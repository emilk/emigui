use crate::{
    emath::{lerp, Align, Pos2, Rect, Vec2},
    CursorIcon, PointerButton, NUM_POINTER_BUTTONS,
};
use crate::{CtxRef, Id, LayerId, Sense, Ui};

// ----------------------------------------------------------------------------

/// The result of adding a widget to a [`Ui`].
///
/// A `Response` lets you know whether or not a widget is being hovered, clicked or dragged.
/// It also lets you easily show a tooltip on hover.
///
/// Whenever something gets added to a `Ui`, a `Response` object is returned.
/// [`ui.add`] returns a `Response`, as does [`ui.button`], and all similar shortcuts.
#[derive(Clone)]
pub struct Response {
    // CONTEXT:
    /// Used for optionally showing a tooltip and checking for more interactions.
    pub ctx: CtxRef,

    // IN:
    /// Which layer the widget is part of.
    pub layer_id: LayerId,

    /// The `Id` of the widget/area this response pertains.
    pub id: Id,

    /// The area of the screen we are talking about.
    pub rect: Rect,

    /// The senses (click and/or drag) that the widget was interested in (if any).
    pub sense: Sense,

    /// Was the widget enabled?
    /// If `false`, there was no interaction attempted (not even hover).
    pub(crate) enabled: bool,

    // OUT:
    /// The pointer is hovering above this widget or the widget was clicked/tapped this frame.
    pub(crate) hovered: bool,

    /// The pointer clicked this thing this frame.
    pub(crate) clicked: [bool; NUM_POINTER_BUTTONS],

    // TODO: `released` for sliders
    /// The thing was double-clicked.
    pub(crate) double_clicked: [bool; NUM_POINTER_BUTTONS],

    /// The widgets is being dragged
    pub(crate) dragged: bool,

    /// The widget was being dragged, but now it has been released.
    pub(crate) drag_released: bool,

    /// Is the pointer button currently down on this widget?
    /// This is true if the pointer is pressing down or dragging a widget
    pub(crate) is_pointer_button_down_on: bool,

    /// Where the pointer (mouse/touch) were when when this widget was clicked or dragged.
    /// `None` if the widget is not being interacted with.
    pub(crate) interact_pointer_pos: Option<Pos2>,

    /// What the underlying data changed?
    /// e.g. the slider was dragged, text was entered in a `TextEdit` etc.
    /// Always `false` for something like a `Button`.
    pub(crate) changed: bool,
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            ctx: _,
            layer_id,
            id,
            rect,
            sense,
            enabled,
            hovered,
            clicked,
            double_clicked,
            dragged,
            drag_released,
            is_pointer_button_down_on,
            interact_pointer_pos,
            changed,
        } = self;
        f.debug_struct("Response")
            .field("layer_id", layer_id)
            .field("id", id)
            .field("rect", rect)
            .field("sense", sense)
            .field("enabled", enabled)
            .field("hovered", hovered)
            .field("clicked", clicked)
            .field("double_clicked", double_clicked)
            .field("dragged", dragged)
            .field("drag_released", drag_released)
            .field("is_pointer_button_down_on", is_pointer_button_down_on)
            .field("interact_pointer_pos", interact_pointer_pos)
            .field("changed", changed)
            .finish()
    }
}

impl Response {
    /// Returns true if this widget was clicked this frame by the primary button.
    ///
    /// Note that the widget must be sensing clicks with [`Sense::click`].
    /// [`crate::Button`] senses clicks; [`crate::Label`] does not (unless you call [`crate::Label::sense`]).
    ///
    /// You can use [`Self::interact`] to sense more things *after* adding a widget.
    #[inline(always)]
    pub fn clicked(&self) -> bool {
        self.clicked[PointerButton::Primary as usize]
    }

    /// Returns true if this widget was clicked this frame by the given button.
    pub fn clicked_by(&self, button: PointerButton) -> bool {
        self.clicked[button as usize]
    }

    /// Returns true if this widget was clicked this frame by the secondary mouse button (e.g. the right mouse button).
    pub fn secondary_clicked(&self) -> bool {
        self.clicked[PointerButton::Secondary as usize]
    }

    /// Returns true if this widget was clicked this frame by the middle mouse button.
    pub fn middle_clicked(&self) -> bool {
        self.clicked[PointerButton::Middle as usize]
    }

    /// Returns true if this widget was double-clicked this frame by the primary button.
    pub fn double_clicked(&self) -> bool {
        self.double_clicked[PointerButton::Primary as usize]
    }

    /// Returns true if this widget was double-clicked this frame by the given button.
    pub fn double_clicked_by(&self, button: PointerButton) -> bool {
        self.double_clicked[button as usize]
    }

    /// `true` if there was a click *outside* this widget this frame.
    pub fn clicked_elsewhere(&self) -> bool {
        // We do not use self.clicked(), because we want to catch all click within our frame,
        // even if we aren't clickable. This is important for windows and such that should close
        // then the user clicks elsewhere.
        let pointer = &self.ctx.input().pointer;
        if let Some(pos) = pointer.interact_pos() {
            pointer.any_click() && !self.rect.contains(pos)
        } else {
            false
        }
    }

    /// Was the widget enabled?
    /// If false, there was no interaction attempted
    /// and the widget should be drawn in a gray disabled look.
    #[inline(always)]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// The pointer is hovering above this widget or the widget was clicked/tapped this frame.
    #[inline(always)]
    pub fn hovered(&self) -> bool {
        self.hovered
    }

    /// This widget has the keyboard focus (i.e. is receiving key presses).
    pub fn has_focus(&self) -> bool {
        self.ctx.memory().has_focus(self.id)
    }

    /// True if this widget has keyboard focus this frame, but didn't last frame.
    pub fn gained_focus(&self) -> bool {
        self.ctx.memory().gained_focus(self.id)
    }

    /// The widget had keyboard focus and lost it,
    /// either because the user pressed tab or clicked somewhere else,
    /// or (in case of a [`crate::TextEdit`]) because the user pressed enter.
    ///
    /// ```
    /// # let mut ui = egui::Ui::__test();
    /// # let mut my_text = String::new();
    /// # fn do_request(_: &str) {}
    /// let response = ui.text_edit_singleline(&mut my_text);
    /// if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
    ///     do_request(&my_text);
    /// }
    /// ```
    pub fn lost_focus(&self) -> bool {
        self.ctx.memory().lost_focus(self.id)
    }

    #[deprecated = "Renamed to lost_focus()"]
    pub fn lost_kb_focus(&self) -> bool {
        self.lost_focus()
    }

    /// Request that this widget get keyboard focus.
    pub fn request_focus(&self) {
        self.ctx.memory().request_focus(self.id)
    }

    /// Surrender keyboard focus for this widget.
    pub fn surrender_focus(&self) {
        self.ctx.memory().surrender_focus(self.id)
    }

    /// The widgets is being dragged.
    ///
    /// To find out which button(s), query [`crate::PointerState::button_down`]
    /// (`ui.input().pointer.button_down(…)`).
    ///
    /// Note that the widget must be sensing drags with [`Sense::drag`].
    /// [`crate::DragValue`] senses drags; [`crate::Label`] does not (unless you call [`crate::Label::sense`]).
    ///
    /// You can use [`Self::interact`] to sense more things *after* adding a widget.
    #[inline(always)]
    pub fn dragged(&self) -> bool {
        self.dragged
    }

    pub fn dragged_by(&self, button: PointerButton) -> bool {
        self.dragged() && self.ctx.input().pointer.button_down(button)
    }

    /// Did a drag on this widgets begin this frame?
    pub fn drag_started(&self) -> bool {
        self.dragged && self.ctx.input().pointer.any_pressed()
    }

    /// The widget was being dragged, but now it has been released.
    pub fn drag_released(&self) -> bool {
        self.drag_released
    }

    /// If dragged, how many points were we dragged and in what direction?
    pub fn drag_delta(&self) -> Vec2 {
        if self.dragged() {
            self.ctx.input().pointer.delta()
        } else {
            Vec2::ZERO
        }
    }

    /// Where the pointer (mouse/touch) were when when this widget was clicked or dragged.
    /// `None` if the widget is not being interacted with.
    pub fn interact_pointer_pos(&self) -> Option<Pos2> {
        self.interact_pointer_pos
    }

    /// If it is a good idea to show a tooltip, where is pointer?
    /// None if the pointer is outside the response area.
    pub fn hover_pos(&self) -> Option<Pos2> {
        if self.hovered() {
            self.ctx.input().pointer.hover_pos()
        } else {
            None
        }
    }

    /// Is the pointer button currently down on this widget?
    /// This is true if the pointer is pressing down or dragging a widget
    #[inline(always)]
    pub fn is_pointer_button_down_on(&self) -> bool {
        self.is_pointer_button_down_on
    }

    /// What the underlying data changed?
    ///
    /// e.g. the slider was dragged, text was entered in a `TextEdit` etc.
    /// Always `false` for something like a `Button`.
    ///
    /// Can sometimes be `true` even though the data didn't changed
    /// (e.g. if the user entered a character and erased it the same frame).
    ///
    /// This is not set if the *view* of the data was changed.
    /// For instance, moving the cursor in a `TextEdit` does not set this to `true`.
    #[inline(always)]
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// Report the data shown by this widget changed.
    ///
    /// This must be called by widgets that represent some mutable data,
    /// e.g. checkboxes, sliders etc.
    ///
    /// This should be called when the *content* changes, but not when the view does.
    /// So we call this when the text of a [`crate::TextEdit`], but not when the cursors changes.
    #[inline(always)]
    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    /// Show this UI if the widget was hovered (i.e. a tooltip).
    ///
    /// The text will not be visible if the widget is not enabled.
    /// If you call this multiple times the tooltips will stack underneath the previous ones.
    pub fn on_hover_ui(self, add_contents: impl FnOnce(&mut Ui)) -> Self {
        if self.should_show_hover_ui() {
            crate::containers::show_tooltip_under(
                &self.ctx,
                self.id.with("__tooltip"),
                &self.rect,
                add_contents,
            );
        }
        self
    }

    /// Show this UI when hovering if the widget is disabled.
    pub fn on_disabled_hover_ui(self, add_contents: impl FnOnce(&mut Ui)) -> Self {
        if !self.enabled && self.ctx.rect_contains_pointer(self.layer_id, self.rect) {
            crate::containers::show_tooltip_under(
                &self.ctx,
                self.id.with("__tooltip"),
                &self.rect,
                add_contents,
            );
        }
        self
    }

    /// Like `on_hover_ui`, but show the ui next to cursor.
    pub fn on_hover_ui_at_pointer(self, add_contents: impl FnOnce(&mut Ui)) -> Self {
        if self.should_show_hover_ui() {
            crate::containers::show_tooltip_at_pointer(
                &self.ctx,
                self.id.with("__tooltip"),
                add_contents,
            );
        }
        self
    }

    fn should_show_hover_ui(&self) -> bool {
        if self.ctx.memory().everything_is_visible() {
            true
        } else if self.hovered && self.ctx.input().pointer.has_pointer() {
            let show_tooltips_only_when_still =
                self.ctx.style().interaction.show_tooltips_only_when_still;
            if show_tooltips_only_when_still {
                if self.ctx.input().pointer.is_still() {
                    true
                } else {
                    // wait for mouse to stop
                    self.ctx.request_repaint();
                    false
                }
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Show this text if the widget was hovered (i.e. a tooltip).
    ///
    /// The text will not be visible if the widget is not enabled.
    /// If you call this multiple times the tooltips will stack underneath the previous ones.
    pub fn on_hover_text(self, text: impl ToString) -> Self {
        self.on_hover_ui(|ui| {
            ui.add(crate::widgets::Label::new(text));
        })
    }

    /// Show this text when hovering if the widget is disabled.
    pub fn on_disabled_hover_text(self, text: impl ToString) -> Self {
        self.on_disabled_hover_ui(|ui| {
            ui.add(crate::widgets::Label::new(text));
        })
    }

    #[deprecated = "Deprecated 2020-10-01: use `on_hover_text` instead."]
    pub fn tooltip_text(self, text: impl ToString) -> Self {
        self.on_hover_text(text)
    }

    /// When hovered, use this icon for the mouse cursor.
    pub fn on_hover_cursor(self, cursor: CursorIcon) -> Self {
        if self.hovered() {
            self.ctx.output().cursor_icon = cursor;
        }
        self
    }

    /// Check for more interactions (e.g. sense clicks on a `Response` returned from a label).
    ///
    /// Note that this call will not add any hover-effects to the widget, so when possible
    /// it is better to give the widget a `Sense` instead, e.g. using `[Label::sense]`.
    ///
    /// ```
    /// # let mut ui = egui::Ui::__test();
    /// let response = ui.label("hello");
    /// assert!(!response.clicked()); // labels don't sense clicks by default
    /// let response = response.interact(egui::Sense::click());
    /// if response.clicked() { /* … */ }
    /// ```
    pub fn interact(&self, sense: Sense) -> Self {
        self.ctx.interact_with_hovered(
            self.layer_id,
            self.id,
            self.rect,
            sense,
            self.enabled,
            self.hovered,
        )
    }

    /// Move the scroll to this UI with the specified alignment.
    ///
    /// ```
    /// # use egui::Align;
    /// # let mut ui = &mut egui::Ui::__test();
    /// egui::ScrollArea::auto_sized().show(ui, |ui| {
    ///     for i in 0..1000 {
    ///         let response = ui.button(format!("Button {}", i));
    ///         if response.clicked() {
    ///             response.scroll_to_me(Align::Center);
    ///         }
    ///     }
    /// });
    /// ```
    pub fn scroll_to_me(&self, align: Align) {
        let scroll_target = lerp(self.rect.y_range(), align.to_factor());
        self.ctx.frame_state().scroll_target = Some((scroll_target, align));
    }

    /// For accessibility.
    ///
    /// Call after interacting and potential calls to [`Self::mark_changed`].
    pub fn widget_info(&self, make_info: impl Fn() -> crate::WidgetInfo) {
        use crate::output::OutputEvent;
        let event = if self.clicked() {
            Some(OutputEvent::Clicked(make_info()))
        } else if self.double_clicked() {
            Some(OutputEvent::DoubleClicked(make_info()))
        } else if self.gained_focus() {
            Some(OutputEvent::FocusGained(make_info()))
        } else if self.changed {
            Some(OutputEvent::ValueChanged(make_info()))
        } else {
            None
        };
        if let Some(event) = event {
            self.ctx.output().events.push(event);
        }
    }

    pub fn context_menu(&self, add_contents: impl FnOnce(&mut Ui, &mut super::context_menu::MenuState)) -> &Self {
        self.ctx.show_context_menu(self, add_contents);
        self
    }
}

impl Response {
    /// A logical "or" operation.
    /// For instance `a.union(b).hovered` means "was either a or b hovered?".
    pub fn union(&self, other: Self) -> Self {
        assert!(self.ctx == other.ctx);
        crate::egui_assert!(
            self.layer_id == other.layer_id,
            "It makes no sense to combine Responses from two different layers"
        );
        Self {
            ctx: other.ctx,
            layer_id: self.layer_id,
            id: self.id,
            rect: self.rect.union(other.rect),
            sense: self.sense.union(other.sense),
            enabled: self.enabled || other.enabled,
            hovered: self.hovered || other.hovered,
            clicked: [
                self.clicked[0] || other.clicked[0],
                self.clicked[1] || other.clicked[1],
                self.clicked[2] || other.clicked[2],
            ],
            double_clicked: [
                self.double_clicked[0] || other.double_clicked[0],
                self.double_clicked[1] || other.double_clicked[1],
                self.double_clicked[2] || other.double_clicked[2],
            ],
            dragged: self.dragged || other.dragged,
            drag_released: self.drag_released || other.drag_released,
            is_pointer_button_down_on: self.is_pointer_button_down_on
                || other.is_pointer_button_down_on,
            interact_pointer_pos: self.interact_pointer_pos.or(other.interact_pointer_pos),
            changed: self.changed || other.changed,
        }
    }
}

/// To summarize the response from many widgets you can use this pattern:
///
/// ```
/// use egui::*;
/// fn draw_vec2(ui: &mut Ui, v: &mut Vec2) -> Response {
///     ui.add(DragValue::new(&mut v.x)) | ui.add(DragValue::new(&mut v.y))
/// }
/// ```
///
/// Now `draw_vec2(ui, foo).hovered` is true if either `DragValue` were hovered.
impl std::ops::BitOr for Response {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

/// To summarize the response from many widgets you can use this pattern:
///
/// ```
/// # let mut ui = egui::Ui::__test();
/// # let (widget_a, widget_b, widget_c) = (egui::Label::new("a"), egui::Label::new("b"), egui::Label::new("c"));
/// let mut response = ui.add(widget_a);
/// response |= ui.add(widget_b);
/// response |= ui.add(widget_c);
/// if response.hovered() { ui.label("You hovered at least one of the widgets"); }
/// ```
impl std::ops::BitOrAssign for Response {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.union(rhs);
    }
}

// ----------------------------------------------------------------------------

/// Returned when we wrap some ui-code and want to return both
/// the results of the inner function and the ui as a whole, e.g.:
///
/// ```
/// # let ui = &mut egui::Ui::__test();
/// let inner_resp = ui.horizontal(|ui| {
///     ui.label("Blah blah");
///     42
/// });
/// inner_resp.response.on_hover_text("You hovered the horizontal layout");
/// assert_eq!(inner_resp.inner, 42);
/// ```
#[derive(Debug)]
pub struct InnerResponse<R> {
    pub inner: R,
    pub response: Response,
}

impl<R> InnerResponse<R> {
    pub fn new(inner: R, response: Response) -> Self {
        Self { inner, response }
    }
}
