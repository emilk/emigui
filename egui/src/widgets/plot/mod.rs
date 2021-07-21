//! Simple plotting library.

mod items;
mod legend;
mod transform;

use std::collections::HashSet;

use items::PlotItem;
pub use items::{
    Arrows, HLine, Line, LineStyle, MarkerShape, PlotImage, Points, Polygon, Text, VLine, Value,
    Values,
};
use legend::LegendWidget;
pub use legend::{Corner, Legend};
use transform::{Bounds, ScreenTransform};

use crate::*;
use color::Hsva;

// ----------------------------------------------------------------------------

/// Information about the plot that has to persist between frames.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone)]
struct PlotMemory {
    bounds: Bounds,
    auto_bounds: bool,
    hovered_entry: Option<String>,
    hidden_items: HashSet<String>,
}

// ----------------------------------------------------------------------------

/// A 2D plot, e.g. a graph of a function.
///
/// `Plot` supports multiple lines and points.
///
/// ```
/// # let ui = &mut egui::Ui::__test();
/// use egui::plot::{Line, Plot, Value, Values};
/// let sin = (0..1000).map(|i| {
///     let x = i as f64 * 0.01;
///     Value::new(x, x.sin())
/// });
/// let line = Line::new(Values::from_values_iter(sin));
/// ui.add(
///     Plot::new("my_plot").line(line).view_aspect(2.0)
/// );
/// ```
pub struct Plot {
    id_source: Id,
    next_auto_color_idx: usize,

    items: Vec<Box<dyn PlotItem>>,

    center_x_axis: bool,
    center_y_axis: bool,
    allow_zoom: bool,
    allow_drag: bool,
    min_auto_bounds: Bounds,
    margin_fraction: Vec2,

    min_size: Vec2,
    width: Option<f32>,
    height: Option<f32>,
    data_aspect: Option<f32>,
    view_aspect: Option<f32>,

    show_x: bool,
    show_y: bool,
    legend_config: Option<Legend>,
}

impl Plot {
    /// Give a unique id for each plot within the same `Ui`.
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            id_source: Id::new(id_source),
            next_auto_color_idx: 0,

            items: Default::default(),

            center_x_axis: false,
            center_y_axis: false,
            allow_zoom: true,
            allow_drag: true,
            min_auto_bounds: Bounds::NOTHING,
            margin_fraction: Vec2::splat(0.05),

            min_size: Vec2::splat(64.0),
            width: None,
            height: None,
            data_aspect: None,
            view_aspect: None,

            show_x: true,
            show_y: true,
            legend_config: None,
        }
    }

    fn auto_color(&mut self) -> Color32 {
        let i = self.next_auto_color_idx;
        self.next_auto_color_idx += 1;
        let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
        let h = i as f32 * golden_ratio;
        Hsva::new(h, 0.85, 0.5, 1.0).into() // TODO: OkLab or some other perspective color space
    }

    /// Add a data lines.
    pub fn line(mut self, mut line: Line) -> Self {
        if line.series.is_empty() {
            return self;
        };

        // Give the stroke an automatic color if no color has been assigned.
        if line.stroke.color == Color32::TRANSPARENT {
            line.stroke.color = self.auto_color();
        }
        self.items.push(Box::new(line));
        self
    }

    /// Add a polygon. The polygon has to be convex.
    pub fn polygon(mut self, mut polygon: Polygon) -> Self {
        if polygon.series.is_empty() {
            return self;
        };

        // Give the stroke an automatic color if no color has been assigned.
        if polygon.stroke.color == Color32::TRANSPARENT {
            polygon.stroke.color = self.auto_color();
        }
        self.items.push(Box::new(polygon));
        self
    }

    /// Add a text.
    pub fn text(mut self, text: Text) -> Self {
        if text.text.is_empty() {
            return self;
        };

        self.items.push(Box::new(text));
        self
    }

    /// Add data points.
    pub fn points(mut self, mut points: Points) -> Self {
        if points.series.is_empty() {
            return self;
        };

        // Give the points an automatic color if no color has been assigned.
        if points.color == Color32::TRANSPARENT {
            points.color = self.auto_color();
        }
        self.items.push(Box::new(points));
        self
    }

    /// Add arrows.
    pub fn arrows(mut self, mut arrows: Arrows) -> Self {
        if arrows.origins.is_empty() || arrows.tips.is_empty() {
            return self;
        };

        // Give the arrows an automatic color if no color has been assigned.
        if arrows.color == Color32::TRANSPARENT {
            arrows.color = self.auto_color();
        }
        self.items.push(Box::new(arrows));
        self
    }

    /// Add an image.
    pub fn image(mut self, image: PlotImage) -> Self {
        self.items.push(Box::new(image));
        self
    }

    /// Add a horizontal line.
    /// Can be useful e.g. to show min/max bounds or similar.
    /// Always fills the full width of the plot.
    pub fn hline(mut self, mut hline: HLine) -> Self {
        if hline.stroke.color == Color32::TRANSPARENT {
            hline.stroke.color = self.auto_color();
        }
        self.items.push(Box::new(hline));
        self
    }

    /// Add a vertical line.
    /// Can be useful e.g. to show min/max bounds or similar.
    /// Always fills the full height of the plot.
    pub fn vline(mut self, mut vline: VLine) -> Self {
        if vline.stroke.color == Color32::TRANSPARENT {
            vline.stroke.color = self.auto_color();
        }
        self.items.push(Box::new(vline));
        self
    }

    /// width / height ratio of the data.
    /// For instance, it can be useful to set this to `1.0` for when the two axes show the same
    /// unit.
    /// By default the plot window's aspect ratio is used.
    pub fn data_aspect(mut self, data_aspect: f32) -> Self {
        self.data_aspect = Some(data_aspect);
        self
    }

    /// width / height ratio of the plot region.
    /// By default no fixed aspect ratio is set (and width/height will fill the ui it is in).
    pub fn view_aspect(mut self, view_aspect: f32) -> Self {
        self.view_aspect = Some(view_aspect);
        self
    }

    /// Width of plot. By default a plot will fill the ui it is in.
    /// If you set [`Self::view_aspect`], the width can be calculated from the height.
    pub fn width(mut self, width: f32) -> Self {
        self.min_size.x = width;
        self.width = Some(width);
        self
    }

    /// Height of plot. By default a plot will fill the ui it is in.
    /// If you set [`Self::view_aspect`], the height can be calculated from the width.
    pub fn height(mut self, height: f32) -> Self {
        self.min_size.y = height;
        self.height = Some(height);
        self
    }

    /// Minimum size of the plot view.
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Show the x-value (e.g. when hovering). Default: `true`.
    pub fn show_x(mut self, show_x: bool) -> Self {
        self.show_x = show_x;
        self
    }

    /// Show the y-value (e.g. when hovering). Default: `true`.
    pub fn show_y(mut self, show_y: bool) -> Self {
        self.show_y = show_y;
        self
    }

    #[deprecated = "Renamed center_x_axis"]
    pub fn symmetrical_x_axis(mut self, on: bool) -> Self {
        self.center_x_axis = on;
        self
    }

    #[deprecated = "Renamed center_y_axis"]
    pub fn symmetrical_y_axis(mut self, on: bool) -> Self {
        self.center_y_axis = on;
        self
    }

    /// Always keep the x-axis centered. Default: `false`.
    pub fn center_x_axis(mut self, on: bool) -> Self {
        self.center_x_axis = on;
        self
    }

    /// Always keep the y-axis centered. Default: `false`.
    pub fn center_y_axis(mut self, on: bool) -> Self {
        self.center_y_axis = on;
        self
    }

    /// Whether to allow zooming in the plot. Default: `true`.
    pub fn allow_zoom(mut self, on: bool) -> Self {
        self.allow_zoom = on;
        self
    }

    /// Whether to allow dragging in the plot to move the bounds. Default: `true`.
    pub fn allow_drag(mut self, on: bool) -> Self {
        self.allow_drag = on;
        self
    }

    /// Expand bounds to include the given x value.
    /// For instance, to always show the y axis, call `plot.include_x(0.0)`.
    pub fn include_x(mut self, x: impl Into<f64>) -> Self {
        self.min_auto_bounds.extend_with_x(x.into());
        self
    }

    /// Expand bounds to include the given y value.
    /// For instance, to always show the x axis, call `plot.include_y(0.0)`.
    pub fn include_y(mut self, y: impl Into<f64>) -> Self {
        self.min_auto_bounds.extend_with_y(y.into());
        self
    }

    #[deprecated = "Use `Plot::legend` instead"]
    /// Whether to show a legend including all named items. Default: `true`.
    pub fn show_legend(mut self, show: bool) -> Self {
        self.legend_config = show.then(Legend::default);
        self
    }

    /// Show a legend including all named items.
    pub fn legend(mut self, legend: Legend) -> Self {
        self.legend_config = Some(legend);
        self
    }
}

impl Widget for Plot {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            id_source,
            next_auto_color_idx: _,
            mut items,
            center_x_axis,
            center_y_axis,
            allow_zoom,
            allow_drag,
            min_auto_bounds,
            margin_fraction,
            width,
            height,
            min_size,
            data_aspect,
            view_aspect,
            mut show_x,
            mut show_y,
            legend_config,
        } = self;

        let plot_id = ui.make_persistent_id(id_source);
        let memory = ui
            .memory()
            .id_data
            .get_mut_or_insert_with(plot_id, || PlotMemory {
                bounds: min_auto_bounds,
                auto_bounds: !min_auto_bounds.is_valid(),
                hovered_entry: None,
                hidden_items: HashSet::new(),
            })
            .clone();

        let PlotMemory {
            mut bounds,
            mut auto_bounds,
            mut hovered_entry,
            mut hidden_items,
        } = memory;

        // Determine the size of the plot in the UI
        let size = {
            let width = width
                .unwrap_or_else(|| {
                    if let (Some(height), Some(aspect)) = (height, view_aspect) {
                        height * aspect
                    } else {
                        ui.available_size_before_wrap_finite().x
                    }
                })
                .at_least(min_size.x);

            let height = height
                .unwrap_or_else(|| {
                    if let Some(aspect) = view_aspect {
                        width / aspect
                    } else {
                        ui.available_size_before_wrap_finite().y
                    }
                })
                .at_least(min_size.y);
            vec2(width, height)
        };

        let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
        let plot_painter = ui.painter().sub_region(rect);

        // Background
        plot_painter.add(Shape::Rect {
            rect,
            corner_radius: 2.0,
            fill: ui.visuals().extreme_bg_color,
            stroke: ui.visuals().widgets.noninteractive.bg_stroke,
        });

        // Legend
        let legend = legend_config
            .and_then(|config| LegendWidget::try_new(rect, config, &items, &hidden_items));
        // Don't show hover cursor when hovering over legend.
        if hovered_entry.is_some() {
            show_x = false;
            show_y = false;
        }
        // Remove the deselected items.
        items.retain(|item| !hidden_items.contains(item.name()));
        // Highlight the hovered items.
        if let Some(hovered_name) = &hovered_entry {
            items
                .iter_mut()
                .filter(|entry| entry.name() == hovered_name)
                .for_each(|entry| entry.highlight());
        }
        // Move highlighted items to front.
        items.sort_by_key(|item| item.highlighted());

        auto_bounds |= response.double_clicked_by(PointerButton::Primary);

        // Set bounds automatically based on content.
        if auto_bounds || !bounds.is_valid() {
            bounds = min_auto_bounds;
            items
                .iter()
                .for_each(|item| bounds.merge(&item.get_bounds()));
            bounds.add_relative_margin(margin_fraction);
        }
        // Make sure they are not empty.
        if !bounds.is_valid() {
            bounds = Bounds::new_symmetrical(1.0);
        }

        // Scale axes so that the origin is in the center.
        if center_x_axis {
            bounds.make_x_symmetrical();
        };
        if center_y_axis {
            bounds.make_y_symmetrical()
        };

        let mut transform = ScreenTransform::new(rect, bounds, center_x_axis, center_y_axis);

        // Enforce equal aspect ratio.
        if let Some(data_aspect) = data_aspect {
            transform.set_aspect(data_aspect as f64);
        }

        // Dragging
        if allow_drag && response.dragged_by(PointerButton::Primary) {
            transform.translate_bounds(-response.drag_delta());
            auto_bounds = false;
        }

        // Zooming
        if allow_zoom {
            if let Some(hover_pos) = response.hover_pos() {
                let zoom_factor = if data_aspect.is_some() {
                    Vec2::splat(ui.input().zoom_delta())
                } else {
                    ui.input().zoom_delta_2d()
                };
                if zoom_factor != Vec2::splat(1.0) {
                    transform.zoom(zoom_factor, hover_pos);
                    auto_bounds = false;
                }

                let scroll_delta = ui.input().scroll_delta;
                if scroll_delta != Vec2::ZERO {
                    transform.translate_bounds(-scroll_delta);
                    auto_bounds = false;
                }
            }
        }

        // Initialize values from functions.
        items
            .iter_mut()
            .for_each(|item| item.initialize(transform.bounds().range_x()));

        let bounds = *transform.bounds();

        let prepared = Prepared {
            items,
            show_x,
            show_y,
            transform,
        };
        prepared.ui(ui, &response);

        if let Some(mut legend) = legend {
            ui.add(&mut legend);
            hidden_items = legend.get_hidden_items();
            hovered_entry = legend.get_hovered_entry_name();
        }

        ui.memory().id_data.insert(
            plot_id,
            PlotMemory {
                bounds,
                auto_bounds,
                hovered_entry,
                hidden_items,
            },
        );

        if show_x || show_y {
            response.on_hover_cursor(CursorIcon::Crosshair)
        } else {
            response
        }
    }
}

struct Prepared {
    items: Vec<Box<dyn PlotItem>>,
    show_x: bool,
    show_y: bool,
    transform: ScreenTransform,
}

impl Prepared {
    fn ui(self, ui: &mut Ui, response: &Response) {
        let mut shapes = Vec::new();

        for d in 0..2 {
            self.paint_axis(ui, d, &mut shapes);
        }

        let transform = &self.transform;

        let mut plot_ui = ui.child_ui(*transform.frame(), Layout::default());
        plot_ui.set_clip_rect(*transform.frame());
        for item in &self.items {
            item.get_shapes(&mut plot_ui, transform, &mut shapes);
        }

        if let Some(pointer) = response.hover_pos() {
            self.hover(ui, pointer, &mut shapes);
        }

        ui.painter().sub_region(*transform.frame()).extend(shapes);
    }

    fn paint_axis(&self, ui: &Ui, axis: usize, shapes: &mut Vec<Shape>) {
        let Self { transform, .. } = self;

        let bounds = transform.bounds();
        let text_style = TextStyle::Body;

        let base: i64 = 10;
        let basef = base as f64;

        let min_line_spacing_in_points = 6.0; // TODO: large enough for a wide label
        let step_size = transform.dvalue_dpos()[axis] * min_line_spacing_in_points;
        let step_size = basef.powi(step_size.abs().log(basef).ceil() as i32);

        let step_size_in_points = (transform.dpos_dvalue()[axis] * step_size).abs() as f32;

        // Where on the cross-dimension to show the label values
        let value_cross = 0.0_f64.clamp(bounds.min[1 - axis], bounds.max[1 - axis]);

        for i in 0.. {
            let value_main = step_size * (bounds.min[axis] / step_size + i as f64).floor();
            if value_main > bounds.max[axis] {
                break;
            }

            let value = if axis == 0 {
                Value::new(value_main, value_cross)
            } else {
                Value::new(value_cross, value_main)
            };
            let pos_in_gui = transform.position_from_value(&value);

            let n = (value_main / step_size).round() as i64;
            let spacing_in_points = if n % (base * base) == 0 {
                step_size_in_points * (basef * basef) as f32 // think line (multiple of 100)
            } else if n % base == 0 {
                step_size_in_points * basef as f32 // medium line (multiple of 10)
            } else {
                step_size_in_points // thin line
            };

            let line_alpha = remap_clamp(
                spacing_in_points,
                (min_line_spacing_in_points as f32)..=300.0,
                0.0..=0.15,
            );

            if line_alpha > 0.0 {
                let line_color = color_from_alpha(ui, line_alpha);

                let mut p0 = pos_in_gui;
                let mut p1 = pos_in_gui;
                p0[1 - axis] = transform.frame().min[1 - axis];
                p1[1 - axis] = transform.frame().max[1 - axis];
                shapes.push(Shape::line_segment([p0, p1], Stroke::new(1.0, line_color)));
            }

            let text_alpha = remap_clamp(spacing_in_points, 40.0..=150.0, 0.0..=0.4);

            if text_alpha > 0.0 {
                let text = emath::round_to_decimals(value_main, 5).to_string(); // hack

                let galley = ui.fonts().layout_single_line(text_style, text);

                let mut text_pos = pos_in_gui + vec2(1.0, -galley.size.y);

                // Make sure we see the labels, even if the axis is off-screen:
                text_pos[1 - axis] = text_pos[1 - axis]
                    .at_most(transform.frame().max[1 - axis] - galley.size[1 - axis] - 2.0)
                    .at_least(transform.frame().min[1 - axis] + 1.0);

                shapes.push(Shape::Text {
                    pos: text_pos,
                    galley,
                    default_color: ui.visuals().text_color(),
                    color_map: epaint::text::TextColorMap::default(),
                    fake_italics: false,
                });
            }
        }

        fn color_from_alpha(ui: &Ui, alpha: f32) -> Color32 {
            if ui.visuals().dark_mode {
                Rgba::from_white_alpha(alpha).into()
            } else {
                Rgba::from_black_alpha((4.0 * alpha).at_most(1.0)).into()
            }
        }
    }

    fn hover(&self, ui: &Ui, pointer: Pos2, shapes: &mut Vec<Shape>) {
        let Self {
            transform,
            show_x,
            show_y,
            items,
            ..
        } = self;

        if !show_x && !show_y {
            return;
        }

        let interact_radius: f32 = 16.0;
        let mut closest_value = None;
        let mut closest_item = None;
        let mut closest_dist_sq = interact_radius.powi(2);
        for item in items {
            if let Some(values) = item.values() {
                for value in &values.values {
                    let pos = transform.position_from_value(value);
                    let dist_sq = pointer.distance_sq(pos);
                    if dist_sq < closest_dist_sq {
                        closest_dist_sq = dist_sq;
                        closest_value = Some(value);
                        closest_item = Some(item.name());
                    }
                }
            }
        }

        let mut prefix = String::new();
        if let Some(name) = closest_item {
            if !name.is_empty() {
                prefix = format!("{}\n", name);
            }
        }

        let line_color = if ui.visuals().dark_mode {
            Color32::from_gray(100).additive()
        } else {
            Color32::from_black_alpha(180)
        };

        let value = if let Some(value) = closest_value {
            let position = transform.position_from_value(value);
            shapes.push(Shape::circle_filled(position, 3.0, line_color));
            *value
        } else {
            transform.value_from_position(pointer)
        };
        let pointer = transform.position_from_value(&value);

        let rect = transform.frame();

        if *show_x {
            // vertical line
            shapes.push(Shape::line_segment(
                [pos2(pointer.x, rect.top()), pos2(pointer.x, rect.bottom())],
                (1.0, line_color),
            ));
        }
        if *show_y {
            // horizontal line
            shapes.push(Shape::line_segment(
                [pos2(rect.left(), pointer.y), pos2(rect.right(), pointer.y)],
                (1.0, line_color),
            ));
        }

        let text = {
            let scale = transform.dvalue_dpos();
            let x_decimals = ((-scale[0].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
            let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
            if *show_x && *show_y {
                format!(
                    "{}x = {:.*}\ny = {:.*}",
                    prefix, x_decimals, value.x, y_decimals, value.y
                )
            } else if *show_x {
                format!("{}x = {:.*}", prefix, x_decimals, value.x)
            } else if *show_y {
                format!("{}y = {:.*}", prefix, y_decimals, value.y)
            } else {
                unreachable!()
            }
        };

        shapes.push(Shape::text(
            ui.fonts(),
            pointer + vec2(3.0, -2.0),
            Align2::LEFT_BOTTOM,
            text,
            TextStyle::Body,
            ui.visuals().text_color(),
        ));
    }
}
