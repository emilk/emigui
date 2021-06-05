use crate::*;

/// Clickable button with text.
///
/// See also [`Ui::button`].
///
/// ```
/// # let ui = &mut egui::Ui::__test();
/// if ui.add(egui::Button::new("Click mew")).clicked() {
///     do_stuff();
/// }
/// # fn do_stuff() {}
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct Button {
    text: String,
    text_color: Option<Color32>,
    text_style: Option<TextStyle>,
    /// None means default for interact
    fill: Option<Color32>,
    sense: Sense,
    small: bool,
    frame: bool,
    wrap: Option<bool>,
    min_size: Vec2,
}

impl Button {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
            text_color: None,
            text_style: None,
            fill: Default::default(),
            sense: Sense::click(),
            small: false,
            frame: true,
            wrap: None,
            min_size: Vec2::ZERO,
        }
    }

    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn text_color_opt(mut self, text_color: Option<Color32>) -> Self {
        self.text_color = text_color;
        self
    }

    pub fn text_style(mut self, text_style: TextStyle) -> Self {
        self.text_style = Some(text_style);
        self
    }

    pub fn fill(mut self, fill: Option<Color32>) -> Self {
        self.fill = fill;
        self
    }

    /// Make this a small button, suitable for embedding into text.
    pub fn small(mut self) -> Self {
        self.text_style = Some(TextStyle::Body);
        self.small = true;
        self
    }

    /// Turn off the frame
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = frame;
        self
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    /// If you set this to `false`, the button will be grayed out and un-clickable.
    /// `enabled(false)` has the same effect as calling `sense(Sense::hover())`.
    ///
    /// This is a convenience for [`Ui::set_enabled`].
    pub fn enabled(mut self, enabled: bool) -> Self {
        if !enabled {
            self.sense = Sense::hover();
        }
        self
    }

    /// If `true`, the text will wrap at the `max_width`.
    /// By default [`Self::wrap`] will be true in vertical layouts
    /// and horizontal layouts with wrapping,
    /// and false on non-wrapping horizontal layouts.
    ///
    /// Note that any `\n` in the button text will always produce a new line.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub(crate) fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }
}

impl Button {
    fn enabled_ui(self, ui: &mut Ui) -> Response {
        let Button {
            text,
            text_color,
            text_style,
            fill,
            sense,
            small,
            frame,
            wrap,
            min_size,
        } = self;

        let text_style = text_style
            .or(ui.style().override_text_style)
            .unwrap_or(TextStyle::Button);

        let mut button_padding = ui.spacing().button_padding;
        if small {
            button_padding.y = 0.0;
        }
        let total_extra = button_padding + button_padding;

        let wrap = wrap.unwrap_or_else(|| ui.wrap_text());
        let galley = if wrap {
            ui.fonts()
                .layout_multiline(text_style, text, ui.available_width() - total_extra.x)
        } else {
            ui.fonts().layout_no_wrap(text_style, text)
        };

        let mut desired_size = galley.size + 2.0 * button_padding;
        if !small {
            desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        }
        desired_size = desired_size.at_least(min_size);

        let (rect, response) = ui.allocate_at_least(desired_size, sense);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, &galley.text));

        if ui.clip_rect().intersects(rect) {
            let visuals = ui.style().interact(&response);
            let text_pos = ui
                .layout()
                .align_size_within_rect(galley.size, rect.shrink2(button_padding))
                .min;

            if frame {
                let fill = fill.unwrap_or(visuals.bg_fill);
                ui.painter().rect(
                    rect.expand(visuals.expansion),
                    visuals.corner_radius,
                    fill,
                    visuals.bg_stroke,
                );
            }

            let text_color = text_color
                .or(ui.visuals().override_text_color)
                .unwrap_or_else(|| visuals.text_color());
            ui.painter().galley(text_pos, galley, text_color);
        }

        response
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let button_enabled = self.sense != Sense::hover();
        if button_enabled || !ui.enabled() {
            self.enabled_ui(ui)
        } else {
            // We need get a temporary disabled `Ui` to get that grayed out look:
            ui.scope(|ui| {
                ui.set_enabled(false);
                self.enabled_ui(ui)
            })
            .inner
        }
    }
}

// ----------------------------------------------------------------------------

// TODO: allow checkbox without a text label
/// Boolean on/off control with text label.
///
/// Usually you'd use [`Ui::checkbox`] instead.
///
/// ```
/// # let ui = &mut egui::Ui::__test();
/// # let mut my_bool = true;
/// // These are equivalent:
/// ui.checkbox(&mut my_bool, "Checked");
/// ui.add(egui::Checkbox::new(&mut my_bool, "Checked"));
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
#[derive(Debug)]
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    text: String,
    text_color: Option<Color32>,
    text_style: Option<TextStyle>,
    left_sided: bool,
}

impl<'a> Checkbox<'a> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(checked: &'a mut bool, text: impl ToString) -> Self {
        Checkbox {
            checked,
            text: text.to_string(),
            text_color: None,
            text_style: None,
            left_sided: false,
        }
    }

    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn text_style(mut self, text_style: TextStyle) -> Self {
        self.text_style = Some(text_style);
        self
    }

    /// If set to `true` the label will be displayed before the checkbox
    pub fn label_on_left(mut self, is_on_left: bool) -> Self {
        self.left_sided = is_on_left;
        self
    }
}

impl<'a> Widget for Checkbox<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Checkbox {
            checked,
            text,
            text_color,
            text_style,
            left_sided,
        } = self;

        let text_style = text_style
            .or(ui.style().override_text_style)
            .unwrap_or(TextStyle::Button);

        let spacing = &ui.spacing();
        let icon_width = spacing.icon_width;
        let icon_spacing = ui.spacing().icon_spacing;
        let button_padding = spacing.button_padding;
        let total_extra = button_padding + vec2(icon_width + icon_spacing, 0.0) + button_padding;

        let galley = if ui.wrap_text() {
            ui.fonts()
                .layout_multiline(text_style, text, ui.available_width() - total_extra.x)
        } else {
            ui.fonts().layout_no_wrap(text_style, text)
        };

        let mut desired_size = total_extra + galley.size;
        desired_size = desired_size.at_least(spacing.interact_size);
        desired_size.y = desired_size.y.max(icon_width);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        if response.clicked() {
            *checked = !*checked;
            response.mark_changed();
        }
        response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, *checked, &galley.text));

        // let visuals = ui.style().interact_selectable(&response, *checked); // too colorful
        let visuals = ui.style().interact(&response);
        let text_pos = if left_sided {
            pos2(rect.min.x, rect.center().y - 0.5 * galley.size.y)
        } else {
            pos2(
                rect.min.x + button_padding.x + icon_width + icon_spacing,
                rect.center().y - 0.5 * galley.size.y,
            )
        };

        let text_size = galley.size.x;

        let (mut small_icon_rect, mut big_icon_rect) = ui.spacing().icon_rectangles(rect);
        if left_sided {
            small_icon_rect = small_icon_rect.translate(vec2(text_size + button_padding.x, 0.));
            big_icon_rect = big_icon_rect.translate(vec2(text_size + button_padding.x, 0.));
        }

        ui.painter().add(Shape::Rect {
            rect: big_icon_rect.expand(visuals.expansion),
            corner_radius: visuals.corner_radius,
            fill: visuals.bg_fill,
            stroke: visuals.bg_stroke,
        });

        if *checked {
            // Check mark:
            ui.painter().add(Shape::line(
                vec![
                    pos2(small_icon_rect.left(), small_icon_rect.center().y),
                    pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
                    pos2(small_icon_rect.right(), small_icon_rect.top()),
                ],
                visuals.fg_stroke,
            ));
        }

        let text_color = text_color
            .or(ui.visuals().override_text_color)
            .unwrap_or_else(|| visuals.text_color());
        ui.painter().galley(text_pos, galley, text_color);
        response
    }
}

// ----------------------------------------------------------------------------

/// One out of several alternatives, either selected or not.
///
/// Usually you'd use [`Ui::radio_value`] or [`Ui::radio`] instead.
///
/// ```
/// # let ui = &mut egui::Ui::__test();
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
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
#[derive(Debug)]
pub struct RadioButton {
    checked: bool,
    text: String,
    text_color: Option<Color32>,
    text_style: Option<TextStyle>,
    left_sided: bool,
}

impl RadioButton {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(checked: bool, text: impl ToString) -> Self {
        Self {
            checked,
            text: text.to_string(),
            text_color: None,
            text_style: None,
            left_sided: false,
        }
    }

    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn text_style(mut self, text_style: TextStyle) -> Self {
        self.text_style = Some(text_style);
        self
    }

    /// If set to `true` the label will be displayed before the checkbox
    pub fn label_on_left(mut self, is_on_left: bool) -> Self {
        self.left_sided = is_on_left;
        self
    }
}

impl Widget for RadioButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let RadioButton {
            checked,
            text,
            text_color,
            text_style,
            left_sided,
        } = self;

        let text_style = text_style
            .or(ui.style().override_text_style)
            .unwrap_or(TextStyle::Button);

        let icon_width = ui.spacing().icon_width;
        let icon_spacing = ui.spacing().icon_spacing;
        let button_padding = ui.spacing().button_padding;
        let total_extra = button_padding + vec2(icon_width + icon_spacing, 0.0) + button_padding;

        let galley = if ui.wrap_text() {
            ui.fonts()
                .layout_multiline(text_style, text, ui.available_width() - total_extra.x)
        } else {
            ui.fonts().layout_no_wrap(text_style, text)
        };
        let text_size = galley.size.x;

        let mut desired_size = total_extra + galley.size;
        desired_size = desired_size.at_least(ui.spacing().interact_size);
        desired_size.y = desired_size.y.max(icon_width);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
        response
            .widget_info(|| WidgetInfo::selected(WidgetType::RadioButton, checked, &galley.text));

        let text_pos = if left_sided {
            pos2(rect.min.x, rect.center().y - 0.5 * galley.size.y)
        } else {
            pos2(
                rect.min.x + button_padding.x + icon_width + icon_spacing,
                rect.center().y - 0.5 * galley.size.y,
            )
        };

        // let visuals = ui.style().interact_selectable(&response, checked); // too colorful
        let visuals = ui.style().interact(&response);

        let (mut small_icon_rect, mut big_icon_rect) = ui.spacing().icon_rectangles(rect);
        if left_sided {
            small_icon_rect = small_icon_rect.translate(vec2(text_size + button_padding.x, 0.));
            big_icon_rect = big_icon_rect.translate(vec2(text_size + button_padding.x, 0.));
        }

        let painter = ui.painter();

        painter.add(Shape::Circle {
            center: big_icon_rect.center(),
            radius: big_icon_rect.width() / 2.0 + visuals.expansion,
            fill: visuals.bg_fill,
            stroke: visuals.bg_stroke,
        });

        if checked {
            painter.add(Shape::Circle {
                center: small_icon_rect.center(),
                radius: small_icon_rect.width() / 3.0,
                fill: visuals.fg_stroke.color, // Intentional to use stroke and not fill
                // fill: ui.visuals().selection.stroke.color, // too much color
                stroke: Default::default(),
            });
        }

        let text_color = text_color
            .or(ui.visuals().override_text_color)
            .unwrap_or_else(|| visuals.text_color());
        painter.galley(text_pos, galley, text_color);
        response
    }
}

// ----------------------------------------------------------------------------

/// A clickable image within a frame.
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
#[derive(Clone, Debug)]
pub struct ImageButton {
    image: widgets::Image,
    sense: Sense,
    frame: bool,
    selected: bool,
}

impl ImageButton {
    pub fn new(texture_id: TextureId, size: impl Into<Vec2>) -> Self {
        Self {
            image: widgets::Image::new(texture_id, size),
            sense: Sense::click(),
            frame: true,
            selected: false,
        }
    }

    /// Select UV range. Default is (0,0) in top-left, (1,1) bottom right.
    pub fn uv(mut self, uv: impl Into<Rect>) -> Self {
        self.image = self.image.uv(uv);
        self
    }

    /// Multiply image color with this. Default is WHITE (no tint).
    pub fn tint(mut self, tint: impl Into<Color32>) -> Self {
        self.image = self.image.tint(tint);
        self
    }

    /// If `true`, mark this button as "selected".
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Turn off the frame
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = frame;
        self
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }
}

impl Widget for ImageButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            image,
            sense,
            frame,
            selected,
        } = self;

        let button_padding = ui.spacing().button_padding;
        let size = image.size() + 2.0 * button_padding;
        let (rect, response) = ui.allocate_exact_size(size, sense);
        response.widget_info(|| WidgetInfo::new(WidgetType::ImageButton));

        if ui.clip_rect().intersects(rect) {
            let visuals = ui.style().interact(&response);

            if selected {
                let selection = ui.visuals().selection;
                ui.painter()
                    .rect(rect, 0.0, selection.bg_fill, selection.stroke);
            } else if frame {
                ui.painter().rect(
                    rect.expand(visuals.expansion),
                    visuals.corner_radius,
                    visuals.bg_fill,
                    visuals.bg_stroke,
                );
            }

            let image_rect = ui
                .layout()
                .align_size_within_rect(image.size(), rect.shrink2(button_padding));
            image.paint_at(ui, image_rect);
        }

        response
    }
}
