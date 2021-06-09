// #![warn(missing_docs)]

use std::sync::{
    atomic::{AtomicU32, Ordering::SeqCst},
    Arc,
};

use crate::{
    animation_manager::AnimationManager,
    data::output::Output,
    frame_state::FrameState,
    input_state::*,
    layers::GraphicLayers,
    localization::{Language, Localization},
    mutex::{Mutex, MutexGuard},
    *,
};
use epaint::{stats::*, text::Fonts, *};

// ----------------------------------------------------------------------------

/// A wrapper around [`Arc`](std::sync::Arc)`<`[`Context`]`>`.
/// This is how you will normally create and access a [`Context`].
///
/// Almost all methods are marked `&self`, `Context` has interior mutability (protected by mutexes).
///
/// [`CtxRef`] is cheap to clone, and any clones refers to the same mutable data.
///
/// # Example:
///
/// ``` no_run
/// # fn handle_output(_: egui::Output) {}
/// # fn paint(_: Vec<egui::ClippedMesh>) {}
/// let mut ctx = egui::CtxRef::default();
///
/// // Game loop:
/// loop {
///     let raw_input = egui::RawInput::default();
///     ctx.begin_frame(raw_input);
///
///     egui::CentralPanel::default().show(&ctx, |ui| {
///         ui.label("Hello world!");
///         if ui.button("Click me").clicked() {
///             /* take some action here */
///         }
///     });
///
///     let (output, shapes) = ctx.end_frame();
///     let clipped_meshes = ctx.tessellate(shapes); // create triangles to paint
///     handle_output(output);
///     paint(clipped_meshes);
/// }
/// ```
///
#[derive(Clone)]
pub struct CtxRef(std::sync::Arc<Context>);

impl std::ops::Deref for CtxRef {
    type Target = Context;

    fn deref(&self) -> &Context {
        &*self.0
    }
}

impl AsRef<Context> for CtxRef {
    fn as_ref(&self) -> &Context {
        self.0.as_ref()
    }
}

impl std::borrow::Borrow<Context> for CtxRef {
    fn borrow(&self) -> &Context {
        self.0.borrow()
    }
}

impl std::cmp::PartialEq for CtxRef {
    fn eq(&self, other: &CtxRef) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Default for CtxRef {
    fn default() -> Self {
        Self(Arc::new(Context {
            // Start with painting an extra frame to compensate for some widgets
            // that take two frames before they "settle":
            repaint_requests: AtomicU32::new(1),
            ..Context::default()
        }))
    }
}

impl CtxRef {
    /// Call at the start of every frame. Match with a call to [`Context::end_frame`].
    ///
    /// This will modify the internal reference to point to a new generation of [`Context`].
    /// Any old clones of this [`CtxRef`] will refer to the old [`Context`], which will not get new input.
    ///
    /// Put your widgets into a [`SidePanel`], [`TopBottomPanel`], [`CentralPanel`], [`Window`] or [`Area`].
    pub fn begin_frame(&mut self, new_input: RawInput) {
        let mut self_: Context = (*self.0).clone();
        self_.begin_frame_mut(new_input);
        *self = Self(Arc::new(self_));
    }

    // ---------------------------------------------------------------------

    /// If the given [`Id`] is not unique, an error will be printed at the given position.
    /// Call this for [`Id`]:s that need interaction or persistence.
    pub(crate) fn register_interaction_id(&self, id: Id, new_pos: Pos2) {
        let prev_pos = self.frame_state().used_ids.insert(id, new_pos);
        if let Some(prev_pos) = prev_pos {
            if prev_pos.distance(new_pos) < 0.1 {
                // Likely same Widget being interacted with twice, which is fine.
                return;
            }

            let show_error = |pos: Pos2, text: String| {
                let painter = self.debug_painter();
                let rect = painter.error(pos, text);
                if let Some(pointer_pos) = self.input.pointer.hover_pos() {
                    if rect.contains(pointer_pos) {
                        painter.error(
                            rect.left_bottom() + vec2(2.0, 4.0),
                            "ID clashes happens when things like Windows or CollapsingHeaders share names,\n\
                             or when things like ScrollAreas and Resize areas aren't given unique id_source:s.",
                        );
                    }
                }
            };

            let id_str = id.short_debug_format();

            if prev_pos.distance(new_pos) < 4.0 {
                show_error(new_pos, format!("Double use of ID {}", id_str));
            } else {
                show_error(prev_pos, format!("First use of ID {}", id_str));
                show_error(new_pos, format!("Second use of ID {}", id_str));
            }

            // TODO: a tooltip explaining this.
        }
    }

    // ---------------------------------------------------------------------

    /// Use `ui.interact` instead
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn interact(
        &self,
        clip_rect: Rect,
        item_spacing: Vec2,
        layer_id: LayerId,
        id: Id,
        rect: Rect,
        sense: Sense,
        enabled: bool,
    ) -> Response {
        let gap = 0.5; // Just to make sure we don't accidentally hover two things at once (a small eps should be sufficient).
        let interact_rect = rect.expand2(
            (0.5 * item_spacing - Vec2::splat(gap))
                .at_least(Vec2::splat(0.0))
                .at_most(Vec2::splat(5.0)),
        ); // make it easier to click
        let hovered = self.rect_contains_pointer(layer_id, clip_rect.intersect(interact_rect));
        self.interact_with_hovered(layer_id, id, rect, sense, enabled, hovered)
    }

    /// You specify if a thing is hovered, and the function gives a `Response`.
    pub(crate) fn interact_with_hovered(
        &self,
        layer_id: LayerId,
        id: Id,
        rect: Rect,
        sense: Sense,
        enabled: bool,
        hovered: bool,
    ) -> Response {
        let hovered = hovered && enabled; // can't even hover disabled widgets

        let mut response = Response {
            ctx: self.clone(),
            layer_id,
            id,
            rect,
            sense,
            enabled,
            hovered,
            clicked: Default::default(),
            double_clicked: Default::default(),
            dragged: false,
            drag_released: false,
            is_pointer_button_down_on: false,
            interact_pointer_pos: None,
            changed: false, // must be set by the widget itself
        };

        if !enabled || !sense.focusable || !layer_id.allow_interaction() {
            // Not interested or allowed input:
            self.memory().surrender_focus(id);
            return response;
        }

        // We only want to focus labels if the screen reader is on.
        let interested_in_focus =
            sense.interactive() || sense.focusable && self.memory().options.screen_reader;

        if interested_in_focus {
            self.memory().interested_in_focus(id);
        }

        if sense.click
            && response.has_focus()
            && (self.input().key_pressed(Key::Space) || self.input().key_pressed(Key::Enter))
        {
            // Space/enter works like a primary click for e.g. selected buttons
            response.clicked[PointerButton::Primary as usize] = true;
        }

        self.register_interaction_id(id, rect.min);

        if sense.click || sense.drag {
            let mut memory = self.memory();

            memory.interaction.click_interest |= hovered && sense.click;
            memory.interaction.drag_interest |= hovered && sense.drag;

            response.dragged = memory.interaction.drag_id == Some(id);
            response.is_pointer_button_down_on =
                memory.interaction.click_id == Some(id) || response.dragged;

            for pointer_event in &self.input.pointer.pointer_events {
                match pointer_event {
                    PointerEvent::Moved(_) => {}
                    PointerEvent::Pressed(_) => {
                        if hovered {
                            if sense.click && memory.interaction.click_id.is_none() {
                                // potential start of a click
                                memory.interaction.click_id = Some(id);
                                response.is_pointer_button_down_on = true;
                            }

                            // HACK: windows have low priority on dragging.
                            // This is so that if you drag a slider in a window,
                            // the slider will steal the drag away from the window.
                            // This is needed because we do window interaction first (to prevent frame delay),
                            // and then do content layout.
                            if sense.drag
                                && (memory.interaction.drag_id.is_none()
                                    || memory.interaction.drag_is_window)
                            {
                                // potential start of a drag
                                memory.interaction.drag_id = Some(id);
                                memory.interaction.drag_is_window = false;
                                memory.window_interaction = None; // HACK: stop moving windows (if any)
                                response.is_pointer_button_down_on = true;
                                response.dragged = true;
                            }
                        }
                    }
                    PointerEvent::Released(click) => {
                        response.drag_released = response.dragged;
                        response.dragged = false;

                        if hovered && response.is_pointer_button_down_on {
                            if let Some(click) = click {
                                let clicked = hovered && response.is_pointer_button_down_on;
                                response.clicked[click.button as usize] = clicked;
                                response.double_clicked[click.button as usize] =
                                    clicked && click.is_double();
                            }
                        }
                    }
                }
            }
        }

        if response.is_pointer_button_down_on {
            response.interact_pointer_pos = self.input().pointer.interact_pos();
        }

        if self.input.pointer.any_down() {
            response.hovered &= response.is_pointer_button_down_on; // we don't hover widgets while interacting with *other* widgets
        }

        if response.has_focus() && response.clicked_elsewhere() {
            self.memory().surrender_focus(id);
        }

        response
    }

    /// Get a full-screen painter for a new or existing layer
    pub fn layer_painter(&self, layer_id: LayerId) -> Painter {
        Painter::new(self.clone(), layer_id, self.input.screen_rect())
    }

    /// Paint on top of everything else
    pub fn debug_painter(&self) -> Painter {
        Self::layer_painter(self, LayerId::debug())
    }
}

// ----------------------------------------------------------------------------

/// This is the first thing you need when working with egui. Create using [`CtxRef`].
///
/// Contains the [`InputState`], [`Memory`], [`Output`], and more.
///
/// Your handle to Egui.
///
/// Almost all methods are marked `&self`, `Context` has interior mutability (protected by mutexes).
/// Multi-threaded access to a [`Context`] is behind the feature flag `multi_threaded`.
/// Normally you'd always do all ui work on one thread, or perhaps use multiple contexts,
/// but if you really want to access the same Context from multiple threads, it *SHOULD* be fine,
/// but you are likely the first person to try it.
#[derive(Default)]
pub struct Context {
    // We clone the Context each frame so we can set a new `input`.
    // This is so we can avoid a mutex lock to access the `InputState`.
    // This means everything else needs to be behind an Arc.
    // We can probably come up with a nicer design.
    //
    /// None until first call to `begin_frame`.
    fonts: Option<Arc<Fonts>>,
    memory: Arc<Mutex<Memory>>,
    animation_manager: Arc<Mutex<AnimationManager>>,
    localization: Arc<Mutex<Localization>>,

    input: InputState,

    /// State that is collected during a frame and then cleared
    frame_state: Arc<Mutex<FrameState>>,

    // The output of a frame:
    graphics: Arc<Mutex<GraphicLayers>>,
    output: Arc<Mutex<Output>>,

    paint_stats: Arc<Mutex<PaintStats>>,

    /// While positive, keep requesting repaints. Decrement at the end of each frame.
    repaint_requests: AtomicU32,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Context {
            fonts: self.fonts.clone(),
            memory: self.memory.clone(),
            animation_manager: self.animation_manager.clone(),
            input: self.input.clone(),
            frame_state: self.frame_state.clone(),
            graphics: self.graphics.clone(),
            output: self.output.clone(),
            paint_stats: self.paint_stats.clone(),
            repaint_requests: self.repaint_requests.load(SeqCst).into(),
            localization: self.localization.clone(),
        }
    }
}

impl Context {
    #[allow(clippy::new_ret_no_self)]
    #[deprecated = "Use CtxRef::default() instead"]
    pub fn new() -> CtxRef {
        CtxRef::default()
    }

    /// How much space is still available after panels has been added.
    /// This is the "background" area, what egui doesn't cover with panels (but may cover with windows).
    /// This is also the area to which windows are constrained.
    pub fn available_rect(&self) -> Rect {
        self.frame_state.lock().available_rect()
    }

    /// Stores all the egui state.
    /// If you want to store/restore egui, serialize this.
    pub fn memory(&self) -> MutexGuard<'_, Memory> {
        self.memory.lock()
    }

    pub(crate) fn graphics(&self) -> MutexGuard<'_, GraphicLayers> {
        self.graphics.lock()
    }

    /// What egui outputs each frame.
    pub fn output(&self) -> MutexGuard<'_, Output> {
        self.output.lock()
    }

    pub(crate) fn frame_state(&self) -> MutexGuard<'_, FrameState> {
        self.frame_state.lock()
    }

    /// Call this if there is need to repaint the UI, i.e. if you are showing an animation.
    /// If this is called at least once in a frame, then there will be another frame right after this.
    /// Call as many times as you wish, only one repaint will be issued.
    pub fn request_repaint(&self) {
        // request two frames of repaint, just to cover some corner cases (frame delays):
        let times_to_repaint = 2;
        self.repaint_requests.store(times_to_repaint, SeqCst);
    }

    #[inline(always)]
    pub fn input(&self) -> &InputState {
        &self.input
    }

    /// Not valid until first call to [`CtxRef::begin_frame()`].
    /// That's because since we don't know the proper `pixels_per_point` until then.
    pub fn fonts(&self) -> &Fonts {
        &*self
            .fonts
            .as_ref()
            .expect("No fonts available until first call to CtxRef::begin_frame()")
    }

    /// The egui texture, containing font characters etc.
    /// Not valid until first call to [`CtxRef::begin_frame()`].
    /// That's because since we don't know the proper `pixels_per_point` until then.
    pub fn texture(&self) -> Arc<epaint::Texture> {
        self.fonts().texture()
    }

    /// Tell `egui` which fonts to use.
    ///
    /// The default `egui` fonts only support latin and cyrillic alphabets,
    /// but you can call this to install additional fonts that support e.g. korean characters.
    ///
    /// The new fonts will become active at the start of the next frame.
    pub fn set_fonts(&self, font_definitions: FontDefinitions) {
        if let Some(current_fonts) = &self.fonts {
            // NOTE: this comparison is expensive since it checks TTF data for equality
            if current_fonts.definitions() == &font_definitions {
                return; // no change - save us from reloading font textures
            }
        }

        self.memory().new_font_definitions = Some(font_definitions);
    }

    /// The [`Style`] used by all subsequent windows, panels etc.
    pub fn style(&self) -> Arc<Style> {
        self.memory().options.style.clone()
    }

    /// The [`Style`] used by all new windows, panels etc.
    ///
    /// You can also use [`Ui::style_mut`] to change the style of a single [`Ui`].
    ///
    /// Example:
    /// ```
    /// # let mut ctx = egui::CtxRef::default();
    /// let mut style: egui::Style = (*ctx.style()).clone();
    /// style.spacing.item_spacing = egui::vec2(10.0, 20.0);
    /// ctx.set_style(style);
    /// ```
    pub fn set_style(&self, style: impl Into<Arc<Style>>) {
        self.memory().options.style = style.into();
    }

    /// The [`Visuals`] used by all subsequent windows, panels etc.
    ///
    /// You can also use [`Ui::visuals_mut`] to change the visuals of a single [`Ui`].
    ///
    /// Example:
    /// ```
    /// # let mut ctx = egui::CtxRef::default();
    /// ctx.set_visuals(egui::Visuals::light()); // Switch to light mode
    /// ```
    pub fn set_visuals(&self, visuals: crate::Visuals) {
        std::sync::Arc::make_mut(&mut self.memory().options.style).visuals = visuals;
    }

    /// The number of physical pixels for each logical point.
    #[inline(always)]
    pub fn pixels_per_point(&self) -> f32 {
        self.input.pixels_per_point()
    }

    /// Set the number of physical pixels for each logical point.
    /// Will become active at the start of the next frame.
    ///
    /// Note that this may be overwritten by input from the integration via [`RawInput::pixels_per_point`].
    /// For instance, when using `egui_web` the browsers native zoom level will always be used.
    pub fn set_pixels_per_point(&self, pixels_per_point: f32) {
        self.memory().new_pixels_per_point = Some(pixels_per_point);
    }

    /// Useful for pixel-perfect rendering
    pub(crate) fn round_to_pixel(&self, point: f32) -> f32 {
        let pixels_per_point = self.pixels_per_point();
        (point * pixels_per_point).round() / pixels_per_point
    }

    /// Useful for pixel-perfect rendering
    pub(crate) fn round_pos_to_pixels(&self, pos: Pos2) -> Pos2 {
        pos2(self.round_to_pixel(pos.x), self.round_to_pixel(pos.y))
    }

    /// Useful for pixel-perfect rendering
    pub(crate) fn round_vec_to_pixels(&self, vec: Vec2) -> Vec2 {
        vec2(self.round_to_pixel(vec.x), self.round_to_pixel(vec.y))
    }

    /// Useful for pixel-perfect rendering
    pub(crate) fn round_rect_to_pixels(&self, rect: Rect) -> Rect {
        Rect {
            min: self.round_pos_to_pixels(rect.min),
            max: self.round_pos_to_pixels(rect.max),
        }
    }

    // ---------------------------------------------------------------------

    /// Constrain the position of a window/area
    /// so it fits within the screen.
    pub(crate) fn constrain_window_rect(&self, window: Rect) -> Rect {
        self.constrain_window_rect_to_area(window, self.available_rect())
    }

    /// Constrain the position of a window/area
    /// so it fits within the provided boundary.
    pub(crate) fn constrain_window_rect_to_area(&self, window: Rect, mut area: Rect) -> Rect {
        if window.width() > area.width() {
            // Allow overlapping side bars.
            // This is important for small screens, e.g. mobiles running the web demo.
            area.max.x = self.input().screen_rect().max.x;
            area.min.x = self.input().screen_rect().min.x;
        }
        if window.height() > area.height() {
            // Allow overlapping top/bottom bars:
            area.max.y = self.input().screen_rect().max.y;
            area.min.y = self.input().screen_rect().min.y;
        }

        let mut pos = window.min;

        // Constrain to screen, unless window is too large to fit:
        let margin_x = (window.width() - area.width()).at_least(0.0);
        let margin_y = (window.height() - area.height()).at_least(0.0);

        pos.x = pos.x.at_most(area.right() + margin_x - window.width()); // move left if needed
        pos.x = pos.x.at_least(area.left() - margin_x); // move right if needed
        pos.y = pos.y.at_most(area.bottom() + margin_y - window.height()); // move right if needed
        pos.y = pos.y.at_least(area.top() - margin_y); // move down if needed

        pos = self.round_pos_to_pixels(pos);

        Rect::from_min_size(pos, window.size())
    }

    // ---------------------------------------------------------------------

    fn begin_frame_mut(&mut self, new_raw_input: RawInput) {
        self.memory().begin_frame(&self.input, &new_raw_input);

        let mut input = std::mem::take(&mut self.input);
        if let Some(new_pixels_per_point) = self.memory().new_pixels_per_point.take() {
            input.pixels_per_point = new_pixels_per_point;
        }

        let lang = self.memory().new_language.take().unwrap_or_default();
        self.set_localization(lang);

        self.input = input.begin_frame(new_raw_input);
        self.frame_state.lock().begin_frame(&self.input);
        {
            // Load new fonts if required:
            let new_font_definitions = self.memory().new_font_definitions.take();
            let pixels_per_point = self.input.pixels_per_point();

            let pixels_per_point_changed = match &self.fonts {
                None => true,
                Some(current_fonts) => {
                    (current_fonts.pixels_per_point() - pixels_per_point).abs() > 1e-3
                }
            };

            if self.fonts.is_none() || new_font_definitions.is_some() || pixels_per_point_changed {
                self.fonts = Some(Arc::new(Fonts::from_definitions(
                    pixels_per_point,
                    new_font_definitions.unwrap_or_default(),
                )));
            }
        }

        // Ensure we register the background area so panels and background ui can catch clicks:
        let screen_rect = self.input.screen_rect();
        self.memory().areas.set_state(
            LayerId::background(),
            containers::area::State {
                pos: screen_rect.min,
                size: screen_rect.size(),
                interactable: true,
            },
        );
    }

    /// Call at the end of each frame.
    /// Returns what has happened this frame [`crate::Output`] as well as what you need to paint.
    /// You can transform the returned shapes into triangles with a call to [`Context::tessellate`].
    #[must_use]
    pub fn end_frame(&self) -> (Output, Vec<ClippedShape>) {
        if self.input.wants_repaint() {
            self.request_repaint();
        }

        self.memory()
            .end_frame(&self.input, &self.frame_state().used_ids);

        self.fonts().end_frame();

        let mut output: Output = std::mem::take(&mut self.output());
        if self.repaint_requests.load(SeqCst) > 0 {
            self.repaint_requests.fetch_sub(1, SeqCst);
            output.needs_repaint = true;
        }

        let shapes = self.drain_paint_lists();
        (output, shapes)
    }

    fn drain_paint_lists(&self) -> Vec<ClippedShape> {
        let memory = self.memory();
        self.graphics().drain(memory.areas.order()).collect()
    }

    /// Tessellate the given shapes into triangle meshes.
    pub fn tessellate(&self, shapes: Vec<ClippedShape>) -> Vec<ClippedMesh> {
        let mut tessellation_options = self.memory().options.tessellation_options;
        tessellation_options.pixels_per_point = self.pixels_per_point();
        tessellation_options.aa_size = 1.0 / self.pixels_per_point();
        let paint_stats = PaintStats::from_shapes(&shapes); // TODO: internal allocations
        let clipped_meshes = tessellator::tessellate_shapes(
            shapes,
            tessellation_options,
            self.fonts().texture().size(),
        );
        *self.paint_stats.lock() = paint_stats.with_clipped_meshes(&clipped_meshes);
        clipped_meshes
    }

    // ---------------------------------------------------------------------

    /// How much space is used by panels and windows.
    pub fn used_rect(&self) -> Rect {
        let mut used = self.frame_state().used_by_panels;
        for window in self.memory().areas.visible_windows() {
            used = used.union(window.rect());
        }
        used
    }

    /// How much space is used by panels and windows.
    /// You can shrink your egui area to this size and still fit all egui components.
    pub fn used_size(&self) -> Vec2 {
        self.used_rect().max - Pos2::new(0.0, 0.0)
    }

    // ---------------------------------------------------------------------

    /// Is the pointer (mouse/touch) over any egui area?
    pub fn is_pointer_over_area(&self) -> bool {
        if let Some(pointer_pos) = self.input.pointer.interact_pos() {
            if let Some(layer) = self.layer_id_at(pointer_pos) {
                if layer.order == Order::Background {
                    !self.frame_state().unused_rect.contains(pointer_pos)
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// True if egui is currently interested in the pointer (mouse or touch).
    /// Could be the pointer is hovering over a [`Window`] or the user is dragging a widget.
    /// If `false`, the pointer is outside of any egui area and so
    /// you may be interested in what it is doing (e.g. controlling your game).
    /// Returns `false` if a drag started outside of egui and then moved over an egui area.
    pub fn wants_pointer_input(&self) -> bool {
        self.is_using_pointer() || (self.is_pointer_over_area() && !self.input().pointer.any_down())
    }

    /// Is egui currently using the pointer position (e.g. dragging a slider).
    /// NOTE: this will return `false` if the pointer is just hovering over an egui area.
    pub fn is_using_pointer(&self) -> bool {
        self.memory().interaction.is_using_pointer()
    }

    #[deprecated = "Renamed wants_pointer_input"]
    pub fn wants_mouse_input(&self) -> bool {
        self.wants_pointer_input()
    }

    #[deprecated = "Renamed is_using_pointer"]
    pub fn is_using_mouse(&self) -> bool {
        self.is_using_pointer()
    }

    /// If `true`, egui is currently listening on text input (e.g. typing text in a [`TextEdit`]).
    pub fn wants_keyboard_input(&self) -> bool {
        self.memory().interaction.focus.focused().is_some()
    }

    // ---------------------------------------------------------------------

    /// Move all the graphics at the given layer.
    /// Can be used to implement drag-and-drop (see relevant demo).
    pub fn translate_layer(&self, layer_id: LayerId, delta: Vec2) {
        if delta != Vec2::ZERO {
            self.graphics().list(layer_id).lock().translate(delta);
        }
    }

    pub fn layer_id_at(&self, pos: Pos2) -> Option<LayerId> {
        let resize_grab_radius_side = self.style().interaction.resize_grab_radius_side;
        self.memory().layer_id_at(pos, resize_grab_radius_side)
    }

    pub(crate) fn rect_contains_pointer(&self, layer_id: LayerId, rect: Rect) -> bool {
        if let Some(pointer_pos) = self.input.pointer.interact_pos() {
            rect.contains(pointer_pos) && self.layer_id_at(pointer_pos) == Some(layer_id)
        } else {
            false
        }
    }

    // ---------------------------------------------------------------------

    /// Wether or not to debug widget layout on hover.
    pub fn debug_on_hover(&self) -> bool {
        self.memory().options.style.debug.debug_on_hover
    }

    /// Turn on/off wether or not to debug widget layout on hover.
    pub fn set_debug_on_hover(&self, debug_on_hover: bool) {
        let mut style = (*self.memory().options.style).clone();
        style.debug.debug_on_hover = debug_on_hover;
        self.set_style(style);
    }
}

/// ## Animation
impl Context {
    /// Returns a value in the range [0, 1], to indicate "how on" this thing is.
    ///
    /// The first time called it will return `if value { 1.0 } else { 0.0 }`
    /// Calling this with `value = true` will always yield a number larger than zero, quickly going towards one.
    /// Calling this with `value = false` will always yield a number less than one, quickly going towards zero.
    ///
    /// The function will call [`Self::request_repaint()`] when appropriate.
    pub fn animate_bool(&self, id: Id, value: bool) -> f32 {
        let animation_time = self.style().animation_time;
        let animated_value =
            self.animation_manager
                .lock()
                .animate_bool(&self.input, animation_time, id, value);
        let animation_in_progress = 0.0 < animated_value && animated_value < 1.0;
        if animation_in_progress {
            self.request_repaint();
        }
        animated_value
    }

    /// Clear memory of any animations.
    pub fn clear_animations(&self) {
        *self.animation_manager.lock() = Default::default();
    }
}

impl Context {
    pub fn settings_ui(&self, ui: &mut Ui) {
        use crate::containers::*;

        CollapsingHeader::new("🎑 Style")
            .default_open(true)
            .show(ui, |ui| {
                self.style_ui(ui);
            });

        CollapsingHeader::new("🔠 Fonts")
            .default_open(false)
            .show(ui, |ui| {
                let mut font_definitions = self.fonts().definitions().clone();
                font_definitions.ui(ui);
                self.fonts().texture().ui(ui);
                self.set_fonts(font_definitions);
            });

        CollapsingHeader::new("✒ Painting")
            .default_open(true)
            .show(ui, |ui| {
                let mut tessellation_options = self.memory().options.tessellation_options;
                tessellation_options.ui(ui);
                ui.vertical_centered(|ui| reset_button(ui, &mut tessellation_options));
                self.memory().options.tessellation_options = tessellation_options;
            });
    }

    pub fn inspection_ui(&self, ui: &mut Ui) {
        use crate::containers::*;
        crate::trace!(ui);

        ui.label(format!("Is using pointer: {}", self.is_using_pointer()))
            .on_hover_text(
                "Is egui currently using the pointer actively (e.g. dragging a slider)?",
            );
        ui.label(format!("Wants pointer input: {}", self.wants_pointer_input()))
            .on_hover_text("Is egui currently interested in the location of the pointer (either because it is in use, or because it is hovering over a window).");
        ui.label(format!(
            "Wants keyboard input: {}",
            self.wants_keyboard_input()
        ))
        .on_hover_text("Is egui currently listening for text input?");
        ui.label(format!(
            "keyboard focus widget: {}",
            self.memory()
                .interaction
                .focus
                .focused()
                .as_ref()
                .map(Id::short_debug_format)
                .unwrap_or_default()
        ))
        .on_hover_text("Is egui currently listening for text input?");
        ui.add_space(16.0);

        ui.label(format!(
            "There are {} text galleys in the layout cache",
            self.fonts().num_galleys_in_cache()
        ))
        .on_hover_text("This is approximately the number of text strings on screen");
        ui.add_space(16.0);

        CollapsingHeader::new("📥 Input")
            .default_open(false)
            .show(ui, |ui| ui.input().clone().ui(ui));

        CollapsingHeader::new("📊 Paint stats")
            .default_open(true)
            .show(ui, |ui| {
                self.paint_stats.lock().ui(ui);
            });
    }

    pub fn memory_ui(&self, ui: &mut crate::Ui) {
        if ui
            .button("Reset all")
            .on_hover_text("Reset all egui state")
            .clicked()
        {
            *self.memory() = Default::default();
        }

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} areas (window positions)",
                self.memory().areas.count()
            ));
            if ui.button("Reset").clicked() {
                self.memory().areas = Default::default();
            }
        });
        ui.indent("areas", |ui| {
            let layers_ids: Vec<LayerId> = self.memory().areas.order().to_vec();
            for layer_id in layers_ids {
                let area = self.memory().areas.get(layer_id.id).cloned();
                if let Some(area) = area {
                    let is_visible = self.memory().areas.is_visible(&layer_id);
                    if ui
                        .label(format!(
                            "{:?} {:?} {}",
                            layer_id.order,
                            area.rect(),
                            if is_visible { "" } else { "(INVISIBLE)" }
                        ))
                        .hovered
                        && is_visible
                    {
                        ui.ctx()
                            .debug_painter()
                            .debug_rect(area.rect(), Color32::RED, "");
                    }
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} collapsing headers",
                self.memory()
                    .id_data
                    .count::<containers::collapsing_header::State>()
            ));
            if ui.button("Reset").clicked() {
                self.memory()
                    .id_data
                    .remove_by_type::<containers::collapsing_header::State>();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} menu bars",
                self.memory().id_data_temp.count::<menu::BarState>()
            ));
            if ui.button("Reset").clicked() {
                self.memory()
                    .id_data_temp
                    .remove_by_type::<menu::BarState>();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} scroll areas",
                self.memory().id_data.count::<scroll_area::State>()
            ));
            if ui.button("Reset").clicked() {
                self.memory().id_data.remove_by_type::<scroll_area::State>();
            }
        });

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} resize areas",
                self.memory().id_data.count::<resize::State>()
            ));
            if ui.button("Reset").clicked() {
                self.memory().id_data.remove_by_type::<resize::State>();
            }
        });

        ui.shrink_width_to_current(); // don't let the text below grow this window wider
        ui.label("NOTE: the position of this window cannot be reset from within itself.");

        ui.collapsing("Interaction", |ui| {
            let interaction = self.memory().interaction.clone();
            interaction.ui(ui);
        });
    }
}

impl Context {
    pub fn style_ui(&self, ui: &mut Ui) {
        let mut style: Style = (*self.style()).clone();
        style.ui(ui);
        self.set_style(style);
    }
}

/// ## Localization
impl Context {
    /// Sets the current language of the default texts within [`crate::widgets`] and [`crate::containers`].
    ///
    /// The languages available for localization can be seen in [`crate::localization::Language`].
    pub fn set_localization(&self, lang: Language) {
        self.localization().set_localization(lang.clone());
        self.memory().new_language = Some(lang);
    }

    /// Use this to access the `Localization` struct stored within context and also all the texts stored within it.
    pub fn localization(&self) -> MutexGuard<'_, Localization> {
        self.localization.lock()
    }

    /// Returns the language that will be set in the next frame
    pub fn lang(&self) -> Language {
        let new_lang = self.memory().new_language.clone().unwrap_or_default();
        new_lang
    }
}
