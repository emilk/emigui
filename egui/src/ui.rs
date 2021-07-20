// #![warn(missing_docs)]

use std::hash::Hash;

use crate::{
    color::*, containers::*, epaint::text::Fonts, layout::*, mutex::MutexGuard, placer::Placer,
    widgets::*, *,
};

// ----------------------------------------------------------------------------

/// This is what you use to place widgets.
///
/// Represents a region of the screen with a type of layout (horizontal or vertical).
///
/// ```
/// # let mut ui = egui::Ui::__test();
/// ui.add(egui::Label::new("Hello World!"));
/// ui.label("A shorter and more convenient way to add a label.");
/// ui.horizontal(|ui| {
///     ui.label("Add widgets");
///     if ui.button("on the same row!").clicked() {
///         /* … */
///     }
/// });
/// ```
pub struct Ui {
    /// ID of this ui.
    /// Generated based on id of parent ui together with
    /// another source of child identity (e.g. window title).
    /// Acts like a namespace for child uis.
    /// Should be unique and persist predictably from one frame to next
    /// so it can be used as a source for storing state (e.g. window position, or if a collapsing header is open).
    id: Id,

    /// This is used to create a unique interact ID for some widgets.
    /// This value is based on where in the hierarchy of widgets this Ui is in,
    /// and the value is increment with each added child widget.
    /// This works as an Id source only as long as new widgets aren't added or removed.
    /// They are therefore only good for Id:s that has no state.
    next_auto_id_source: u64,

    /// Specifies paint layer, clip rectangle and a reference to `Context`.
    painter: Painter,

    /// The `Style` (visuals, spacing, etc) of this ui.
    /// Commonly many `Ui`:s share the same `Style`.
    /// The `Ui` implements copy-on-write for this.
    style: std::sync::Arc<Style>,

    /// Handles the `Ui` size and the placement of new widgets.
    placer: Placer,

    /// If false we are unresponsive to input,
    /// and all widgets will assume a gray style.
    enabled: bool,
}

impl Ui {
    // ------------------------------------------------------------------------
    // Creation:

    /// Create a new `Ui`.
    ///
    /// Normally you would not use this directly, but instead use
    /// [`SidePanel`], [`TopBottomPanel`], [`CentralPanel`], [`Window`] or [`Area`].
    pub fn new(ctx: CtxRef, layer_id: LayerId, id: Id, max_rect: Rect, clip_rect: Rect) -> Self {
        let style = ctx.style();
        Ui {
            id,
            next_auto_id_source: id.with("auto").value(),
            painter: Painter::new(ctx, layer_id, clip_rect),
            style,
            placer: Placer::new(max_rect, Layout::default()),
            enabled: true,
        }
    }

    /// Create a new `Ui` at a specific region.
    pub fn child_ui(&mut self, max_rect: Rect, layout: Layout) -> Self {
        crate::egui_assert!(!max_rect.any_nan());
        let next_auto_id_source = Id::new(self.next_auto_id_source).with("child").value();
        self.next_auto_id_source = self.next_auto_id_source.wrapping_add(1);

        Ui {
            id: self.id.with("child"),
            next_auto_id_source,
            painter: self.painter.clone(),
            style: self.style.clone(),
            placer: Placer::new(max_rect, layout),
            enabled: self.enabled,
        }
    }

    /// Empty `Ui` for use in tests.
    pub fn __test() -> Self {
        let mut ctx = CtxRef::default();
        ctx.begin_frame(Default::default());
        let id = Id::new("__test");
        let layer_id = LayerId::new(Order::Middle, id);
        let rect = Rect::from_min_size(Pos2::new(0.0, 0.0), vec2(1000.0, 1000.0));
        Self::new(ctx, layer_id, id, rect, rect)
    }

    // -------------------------------------------------

    /// A unique identity of this `Ui`.
    #[inline(always)]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Style options for this `Ui` and its children.
    #[inline(always)]
    pub fn style(&self) -> &std::sync::Arc<Style> {
        &self.style
    }

    /// Mutably borrow internal `Style`.
    /// Changes apply to this `Ui` and its subsequent children.
    ///
    /// To set the style of all `Ui`:s, use [`Context::set_style`].
    ///
    /// Example:
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.style_mut().body_text_style = egui::TextStyle::Heading;
    /// ```
    pub fn style_mut(&mut self) -> &mut Style {
        std::sync::Arc::make_mut(&mut self.style) // clone-on-write
    }

    /// Changes apply to this `Ui` and its subsequent children.
    ///
    /// To set the visuals of all `Ui`:s, use [`Context::set_visuals`].
    pub fn set_style(&mut self, style: impl Into<std::sync::Arc<Style>>) {
        self.style = style.into();
    }

    /// Reset to the default style set in [`Context`].
    pub fn reset_style(&mut self) {
        self.style = self.ctx().style();
    }

    /// The current spacing options for this `Ui`.
    /// Short for `ui.style().spacing`.
    #[inline(always)]
    pub fn spacing(&self) -> &crate::style::Spacing {
        &self.style.spacing
    }

    /// Mutably borrow internal `Spacing`.
    /// Changes apply to this `Ui` and its subsequent children.
    ///
    /// Example:
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.spacing_mut().item_spacing = egui::vec2(10.0, 2.0);
    /// ```
    pub fn spacing_mut(&mut self) -> &mut crate::style::Spacing {
        &mut self.style_mut().spacing
    }

    /// The current visuals settings of this `Ui`.
    /// Short for `ui.style().visuals`.
    #[inline(always)]
    pub fn visuals(&self) -> &crate::Visuals {
        &self.style.visuals
    }

    /// Mutably borrow internal `visuals`.
    /// Changes apply to this `Ui` and its subsequent children.
    ///
    /// To set the visuals of all `Ui`:s, use [`Context::set_visuals`].
    ///
    /// Example:
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
    /// ```
    pub fn visuals_mut(&mut self) -> &mut crate::Visuals {
        &mut self.style_mut().visuals
    }

    /// Get a reference to the parent [`CtxRef`].
    #[inline(always)]
    pub fn ctx(&self) -> &CtxRef {
        self.painter.ctx()
    }

    /// Use this to paint stuff within this `Ui`.
    #[inline(always)]
    pub fn painter(&self) -> &Painter {
        &self.painter
    }

    /// If `false`, the `Ui` does not allow any interaction and
    /// the widgets in it will draw with a gray look.
    #[inline(always)]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Calling `set_enabled(false)` will cause the `Ui` to deny all future interaction
    /// and all the widgets will draw with a gray look.
    ///
    /// Calling `set_enabled(true)` has no effect - it will NOT re-enable the `Ui` once disabled.
    ///
    /// ### Example
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// # let mut enabled = true;
    /// ui.group(|ui|{
    ///     ui.checkbox(&mut enabled, "Enable subsection");
    ///     ui.set_enabled(enabled);
    ///     if ui.button("Button that is not always clickable").clicked() {
    ///         /* … */
    ///     }
    /// });
    /// ```
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled &= enabled;
        if !self.enabled && self.visible() {
            self.painter
                .set_fade_to_color(Some(self.visuals().window_fill()));
        }
    }

    /// If `false`, any widgets added to the `Ui` will be invisible and non-interactive.
    #[inline(always)]
    pub fn visible(&self) -> bool {
        self.painter.visible()
    }

    /// Calling `set_visible(false)` will cause all further widgets to be invisible,
    /// yet still allocate space.
    ///
    /// The widgets will not be interactive (`set_visible(false)` implies `set_enabled(false)`).
    ///
    /// Calling `set_visible(true)` has no effect.
    ///
    /// ### Example
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// # let mut visible = true;
    /// ui.group(|ui|{
    ///     ui.checkbox(&mut visible, "Show subsection");
    ///     ui.set_visible(visible);
    ///     if ui.button("Button that is not always shown").clicked() {
    ///         /* … */
    ///     }
    /// });
    /// ```
    pub fn set_visible(&mut self, visible: bool) {
        self.set_enabled(visible);
        if !visible {
            self.painter.set_invisible();
        }
    }

    #[inline(always)]
    pub fn layout(&self) -> &Layout {
        self.placer.layout()
    }

    /// Should text wrap in this `Ui`?
    /// This is determined first by [`Style::wrap`], and then by the layout of this `Ui`.
    pub fn wrap_text(&self) -> bool {
        if let Some(wrap) = self.style.wrap {
            wrap
        } else if let Some(grid) = self.placer.grid() {
            grid.wrap_text()
        } else {
            // In vertical layouts we wrap text, but in horizontal we keep going.
            self.layout().is_vertical()
        }
    }

    /// Create a painter for a sub-region of this Ui.
    ///
    /// The clip-rect of the returned `Painter` will be the intersection
    /// of the given rectangle and the `clip_rect()` of this `Ui`.
    pub fn painter_at(&self, rect: Rect) -> Painter {
        self.painter().sub_region(rect)
    }

    /// Use this to paint stuff within this `Ui`.
    pub fn layer_id(&self) -> LayerId {
        self.painter().layer_id()
    }

    /// The `Input` of the `Context` associated with the `Ui`.
    /// Equivalent to `.ctx().input()`.
    #[inline(always)]
    pub fn input(&self) -> &InputState {
        self.ctx().input()
    }

    /// The `Memory` of the `Context` associated with the `Ui`.
    /// Equivalent to `.ctx().memory()`.
    pub fn memory(&self) -> MutexGuard<'_, Memory> {
        self.ctx().memory()
    }

    /// The `Output` of the `Context` associated with the `Ui`.
    /// Equivalent to `.ctx().output()`.
    pub fn output(&self) -> MutexGuard<'_, Output> {
        self.ctx().output()
    }

    /// The `Fonts` of the `Context` associated with the `Ui`.
    /// Equivalent to `.ctx().fonts()`.
    pub fn fonts(&self) -> &Fonts {
        self.ctx().fonts()
    }

    /// Screen-space rectangle for clipping what we paint in this ui.
    /// This is used, for instance, to avoid painting outside a window that is smaller than its contents.
    pub fn clip_rect(&self) -> Rect {
        self.painter.clip_rect()
    }

    /// Screen-space rectangle for clipping what we paint in this ui.
    /// This is used, for instance, to avoid painting outside a window that is smaller than its contents.
    pub fn set_clip_rect(&mut self, clip_rect: Rect) {
        self.painter.set_clip_rect(clip_rect);
    }
}

// ------------------------------------------------------------------------

/// # Sizes etc
impl Ui {
    /// Where and how large the `Ui` is already.
    /// All widgets that have been added ot this `Ui` fits within this rectangle.
    ///
    /// No matter what, the final Ui will be at least this large.
    ///
    /// This will grow as new widgets are added, but never shrink.
    pub fn min_rect(&self) -> Rect {
        self.placer.min_rect()
    }

    /// Size of content; same as `min_rect().size()`
    pub fn min_size(&self) -> Vec2 {
        self.min_rect().size()
    }

    /// New widgets will *try* to fit within this rectangle.
    ///
    /// Text labels will wrap to fit within `max_rect`.
    /// Separator lines will span the `max_rect`.
    ///
    /// If a new widget doesn't fit within the `max_rect` then the
    /// `Ui` will make room for it by expanding both `min_rect` and `max_rect`.
    pub fn max_rect(&self) -> Rect {
        self.placer.max_rect()
    }

    /// This is like `max_rect()`, but will never be infinite.
    /// This can be useful for widgets that expand to fit the available space.
    pub fn max_rect_finite(&self) -> Rect {
        self.placer.max_rect_finite()
    }

    /// Used for animation, kind of hacky
    pub(crate) fn force_set_min_rect(&mut self, min_rect: Rect) {
        self.placer.force_set_min_rect(min_rect)
    }

    // ------------------------------------------------------------------------

    /// Set the maximum size of the ui.
    /// You won't be able to shrink it below the current minimum size.
    pub fn set_max_size(&mut self, size: Vec2) {
        self.set_max_width(size.x);
        self.set_max_height(size.y);
    }

    /// Set the maximum width of the ui.
    /// You won't be able to shrink it below the current minimum size.
    pub fn set_max_width(&mut self, width: f32) {
        self.placer.set_max_width(width);
    }

    /// Set the maximum height of the ui.
    /// You won't be able to shrink it below the current minimum size.
    pub fn set_max_height(&mut self, height: f32) {
        self.placer.set_max_height(height);
    }

    // ------------------------------------------------------------------------

    /// Set the minimum size of the ui.
    /// This can't shrink the ui, only make it larger.
    pub fn set_min_size(&mut self, size: Vec2) {
        self.set_min_width(size.x);
        self.set_min_height(size.y);
    }

    /// Set the minimum width of the ui.
    /// This can't shrink the ui, only make it larger.
    pub fn set_min_width(&mut self, width: f32) {
        self.placer.set_min_width(width);
    }

    /// Set the minimum height of the ui.
    /// This can't shrink the ui, only make it larger.
    pub fn set_min_height(&mut self, height: f32) {
        self.placer.set_min_height(height);
    }

    // ------------------------------------------------------------------------

    /// Helper: shrinks the max width to the current width,
    /// so further widgets will try not to be wider than previous widgets.
    /// Useful for normal vertical layouts.
    pub fn shrink_width_to_current(&mut self) {
        self.set_max_width(self.min_rect().width())
    }

    /// Helper: shrinks the max height to the current height,
    /// so further widgets will try not to be wider than previous widgets.
    pub fn shrink_height_to_current(&mut self) {
        self.set_max_height(self.min_rect().height())
    }

    /// Expand the `min_rect` and `max_rect` of this ui to include a child at the given rect.
    pub fn expand_to_include_rect(&mut self, rect: Rect) {
        self.placer.expand_to_include_rect(rect);
    }

    /// `ui.set_width_range(min..=max);` is equivalent to `ui.set_min_width(min); ui.set_max_width(max);`.
    pub fn set_width_range(&mut self, width: std::ops::RangeInclusive<f32>) {
        self.set_min_width(*width.start());
        self.set_max_width(*width.end());
    }

    /// Set both the minimum and maximum width.
    pub fn set_width(&mut self, width: f32) {
        self.set_min_width(width);
        self.set_max_width(width);
    }

    /// Set both the minimum and maximum height.
    pub fn set_height(&mut self, height: f32) {
        self.set_min_height(height);
        self.set_max_height(height);
    }

    /// Ensure we are big enough to contain the given x-coordinate.
    /// This is sometimes useful to expand an ui to stretch to a certain place.
    pub fn expand_to_include_x(&mut self, x: f32) {
        self.placer.expand_to_include_x(x);
    }

    // ------------------------------------------------------------------------
    // Layout related measures:

    /// The available space at the moment, given the current cursor.
    /// This how much more space we can take up without overflowing our parent.
    /// Shrinks as widgets allocate space and the cursor moves.
    /// A small size should be interpreted as "as little as possible".
    /// An infinite size should be interpreted as "as much as you want".
    pub fn available_size(&self) -> Vec2 {
        self.placer.available_size()
    }

    pub fn available_width(&self) -> f32 {
        self.available_size().x
    }

    /// In case of a wrapping layout, how much space is left on this row/column?
    pub fn available_size_before_wrap(&self) -> Vec2 {
        self.placer.available_rect_before_wrap().size()
    }

    /// This is like `available_size_before_wrap()`, but will never be infinite.
    /// This can be useful for widgets that expand to fit the available space.
    /// In most layouts the next widget will be put in the top left corner of this `Rect`.
    pub fn available_size_before_wrap_finite(&self) -> Vec2 {
        self.placer.available_rect_before_wrap_finite().size()
    }

    pub fn available_rect_before_wrap(&self) -> Rect {
        self.placer.available_rect_before_wrap()
    }

    /// This is like `available_rect_before_wrap()`, but will never be infinite.
    /// This can be useful for widgets that expand to fit the available space.
    /// In most layouts the next widget will be put in the top left corner of this `Rect`.
    pub fn available_rect_before_wrap_finite(&self) -> Rect {
        self.placer.available_rect_before_wrap_finite()
    }
}

/// # `Id` creation
impl Ui {
    /// Use this to generate widget ids for widgets that have persistent state in `Memory`.
    pub fn make_persistent_id<IdSource>(&self, id_source: IdSource) -> Id
    where
        IdSource: Hash + std::fmt::Debug,
    {
        self.id.with(&id_source)
    }

    pub(crate) fn next_auto_id(&self) -> Id {
        Id::new(self.next_auto_id_source)
    }

    pub(crate) fn auto_id_with<IdSource>(&self, id_source: IdSource) -> Id
    where
        IdSource: Hash + std::fmt::Debug,
    {
        Id::new(self.next_auto_id_source).with(id_source)
    }

    pub fn skip_ahead_auto_ids(&mut self, count: usize) {
        self.next_auto_id_source = self.next_auto_id_source.wrapping_add(count as u64);
    }
}

/// # Interaction
impl Ui {
    /// Check for clicks, drags and/or hover on a specific region of this `Ui`.
    pub fn interact(&self, rect: Rect, id: Id, sense: Sense) -> Response {
        self.ctx().interact(
            self.clip_rect(),
            self.spacing().item_spacing,
            self.layer_id(),
            id,
            rect,
            sense,
            self.enabled,
        )
    }

    /// Is the pointer (mouse/touch) above this rectangle in this `Ui`?
    ///
    /// The `clip_rect` and layer of this `Ui` will be respected, so, for instance,
    /// if this `Ui` is behind some other window, this will always return `false`.
    pub fn rect_contains_pointer(&self, rect: Rect) -> bool {
        self.ctx()
            .rect_contains_pointer(self.layer_id(), self.clip_rect().intersect(rect))
    }

    /// Is the pointer (mouse/touch) above this `Ui`?
    /// Equivalent to `ui.rect_contains_pointer(ui.min_rect())`
    pub fn ui_contains_pointer(&self) -> bool {
        self.rect_contains_pointer(self.min_rect())
    }

    #[deprecated = "renamed rect_contains_pointer"]
    pub fn rect_contains_mouse(&self, rect: Rect) -> bool {
        self.rect_contains_pointer(rect)
    }

    #[deprecated = "renamed ui_contains_pointer"]
    pub fn ui_contains_mouse(&self) -> bool {
        self.ui_contains_pointer()
    }

    #[deprecated = "Use: interact(rect, id, Sense::hover())"]
    pub fn interact_hover(&self, rect: Rect) -> Response {
        self.interact(rect, self.auto_id_with("hover_rect"), Sense::hover())
    }

    #[deprecated = "Use: rect_contains_pointer()"]
    pub fn hovered(&self, rect: Rect) -> bool {
        self.interact(rect, self.id, Sense::hover()).hovered
    }
}

/// # Allocating space: where do I put my widgets?
impl Ui {
    /// Allocate space for a widget and check for interaction in the space.
    /// Returns a `Response` which contains a rectangle, id, and interaction info.
    ///
    /// ## How sizes are negotiated
    /// Each widget should have a *minimum desired size* and a *desired size*.
    /// When asking for space, ask AT LEAST for your minimum, and don't ask for more than you need.
    /// If you want to fill the space, ask about `available_size()` and use that.
    ///
    /// You may get MORE space than you asked for, for instance
    /// for justified layouts, like in menus.
    ///
    /// You will never get a rectangle that is smaller than the amount of space you asked for.
    ///
    /// ```
    /// # let mut ui = egui::Ui::__test();
    /// let response = ui.allocate_response(egui::vec2(100.0, 200.0), egui::Sense::click());
    /// if response.clicked() { /* … */ }
    /// ui.painter().rect_stroke(response.rect, 0.0, (1.0, egui::Color32::WHITE));
    /// ```
    pub fn allocate_response(&mut self, desired_size: Vec2, sense: Sense) -> Response {
        let (id, rect) = self.allocate_space(desired_size);
        self.interact(rect, id, sense)
    }

    /// Returns a `Rect` with exactly what you asked for.
    ///
    /// The response rect will be larger if this is part of a justified layout or similar.
    /// This means that if this is a narrow widget in a wide justified layout, then
    /// the widget will react to interactions outside the returned `Rect`.
    pub fn allocate_exact_size(&mut self, desired_size: Vec2, sense: Sense) -> (Rect, Response) {
        let response = self.allocate_response(desired_size, sense);
        let rect = self
            .placer
            .align_size_within_rect(desired_size, response.rect);
        (rect, response)
    }

    /// Allocate at least as much space as needed, and interact with that rect.
    ///
    /// The returned `Rect` will be the same size as `Response::rect`.
    pub fn allocate_at_least(&mut self, desired_size: Vec2, sense: Sense) -> (Rect, Response) {
        let response = self.allocate_response(desired_size, sense);
        (response.rect, response)
    }

    /// Reserve this much space and move the cursor.
    /// Returns where to put the widget.
    ///
    /// ## How sizes are negotiated
    /// Each widget should have a *minimum desired size* and a *desired size*.
    /// When asking for space, ask AT LEAST for your minimum, and don't ask for more than you need.
    /// If you want to fill the space, ask about `available().size()` and use that.
    ///
    /// You may get MORE space than you asked for, for instance
    /// for justified layouts, like in menus.
    ///
    /// You will never get a rectangle that is smaller than the amount of space you asked for.
    ///
    /// Returns an automatic `Id` (which you can use for interaction) and the `Rect` of where to put your widget.
    ///
    /// ```
    /// # let mut ui = egui::Ui::__test();
    /// let (id, rect) = ui.allocate_space(egui::vec2(100.0, 200.0));
    /// let response = ui.interact(rect, id, egui::Sense::click());
    /// ```
    pub fn allocate_space(&mut self, desired_size: Vec2) -> (Id, Rect) {
        // For debug rendering
        let original_available = self.available_size_before_wrap();
        let too_wide = desired_size.x > original_available.x;
        let too_high = desired_size.y > original_available.y;

        let rect = self.allocate_space_impl(desired_size);

        if self.style().debug.debug_on_hover && self.rect_contains_pointer(rect) {
            let painter = self.ctx().debug_painter();
            painter.rect_stroke(rect, 4.0, (1.0, Color32::LIGHT_BLUE));
            self.placer.debug_paint_cursor(&painter, "next");
        }

        let debug_expand_width = self.style().debug.show_expand_width;
        let debug_expand_height = self.style().debug.show_expand_height;

        if (debug_expand_width && too_wide) || (debug_expand_height && too_high) {
            self.painter
                .rect_stroke(rect, 0.0, (1.0, Color32::LIGHT_BLUE));

            let stroke = Stroke::new(2.5, Color32::from_rgb(200, 0, 0));
            let paint_line_seg = |a, b| self.painter().line_segment([a, b], stroke);

            if debug_expand_width && too_wide {
                paint_line_seg(rect.left_top(), rect.left_bottom());
                paint_line_seg(rect.left_center(), rect.right_center());
                paint_line_seg(
                    pos2(rect.left() + original_available.x, rect.top()),
                    pos2(rect.left() + original_available.x, rect.bottom()),
                );
                paint_line_seg(rect.right_top(), rect.right_bottom());
            }

            if debug_expand_height && too_high {
                paint_line_seg(rect.left_top(), rect.right_top());
                paint_line_seg(rect.center_top(), rect.center_bottom());
                paint_line_seg(rect.left_bottom(), rect.right_bottom());
            }
        }

        let id = Id::new(self.next_auto_id_source);
        self.next_auto_id_source = self.next_auto_id_source.wrapping_add(1);

        (id, rect)
    }

    /// Reserve this much space and move the cursor.
    /// Returns where to put the widget.
    fn allocate_space_impl(&mut self, desired_size: Vec2) -> Rect {
        let item_spacing = self.spacing().item_spacing;
        let frame_rect = self.placer.next_space(desired_size, item_spacing);
        let widget_rect = self.placer.justify_and_align(frame_rect, desired_size);

        self.placer
            .advance_after_rects(frame_rect, widget_rect, item_spacing);

        widget_rect
    }

    /// Allocate a specific part of the `Ui`.
    ///
    /// Ignore the layout of the `Ui`: just put my widget here!
    /// The layout cursor will advance to past this `rect`.
    pub fn allocate_rect(&mut self, rect: Rect, sense: Sense) -> Response {
        let id = self.advance_cursor_after_rect(rect);
        self.interact(rect, id, sense)
    }

    pub(crate) fn advance_cursor_after_rect(&mut self, rect: Rect) -> Id {
        let item_spacing = self.spacing().item_spacing;
        self.placer.advance_after_rects(rect, rect, item_spacing);

        if self.style().debug.debug_on_hover && self.rect_contains_pointer(rect) {
            let painter = self.ctx().debug_painter();
            painter.rect_stroke(rect, 4.0, (1.0, Color32::LIGHT_BLUE));
            self.placer.debug_paint_cursor(&painter, "next");
        }

        let id = Id::new(self.next_auto_id_source);
        self.next_auto_id_source = self.next_auto_id_source.wrapping_add(1);
        id
    }

    pub(crate) fn placer(&self) -> &Placer {
        &self.placer
    }

    pub(crate) fn cursor(&self) -> Rect {
        self.placer.cursor()
    }

    /// Where do we expect a zero-sized widget to be placed?
    pub(crate) fn next_widget_position(&self) -> Pos2 {
        self.placer.next_widget_position()
    }

    /// Allocated the given space and then adds content to that space.
    /// If the contents overflow, more space will be allocated.
    /// When finished, the amount of space actually used (`min_rect`) will be allocated.
    /// So you can request a lot of space and then use less.
    #[inline(always)]
    pub fn allocate_ui<R>(
        &mut self,
        desired_size: Vec2,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R> {
        self.allocate_ui_with_layout(desired_size, *self.layout(), add_contents)
    }

    /// Allocated the given space and then adds content to that space.
    /// If the contents overflow, more space will be allocated.
    /// When finished, the amount of space actually used (`min_rect`) will be allocated.
    /// So you can request a lot of space and then use less.
    #[inline(always)]
    pub fn allocate_ui_with_layout<R>(
        &mut self,
        desired_size: Vec2,
        layout: Layout,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R> {
        self.allocate_ui_with_layout_dyn(desired_size, layout, Box::new(add_contents))
    }

    fn allocate_ui_with_layout_dyn<'c, R>(
        &mut self,
        desired_size: Vec2,
        layout: Layout,
        add_contents: Box<dyn FnOnce(&mut Self) -> R + 'c>,
    ) -> InnerResponse<R> {
        crate::egui_assert!(desired_size.x >= 0.0 && desired_size.y >= 0.0);
        let item_spacing = self.spacing().item_spacing;
        let frame_rect = self.placer.next_space(desired_size, item_spacing);
        let child_rect = self.placer.justify_and_align(frame_rect, desired_size);

        let mut child_ui = self.child_ui(child_rect, layout);
        let ret = add_contents(&mut child_ui);
        let final_child_rect = child_ui.min_rect();

        self.placer
            .advance_after_rects(final_child_rect, final_child_rect, item_spacing);

        if self.style().debug.debug_on_hover && self.rect_contains_pointer(final_child_rect) {
            let painter = self.ctx().debug_painter();
            painter.rect_stroke(frame_rect, 4.0, (1.0, Color32::LIGHT_BLUE));
            painter.rect_stroke(final_child_rect, 4.0, (1.0, Color32::LIGHT_BLUE));
            self.placer.debug_paint_cursor(&painter, "next");
        }

        let response = self.interact(final_child_rect, child_ui.id, Sense::hover());
        InnerResponse::new(ret, response)
    }

    /// Allocated the given rectangle and then adds content to that rectangle.
    /// If the contents overflow, more space will be allocated.
    /// When finished, the amount of space actually used (`min_rect`) will be allocated.
    /// So you can request a lot of space and then use less.
    pub fn allocate_ui_at_rect<R>(
        &mut self,
        max_rect: Rect,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R> {
        let mut child_ui = self.child_ui(max_rect, *self.layout());
        let ret = add_contents(&mut child_ui);
        let final_child_rect = child_ui.min_rect();

        self.placer.advance_after_rects(
            final_child_rect,
            final_child_rect,
            self.spacing().item_spacing,
        );

        let response = self.interact(final_child_rect, child_ui.id, Sense::hover());
        InnerResponse::new(ret, response)
    }

    /// Convenience function to get a region to paint on.
    ///
    /// Note that egui uses screen coordinates for everything.
    ///
    /// ```
    /// # use egui::*;
    /// # let mut ui = &mut egui::Ui::__test();
    /// # use std::f32::consts::TAU;
    /// let size = Vec2::splat(16.0);
    /// let (response, painter) = ui.allocate_painter(size, Sense::hover());
    /// let rect = response.rect;
    /// let c = rect.center();
    /// let r = rect.width() / 2.0 - 1.0;
    /// let color = Color32::from_gray(128);
    /// let stroke = Stroke::new(1.0, color);
    /// painter.circle_stroke(c, r, stroke);
    /// painter.line_segment([c - vec2(0.0, r), c + vec2(0.0, r)], stroke);
    /// painter.line_segment([c, c + r * Vec2::angled(TAU * 1.0 / 8.0)], stroke);
    /// painter.line_segment([c, c + r * Vec2::angled(TAU * 3.0 / 8.0)], stroke);
    /// ```
    pub fn allocate_painter(&mut self, desired_size: Vec2, sense: Sense) -> (Response, Painter) {
        let response = self.allocate_response(desired_size, sense);
        let clip_rect = self.clip_rect().intersect(response.rect); // Make sure we don't paint out of bounds
        let painter = Painter::new(self.ctx().clone(), self.layer_id(), clip_rect);
        (response, painter)
    }

    /// Move the scroll to this cursor position with the specified alignment.
    ///
    /// ```
    /// # use egui::Align;
    /// # let mut ui = &mut egui::Ui::__test();
    /// egui::ScrollArea::auto_sized().show(ui, |ui| {
    ///     let scroll_bottom = ui.button("Scroll to bottom.").clicked();
    ///     for i in 0..1000 {
    ///         ui.label(format!("Item {}", i));
    ///     }
    ///
    ///     if scroll_bottom {
    ///         ui.scroll_to_cursor(Align::BOTTOM);
    ///     }
    /// });
    /// ```
    pub fn scroll_to_cursor(&mut self, align: Align) {
        let target_y = self.next_widget_position().y;
        self.ctx().frame_state().scroll_target = Some((target_y, align));
    }
}

/// # Adding widgets
impl Ui {
    /// Add a [`Widget`] to this `Ui` at a location dependent on the current [`Layout`].
    ///
    /// The returned [`Response`] can be used to check for interactions,
    /// as well as adding tooltips using [`Response::on_hover_text`].
    ///
    /// See also [`Self::add_sized`] and [`Self::put`].
    ///
    /// ```
    /// # let mut ui = egui::Ui::__test();
    /// # let mut my_value = 42;
    /// let response = ui.add(egui::Slider::new(&mut my_value, 0..=100));
    /// response.on_hover_text("Drag me!");
    /// ```
    #[inline(always)]
    pub fn add(&mut self, widget: impl Widget) -> Response {
        widget.ui(self)
    }

    /// Add a [`Widget`] to this `Ui` with a given size.
    /// The widget will attempt to fit within the given size, but some widgets may overflow.
    ///
    /// To fill all remaining area, use `ui.add_sized(ui.available_size(), widget);`
    ///
    /// See also [`Self::add`] and [`Self::put`].
    ///
    /// ```
    /// # let mut ui = egui::Ui::__test();
    /// # let mut my_value = 42;
    /// ui.add_sized([40.0, 20.0], egui::DragValue::new(&mut my_value));
    /// ```
    pub fn add_sized(&mut self, max_size: impl Into<Vec2>, widget: impl Widget) -> Response {
        // TODO: configure to overflow to main_dir instead of centered overflow
        // to handle the bug mentioned at https://github.com/emilk/egui/discussions/318#discussioncomment-627578
        // and fixed in https://github.com/emilk/egui/commit/035166276322b3f2324bd8b97ffcedc63fa8419f
        //
        // Make sure we keep the same main direction since it changes e.g. how text is wrapped:
        let layout = Layout::centered_and_justified(self.layout().main_dir());
        self.allocate_ui_with_layout(max_size.into(), layout, |ui| ui.add(widget))
            .inner
    }

    /// Add a [`Widget`] to this `Ui` at a specific location (manual layout).
    ///
    /// See also [`Self::add`] and [`Self::add_sized`].
    pub fn put(&mut self, max_rect: Rect, widget: impl Widget) -> Response {
        self.allocate_ui_at_rect(max_rect, |ui| {
            ui.centered_and_justified(|ui| ui.add(widget)).inner
        })
        .inner
    }

    /// Add extra space before the next widget.
    ///
    /// The direction is dependent on the layout.
    /// This will be in addition to the [`Spacing::item_spacing`}.
    ///
    /// [`Self::min_rect`] will expand to contain the space.
    #[inline(always)]
    pub fn add_space(&mut self, amount: f32) {
        self.placer.advance_cursor(amount);
    }

    #[deprecated = "Use add_space instead"]
    pub fn advance_cursor(&mut self, amount: f32) {
        self.add_space(amount);
    }

    /// Shortcut for `add(Label::new(text))`
    ///
    /// See also [`Label`].
    #[inline(always)]
    pub fn label(&mut self, label: impl Into<Label>) -> Response {
        label.into().ui(self)
    }

    /// Shortcut for `add(Label::new(text).text_color(color))`
    pub fn colored_label(
        &mut self,
        color: impl Into<Color32>,
        label: impl Into<Label>,
    ) -> Response {
        label.into().text_color(color).ui(self)
    }

    /// Shortcut for `add(Label::new(text).heading())`
    pub fn heading(&mut self, label: impl Into<Label>) -> Response {
        label.into().heading().ui(self)
    }

    /// Shortcut for `add(Label::new(text).monospace())`
    pub fn monospace(&mut self, label: impl Into<Label>) -> Response {
        label.into().monospace().ui(self)
    }

    /// Show text as monospace with a gray background.
    ///
    /// Shortcut for `add(Label::new(text).code())`
    pub fn code(&mut self, label: impl Into<Label>) -> Response {
        label.into().code().ui(self)
    }

    /// Shortcut for `add(Label::new(text).small())`
    pub fn small(&mut self, label: impl Into<Label>) -> Response {
        label.into().small().ui(self)
    }

    /// Shortcut for `add(Hyperlink::new(url))`
    ///
    /// See also [`Hyperlink`].
    pub fn hyperlink(&mut self, url: impl ToString) -> Response {
        Hyperlink::new(url).ui(self)
    }

    /// Shortcut for `add(Hyperlink::new(url).text(label))`
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.hyperlink_to("egui on GitHub", "https://www.github.com/emilk/egui/");
    /// ```
    ///
    /// See also [`Hyperlink`].
    pub fn hyperlink_to(&mut self, label: impl ToString, url: impl ToString) -> Response {
        Hyperlink::new(url).text(label).ui(self)
    }

    #[deprecated = "Use `text_edit_singleline` or `text_edit_multiline`"]
    pub fn text_edit(&mut self, text: &mut String) -> Response {
        self.text_edit_multiline(text)
    }

    /// No newlines (`\n`) allowed. Pressing enter key will result in the `TextEdit` losing focus (`response.lost_focus`).
    ///
    /// See also [`TextEdit`].
    pub fn text_edit_singleline<S: widgets::text_edit::TextBuffer>(
        &mut self,
        text: &mut S,
    ) -> Response {
        TextEdit::singleline(text).ui(self)
    }

    /// A `TextEdit` for multiple lines. Pressing enter key will create a new line.
    ///
    /// See also [`TextEdit`].
    pub fn text_edit_multiline<S: widgets::text_edit::TextBuffer>(
        &mut self,
        text: &mut S,
    ) -> Response {
        TextEdit::multiline(text).ui(self)
    }

    /// A `TextEdit` for code editing.
    ///
    /// This will be multiline, monospace, and will insert tabs instead of moving focus.
    ///
    /// See also [`TextEdit::code_editor`].
    pub fn code_editor<S: widgets::text_edit::TextBuffer>(&mut self, text: &mut S) -> Response {
        self.add(TextEdit::multiline(text).code_editor())
    }

    /// Usage: `if ui.button("Click me").clicked() { … }`
    ///
    /// Shortcut for `add(Button::new(text))`
    ///
    /// See also [`Button`].
    #[must_use = "You should check if the user clicked this with `if ui.button(…).clicked() { … } "]
    #[inline(always)]
    pub fn button(&mut self, text: impl ToString) -> Response {
        Button::new(text).ui(self)
    }

    /// A button as small as normal body text.
    ///
    /// Usage: `if ui.small_button("Click me").clicked() { … }`
    ///
    /// Shortcut for `add(Button::new(text).small())`
    #[must_use = "You should check if the user clicked this with `if ui.small_button(…).clicked() { … } "]
    pub fn small_button(&mut self, text: impl ToString) -> Response {
        Button::new(text).small().ui(self)
    }

    /// Show a checkbox.
    pub fn checkbox(&mut self, checked: &mut bool, text: impl ToString) -> Response {
        Checkbox::new(checked, text).ui(self)
    }

    /// Show a [`RadioButton`].
    /// Often you want to use [`Self::radio_value`] instead.
    #[must_use = "You should check if the user clicked this with `if ui.radio(…).clicked() { … } "]
    pub fn radio(&mut self, selected: bool, text: impl ToString) -> Response {
        RadioButton::new(selected, text).ui(self)
    }

    /// Show a [`RadioButton`]. It is selected if `*current_value == selected_value`.
    /// If clicked, `selected_value` is assigned to `*current_value`.
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    ///
    /// #[derive(PartialEq)]
    /// enum Enum { First, Second, Third }
    /// let mut my_enum = Enum::First;
    ///
    /// ui.radio_value(&mut my_enum, Enum::First, "First");
    ///
    /// // is equivalent to:
    ///
    /// if ui.add(egui::RadioButton::new(my_enum == Enum::First, "First")).clicked() {
    ///     my_enum = Enum::First
    /// }
    pub fn radio_value<Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        selected_value: Value,
        text: impl ToString,
    ) -> Response {
        let mut response = self.radio(*current_value == selected_value, text);
        if response.clicked() {
            *current_value = selected_value;
            response.mark_changed();
        }
        response
    }

    /// Show a label which can be selected or not.
    ///
    /// See also [`SelectableLabel`].
    #[must_use = "You should check if the user clicked this with `if ui.selectable_label(…).clicked() { … } "]
    pub fn selectable_label(&mut self, checked: bool, text: impl ToString) -> Response {
        SelectableLabel::new(checked, text).ui(self)
    }

    /// Show selectable text. It is selected if `*current_value == selected_value`.
    /// If clicked, `selected_value` is assigned to `*current_value`.
    ///
    /// Example: `ui.selectable_value(&mut my_enum, Enum::Alternative, "Alternative")`.
    ///
    /// See also [`SelectableLabel`].
    pub fn selectable_value<Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        selected_value: Value,
        text: impl ToString,
    ) -> Response {
        let mut response = self.selectable_label(*current_value == selected_value, text);
        if response.clicked() {
            *current_value = selected_value;
            response.mark_changed();
        }
        response
    }

    /// Shortcut for `add(Separator::default())` (see [`Separator`]).
    #[inline(always)]
    pub fn separator(&mut self) -> Response {
        Separator::default().ui(self)
    }

    /// Modify an angle. The given angle should be in radians, but is shown to the user in degrees.
    /// The angle is NOT wrapped, so the user may select, for instance 720° = 2𝞃 = 4π
    pub fn drag_angle(&mut self, radians: &mut f32) -> Response {
        let mut degrees = radians.to_degrees();
        let mut response = self.add(DragValue::new(&mut degrees).speed(1.0).suffix("°"));

        // only touch `*radians` if we actually changed the degree value
        if degrees != radians.to_degrees() {
            *radians = degrees.to_radians();
            response.changed = true;
        }

        response
    }

    /// Modify an angle. The given angle should be in radians,
    /// but is shown to the user in fractions of one Tau (i.e. fractions of one turn).
    /// The angle is NOT wrapped, so the user may select, for instance 2𝞃 (720°)
    pub fn drag_angle_tau(&mut self, radians: &mut f32) -> Response {
        use std::f32::consts::TAU;

        let mut taus = *radians / TAU;
        let mut response = self
            .add(DragValue::new(&mut taus).speed(0.01).suffix("τ"))
            .on_hover_text("1τ = one turn, 0.5τ = half a turn, etc. 0.25τ = 90°");

        // only touch `*radians` if we actually changed the value
        if taus != *radians / TAU {
            *radians = taus * TAU;
            response.changed = true;
        }

        response
    }

    /// Show an image here with the given size.
    ///
    /// See also [`Image`].
    #[inline(always)]
    pub fn image(&mut self, texture_id: TextureId, size: impl Into<Vec2>) -> Response {
        Image::new(texture_id, size).ui(self)
    }
}

/// # Colors
impl Ui {
    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    pub fn color_edit_button_srgba(&mut self, srgba: &mut Color32) -> Response {
        color_picker::color_edit_button_srgba(self, srgba, color_picker::Alpha::BlendOrAdditive)
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    pub fn color_edit_button_hsva(&mut self, hsva: &mut Hsva) -> Response {
        color_picker::color_edit_button_hsva(self, hsva, color_picker::Alpha::BlendOrAdditive)
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in `sRGB` space.
    pub fn color_edit_button_srgb(&mut self, srgb: &mut [u8; 3]) -> Response {
        let mut hsva = Hsva::from_srgb(*srgb);
        let response =
            color_picker::color_edit_button_hsva(self, &mut hsva, color_picker::Alpha::Opaque);
        *srgb = hsva.to_srgb();
        response
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in linear RGB space.
    pub fn color_edit_button_rgb(&mut self, rgb: &mut [f32; 3]) -> Response {
        let mut hsva = Hsva::from_rgb(*rgb);
        let response =
            color_picker::color_edit_button_hsva(self, &mut hsva, color_picker::Alpha::Opaque);
        *rgb = hsva.to_rgb();
        response
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in `sRGBA` space with premultiplied alpha
    pub fn color_edit_button_srgba_premultiplied(&mut self, srgba: &mut [u8; 4]) -> Response {
        let mut color = Color32::from_rgba_premultiplied(srgba[0], srgba[1], srgba[2], srgba[3]);
        let response = self.color_edit_button_srgba(&mut color);
        *srgba = color.to_array();
        response
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in `sRGBA` space without premultiplied alpha.
    /// If unsure, what "premultiplied alpha" is, then this is probably the function you want to use.
    pub fn color_edit_button_srgba_unmultiplied(&mut self, srgba: &mut [u8; 4]) -> Response {
        let mut hsva = Hsva::from_srgba_unmultiplied(*srgba);
        let response =
            color_picker::color_edit_button_hsva(self, &mut hsva, color_picker::Alpha::OnlyBlend);
        *srgba = hsva.to_srgba_unmultiplied();
        response
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in linear RGBA space with premultiplied alpha
    pub fn color_edit_button_rgba_premultiplied(&mut self, rgba: &mut [f32; 4]) -> Response {
        let mut hsva = Hsva::from_rgba_premultiplied(*rgba);
        let response = color_picker::color_edit_button_hsva(
            self,
            &mut hsva,
            color_picker::Alpha::BlendOrAdditive,
        );
        *rgba = hsva.to_rgba_premultiplied();
        response
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in linear RGBA space without premultiplied alpha.
    /// If unsure, what "premultiplied alpha" is, then this is probably the function you want to use.
    pub fn color_edit_button_rgba_unmultiplied(&mut self, rgba: &mut [f32; 4]) -> Response {
        let mut hsva = Hsva::from_rgba_unmultiplied(*rgba);
        let response =
            color_picker::color_edit_button_hsva(self, &mut hsva, color_picker::Alpha::OnlyBlend);
        *rgba = hsva.to_rgba_unmultiplied();
        response
    }
}

/// # Adding Containers / Sub-uis:
impl Ui {
    /// Put into a [`Frame::group`], visually grouping the contents together
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.group(|ui|{
    ///     ui.label("Within a frame");
    /// });
    /// ```
    ///
    /// Se also [`Self::scope`].
    pub fn group<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        crate::Frame::group(self.style()).show(self, add_contents)
    }

    /// Create a scoped child ui.
    ///
    /// You can use this to temporarily change the [`Style`] of a sub-region, for instance:
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.scope(|ui|{
    ///     ui.spacing_mut().slider_width = 200.0; // Temporary change
    ///     // …
    /// });
    /// ```
    pub fn scope<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let child_rect = self.available_rect_before_wrap();
        let next_auto_id_source = self.next_auto_id_source;
        let mut child_ui = self.child_ui(child_rect, *self.layout());
        self.next_auto_id_source = next_auto_id_source; // HACK: we want `scope` to only increment this once, so that `ui.scope` is equivalent to `ui.allocate_space`.
        let ret = add_contents(&mut child_ui);
        let response = self.allocate_rect(child_ui.min_rect(), Sense::hover());
        InnerResponse::new(ret, response)
    }

    #[deprecated = "Renamed scope()"]
    pub fn wrap<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.scope(add_contents)
    }

    /// Redirect shapes to another paint layer.
    pub fn with_layer_id<R>(
        &mut self,
        layer_id: LayerId,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R> {
        self.scope(|ui| {
            ui.painter.set_layer_id(layer_id);
            add_contents(ui)
        })
    }

    #[deprecated = "Use `ui.allocate_ui` instead"]
    pub fn add_custom_contents(
        &mut self,
        desired_size: Vec2,
        add_contents: impl FnOnce(&mut Ui),
    ) -> Rect {
        self.allocate_ui(desired_size, add_contents).response.rect
    }

    /// A [`CollapsingHeader`] that starts out collapsed.
    pub fn collapsing<R>(
        &mut self,
        heading: impl ToString,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        CollapsingHeader::new(heading).show(self, add_contents)
    }

    /// Create a child ui which is indented to the right.
    ///
    /// The `id_source` here be anything at all.
    // TODO: remove `id_source` argument?
    #[inline(always)]
    pub fn indent<R>(
        &mut self,
        id_source: impl Hash,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.indent_dyn(id_source, Box::new(add_contents))
    }

    fn indent_dyn<'c, R>(
        &mut self,
        id_source: impl Hash,
        add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
    ) -> InnerResponse<R> {
        assert!(
            self.layout().is_vertical(),
            "You can only indent vertical layouts, found {:?}",
            self.layout()
        );

        let indent = self.spacing().indent;
        let mut child_rect = self.placer.available_rect_before_wrap();
        child_rect.min.x += indent;

        let mut child_ui = Self {
            id: self.id.with(id_source),
            ..self.child_ui(child_rect, *self.layout())
        };
        let ret = add_contents(&mut child_ui);

        let end_with_horizontal_line = self.spacing().indent_ends_with_horizontal_line;

        if end_with_horizontal_line {
            child_ui.add_space(4.0);
        }

        // draw a faint line on the left to mark the indented section
        let stroke = self.visuals().widgets.noninteractive.bg_stroke;
        let left_top = child_rect.min - 0.5 * indent * Vec2::X;
        let left_top = self.painter().round_pos_to_pixels(left_top);
        let left_bottom = pos2(left_top.x, child_ui.min_rect().bottom() - 2.0);
        let left_bottom = self.painter().round_pos_to_pixels(left_bottom);
        self.painter.line_segment([left_top, left_bottom], stroke);
        if end_with_horizontal_line {
            let fudge = 2.0; // looks nicer with button rounding in collapsing headers
            let right_bottom = pos2(child_ui.min_rect().right() - fudge, left_bottom.y);
            self.painter
                .line_segment([left_bottom, right_bottom], stroke);
        }

        let response = self.allocate_rect(child_ui.min_rect(), Sense::hover());
        InnerResponse::new(ret, response)
    }

    /// Start a ui with horizontal layout.
    /// After you have called this, the function registers the contents as any other widget.
    ///
    /// Elements will be centered on the Y axis, i.e.
    /// adjusted up and down to lie in the center of the horizontal layout.
    /// The initial height is `style.spacing.interact_size.y`.
    /// Centering is almost always what you want if you are
    /// planning to to mix widgets or use different types of text.
    ///
    /// The returned `Response` will only have checked for mouse hover
    /// but can be used for tooltips (`on_hover_text`).
    /// It also contains the `Rect` used by the horizontal layout.
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.horizontal(|ui|{
    ///     ui.label("Same");
    ///     ui.label("row");
    /// });
    /// ```
    #[inline(always)]
    pub fn horizontal<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.horizontal_with_main_wrap(false, add_contents)
    }

    /// Like `horizontal`, but will set up the spacing to match that of a normal label.
    ///
    /// In particular, the space between widgets is the same width as the space character.
    ///
    /// You can still add any widgets to the layout (not only Labels).
    #[deprecated = "Use horizontal instead and set the desired spacing manually with `ui.spacing_mut().item_spacing`"]
    pub fn horizontal_for_text<R>(
        &mut self,
        text_style: TextStyle,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.scope(|ui| {
            let row_height = ui.fonts().row_height(text_style);
            let space_width = ui.fonts().glyph_width(text_style, ' ');
            let spacing = ui.spacing_mut();
            spacing.interact_size.y = row_height;
            spacing.item_spacing.x = space_width;
            spacing.item_spacing.y = 0.0;
            ui.horizontal(add_contents).inner
        })
    }

    /// Start a ui with horizontal layout that wraps to a new row
    /// when it reaches the right edge of the `max_size`.
    /// After you have called this, the function registers the contents as any other widget.
    ///
    /// Elements will be centered on the Y axis, i.e.
    /// adjusted up and down to lie in the center of the horizontal layout.
    /// The initial height is `style.spacing.interact_size.y`.
    /// Centering is almost always what you want if you are
    /// planning to to mix widgets or use different types of text.
    ///
    /// The returned `Response` will only have checked for mouse hover
    /// but can be used for tooltips (`on_hover_text`).
    /// It also contains the `Rect` used by the horizontal layout.
    pub fn horizontal_wrapped<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.horizontal_with_main_wrap(true, add_contents)
    }

    /// Like `horizontal_wrapped`, but will set up the spacing and
    /// line size to match that of a normal label.
    ///
    /// In particular, the space between widgets is the same width as the space character
    /// and the line spacing is the same as that for text.
    ///
    /// You can still add any widgets to the layout (not only Labels).
    #[deprecated = "Use horizontal_wrapped instead and set the desired spacing manually with `ui.spacing_mut().item_spacing`"]
    pub fn horizontal_wrapped_for_text<R>(
        &mut self,
        text_style: TextStyle,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.scope(|ui| {
            let row_height = ui.fonts().row_height(text_style);
            let space_width = ui.fonts().glyph_width(text_style, ' ');
            let spacing = ui.spacing_mut();
            spacing.interact_size.y = row_height;
            spacing.item_spacing.x = space_width;
            spacing.item_spacing.y = 0.0;
            ui.horizontal_wrapped(add_contents).inner
        })
    }

    #[inline(always)]
    fn horizontal_with_main_wrap<R>(
        &mut self,
        main_wrap: bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.horizontal_with_main_wrap_dyn(main_wrap, Box::new(add_contents))
    }

    fn horizontal_with_main_wrap_dyn<'c, R>(
        &mut self,
        main_wrap: bool,
        add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
    ) -> InnerResponse<R> {
        let initial_size = vec2(
            self.available_size_before_wrap_finite().x,
            self.spacing().interact_size.y, // Assume there will be something interactive on the horizontal layout
        );

        let layout = if self.placer.prefer_right_to_left() {
            Layout::right_to_left()
        } else {
            Layout::left_to_right()
        }
        .with_main_wrap(main_wrap);

        self.allocate_ui_with_layout_dyn(initial_size, layout, add_contents)
    }

    /// Start a ui with vertical layout.
    /// Widgets will be left-justified.
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.vertical(|ui|{
    ///     ui.label("over");
    ///     ui.label("under");
    /// });
    /// ```
    #[inline(always)]
    pub fn vertical<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.with_layout(Layout::top_down(Align::Min), add_contents)
    }

    /// Start a ui with vertical layout.
    /// Widgets will be horizontally centered.
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.vertical_centered(|ui|{
    ///     ui.label("over");
    ///     ui.label("under");
    /// });
    /// ```
    pub fn vertical_centered<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.with_layout(Layout::top_down(Align::Center), add_contents)
    }

    /// Start a ui with vertical layout.
    /// Widgets will be horizontally centered and justified (fill full width).
    ///
    /// ```
    /// # let ui = &mut egui::Ui::__test();
    /// ui.vertical_centered_justified(|ui|{
    ///     ui.label("over");
    ///     ui.label("under");
    /// });
    /// ```
    pub fn vertical_centered_justified<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.with_layout(
            Layout::top_down(Align::Center).with_cross_justify(true),
            add_contents,
        )
    }

    /// The new layout will take up all available space.
    ///
    /// Consider using [`Self::allocate_ui_with_layout`] instead,
    /// or the helpers [`Self::horizontal]`, [`Self::vertical`], etc.
    pub fn with_layout<R>(
        &mut self,
        layout: Layout,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R> {
        self.with_layout_dyn(layout, Box::new(add_contents))
    }

    fn with_layout_dyn<'c, R>(
        &mut self,
        layout: Layout,
        add_contents: Box<dyn FnOnce(&mut Self) -> R + 'c>,
    ) -> InnerResponse<R> {
        let mut child_ui = self.child_ui(self.available_rect_before_wrap(), layout);
        let inner = add_contents(&mut child_ui);
        let rect = child_ui.min_rect();
        let item_spacing = self.spacing().item_spacing;
        self.placer.advance_after_rects(rect, rect, item_spacing);

        if self.style().debug.debug_on_hover && self.rect_contains_pointer(rect) {
            let painter = self.ctx().debug_painter();
            painter.rect_stroke(rect, 4.0, (1.0, Color32::LIGHT_BLUE));
            self.placer.debug_paint_cursor(&painter, "next");
        }

        InnerResponse::new(inner, self.interact(rect, child_ui.id, Sense::hover()))
    }

    /// This will make the next added widget centered and justified in the available space.
    pub fn centered_and_justified<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R> {
        self.with_layout(
            Layout::centered_and_justified(Direction::TopDown),
            add_contents,
        )
    }

    pub(crate) fn set_grid(&mut self, grid: grid::GridLayout) {
        self.placer.set_grid(grid);
    }

    pub(crate) fn save_grid(&mut self) {
        self.placer.save_grid();
    }

    pub(crate) fn is_grid(&self) -> bool {
        self.placer.is_grid()
    }

    pub(crate) fn grid(&self) -> Option<&grid::GridLayout> {
        self.placer.grid()
    }

    /// Move to the next row in a grid layout or wrapping layout.
    /// Otherwise does nothing.
    pub fn end_row(&mut self) {
        self.placer
            .end_row(self.spacing().item_spacing, &self.painter().clone());
    }

    /// Set row height in horizontal wrapping layout.
    pub fn set_row_height(&mut self, height: f32) {
        self.placer.set_row_height(height);
    }

    /// Temporarily split split an Ui into several columns.
    ///
    /// ```
    /// # let mut ui = egui::Ui::__test();
    /// ui.columns(2, |columns| {
    ///     columns[0].label("First column");
    ///     columns[1].label("Second column");
    /// });
    /// ```
    pub fn columns<F, R>(&mut self, num_columns: usize, add_contents: F) -> R
    where
        F: FnOnce(&mut [Self]) -> R,
    {
        // TODO: ensure there is space
        let spacing = self.spacing().item_spacing.x;
        let total_spacing = spacing * (num_columns as f32 - 1.0);
        let column_width = (self.available_width() - total_spacing) / (num_columns as f32);
        let top_left = self.cursor().min;

        let mut columns: Vec<Self> = (0..num_columns)
            .map(|col_idx| {
                let pos = top_left + vec2((col_idx as f32) * (column_width + spacing), 0.0);
                let child_rect = Rect::from_min_max(
                    pos,
                    pos2(pos.x + column_width, self.max_rect().right_bottom().y),
                );
                let mut column_ui =
                    self.child_ui(child_rect, Layout::top_down_justified(Align::LEFT));
                column_ui.set_width(column_width);
                column_ui
            })
            .collect();

        let result = add_contents(&mut columns[..]);

        let mut max_column_width = column_width;
        let mut max_height = 0.0;
        for column in &columns {
            max_column_width = max_column_width.max(column.min_rect().width());
            max_height = column.min_size().y.max(max_height);
        }

        // Make sure we fit everything next frame:
        let total_required_width = total_spacing + max_column_width * (num_columns as f32);

        let size = vec2(self.available_width().max(total_required_width), max_height);
        self.advance_cursor_after_rect(Rect::from_min_size(top_left, size));
        result
    }
}

// ----------------------------------------------------------------------------

/// # Debug stuff
impl Ui {
    /// Shows where the next widget is going to be placed
    pub fn debug_paint_cursor(&self) {
        self.placer.debug_paint_cursor(&self.painter, "next");
    }

    /// Shows the given text where the next widget is to be placed
    /// if when [`Context::set_debug_on_hover`] has been turned on and the mouse is hovering the Ui.
    pub fn trace_location(&self, text: impl ToString) {
        let rect = self.max_rect_finite();
        if self.style().debug.debug_on_hover && self.rect_contains_pointer(rect) {
            self.placer
                .debug_paint_cursor(&self.ctx().debug_painter(), text);
        }
    }
}
