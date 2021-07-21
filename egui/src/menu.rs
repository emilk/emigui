//! Menu bar functionality (very basic so far).
//!
//! Usage:
//! ```
//! fn show_menu(ui: &mut egui::Ui) {
//!     use egui::{menu, Button};
//!
//!     menu::bar(ui, |ui| {
//!         menu::menu(ui, "File", |ui| {
//!             if ui.button("Open").clicked() {
//!                 // ...
//!             }
//!         });
//!     });
//! }
//! ```

use crate::{widgets::*, *};
use epaint::Stroke;

/// What is saved between frames.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub(crate) struct BarState {
    open_menu: Option<Id>,
}

impl BarState {
    fn load(ctx: &Context, bar_id: &Id) -> Self {
        *ctx.memory().id_data_temp.get_or_default(*bar_id)
    }

    fn save(self, ctx: &Context, bar_id: Id) {
        ctx.memory().id_data_temp.insert(bar_id, self);
    }
}

/// The menu bar goes well in a [`TopBottomPanel::top`],
/// but can also be placed in a `Window`.
/// In the latter case you may want to wrap it in `Frame`.
pub fn bar<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    ui.horizontal(|ui| {
        let mut style = (**ui.style()).clone();
        style.spacing.button_padding = vec2(2.0, 0.0);
        // style.visuals.widgets.active.bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.active.bg_stroke = Stroke::none();
        // style.visuals.widgets.hovered.bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.hovered.bg_stroke = Stroke::none();
        style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.inactive.bg_stroke = Stroke::none();
        ui.set_style(style);

        // Take full width and fixed height:
        let height = ui.spacing().interact_size.y;
        ui.set_min_size(vec2(ui.available_width(), height));

        add_contents(ui)
    })
}

/// Construct a top level menu in a menu bar. This would be e.g. "File", "Edit" etc.
///
/// Returns `None` if the menu is not open.
pub fn menu<R>(
    ui: &mut Ui,
    title: impl ToString,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    menu_impl(ui, title, Box::new(add_contents))
}

pub(crate) fn menu_ui<'c, R>(
    ctx: &CtxRef,
    menu_id: Id,
    pos: Pos2,
    mut style: Style,
    add_contents: impl FnOnce(&mut Ui) -> R + 'c,
) -> InnerResponse<R> {
    let area = Area::new(menu_id)
        .order(Order::Foreground)
        .fixed_pos(pos)
        .drag_bounds(Rect::EVERYTHING);
    let frame = Frame::menu(&style);

    area.show(ctx, |ui| {
        frame
            .show(ui, |ui| {
                style.spacing.button_padding = vec2(2.0, 0.0);
                // style.visuals.widgets.active.bg_fill = Color32::TRANSPARENT;
                style.visuals.widgets.active.bg_stroke = Stroke::none();
                // style.visuals.widgets.hovered.bg_fill = Color32::TRANSPARENT;
                style.visuals.widgets.hovered.bg_stroke = Stroke::none();
                style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                style.visuals.widgets.inactive.bg_stroke = Stroke::none();
                ui.set_style(style);
                ui.with_layout(Layout::top_down_justified(Align::LEFT), add_contents)
                    .inner
            })
            .inner
    })
}
#[allow(clippy::needless_pass_by_value)]
fn menu_impl<'c, R>(
    ui: &mut Ui,
    title: impl ToString,
    add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
) -> Option<R> {
    let title = title.to_string();
    let bar_id = ui.id();
    let menu_id = bar_id.with(&title);

    let mut bar_state = BarState::load(ui.ctx(), &bar_id);

    let mut button = Button::new(title);

    if bar_state.open_menu == Some(menu_id) {
        button = button.fill(ui.visuals().widgets.open.bg_fill);
        button = button.stroke(ui.visuals().widgets.open.bg_stroke);
    }

    let button_response = ui.add(button);
    if button_response.clicked() {
        // Toggle
        if bar_state.open_menu == Some(menu_id) {
            bar_state.open_menu = None;
        } else {
            bar_state.open_menu = Some(menu_id);
        }
    } else if button_response.hovered() && bar_state.open_menu.is_some() {
        bar_state.open_menu = Some(menu_id);
    }

    let inner = if bar_state.open_menu == Some(menu_id) || ui.ctx().memory().everything_is_visible()
    {
        let inner = menu_ui(
            ui.ctx(),
            menu_id,
            button_response.rect.left_bottom(),
            ui.style().as_ref().clone(),
            add_contents,
        )
        .inner;

        // TODO: this prevents sub-menus in menus. We should fix that.
        if ui.input().key_pressed(Key::Escape) || button_response.clicked_elsewhere() {
            bar_state.open_menu = None;
        }
        Some(inner)
    } else {
        None
    };

    bar_state.save(ui.ctx(), bar_id);
    inner
}
