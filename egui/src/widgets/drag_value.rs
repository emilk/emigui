#![allow(clippy::needless_pass_by_value)] // False positives with `impl ToString`

use std::ops::RangeInclusive;

use crate::*;

// ----------------------------------------------------------------------------

/// Same state for all [`DragValue`]s.
#[derive(Clone, Debug, Default)]
pub(crate) struct MonoState {
    last_dragged_id: Option<Id>,
    last_dragged_value: Option<f64>,
    /// For temporary edit of a `DragValue` value.
    /// Couples with the current focus id.
    edit_string: Option<String>,
}

impl MonoState {
    pub(crate) fn end_frame(&mut self, input: &InputState) {
        if input.pointer.any_pressed() || input.pointer.any_released() {
            self.last_dragged_id = None;
            self.last_dragged_value = None;
        }
    }
}

// ----------------------------------------------------------------------------

/// Combined into one function (rather than two) to make it easier
/// for the borrow checker.
type GetSetValue<'a> = Box<dyn 'a + FnMut(Option<f64>) -> f64>;

fn get(get_set_value: &mut GetSetValue<'_>) -> f64 {
    (get_set_value)(None)
}

fn set(get_set_value: &mut GetSetValue<'_>, value: f64) {
    (get_set_value)(Some(value));
}

/// A numeric value that you can change by dragging the number. More compact than a [`Slider`].
///
/// ```
/// # let ui = &mut egui::Ui::__test();
/// # let mut my_f32: f32 = 0.0;
/// ui.add(egui::DragValue::new(&mut my_f32).speed(0.1));
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct DragValue<'a> {
    get_set_value: GetSetValue<'a>,
    speed: f64,
    prefix: String,
    suffix: String,
    clamp_range: RangeInclusive<f64>,
    min_decimals: usize,
    max_decimals: Option<usize>,
}

macro_rules! impl_integer_constructor {
    ($int:ident) => {
        #[deprecated = "Use DragValue::new instead"]
        pub fn $int(value: &'a mut $int) -> Self {
            Self::from_get_set(move |v: Option<f64>| {
                if let Some(v) = v {
                    *value = v.round() as $int;
                }
                *value as f64
            })
            .max_decimals(0)
            .clamp_range($int::MIN..=$int::MAX)
            .speed(0.25)
        }
    };
}

impl<'a> DragValue<'a> {
    pub fn new<Num: emath::Numeric>(value: &'a mut Num) -> Self {
        let slf = Self::from_get_set(move |v: Option<f64>| {
            if let Some(v) = v {
                *value = Num::from_f64(v)
            }
            value.to_f64()
        });

        if Num::INTEGRAL {
            slf.max_decimals(0)
                .clamp_range(Num::MIN..=Num::MAX)
                .speed(0.25)
        } else {
            slf
        }
    }

    #[deprecated = "Use DragValue::new instead"]
    pub fn f32(value: &'a mut f32) -> Self {
        Self::from_get_set(move |v: Option<f64>| {
            if let Some(v) = v {
                *value = v as f32
            }
            *value as f64
        })
    }

    #[deprecated = "Use DragValue::new instead"]
    pub fn f64(value: &'a mut f64) -> Self {
        Self::from_get_set(move |v: Option<f64>| {
            if let Some(v) = v {
                *value = v
            }
            *value
        })
    }

    impl_integer_constructor!(i8);
    impl_integer_constructor!(u8);
    impl_integer_constructor!(i16);
    impl_integer_constructor!(u16);
    impl_integer_constructor!(i32);
    impl_integer_constructor!(u32);
    impl_integer_constructor!(i64);
    impl_integer_constructor!(u64);
    impl_integer_constructor!(isize);
    impl_integer_constructor!(usize);

    pub fn from_get_set(get_set_value: impl 'a + FnMut(Option<f64>) -> f64) -> Self {
        Self {
            get_set_value: Box::new(get_set_value),
            speed: 1.0,
            prefix: Default::default(),
            suffix: Default::default(),
            clamp_range: f64::NEG_INFINITY..=f64::INFINITY,
            min_decimals: 0,
            max_decimals: None,
        }
    }

    /// How much the value changes when dragged one point (logical pixel).
    pub fn speed(mut self, speed: impl Into<f64>) -> Self {
        self.speed = speed.into();
        self
    }

    /// Clamp incoming and outgoing values to this range.
    pub fn clamp_range<Num: emath::Numeric>(mut self, clamp_range: RangeInclusive<Num>) -> Self {
        self.clamp_range = clamp_range.start().to_f64()..=clamp_range.end().to_f64();
        self
    }

    #[deprecated = "Use clamp_range"]
    pub fn clamp_range_f64(mut self, clamp_range: RangeInclusive<f64>) -> Self {
        self.clamp_range = clamp_range;
        self
    }

    #[deprecated = "Renamed clamp_range"]
    pub fn range(self, clamp_range: RangeInclusive<f32>) -> Self {
        self.clamp_range(clamp_range)
    }

    /// Show a prefix before the number, e.g. "x: "
    pub fn prefix(mut self, prefix: impl ToString) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// Add a suffix to the number, this can be e.g. a unit ("°" or " m")
    pub fn suffix(mut self, suffix: impl ToString) -> Self {
        self.suffix = suffix.to_string();
        self
    }

    // TODO: we should also have a "min precision".
    /// Set a minimum number of decimals to display.
    /// Normally you don't need to pick a precision, as the slider will intelligently pick a precision for you.
    /// Regardless of precision the slider will use "smart aim" to help the user select nice, round values.
    pub fn min_decimals(mut self, min_decimals: usize) -> Self {
        self.min_decimals = min_decimals;
        self
    }

    // TODO: we should also have a "max precision".
    /// Set a maximum number of decimals to display.
    /// Values will also be rounded to this number of decimals.
    /// Normally you don't need to pick a precision, as the slider will intelligently pick a precision for you.
    /// Regardless of precision the slider will use "smart aim" to help the user select nice, round values.
    pub fn max_decimals(mut self, max_decimals: usize) -> Self {
        self.max_decimals = Some(max_decimals);
        self
    }

    pub fn max_decimals_opt(mut self, max_decimals: Option<usize>) -> Self {
        self.max_decimals = max_decimals;
        self
    }

    /// Set an exact number of decimals to display.
    /// Values will also be rounded to this number of decimals.
    /// Normally you don't need to pick a precision, as the slider will intelligently pick a precision for you.
    /// Regardless of precision the slider will use "smart aim" to help the user select nice, round values.
    pub fn fixed_decimals(mut self, num_decimals: usize) -> Self {
        self.min_decimals = num_decimals;
        self.max_decimals = Some(num_decimals);
        self
    }
}

impl<'a> Widget for DragValue<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            mut get_set_value,
            speed,
            clamp_range,
            prefix,
            suffix,
            min_decimals,
            max_decimals,
        } = self;

        let is_slow_speed =
            ui.input().modifiers.shift_only() && ui.memory().is_being_dragged(ui.next_auto_id());

        let old_value = get(&mut get_set_value);
        let value = clamp_to_range(old_value, clamp_range.clone());
        if old_value != value {
            set(&mut get_set_value, value);
        }
        let aim_rad = ui.input().aim_radius() as f64;

        let auto_decimals = (aim_rad / speed.abs()).log10().ceil().clamp(0.0, 15.0) as usize;
        let auto_decimals = auto_decimals + is_slow_speed as usize;

        let max_decimals = max_decimals.unwrap_or(auto_decimals + 2);
        let auto_decimals = auto_decimals.clamp(min_decimals, max_decimals);
        let value_text = if value == 0.0 {
            "0".to_owned()
        } else {
            emath::format_with_decimals_in_range(value, auto_decimals..=max_decimals)
        };

        let kb_edit_id = ui.auto_id_with("edit");
        let is_kb_editing = ui.memory().has_focus(kb_edit_id);

        let mut response = if is_kb_editing {
            let button_width = ui.spacing().interact_size.x;
            let mut value_text = ui
                .memory()
                .drag_value
                .edit_string
                .take()
                .unwrap_or(value_text);
            let response = ui.add(
                TextEdit::singleline(&mut value_text)
                    .id(kb_edit_id)
                    .desired_width(button_width)
                    .text_style(TextStyle::Monospace),
            );
            if let Ok(parsed_value) = value_text.parse() {
                let parsed_value = clamp_to_range(parsed_value, clamp_range);
                set(&mut get_set_value, parsed_value)
            }
            if ui.input().key_pressed(Key::Enter) {
                ui.memory().surrender_focus(kb_edit_id);
                ui.memory().drag_value.edit_string = None;
            } else {
                ui.memory().drag_value.edit_string = Some(value_text);
            }
            response
        } else {
            let button = Button::new(format!("{}{}{}", prefix, value_text, suffix))
                .sense(Sense::click_and_drag())
                .text_style(TextStyle::Monospace)
                .wrap(false)
                .min_size(ui.spacing().interact_size); // TODO: find some more generic solution to this

            let response = ui.add(button);
            let slider_tooltip_text = &ui.ctx().localization().slider_tooltip;
            let response = response
                .on_hover_cursor(CursorIcon::ResizeHorizontal)
                .on_hover_text(format!(
                    "{}{}{}\n{}",
                    prefix,
                    value as f32, // Show full precision value on-hover. TODO: figure out f64 vs f32
                    suffix,
                    slider_tooltip_text
                ));

            if response.clicked() {
                ui.memory().request_focus(kb_edit_id);
                ui.memory().drag_value.edit_string = None; // Filled in next frame
            } else if response.dragged() {
                ui.output().cursor_icon = CursorIcon::ResizeHorizontal;

                let mdelta = response.drag_delta();
                let delta_points = mdelta.x - mdelta.y; // Increase to the right and up

                let speed = if is_slow_speed { speed / 10.0 } else { speed };

                let delta_value = delta_points as f64 * speed;

                if delta_value != 0.0 {
                    let mut drag_state = std::mem::take(&mut ui.memory().drag_value);

                    // Since we round the value being dragged, we need to store the full precision value in memory:
                    let stored_value = (drag_state.last_dragged_id == Some(response.id))
                        .then(|| drag_state.last_dragged_value)
                        .flatten();
                    let stored_value = stored_value.unwrap_or(value);
                    let stored_value = stored_value + delta_value as f64;

                    let aim_delta = aim_rad * speed;
                    let rounded_new_value = emath::smart_aim::best_in_range_f64(
                        stored_value - aim_delta,
                        stored_value + aim_delta,
                    );
                    let rounded_new_value =
                        emath::round_to_decimals(rounded_new_value, auto_decimals);
                    let rounded_new_value = clamp_to_range(rounded_new_value, clamp_range);
                    set(&mut get_set_value, rounded_new_value);

                    drag_state.last_dragged_id = Some(response.id);
                    drag_state.last_dragged_value = Some(stored_value);
                    ui.memory().drag_value = drag_state;
                }
            } else if response.has_focus() {
                let change = ui.input().num_presses(Key::ArrowUp) as f64
                    + ui.input().num_presses(Key::ArrowRight) as f64
                    - ui.input().num_presses(Key::ArrowDown) as f64
                    - ui.input().num_presses(Key::ArrowLeft) as f64;

                if change != 0.0 {
                    let new_value = value + speed * change;
                    let new_value = emath::round_to_decimals(new_value, auto_decimals);
                    let new_value = clamp_to_range(new_value, clamp_range);
                    set(&mut get_set_value, new_value);
                }
            }

            response
        };

        response.changed = get(&mut get_set_value) != old_value;

        response.widget_info(|| WidgetInfo::drag_value(value));
        response
    }
}

fn clamp_to_range(x: f64, range: RangeInclusive<f64>) -> f64 {
    x.clamp(
        range.start().min(*range.end()),
        range.start().max(*range.end()),
    )
}
