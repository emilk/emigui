//! Color picker widgets.

use crate::util::Cache;
use crate::*;
use epaint::{color::*, *};

fn contrast_color(color: impl Into<Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

/// Number of vertices per dimension in the color sliders.
/// We need at least 6 for hues, and more for smooth 2D areas.
/// Should always be a multiple of 6 to hit the peak hues in HSV/HSL (every 60°).
const N: u32 = 6 * 6;

fn background_checkers(painter: &Painter, rect: Rect) {
    let rect = rect.shrink(0.5); // Small hack to avoid the checkers from peeking through the sides
    if !rect.is_positive() {
        return;
    }

    let mut top_color = Color32::from_gray(128);
    let mut bottom_color = Color32::from_gray(32);
    let checker_size = Vec2::splat(rect.height() / 2.0);
    let n = (rect.width() / checker_size.x).round() as u32;

    let mut mesh = Mesh::default();
    for i in 0..n {
        let x = lerp(rect.left()..=rect.right(), i as f32 / (n as f32));
        mesh.add_colored_rect(
            Rect::from_min_size(pos2(x, rect.top()), checker_size),
            top_color,
        );
        mesh.add_colored_rect(
            Rect::from_min_size(pos2(x, rect.center().y), checker_size),
            bottom_color,
        );
        std::mem::swap(&mut top_color, &mut bottom_color);
    }
    painter.add(Shape::mesh(mesh));
}

pub fn show_color(ui: &mut Ui, color: impl Into<Hsva>, desired_size: Vec2) -> Response {
    show_hsva(ui, color.into(), desired_size)
}

fn show_hsva(ui: &mut Ui, color: Hsva, desired_size: Vec2) -> Response {
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::hover());
    background_checkers(ui.painter(), rect);
    if true {
        let left = Rect::from_min_max(rect.left_top(), rect.center_bottom());
        let right = Rect::from_min_max(rect.center_top(), rect.right_bottom());
        ui.painter().rect_filled(left, 0.0, color);
        ui.painter().rect_filled(right, 0.0, color.to_opaque());
    } else {
        ui.painter().add(Shape::Rect {
            rect,
            corner_radius: 2.0,
            fill: color.into(),
            stroke: Stroke::new(3.0, color.to_opaque()),
        });
    }
    response
}

fn color_button(ui: &mut Ui, color: Color32) -> Response {
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| WidgetInfo::new(WidgetType::ColorButton));
    let visuals = ui.style().interact(&response);
    let rect = rect.expand(visuals.expansion);

    background_checkers(ui.painter(), rect);

    let left_half = Rect::from_min_max(rect.left_top(), rect.center_bottom());
    let right_half = Rect::from_min_max(rect.center_top(), rect.right_bottom());
    ui.painter().rect_filled(left_half, 0.0, color);
    ui.painter().rect_filled(right_half, 0.0, color.to_opaque());

    let corner_radius = visuals.corner_radius.at_most(2.0);
    ui.painter()
        .rect_stroke(rect, corner_radius, (2.0, visuals.bg_fill)); // fill is intentional!

    response
}

fn color_slider_1d(ui: &mut Ui, value: &mut f32, color_at: impl Fn(f32) -> Color32) -> Response {
    #![allow(clippy::identity_op)]

    let desired_size = vec2(
        ui.spacing().slider_width,
        ui.spacing().interact_size.y * 2.0,
    );
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
    }

    let visuals = ui.style().interact(&response);

    background_checkers(ui.painter(), rect); // for alpha:

    {
        // fill color:
        let mut mesh = Mesh::default();
        for i in 0..=N {
            let t = i as f32 / (N as f32);
            let color = color_at(t);
            let x = lerp(rect.left()..=rect.right(), t);
            mesh.colored_vertex(pos2(x, rect.top()), color);
            mesh.colored_vertex(pos2(x, rect.bottom()), color);
            if i < N {
                mesh.add_triangle(2 * i + 0, 2 * i + 1, 2 * i + 2);
                mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
            }
        }
        ui.painter().add(Shape::mesh(mesh));
    }

    ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke); // outline

    {
        // Show where the slider is at:
        let x = lerp(rect.left()..=rect.right(), *value);
        let r = rect.height() / 4.0;
        let picked_color = color_at(*value);
        ui.painter().add(Shape::convex_polygon(
            vec![
                pos2(x - r, rect.bottom()),
                pos2(x + r, rect.bottom()),
                pos2(x, rect.center().y),
            ],
            picked_color,
            Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        ));
    }

    response
}

fn color_slider_2d(
    ui: &mut Ui,
    x_value: &mut f32,
    y_value: &mut f32,
    color_at: impl Fn(f32, f32) -> Color32,
) -> Response {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *x_value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
        *y_value = remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }

    let visuals = ui.style().interact(&response);
    let mut mesh = Mesh::default();

    for xi in 0..=N {
        for yi in 0..=N {
            let xt = xi as f32 / (N as f32);
            let yt = yi as f32 / (N as f32);
            let color = color_at(xt, yt);
            let x = lerp(rect.left()..=rect.right(), xt);
            let y = lerp(rect.bottom()..=rect.top(), yt);
            mesh.colored_vertex(pos2(x, y), color);

            if xi < N && yi < N {
                let x_offset = 1;
                let y_offset = N + 1;
                let tl = yi * y_offset + xi;
                mesh.add_triangle(tl, tl + x_offset, tl + y_offset);
                mesh.add_triangle(tl + x_offset, tl + y_offset, tl + y_offset + x_offset);
            }
        }
    }
    ui.painter().add(Shape::mesh(mesh)); // fill

    ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke); // outline

    // Show where the slider is at:
    let x = lerp(rect.left()..=rect.right(), *x_value);
    let y = lerp(rect.bottom()..=rect.top(), *y_value);
    let picked_color = color_at(*x_value, *y_value);
    ui.painter().add(Shape::Circle {
        center: pos2(x, y),
        radius: rect.width() / 12.0,
        fill: picked_color,
        stroke: Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
    });

    response
}

/// What options to show for alpha
#[derive(Clone, Copy, PartialEq)]
pub enum Alpha {
    // Set alpha to 1.0, and show no option for it.
    Opaque,
    // Only show normal blend options for it.
    OnlyBlend,
    // Show both blend and additive options.
    BlendOrAdditive,
}

fn color_text_ui(ui: &mut Ui, color: impl Into<Color32>) {
    let color = color.into();
    let button_text = ui.ctx().localization().click_copy;
    ui.horizontal(|ui| {
        let [r, g, b, a] = color.to_array();

        ui.label(format!(
            "RGBA (premultiplied): rgba({}, {}, {}, {})",
            r, g, b, a
        ));
        if ui.button("📋").on_hover_text(button_text).clicked() {
            ui.output().copied_text = format!("{}, {}, {}, {}", r, g, b, a);
        }
    });
}

fn color_picker_hsvag_2d(ui: &mut Ui, hsva: &mut HsvaGamma, alpha: Alpha) {
    color_text_ui(ui, *hsva);

    let blending_text = ui.ctx().localization().cp_blending;
    let additive_text = ui.ctx().localization().cp_additive;
    let normal_text = ui.ctx().localization().cp_normal;

    if alpha == Alpha::BlendOrAdditive {
        // We signal additive blending by storing a negative alpha (a bit ironic).
        let a = &mut hsva.a;
        let mut additive = *a < 0.0;
        ui.horizontal(|ui| {
            ui.label(blending_text);
            ui.radio_value(&mut additive, false, additive_text);
            ui.radio_value(&mut additive, true, normal_text);

            if additive {
                *a = -a.abs();
            }

            if !additive {
                *a = a.abs();
            }
        });
    }
    let additive = hsva.a < 0.0;

    // Using different grid ids avoid some flickering when switching between
    // (the grid remembers the sizes of its contents).
    let grid_id = if alpha == Alpha::Opaque {
        "hsva_color_picker_opaque"
    } else if additive {
        "hsva_color_picker_additive"
    } else {
        "hsva_color_picker_normal"
    };

    crate::Grid::new(grid_id).show(ui, |ui| {
        let current_color_size = vec2(
            ui.spacing().slider_width,
            ui.spacing().interact_size.y * 2.0,
        );

        let opaque = HsvaGamma { a: 1.0, ..*hsva };

        let selected_color_text = ui.ctx().localization().cp_selected_color;

        if alpha == Alpha::Opaque {
            hsva.a = 1.0;
            show_color(ui, *hsva, current_color_size);
            ui.label(selected_color_text);
            ui.end_row();
        } else {
            let a = &mut hsva.a;

            if alpha == Alpha::OnlyBlend {
                if *a < 0.0 {
                    *a = 0.5; // was additive, but isn't allowed to be
                }
                color_slider_1d(ui, a, |a| HsvaGamma { a, ..opaque }.into());
                ui.label("Alpha");
                ui.end_row();
            } else if !additive {
                color_slider_1d(ui, a, |a| HsvaGamma { a, ..opaque }.into());
                ui.label("Alpha");
                ui.end_row();
            }

            show_color(ui, *hsva, current_color_size);
            ui.label(selected_color_text);
            ui.end_row();
        }

        ui.separator(); // TODO: fix ever-expansion
        ui.end_row();

        let HsvaGamma { h, s, v, a: _ } = hsva;

        color_slider_1d(ui, h, |h| {
            HsvaGamma {
                h,
                s: 1.0,
                v: 1.0,
                a: 1.0,
            }
            .into()
        });

        let hue_text = ui.ctx().localization().cp_hue;
        let saturation_text = ui.ctx().localization().cp_saturation;
        let value_text = ui.ctx().localization().cp_value;

        ui.label(hue_text);
        ui.end_row();

        color_slider_1d(ui, s, |s| HsvaGamma { s, ..opaque }.into());
        ui.label(saturation_text);
        ui.end_row();

        color_slider_1d(ui, v, |v| HsvaGamma { v, ..opaque }.into());
        ui.label(value_text);
        ui.end_row();

        color_slider_2d(ui, v, s, |v, s| HsvaGamma { s, v, ..opaque }.into());
        ui.label(format!("{} / {}", value_text, saturation_text));
        ui.end_row();
    });
}

/// return true on change
fn color_picker_hsva_2d(ui: &mut Ui, hsva: &mut Hsva, alpha: Alpha) -> bool {
    let mut hsvag = HsvaGamma::from(*hsva);
    color_picker_hsvag_2d(ui, &mut hsvag, alpha);
    let new_hasva = Hsva::from(hsvag);
    if *hsva == new_hasva {
        false
    } else {
        *hsva = new_hasva;
        true
    }
}

pub fn color_edit_button_hsva(ui: &mut Ui, hsva: &mut Hsva, alpha: Alpha) -> Response {
    let pupup_id = ui.auto_id_with("popup");

    let edit_text = ui.ctx().localization().cp_edit;
    let mut button_response = color_button(ui, (*hsva).into()).on_hover_text(edit_text);

    if button_response.clicked() {
        ui.memory().toggle_popup(pupup_id);
    }
    // TODO: make it easier to show a temporary popup that closes when you click outside it
    if ui.memory().is_popup_open(pupup_id) {
        let area_response = Area::new(pupup_id)
            .order(Order::Foreground)
            .default_pos(button_response.rect.max)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().slider_width = 256.0;
                Frame::popup(ui.style()).show(ui, |ui| {
                    if color_picker_hsva_2d(ui, hsva, alpha) {
                        button_response.mark_changed();
                    }
                });
            });

        if !button_response.clicked()
            && (ui.input().key_pressed(Key::Escape) || area_response.clicked_elsewhere())
        {
            ui.memory().close_popup();
        }
    }

    button_response
}

/// Shows a button with the given color.
/// If the user clicks the button, a full color picker is shown.
pub fn color_edit_button_srgba(ui: &mut Ui, srgba: &mut Color32, alpha: Alpha) -> Response {
    // To ensure we keep hue slider when `srgba` is gray we store the
    // full `Hsva` in a cache:

    let mut hsva = ui
        .ctx()
        .memory()
        .data_temp
        .get_or_default::<Cache<Color32, Hsva>>()
        .get(srgba)
        .cloned()
        .unwrap_or_else(|| Hsva::from(*srgba));

    let response = color_edit_button_hsva(ui, &mut hsva, alpha);

    *srgba = Color32::from(hsva);

    ui.ctx()
        .memory()
        .data_temp
        .get_mut_or_default::<Cache<Color32, Hsva>>()
        .set(*srgba, hsva);

    response
}
