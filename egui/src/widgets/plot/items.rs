//! Contains items that can be added to a plot.

use std::ops::RangeInclusive;

use super::transform::{Bounds, ScreenTransform};
use crate::*;

/// A value in the value-space of the plot.
///
/// Uses f64 for improved accuracy to enable plotting
/// large values (e.g. unix time on x axis).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Value {
    /// This is often something monotonically increasing, such as time, but doesn't have to be.
    /// Goes from left to right.
    pub x: f64,
    /// Goes from bottom to top (inverse of everything else in egui!).
    pub y: f64,
}

impl Value {
    #[inline(always)]
    pub fn new(x: impl Into<f64>, y: impl Into<f64>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

// ----------------------------------------------------------------------------

/// A horizontal line in a plot, filling the full width
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HLine {
    pub(super) y: f64,
    pub(super) stroke: Stroke,
}

impl HLine {
    pub fn new(y: impl Into<f64>, stroke: impl Into<Stroke>) -> Self {
        Self {
            y: y.into(),
            stroke: stroke.into(),
        }
    }
}

/// A vertical line in a plot, filling the full width
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VLine {
    pub(super) x: f64,
    pub(super) stroke: Stroke,
}

impl VLine {
    pub fn new(x: impl Into<f64>, stroke: impl Into<Stroke>) -> Self {
        Self {
            x: x.into(),
            stroke: stroke.into(),
        }
    }
}

pub(super) trait PlotItem {
    fn get_shapes(&self, transform: &ScreenTransform, shapes: &mut Vec<Shape>);
    fn series_mut(&mut self) -> &mut Values;
    fn get_bounds(&self) -> Bounds;
    fn closest<'a>(
        &'a self,
        ui: &'a Ui,
        pointer: Pos2,
        transform: &'a ScreenTransform,
        show_x: bool,
        show_y: bool,
    ) -> Option<HoverElement<'a>>;
    fn name(&self) -> &str;
    fn color(&self) -> Color32;
    fn highlight(&mut self);
    fn highlighted(&self) -> bool;
}

// ----------------------------------------------------------------------------

/// Describes a function y = f(x) with an optional range for x and a number of points.
struct ExplicitGenerator {
    function: Box<dyn Fn(f64) -> f64>,
    x_range: RangeInclusive<f64>,
    points: usize,
}

pub struct Values {
    pub(super) values: Vec<Value>,
    generator: Option<ExplicitGenerator>,
}

impl Default for Values {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            generator: None,
        }
    }
}

impl Values {
    pub fn from_values(values: Vec<Value>) -> Self {
        Self {
            values,
            generator: None,
        }
    }

    pub fn from_values_iter(iter: impl Iterator<Item = Value>) -> Self {
        Self::from_values(iter.collect())
    }

    /// Draw a line based on a function `y=f(x)`, a range (which can be infinite) for x and the number of points.
    pub fn from_explicit_callback(
        function: impl Fn(f64) -> f64 + 'static,
        x_range: RangeInclusive<f64>,
        points: usize,
    ) -> Self {
        let generator = ExplicitGenerator {
            function: Box::new(function),
            x_range,
            points,
        };

        Self {
            values: Vec::new(),
            generator: Some(generator),
        }
    }

    /// Draw a line based on a function `(x,y)=f(t)`, a range for t and the number of points.
    pub fn from_parametric_callback(
        function: impl Fn(f64) -> (f64, f64),
        t_range: RangeInclusive<f64>,
        points: usize,
    ) -> Self {
        let increment = (t_range.end() - t_range.start()) / (points - 1) as f64;
        let values = (0..points).map(|i| {
            let t = t_range.start() + i as f64 * increment;
            let (x, y) = function(t);
            Value { x, y }
        });
        Self::from_values_iter(values)
    }

    /// From a series of y-values.
    /// The x-values will be the indices of these values
    pub fn from_ys_f32(ys: &[f32]) -> Self {
        let values: Vec<Value> = ys
            .iter()
            .enumerate()
            .map(|(i, &y)| Value {
                x: i as f64,
                y: y as f64,
            })
            .collect();
        Self::from_values(values)
    }

    /// Returns true if there are no data points available and there is no function to generate any.
    pub(super) fn is_empty(&self) -> bool {
        self.generator.is_none() && self.values.is_empty()
    }

    /// If initialized with a generator function, this will generate `n` evenly spaced points in the
    /// given range.
    pub(super) fn generate_points(&mut self, x_range: RangeInclusive<f64>) {
        if let Some(generator) = self.generator.take() {
            if let Some(intersection) = Self::range_intersection(&x_range, &generator.x_range) {
                let increment =
                    (intersection.end() - intersection.start()) / (generator.points - 1) as f64;
                self.values = (0..generator.points)
                    .map(|i| {
                        let x = intersection.start() + i as f64 * increment;
                        let y = (generator.function)(x);
                        Value { x, y }
                    })
                    .collect();
            }
        }
    }

    /// Returns the intersection of two ranges if they intersect.
    fn range_intersection(
        range1: &RangeInclusive<f64>,
        range2: &RangeInclusive<f64>,
    ) -> Option<RangeInclusive<f64>> {
        let start = range1.start().max(*range2.start());
        let end = range1.end().min(*range2.end());
        (start < end).then(|| start..=end)
    }

    pub(super) fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        self.values
            .iter()
            .for_each(|value| bounds.extend_with(value));
        bounds
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MarkerShape {
    Circle,
    Diamond,
    Square,
    Cross,
    Plus,
    Up,
    Down,
    Left,
    Right,
    Asterisk,
}

impl MarkerShape {
    /// Get a vector containing all marker shapes.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Circle,
            Self::Diamond,
            Self::Square,
            Self::Cross,
            Self::Plus,
            Self::Up,
            Self::Down,
            Self::Left,
            Self::Right,
            Self::Asterisk,
        ]
    }
}

/// A series of values forming a path.
pub struct Line {
    pub(super) series: Values,
    pub(super) stroke: Stroke,
    pub(super) name: String,
    pub(super) highlight: bool,
}

impl Line {
    pub fn new(series: Values) -> Self {
        Self {
            series,
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: Default::default(),
            highlight: false,
        }
    }

    /// Highlight this line in the plot by scaling up the line and marker size.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Add a stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Stroke width. A high value means the plot thickens.
    pub fn width(mut self, width: f32) -> Self {
        self.stroke.width = width;
        self
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Name of this line.
    ///
    /// This name will show up in the plot legend, if legends are turned on. Multiple lines may
    /// share the same name, in which case they will also share an entry in the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for Line {
    fn get_shapes(&self, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let Self {
            series,
            mut stroke,
            highlight,
            ..
        } = self;

        if *highlight {
            stroke.width *= 2.0;
        }

        let values_tf: Vec<_> = series
            .values
            .iter()
            .map(|v| transform.position_from_value(v))
            .collect();

        let line_shape = if values_tf.len() > 1 {
            Shape::line(values_tf, stroke)
        } else {
            Shape::circle_filled(values_tf[0], stroke.width / 2.0, stroke.color)
        };
        shapes.push(line_shape);
    }

    fn series_mut(&mut self) -> &mut Values {
        &mut self.series
    }

    fn get_bounds(&self) -> Bounds {
        self.series.get_bounds()
    }

    fn closest<'a>(
        &'a self,
        ui: &'a Ui,
        pointer: Pos2,
        transform: &'a ScreenTransform,
        show_x: bool,
        show_y: bool,
    ) -> Option<HoverElement<'a>> {
        let mut closest_value = None;
        let mut closest_item = None;
        let mut closest_dist_sq = f32::MAX;
        for value in &self.series.values {
            let pos = transform.position_from_value(value);
            let dist_sq = pointer.distance_sq(pos);
            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                closest_value = Some(value);
                closest_item = Some(self.name());
            }
        }
        closest_value
            .zip(closest_item)
            .map(move |(value, name)| HoverElement {
                distance_square: closest_dist_sq,
                hover_shapes: Box::new(move |mut shapes| {
                    let line_color = if ui.visuals().dark_mode {
                        Color32::from_gray(100).additive()
                    } else {
                        Color32::from_black_alpha(180)
                    };

                    let position = transform.position_from_value(value);
                    shapes.push(Shape::circle_filled(position, 3.0, line_color));

                    rulers_at_value(
                        ui,
                        position,
                        transform,
                        show_x,
                        show_y,
                        *value,
                        name,
                        &mut shapes,
                    );
                }),
            })
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        self.stroke.color
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }
}

/// A set of points.
pub struct Points {
    pub(super) series: Values,
    pub(super) shape: MarkerShape,
    /// Color of the marker. `Color32::TRANSPARENT` means that it will be picked automatically.
    pub(super) color: Color32,
    /// Whether to fill the marker. Does not apply to all types.
    pub(super) filled: bool,
    /// The maximum extent of the marker from its center.
    pub(super) radius: f32,
    pub(super) name: String,
    pub(super) highlight: bool,
}

impl Points {
    pub fn new(series: Values) -> Self {
        Self {
            series,
            shape: MarkerShape::Circle,
            color: Color32::TRANSPARENT,
            filled: true,
            radius: 1.0,
            name: Default::default(),
            highlight: false,
        }
    }

    /// Set the shape of the markers.
    pub fn shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }

    /// Highlight these points in the plot by scaling up their markers.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Set the marker's color.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    /// Whether to fill the marker.
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Set the maximum extent of the marker around its position.
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Name of this set of points.
    ///
    /// This name will show up in the plot legend, if legends are turned on. Multiple sets of points
    /// may share the same name, in which case they will also share an entry in the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for Points {
    fn get_shapes(&self, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let sqrt_3 = 3f32.sqrt();
        let frac_sqrt_3_2 = 3f32.sqrt() / 2.0;
        let frac_1_sqrt_2 = 1.0 / 2f32.sqrt();

        let Self {
            series,
            shape,
            color,
            filled,
            mut radius,
            highlight,
            ..
        } = self;

        if *highlight {
            radius *= 2f32.sqrt();
        }

        let stroke_size = radius / 5.0;

        let default_stroke = Stroke::new(stroke_size, *color);
        let stroke = (!filled).then(|| default_stroke).unwrap_or_default();
        let fill = filled.then(|| *color).unwrap_or_default();

        series
            .values
            .iter()
            .map(|value| transform.position_from_value(value))
            .for_each(|center| {
                let tf = |dx: f32, dy: f32| -> Pos2 { center + radius * vec2(dx, dy) };

                match shape {
                    MarkerShape::Circle => {
                        shapes.push(Shape::Circle {
                            center,
                            radius,
                            fill,
                            stroke,
                        });
                    }
                    MarkerShape::Diamond => {
                        let points = vec![tf(1.0, 0.0), tf(0.0, -1.0), tf(-1.0, 0.0), tf(0.0, 1.0)];
                        shapes.push(Shape::Path {
                            points,
                            closed: true,
                            fill,
                            stroke,
                        });
                    }
                    MarkerShape::Square => {
                        let points = vec![
                            tf(frac_1_sqrt_2, frac_1_sqrt_2),
                            tf(frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(-frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(-frac_1_sqrt_2, frac_1_sqrt_2),
                        ];
                        shapes.push(Shape::Path {
                            points,
                            closed: true,
                            fill,
                            stroke,
                        });
                    }
                    MarkerShape::Cross => {
                        let diagonal1 = [
                            tf(-frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(frac_1_sqrt_2, frac_1_sqrt_2),
                        ];
                        let diagonal2 = [
                            tf(frac_1_sqrt_2, -frac_1_sqrt_2),
                            tf(-frac_1_sqrt_2, frac_1_sqrt_2),
                        ];
                        shapes.push(Shape::line_segment(diagonal1, default_stroke));
                        shapes.push(Shape::line_segment(diagonal2, default_stroke));
                    }
                    MarkerShape::Plus => {
                        let horizontal = [tf(-1.0, 0.0), tf(1.0, 0.0)];
                        let vertical = [tf(0.0, -1.0), tf(0.0, 1.0)];
                        shapes.push(Shape::line_segment(horizontal, default_stroke));
                        shapes.push(Shape::line_segment(vertical, default_stroke));
                    }
                    MarkerShape::Up => {
                        let points =
                            vec![tf(0.0, -1.0), tf(-0.5 * sqrt_3, 0.5), tf(0.5 * sqrt_3, 0.5)];
                        shapes.push(Shape::Path {
                            points,
                            closed: true,
                            fill,
                            stroke,
                        });
                    }
                    MarkerShape::Down => {
                        let points = vec![
                            tf(0.0, 1.0),
                            tf(-0.5 * sqrt_3, -0.5),
                            tf(0.5 * sqrt_3, -0.5),
                        ];
                        shapes.push(Shape::Path {
                            points,
                            closed: true,
                            fill,
                            stroke,
                        });
                    }
                    MarkerShape::Left => {
                        let points =
                            vec![tf(-1.0, 0.0), tf(0.5, -0.5 * sqrt_3), tf(0.5, 0.5 * sqrt_3)];
                        shapes.push(Shape::Path {
                            points,
                            closed: true,
                            fill,
                            stroke,
                        });
                    }
                    MarkerShape::Right => {
                        let points = vec![
                            tf(1.0, 0.0),
                            tf(-0.5, -0.5 * sqrt_3),
                            tf(-0.5, 0.5 * sqrt_3),
                        ];
                        shapes.push(Shape::Path {
                            points,
                            closed: true,
                            fill,
                            stroke,
                        });
                    }
                    MarkerShape::Asterisk => {
                        let vertical = [tf(0.0, -1.0), tf(0.0, 1.0)];
                        let diagonal1 = [tf(-frac_sqrt_3_2, 0.5), tf(frac_sqrt_3_2, -0.5)];
                        let diagonal2 = [tf(-frac_sqrt_3_2, -0.5), tf(frac_sqrt_3_2, 0.5)];
                        shapes.push(Shape::line_segment(vertical, default_stroke));
                        shapes.push(Shape::line_segment(diagonal1, default_stroke));
                        shapes.push(Shape::line_segment(diagonal2, default_stroke));
                    }
                }
            });
    }

    fn series_mut(&mut self) -> &mut Values {
        &mut self.series
    }

    fn get_bounds(&self) -> Bounds {
        self.series.get_bounds()
    }

    fn closest<'a>(
        &'a self,
        ui: &'a Ui,
        pointer: Pos2,
        transform: &'a ScreenTransform,
        show_x: bool,
        show_y: bool,
    ) -> Option<HoverElement<'a>> {
        let mut closest_value = None;
        let mut closest_item = None;
        let mut closest_dist_sq = f32::MAX;
        for value in &self.series.values {
            let pos = transform.position_from_value(value);
            let dist_sq = pointer.distance_sq(pos);
            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                closest_value = Some(value);
                closest_item = Some(self.name());
            }
        }
        closest_value
            .zip(closest_item)
            .map(move |(value, name)| HoverElement {
                distance_square: closest_dist_sq,
                hover_shapes: Box::new(move |mut shapes| {
                    let line_color = if ui.visuals().dark_mode {
                        Color32::from_gray(100).additive()
                    } else {
                        Color32::from_black_alpha(180)
                    };

                    let position = transform.position_from_value(value);
                    shapes.push(Shape::circle_filled(position, 3.0, line_color));

                    rulers_at_value(
                        ui,
                        position,
                        transform,
                        show_x,
                        show_y,
                        *value,
                        name,
                        &mut shapes,
                    );
                }),
            })
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        self.color
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Vertical
    }
}

/// One bar in a bar chart. Potentially floating, allowing stacked bar charts.
/// Width can be changed to allow variable width histograms.
#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    /// Position on the key axis (X if vertical, Y if horizontal)
    pub position: f64,
    pub orientation: Orientation,
    pub name: String,
    pub height: f64,
    pub base_offset: Option<f64>,
    pub width: f64,
    pub stroke: Stroke,
    pub fill: Color32,
}

impl Bar {
    /// Create a bar. Its `orientation` is set by its [[`BarChart`]] parent.
    ///
    /// - `position`: Position on the key axis (X if vertical, Y if horizontal).
    /// - `height`: Height of the bar.
    ///
    /// By default the bar is vertical and its base is at zero.
    pub fn new(position: f64, height: f64) -> Bar {
        Bar {
            position,
            orientation: Orientation::default(),
            name: Default::default(),
            height,
            base_offset: None,
            width: 0.5,
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            fill: Color32::TRANSPARENT,
        }
    }

    /// Name of this bar.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    /// Add a custom stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Add a custom fill color.
    pub fn fill(mut self, color: impl Into<Color32>) -> Self {
        self.fill = color.into();
        self
    }

    /// Offset the base of the bar.
    /// This offset is on the Y axis for a vertical bar
    /// and on the X axis for a horizontal bar.
    pub fn base_offset(mut self, offset: f64) -> Self {
        self.base_offset = Some(offset);
        self
    }

    /// Set the bar width.
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Set orientation of the element as vertical. Key axis is X.
    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    /// Set orientation of the element as horizontal. Key axis is Y.
    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }

    fn point_at(&self, key: f64, value: f64) -> Value {
        match self.orientation {
            Orientation::Horizontal => Value::new(value, key),
            Orientation::Vertical => Value::new(key, value),
        }
    }

    fn lower(&self) -> f64 {
        if self.height.is_sign_positive() {
            self.base_offset.unwrap_or(0.0)
        } else {
            self.base_offset
                .map(|o| o + self.height)
                .unwrap_or(self.height)
        }
    }

    fn upper(&self) -> f64 {
        if self.height.is_sign_positive() {
            self.base_offset
                .map(|o| o + self.height)
                .unwrap_or(self.height)
        } else {
            self.base_offset.unwrap_or(0.0)
        }
    }

    fn bounds_min(&self) -> Value {
        self.point_at(self.position - self.width / 2.0, self.lower())
    }

    fn bounds_max(&self) -> Value {
        self.point_at(self.position + self.width / 2.0, self.upper())
    }

    fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        bounds.extend_with(&self.bounds_min());
        bounds.extend_with(&self.bounds_max());
        bounds
    }

    fn shapes(&self, transform: &ScreenTransform, highlighted: bool, shapes: &mut Vec<Shape>) {
        let (stroke, fill) = if highlighted {
            highlighted_color(self.stroke, self.fill)
        } else {
            (self.stroke, self.fill)
        };

        let rect = transform.rect_from_values(&self.bounds_min(), &self.bounds_max());
        let rect = Shape::Rect {
            rect,
            corner_radius: 0.0,
            fill,
            stroke,
        };
        shapes.push(rect);
    }

    fn default_values_format(&self, transform: &ScreenTransform) -> String {
        let scale = transform.dvalue_dpos();
        let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
        format!("\n{:.*}", y_decimals, self.height)
    }

    fn rulers(
        &self,
        parent: &BarChart,
        ui: &Ui,
        transform: &ScreenTransform,
        show_x: bool,
        show_y: bool,
        shapes: &mut Vec<Shape>,
    ) {
        let value = self
            .base_offset
            .map(|o| o + self.height)
            .unwrap_or(self.height);
        let value_center = self.point_at(self.position, value);
        let value_right_end = self.point_at(self.position + self.width / 2.0, value);

        let show_position = show_x && self.orientation == Orientation::Vertical
            || show_y && self.orientation == Orientation::Horizontal;
        let show_values = show_y && self.orientation == Orientation::Vertical
            || show_x && self.orientation == Orientation::Horizontal;

        let line_color = rulers_color(ui);
        if show_position {
            let upper_pos = transform.position_from_value(&value_center);
            let line = match self.orientation {
                Orientation::Horizontal => horizontal_line(upper_pos, transform, line_color),
                Orientation::Vertical => vertical_line(upper_pos, transform, line_color),
            };
            shapes.push(line);
        }

        let push_value_ruler = |value: Value, shapes: &mut Vec<Shape>| {
            let position = transform.position_from_value(&value);
            let line = match self.orientation {
                Orientation::Horizontal => vertical_line(position, transform, line_color),
                Orientation::Vertical => horizontal_line(position, transform, line_color),
            };
            shapes.push(line);
        };

        if show_values {
            push_value_ruler(value_center, shapes);
            if self.base_offset.is_some() {
                push_value_ruler(
                    self.point_at(self.position, self.base_offset.unwrap_or(0.0)),
                    shapes,
                );
            }
        }

        let text = match parent.element_formatter {
            None => {
                let mut text = String::new();
                if !self.name.is_empty() {
                    text.push_str(&self.name);
                }

                if show_values {
                    text.push_str(&self.default_values_format(transform));
                }
                text
            }
            Some(ref formatter) => formatter(self, parent),
        };

        shapes.push(Shape::text(
            ui.fonts(),
            transform.position_from_value(&value_right_end) + vec2(3.0, -2.0),
            Align2::LEFT_BOTTOM,
            text,
            TextStyle::Body,
            ui.visuals().text_color(),
        ));
    }
}

/// A bar chart.
pub struct BarChart {
    pub(super) bars: Vec<Bar>,
    pub(super) default_color: Color32,
    pub name: String,
    /// A custom element formatter
    element_formatter: Option<Box<dyn Fn(&Bar, &BarChart) -> String>>,
    highlight: bool,
    dummy_values: Values,
}

impl BarChart {
    /// Create a bar chart. It defaults to vertically oriented elements.
    pub fn new(bars: Vec<Bar>) -> BarChart {
        BarChart {
            bars,
            default_color: Color32::TRANSPARENT,
            name: String::new(),
            element_formatter: None,
            highlight: false,
            dummy_values: Values::default(),
        }
    }

    /// Set the default color. It is set on all elements that do not already have a specific color.
    /// This is the color that shows up in the legend.
    /// It can be overridden at the bar level (see [[`Bar`]]).
    /// Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        let plot_color = color.into();
        self.default_color = plot_color;
        self.bars.iter_mut().for_each(|b| {
            if b.fill == Color32::TRANSPARENT && b.stroke.color == Color32::TRANSPARENT {
                b.fill = plot_color.linear_multiply(0.2);
                b.stroke.color = plot_color;
            }
        });
        self
    }

    /// Name of this chart.
    ///
    /// This name will show up in the plot legend, if legends are turned on. Multiple charts may
    /// share the same name, in which case they will also share an entry in the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set all elements to be in a vertical orientation.
    /// Key axis will be X and bar values will be on the Y axis.
    pub fn vertical(mut self) -> Self {
        self.bars.iter_mut().for_each(|b| {
            b.orientation = Orientation::Vertical;
        });
        self
    }

    /// Set all elements to be in a horizontal orientation.
    /// Key axis will be Y and bar values will be on the X axis.
    pub fn horizontal(mut self) -> Self {
        self.bars.iter_mut().for_each(|b| {
            b.orientation = Orientation::Horizontal;
        });
        self
    }

    /// Set the width of all its elements.
    pub fn width(mut self, width: f64) -> Self {
        self.bars.iter_mut().for_each(|b| {
            b.width = width;
        });
        self
    }

    /// Highlight all plot elements.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Add a custom way to format an element.
    /// Can be used to display a set number of decimals or custom labels.
    pub fn element_formatter(mut self, formatter: Box<dyn Fn(&Bar, &BarChart) -> String>) -> Self {
        self.element_formatter = Some(formatter);
        self
    }

    /// Stacks the bars on top of another chart.
    /// Positive values are stacked on top of other positive values.
    /// Negative values are stacked below other negative values.
    pub fn stack_on(mut self, others: &[&BarChart]) -> Self {
        for (index, mut bar) in self.bars.iter_mut().enumerate() {
            if bar.height.is_sign_positive() {
                let mut max = 0.0;
                for other_chart in others {
                    if let Some(other_bar) = other_chart.bars.get(index) {
                        let other_upper = other_bar.upper();
                        if other_upper > max {
                            max = other_upper;
                        }
                    }
                }
                if max > 0.0 {
                    bar.base_offset = Some(max);
                }
            } else {
                let mut min = 0.0;
                for other_chart in others {
                    if let Some(other_bar) = other_chart.bars.get(index) {
                        let other_lower = other_bar.lower();
                        if other_lower < min {
                            min = other_lower;
                        }
                    }
                }
                if min.abs() > 0.0 {
                    bar.base_offset = Some(min);
                }
            }
        }
        self
    }
}

impl PlotItem for BarChart {
    fn get_shapes(&self, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        self.bars.iter().for_each(|b| {
            b.shapes(transform, self.highlight, shapes);
        });
    }

    fn series_mut(&mut self) -> &mut Values {
        &mut self.dummy_values
    }

    fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        self.bars.iter().for_each(|b| {
            bounds.merge(&b.bounds());
        });
        bounds
    }

    fn closest<'a>(
        &'a self,
        ui: &'a Ui,
        pointer: Pos2,
        transform: &'a ScreenTransform,
        show_x: bool,
        show_y: bool,
    ) -> Option<HoverElement<'a>> {
        let mut closest = None;
        let mut closest_dist_sq = f32::MAX;
        for bar in &self.bars {
            let box_rect: Rect = transform.rect_from_values(&bar.bounds_min(), &bar.bounds_max());
            let dist_sq = pointer.distance_from_rect_sq(box_rect);
            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                closest = Some(bar);
            }
        }
        closest.map(move |bar| HoverElement {
            distance_square: closest_dist_sq,
            hover_shapes: Box::new(move |mut shapes| {
                bar.shapes(transform, true, &mut shapes);
                bar.rulers(self, ui, transform, show_x, show_y, &mut shapes);
            }),
        })
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        self.default_color
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }
}

/// A boxplot. This is a low level element, it will not compute quartiles and whiskers,
/// letting one use their preferred formula. Use [[`Points`]] to draw the outliers.
#[derive(Clone, Debug, PartialEq)]
pub struct Boxplot {
    /// Position on the key axis (X if vertical, Y if horizontal)
    pub position: f64,
    pub orientation: Orientation,
    pub name: String,
    pub lower_whisker: f64,
    pub quartile1: f64,
    pub median: f64,
    pub quartile3: f64,
    pub upper_whisker: f64,
    pub box_width: f64,
    pub whisker_width: f64,
    pub stroke: Stroke,
    pub fill: Color32,
}

impl Boxplot {
    /// Create a boxplot. Its `orientation` is set by its [[`BoxplotSeries`]] parent.
    ///
    /// - `position`: Position on the key axis (X if vertical, Y if horizontal).
    /// - `lower_whisker`: Value of the whisker with lowest value. The whisker is not drawn if `lower_whisker >= quartile1`.
    /// - `quartile1`: Value of the side of the box with lowest value.
    /// - `median`: Value of the middle bar inside the box.
    /// - `quartile3`: Value of the side of the box with highest value.
    /// - `upper_whisker`: Value of the whisker with highest value. The whisker is not drawn if `upper_whisker <= quartile3`.
    pub fn new(
        position: f64,
        lower_whisker: f64,
        quartile1: f64,
        median: f64,
        quartile3: f64,
        upper_whisker: f64,
    ) -> Boxplot {
        Boxplot {
            position,
            orientation: Orientation::default(),
            name: Default::default(),
            lower_whisker,
            quartile1,
            median,
            quartile3,
            upper_whisker,
            box_width: 0.25,
            whisker_width: 0.15,
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            fill: Color32::TRANSPARENT,
        }
    }

    /// Name of this boxplot.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    /// Add a custom stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Add a custom fill color.
    pub fn fill(mut self, color: impl Into<Color32>) -> Self {
        self.fill = color.into();
        self
    }

    /// Set the box width.
    pub fn box_width(mut self, width: f64) -> Self {
        self.box_width = width;
        self
    }

    /// Set the whisker width.
    pub fn whisker_width(mut self, width: f64) -> Self {
        self.whisker_width = width;
        self
    }

    /// Set orientation of the element as vertical. Key axis is X.
    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }

    /// Set orientation of the element as horizontal. Key axis is Y.
    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }

    fn point_at(&self, key: f64, value: f64) -> Value {
        match self.orientation {
            Orientation::Horizontal => Value::new(value, key),
            Orientation::Vertical => Value::new(key, value),
        }
    }

    fn bounds_min(&self) -> Value {
        let key = self.position - self.box_width.max(self.whisker_width) / 2.0;
        let value = self.lower_whisker;
        self.point_at(key, value)
    }

    fn bounds_max(&self) -> Value {
        let key = self.position + self.box_width.max(self.whisker_width) / 2.0;
        let value = self.upper_whisker;
        self.point_at(key, value)
    }

    fn bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        bounds.extend_with(&self.bounds_min());
        bounds.extend_with(&self.bounds_max());
        bounds
    }

    fn shapes(&self, transform: &ScreenTransform, highlighted: bool, shapes: &mut Vec<Shape>) {
        let (stroke, fill) = if highlighted {
            highlighted_color(self.stroke, self.fill)
        } else {
            (self.stroke, self.fill)
        };
        let rect = transform.rect_from_values(
            &self.point_at(self.position - self.box_width / 2.0, self.quartile1),
            &self.point_at(self.position + self.box_width / 2.0, self.quartile3),
        );
        let rect = Shape::Rect {
            rect,
            corner_radius: 0.0,
            fill,
            stroke,
        };
        shapes.push(rect);
        let line_between = |v1, v2| {
            Shape::line_segment(
                [
                    transform.position_from_value(&v1),
                    transform.position_from_value(&v2),
                ],
                stroke,
            )
        };
        let median = line_between(
            self.point_at(self.position - self.box_width / 2.0, self.median),
            self.point_at(self.position + self.box_width / 2.0, self.median),
        );
        shapes.push(median);
        if self.upper_whisker > self.quartile3 {
            let high_whisker = line_between(
                self.point_at(self.position, self.quartile3),
                self.point_at(self.position, self.upper_whisker),
            );
            shapes.push(high_whisker);
            if self.box_width > 0.0 {
                let high_whisker_end = line_between(
                    self.point_at(self.position - self.whisker_width / 2.0, self.upper_whisker),
                    self.point_at(self.position + self.whisker_width / 2.0, self.upper_whisker),
                );
                shapes.push(high_whisker_end);
            }
        }
        if self.lower_whisker < self.quartile1 {
            let low_whisker = line_between(
                self.point_at(self.position, self.quartile1),
                self.point_at(self.position, self.lower_whisker),
            );
            shapes.push(low_whisker);
            if self.box_width > 0.0 {
                let low_whisker_end = line_between(
                    self.point_at(self.position - self.whisker_width / 2.0, self.lower_whisker),
                    self.point_at(self.position + self.whisker_width / 2.0, self.lower_whisker),
                );
                shapes.push(low_whisker_end);
            }
        }
    }

    fn default_values_format(&self, transform: &ScreenTransform) -> String {
        let scale = transform.dvalue_dpos();
        let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
        format!(
            "\nMax = {max:.decimals$}\
             \nQuartile 3 = {q3:.decimals$}\
             \nMedian = {med:.decimals$}\
             \nQuartile 1 = {q1:.decimals$}\
             \nMin = {min:.decimals$}",
            max = self.upper_whisker,
            q3 = self.quartile3,
            med = self.median,
            q1 = self.quartile1,
            min = self.lower_whisker,
            decimals = y_decimals
        )
    }

    fn rulers(
        &self,
        parent: &BoxplotSeries,
        ui: &Ui,
        transform: &ScreenTransform,
        show_x: bool,
        show_y: bool,
        shapes: &mut Vec<Shape>,
    ) {
        let median = self.point_at(self.position, self.median);
        let q1 = self.point_at(self.position, self.quartile1);
        let q3 = self.point_at(self.position, self.quartile3);
        let upper = self.point_at(self.position, self.upper_whisker);
        let lower = self.point_at(self.position, self.lower_whisker);

        let show_position = show_x && self.orientation == Orientation::Vertical
            || show_y && self.orientation == Orientation::Horizontal;
        let show_values = show_y && self.orientation == Orientation::Vertical
            || show_x && self.orientation == Orientation::Horizontal;

        let line_color = rulers_color(ui);
        if show_position {
            let median = transform.position_from_value(&median);
            let line = match self.orientation {
                Orientation::Horizontal => horizontal_line(median, transform, line_color),
                Orientation::Vertical => vertical_line(median, transform, line_color),
            };
            shapes.push(line);
        }

        let push_value_ruler = |value: Value, shapes: &mut Vec<Shape>| {
            let position = transform.position_from_value(&value);
            let line = match self.orientation {
                Orientation::Horizontal => vertical_line(position, transform, line_color),
                Orientation::Vertical => horizontal_line(position, transform, line_color),
            };
            shapes.push(line);
        };

        if show_values {
            push_value_ruler(median, shapes);
            push_value_ruler(q1, shapes);
            push_value_ruler(q3, shapes);
            push_value_ruler(upper, shapes);
            push_value_ruler(lower, shapes);
        }

        let text = match parent.element_formatter {
            None => {
                let mut text = String::new();
                if !self.name.is_empty() {
                    text.push_str(&self.name);
                }

                if show_values {
                    text.push_str(&self.default_values_format(transform));
                }
                text
            }
            Some(ref formatter) => formatter(self, parent),
        };

        shapes.push(Shape::text(
            ui.fonts(),
            transform.position_from_value(&upper) + vec2(3.0, -2.0),
            Align2::LEFT_BOTTOM,
            text,
            TextStyle::Body,
            ui.visuals().text_color(),
        ));
    }
}

/// A series of boxplots.
pub struct BoxplotSeries {
    pub(super) plots: Vec<Boxplot>,
    pub(super) default_color: Color32,
    pub name: String,
    /// A custom element formatter
    element_formatter: Option<Box<dyn Fn(&Boxplot, &BoxplotSeries) -> String>>,
    highlight: bool,
    dummy_values: Values,
}

impl BoxplotSeries {
    /// Create a series of boxplots. It defaults to vertically oriented elements.
    pub fn new(plots: Vec<Boxplot>) -> BoxplotSeries {
        BoxplotSeries {
            plots,
            default_color: Color32::TRANSPARENT,
            name: String::new(),
            element_formatter: None,
            highlight: false,
            dummy_values: Values::default(),
        }
    }

    /// Set the default color. It is set on all elements that do not already have a specific color.
    /// This is the color that shows up in the legend.
    /// It can be overridden at the boxplot level (see [[`Boxplot`]]).
    /// Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        let plot_color = color.into();
        self.default_color = plot_color;
        self.plots.iter_mut().for_each(|boxplot| {
            if boxplot.fill == Color32::TRANSPARENT && boxplot.stroke.color == Color32::TRANSPARENT
            {
                boxplot.fill = plot_color.linear_multiply(0.2);
                boxplot.stroke.color = plot_color;
            }
        });
        self
    }

    /// Name of this series of boxplots.
    ///
    /// This name will show up in the plot legend, if legends are turned on. Multiple series may
    /// share the same name, in which case they will also share an entry in the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set all elements to be in a vertical orientation.
    /// Key axis will be X and values will be on the Y axis.
    pub fn vertical(mut self) -> Self {
        self.plots.iter_mut().for_each(|boxplot| {
            boxplot.orientation = Orientation::Vertical;
        });
        self
    }

    /// Set all elements to be in a horizontal orientation.
    /// Key axis will be Y and values will be on the X axis.
    pub fn horizontal(mut self) -> Self {
        self.plots.iter_mut().for_each(|boxplot| {
            boxplot.orientation = Orientation::Horizontal;
        });
        self
    }

    /// Highlight all plot elements.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Add a custom way to format an element.
    /// Can be used to display a set number of decimals or custom labels.
    pub fn element_formatter(
        mut self,
        formatter: Box<dyn Fn(&Boxplot, &BoxplotSeries) -> String>,
    ) -> Self {
        self.element_formatter = Some(formatter);
        self
    }
}

impl PlotItem for BoxplotSeries {
    fn get_shapes(&self, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        self.plots.iter().for_each(|b| {
            b.shapes(transform, self.highlight, shapes);
        });
    }

    fn series_mut(&mut self) -> &mut Values {
        &mut self.dummy_values
    }

    fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        self.plots.iter().for_each(|b| {
            bounds.merge(&b.bounds());
        });
        bounds
    }

    fn closest<'a>(
        &'a self,
        ui: &'a Ui,
        pointer: Pos2,
        transform: &'a ScreenTransform,
        show_x: bool,
        show_y: bool,
    ) -> Option<HoverElement<'a>> {
        let mut closest = None;
        let mut closest_dist_sq = f32::MAX;
        for boxplot in &self.plots {
            let box_rect: Rect =
                transform.rect_from_values(&boxplot.bounds_min(), &boxplot.bounds_max());
            let dist_sq = pointer.distance_from_rect_sq(box_rect);
            if dist_sq < closest_dist_sq {
                closest_dist_sq = dist_sq;
                closest = Some(boxplot);
            }
        }
        closest.map(move |boxplot| HoverElement {
            distance_square: closest_dist_sq,
            hover_shapes: Box::new(move |mut shapes| {
                boxplot.shapes(transform, true, &mut shapes);
                boxplot.rulers(self, ui, transform, show_x, show_y, &mut shapes);
            }),
        })
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        self.default_color
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }
}

pub(crate) struct HoverElement<'a> {
    pub(crate) distance_square: f32,
    // Note: the Box<dyn Fn> here is a compromise between an owned Vec<Shape>
    //       (overhead of precalculating the shapes) and an impl Fn
    //       (typing all the way up to PlotItem with trait object safety workarounds)
    pub(crate) hover_shapes: Box<dyn Fn(&mut Vec<Shape>) + 'a>,
}

fn highlighted_color(mut stroke: Stroke, fill: Color32) -> (Stroke, Color32) {
    stroke.width *= 2.0;
    let fill = Rgba::from(fill);
    let fill_alpha = (2.0 * fill.a()).at_most(1.0);
    let fill = fill.to_opaque().multiply(fill_alpha);
    (stroke, fill.into())
}

fn rulers_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_gray(100).additive()
    } else {
        Color32::from_black_alpha(180)
    }
}

fn vertical_line(pointer: Pos2, transform: &ScreenTransform, line_color: Color32) -> Shape {
    let frame = transform.frame();
    Shape::line_segment(
        [
            pos2(pointer.x, frame.top()),
            pos2(pointer.x, frame.bottom()),
        ],
        (1.0, line_color),
    )
}

fn horizontal_line(pointer: Pos2, transform: &ScreenTransform, line_color: Color32) -> Shape {
    let frame = transform.frame();
    Shape::line_segment(
        [
            pos2(frame.left(), pointer.y),
            pos2(frame.right(), pointer.y),
        ],
        (1.0, line_color),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rulers_at_value(
    ui: &Ui,
    pointer: Pos2,
    transform: &ScreenTransform,
    show_x: bool,
    show_y: bool,
    value: Value,
    name: &str,
    shapes: &mut Vec<Shape>,
) {
    let line_color = rulers_color(ui);
    if show_x {
        shapes.push(vertical_line(pointer, transform, line_color));
    }
    if show_y {
        shapes.push(horizontal_line(pointer, transform, line_color));
    }

    let mut prefix = String::new();

    if !name.is_empty() {
        prefix = format!("{}\n", name);
    }

    let text = {
        let scale = transform.dvalue_dpos();
        let x_decimals = ((-scale[0].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
        let y_decimals = ((-scale[1].abs().log10()).ceil().at_least(0.0) as usize).at_most(6);
        if show_x && show_y {
            format!(
                "{}x = {:.*}\ny = {:.*}",
                prefix, x_decimals, value.x, y_decimals, value.y
            )
        } else if show_x {
            format!("{}x = {:.*}", prefix, x_decimals, value.x)
        } else if show_y {
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
