//! Contains items that can be added to a plot.

use std::ops::{Bound, RangeBounds, RangeInclusive};

use epaint::{text::TextColorMap, Mesh};

use super::transform::{Bounds, ScreenTransform};
use crate::*;

const DEFAULT_FILL_ALPHA: f32 = 0.05;

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LineStyle {
    Solid,
    Dotted { spacing: f32 },
    Dashed { length: f32 },
}

impl LineStyle {
    pub fn dashed_loose() -> Self {
        Self::Dashed { length: 10.0 }
    }

    pub fn dashed_dense() -> Self {
        Self::Dashed { length: 5.0 }
    }

    pub fn dotted_loose() -> Self {
        Self::Dotted { spacing: 10.0 }
    }

    pub fn dotted_dense() -> Self {
        Self::Dotted { spacing: 5.0 }
    }

    fn style_line(
        &self,
        line: Vec<Pos2>,
        mut stroke: Stroke,
        highlight: bool,
        shapes: &mut Vec<Shape>,
    ) {
        match line.len() {
            0 => {}
            1 => {
                let mut radius = stroke.width / 2.0;
                if highlight {
                    radius *= 2f32.sqrt();
                }
                shapes.push(Shape::circle_filled(line[0], radius, stroke.color));
            }
            _ => {
                match self {
                    LineStyle::Solid => {
                        if highlight {
                            stroke.width *= 2.0;
                        }
                        shapes.push(Shape::line(line, stroke));
                    }
                    LineStyle::Dotted { spacing } => {
                        // Take the stroke width for the radius even though it's not "correct", otherwise
                        // the dots would become too small.
                        let mut radius = stroke.width;
                        if highlight {
                            radius *= 2f32.sqrt();
                        }
                        shapes.extend(Shape::dotted_line(&line, stroke.color, *spacing, radius))
                    }
                    LineStyle::Dashed { length } => {
                        if highlight {
                            stroke.width *= 2.0;
                        }
                        let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
                        shapes.extend(Shape::dashed_line(
                            &line,
                            stroke,
                            *length,
                            length * golden_ratio,
                        ))
                    }
                }
            }
        }
    }
}

impl ToString for LineStyle {
    fn to_string(&self) -> String {
        match self {
            LineStyle::Solid => "Solid".into(),
            LineStyle::Dotted { spacing } => format!("Dotted{}Px", spacing),
            LineStyle::Dashed { length } => format!("Dashed{}Px", length),
        }
    }
}

// ----------------------------------------------------------------------------

/// A horizontal line in a plot, filling the full width
#[derive(Clone, Debug, PartialEq)]
pub struct HLine {
    pub(super) y: f64,
    pub(super) stroke: Stroke,
    pub(super) name: String,
    pub(super) highlight: bool,
    pub(super) style: LineStyle,
}

impl HLine {
    pub fn new(y: impl Into<f64>) -> Self {
        Self {
            y: y.into(),
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: String::default(),
            highlight: false,
            style: LineStyle::Solid,
        }
    }

    /// Highlight this line in the plot by scaling up the line.
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
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.stroke.width = width.into();
        self
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Set the line's style. Default is `LineStyle::Solid`.
    pub fn style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Name of this horizontal line.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for HLine {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let HLine {
            y,
            stroke,
            highlight,
            style,
            ..
        } = self;
        let points = vec![
            transform.position_from_value(&Value::new(transform.bounds().min[0], *y)),
            transform.position_from_value(&Value::new(transform.bounds().max[0], *y)),
        ];
        style.style_line(points, *stroke, *highlight, shapes);
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn name(&self) -> &str {
        &self.name
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

    fn values(&self) -> Option<&Values> {
        None
    }

    fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        bounds.min[1] = self.y;
        bounds.max[1] = self.y;
        bounds
    }
}

/// A vertical line in a plot, filling the full width
#[derive(Clone, Debug, PartialEq)]
pub struct VLine {
    pub(super) x: f64,
    pub(super) stroke: Stroke,
    pub(super) name: String,
    pub(super) highlight: bool,
    pub(super) style: LineStyle,
}

impl VLine {
    pub fn new(x: impl Into<f64>) -> Self {
        Self {
            x: x.into(),
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: String::default(),
            highlight: false,
            style: LineStyle::Solid,
        }
    }

    /// Highlight this line in the plot by scaling up the line.
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
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.stroke.width = width.into();
        self
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Set the line's style. Default is `LineStyle::Solid`.
    pub fn style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Name of this vertical line.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for VLine {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let VLine {
            x,
            stroke,
            highlight,
            style,
            ..
        } = self;
        let points = vec![
            transform.position_from_value(&Value::new(*x, transform.bounds().min[1])),
            transform.position_from_value(&Value::new(*x, transform.bounds().max[1])),
        ];
        style.style_line(points, *stroke, *highlight, shapes)
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn name(&self) -> &str {
        &self.name
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

    fn values(&self) -> Option<&Values> {
        None
    }

    fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        bounds.min[0] = self.x;
        bounds.max[0] = self.x;
        bounds
    }
}

/// Trait shared by things that can be drawn in the plot.
pub(super) trait PlotItem {
    fn get_shapes(&self, ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>);
    fn initialize(&mut self, x_range: RangeInclusive<f64>);
    fn name(&self) -> &str;
    fn color(&self) -> Color32;
    fn highlight(&mut self);
    fn highlighted(&self) -> bool;
    fn values(&self) -> Option<&Values>;
    fn get_bounds(&self) -> Bounds;
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
        x_range: impl RangeBounds<f64>,
        points: usize,
    ) -> Self {
        let start = match x_range.start_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => f64::NEG_INFINITY,
        };
        let end = match x_range.end_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => f64::INFINITY,
        };
        let x_range = start..=end;

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
    /// The range may be specified as start..end or as start..=end.
    pub fn from_parametric_callback(
        function: impl Fn(f64) -> (f64, f64),
        t_range: impl RangeBounds<f64>,
        points: usize,
    ) -> Self {
        let start = match t_range.start_bound() {
            Bound::Included(x) => x,
            Bound::Excluded(_) => unreachable!(),
            Bound::Unbounded => panic!("The range for parametric functions must be bounded!"),
        };
        let end = match t_range.end_bound() {
            Bound::Included(x) | Bound::Excluded(x) => x,
            Bound::Unbounded => panic!("The range for parametric functions must be bounded!"),
        };
        let last_point_included = matches!(t_range.end_bound(), Bound::Included(_));
        let increment = if last_point_included {
            (end - start) / (points - 1) as f64
        } else {
            (end - start) / points as f64
        };
        let values = (0..points).map(|i| {
            let t = start + i as f64 * increment;
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
    pub fn all() -> impl Iterator<Item = MarkerShape> {
        [
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
        .iter()
        .copied()
    }
}

/// A series of values forming a path.
pub struct Line {
    pub(super) series: Values,
    pub(super) stroke: Stroke,
    pub(super) name: String,
    pub(super) highlight: bool,
    pub(super) fill: Option<f32>,
    pub(super) style: LineStyle,
}

impl Line {
    pub fn new(series: Values) -> Self {
        Self {
            series,
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: Default::default(),
            highlight: false,
            fill: None,
            style: LineStyle::Solid,
        }
    }

    /// Highlight this line in the plot by scaling up the line.
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
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.stroke.width = width.into();
        self
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Fill the area between this line and a given horizontal reference line.
    pub fn fill(mut self, y_reference: impl Into<f32>) -> Self {
        self.fill = Some(y_reference.into());
        self
    }

    /// Set the line's style. Default is `LineStyle::Solid`.
    pub fn style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Name of this line.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

/// Returns the x-coordinate of a possible intersection between a line segment from `p1` to `p2` and
/// a horizontal line at the given y-coordinate.
fn y_intersection(p1: &Pos2, p2: &Pos2, y: f32) -> Option<f32> {
    ((p1.y > y && p2.y < y) || (p1.y < y && p2.y > y))
        .then(|| ((y * (p1.x - p2.x)) - (p1.x * p2.y - p1.y * p2.x)) / (p1.y - p2.y))
}

impl PlotItem for Line {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let Self {
            series,
            stroke,
            highlight,
            mut fill,
            style,
            ..
        } = self;

        let values_tf: Vec<_> = series
            .values
            .iter()
            .map(|v| transform.position_from_value(v))
            .collect();
        let n_values = values_tf.len();

        // Fill the area between the line and a reference line, if required.
        if n_values < 2 {
            fill = None;
        }
        if let Some(y_reference) = fill {
            let mut fill_alpha = DEFAULT_FILL_ALPHA;
            if *highlight {
                fill_alpha = (2.0 * fill_alpha).at_most(1.0);
            }
            let y = transform
                .position_from_value(&Value::new(0.0, y_reference))
                .y;
            let fill_color = Rgba::from(stroke.color)
                .to_opaque()
                .multiply(fill_alpha)
                .into();
            let mut mesh = Mesh::default();
            let expected_intersections = 20;
            mesh.reserve_triangles((n_values - 1) * 2);
            mesh.reserve_vertices(n_values * 2 + expected_intersections);
            values_tf[0..n_values - 1].windows(2).for_each(|w| {
                let i = mesh.vertices.len() as u32;
                mesh.colored_vertex(w[0], fill_color);
                mesh.colored_vertex(pos2(w[0].x, y), fill_color);
                if let Some(x) = y_intersection(&w[0], &w[1], y) {
                    let point = pos2(x, y);
                    mesh.colored_vertex(point, fill_color);
                    mesh.add_triangle(i, i + 1, i + 2);
                    mesh.add_triangle(i + 2, i + 3, i + 4);
                } else {
                    mesh.add_triangle(i, i + 1, i + 2);
                    mesh.add_triangle(i + 1, i + 2, i + 3);
                }
            });
            let last = values_tf[n_values - 1];
            mesh.colored_vertex(last, fill_color);
            mesh.colored_vertex(pos2(last.x, y), fill_color);
            shapes.push(Shape::Mesh(mesh));
        }
        style.style_line(values_tf, *stroke, *highlight, shapes);
    }

    fn initialize(&mut self, x_range: RangeInclusive<f64>) {
        self.series.generate_points(x_range);
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

    fn values(&self) -> Option<&Values> {
        Some(&self.series)
    }

    fn get_bounds(&self) -> Bounds {
        self.series.get_bounds()
    }
}

/// A convex polygon.
pub struct Polygon {
    pub(super) series: Values,
    pub(super) stroke: Stroke,
    pub(super) name: String,
    pub(super) highlight: bool,
    pub(super) fill_alpha: f32,
    pub(super) style: LineStyle,
}

impl Polygon {
    pub fn new(series: Values) -> Self {
        Self {
            series,
            stroke: Stroke::new(1.0, Color32::TRANSPARENT),
            name: Default::default(),
            highlight: false,
            fill_alpha: DEFAULT_FILL_ALPHA,
            style: LineStyle::Solid,
        }
    }

    /// Highlight this polygon in the plot by scaling up the stroke and reducing the fill
    /// transparency.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Add a custom stroke.
    pub fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.stroke = stroke.into();
        self
    }

    /// Set the stroke width.
    pub fn width(mut self, width: impl Into<f32>) -> Self {
        self.stroke.width = width.into();
        self
    }

    /// Stroke color. Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.stroke.color = color.into();
        self
    }

    /// Alpha of the filled area.
    pub fn fill_alpha(mut self, alpha: impl Into<f32>) -> Self {
        self.fill_alpha = alpha.into();
        self
    }

    /// Set the outline's style. Default is `LineStyle::Solid`.
    pub fn style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    /// Name of this polygon.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for Polygon {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let Self {
            series,
            stroke,
            highlight,
            mut fill_alpha,
            style,
            ..
        } = self;

        if *highlight {
            fill_alpha = (2.0 * fill_alpha).at_most(1.0);
        }

        let mut values_tf: Vec<_> = series
            .values
            .iter()
            .map(|v| transform.position_from_value(v))
            .collect();

        let fill = Rgba::from(stroke.color).to_opaque().multiply(fill_alpha);

        let shape = Shape::Path {
            points: values_tf.clone(),
            closed: true,
            fill: fill.into(),
            stroke: Stroke::none(),
        };
        shapes.push(shape);
        values_tf.push(*values_tf.first().unwrap());
        style.style_line(values_tf, *stroke, *highlight, shapes);
    }

    fn initialize(&mut self, x_range: RangeInclusive<f64>) {
        self.series.generate_points(x_range);
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

    fn values(&self) -> Option<&Values> {
        Some(&self.series)
    }

    fn get_bounds(&self) -> Bounds {
        self.series.get_bounds()
    }
}

/// Text inside the plot.
pub struct Text {
    pub(super) text: String,
    pub(super) style: TextStyle,
    pub(super) position: Value,
    pub(super) name: String,
    pub(super) highlight: bool,
    pub(super) color: Color32,
    pub(super) anchor: Align2,
}

impl Text {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(position: Value, text: impl ToString) -> Self {
        Self {
            text: text.to_string(),
            style: TextStyle::Small,
            position,
            name: Default::default(),
            highlight: false,
            color: Color32::TRANSPARENT,
            anchor: Align2::CENTER_CENTER,
        }
    }

    /// Highlight this text in the plot by drawing a rectangle around it.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Text style. Default is `TextStyle::Small`.
    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Text color. Default is `Color32::TRANSPARENT` which means a color will be auto-assigned.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self
    }

    /// Anchor position of the text. Default is `Align2::CENTER_CENTER`.
    pub fn anchor(mut self, anchor: Align2) -> Self {
        self.anchor = anchor;
        self
    }

    /// Name of this text.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for Text {
    fn get_shapes(&self, ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let color = if self.color == Color32::TRANSPARENT {
            ui.style().visuals.text_color()
        } else {
            self.color
        };
        let pos = transform.position_from_value(&self.position);
        let galley = ui
            .fonts()
            .layout_multiline(self.style, self.text.clone(), f32::INFINITY);
        let rect = self
            .anchor
            .anchor_rect(Rect::from_min_size(pos, galley.size));
        shapes.push(Shape::Text {
            pos: rect.min,
            galley,
            color_map: TextColorMap::default(),
            default_color: color,
            fake_italics: false,
        });
        if self.highlight {
            shapes.push(Shape::rect_stroke(
                rect.expand(2.0),
                1.0,
                Stroke::new(0.5, color),
            ));
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

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

    fn values(&self) -> Option<&Values> {
        None
    }

    fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        bounds.extend_with(&self.position);
        bounds
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
    pub(super) stems: Option<f32>,
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
            stems: None,
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
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self
    }

    /// Whether to fill the marker.
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Whether to add stems between the markers and a horizontal reference line.
    pub fn stems(mut self, y_reference: impl Into<f32>) -> Self {
        self.stems = Some(y_reference.into());
        self
    }

    /// Set the maximum extent of the marker around its position.
    pub fn radius(mut self, radius: impl Into<f32>) -> Self {
        self.radius = radius.into();
        self
    }

    /// Name of this set of points.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for Points {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
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
            stems,
            ..
        } = self;

        let stroke_size = radius / 5.0;

        let default_stroke = Stroke::new(stroke_size, *color);
        let mut stem_stroke = default_stroke;
        let stroke = (!filled)
            .then(|| default_stroke)
            .unwrap_or_else(Stroke::none);
        let fill = filled.then(|| *color).unwrap_or_default();

        if *highlight {
            radius *= 2f32.sqrt();
            stem_stroke.width *= 2.0;
        }

        let y_reference =
            stems.map(|y| transform.position_from_value(&Value::new(0.0, y)).y as f32);

        series
            .values
            .iter()
            .map(|value| transform.position_from_value(value))
            .for_each(|center| {
                let tf = |dx: f32, dy: f32| -> Pos2 { center + radius * vec2(dx, dy) };

                if let Some(y) = y_reference {
                    let stem = Shape::line_segment([center, pos2(center.x, y)], stem_stroke);
                    shapes.push(stem);
                }

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

    fn initialize(&mut self, x_range: RangeInclusive<f64>) {
        self.series.generate_points(x_range);
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

    fn values(&self) -> Option<&Values> {
        Some(&self.series)
    }

    fn get_bounds(&self) -> Bounds {
        self.series.get_bounds()
    }
}

/// A set of arrows.
pub struct Arrows {
    pub(super) origins: Values,
    pub(super) tips: Values,
    pub(super) color: Color32,
    pub(super) name: String,
    pub(super) highlight: bool,
}

impl Arrows {
    pub fn new(origins: Values, tips: Values) -> Self {
        Self {
            origins,
            tips,
            color: Color32::TRANSPARENT,
            name: Default::default(),
            highlight: false,
        }
    }

    /// Highlight these arrows in the plot.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Set the arrows' color.
    pub fn color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self
    }

    /// Name of this set of arrows.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for Arrows {
    fn get_shapes(&self, _ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        use crate::emath::*;
        let Self {
            origins,
            tips,
            color,
            highlight,
            ..
        } = self;
        let stroke = Stroke::new(if *highlight { 2.0 } else { 1.0 }, *color);
        origins
            .values
            .iter()
            .zip(tips.values.iter())
            .map(|(origin, tip)| {
                (
                    transform.position_from_value(origin),
                    transform.position_from_value(tip),
                )
            })
            .for_each(|(origin, tip)| {
                let vector = tip - origin;
                let rot = Rot2::from_angle(std::f32::consts::TAU / 10.0);
                let tip_length = vector.length() / 4.0;
                let tip = origin + vector;
                let dir = vector.normalized();
                shapes.push(Shape::line_segment([origin, tip], stroke));
                shapes.push(Shape::line(
                    vec![
                        tip - tip_length * (rot.inverse() * dir),
                        tip,
                        tip - tip_length * (rot * dir),
                    ],
                    stroke,
                ));
            });
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {
        self.origins
            .generate_points(f64::NEG_INFINITY..=f64::INFINITY);
        self.tips.generate_points(f64::NEG_INFINITY..=f64::INFINITY);
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

    fn values(&self) -> Option<&Values> {
        Some(&self.origins)
    }

    fn get_bounds(&self) -> Bounds {
        self.origins.get_bounds()
    }
}

/// An image in the plot.
pub struct PlotImage {
    pub(super) position: Value,
    pub(super) texture_id: TextureId,
    pub(super) uv: Rect,
    pub(super) size: Vec2,
    pub(super) bg_fill: Color32,
    pub(super) tint: Color32,
    pub(super) highlight: bool,
    pub(super) name: String,
}

impl PlotImage {
    /// Create a new image with position and size in plot coordinates.
    pub fn new(texture_id: TextureId, position: Value, size: impl Into<Vec2>) -> Self {
        Self {
            position,
            name: Default::default(),
            highlight: false,
            texture_id,
            uv: Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            size: size.into(),
            bg_fill: Default::default(),
            tint: Color32::WHITE,
        }
    }

    /// Highlight this image in the plot.
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }

    /// Select UV range. Default is (0,0) in top-left, (1,1) bottom right.
    pub fn uv(mut self, uv: impl Into<Rect>) -> Self {
        self.uv = uv.into();
        self
    }

    /// A solid color to put behind the image. Useful for transparent images.
    pub fn bg_fill(mut self, bg_fill: impl Into<Color32>) -> Self {
        self.bg_fill = bg_fill.into();
        self
    }

    /// Multiply image color with this. Default is WHITE (no tint).
    pub fn tint(mut self, tint: impl Into<Color32>) -> Self {
        self.tint = tint.into();
        self
    }

    /// Name of this image.
    ///
    /// This name will show up in the plot legend, if legends are turned on.
    ///
    /// Multiple plot items may share the same name, in which case they will also share an entry in
    /// the legend.
    #[allow(clippy::needless_pass_by_value)]
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
}

impl PlotItem for PlotImage {
    fn get_shapes(&self, ui: &mut Ui, transform: &ScreenTransform, shapes: &mut Vec<Shape>) {
        let Self {
            position,
            texture_id,
            uv,
            size,
            bg_fill,
            tint,
            highlight,
            ..
        } = self;
        let rect = {
            let left_top = Value::new(
                position.x as f32 - size.x / 2.0,
                position.y as f32 - size.y / 2.0,
            );
            let right_bottom = Value::new(
                position.x as f32 + size.x / 2.0,
                position.y as f32 + size.y / 2.0,
            );
            let left_top_tf = transform.position_from_value(&left_top);
            let right_bottom_tf = transform.position_from_value(&right_bottom);
            Rect::from_two_pos(left_top_tf, right_bottom_tf)
        };
        Image::new(*texture_id, *size)
            .bg_fill(*bg_fill)
            .tint(*tint)
            .uv(*uv)
            .paint_at(ui, rect);
        if *highlight {
            shapes.push(Shape::rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, ui.visuals().strong_text_color()),
            ))
        }
    }

    fn initialize(&mut self, _x_range: RangeInclusive<f64>) {}

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn color(&self) -> Color32 {
        Color32::TRANSPARENT
    }

    fn highlight(&mut self) {
        self.highlight = true;
    }

    fn highlighted(&self) -> bool {
        self.highlight
    }

    fn values(&self) -> Option<&Values> {
        None
    }

    fn get_bounds(&self) -> Bounds {
        let mut bounds = Bounds::NOTHING;
        let left_top = Value::new(
            self.position.x as f32 - self.size.x / 2.0,
            self.position.y as f32 - self.size.y / 2.0,
        );
        let right_bottom = Value::new(
            self.position.x as f32 + self.size.x / 2.0,
            self.position.y as f32 + self.size.y / 2.0,
        );
        bounds.extend_with(&left_top);
        bounds.extend_with(&right_bottom);
        bounds
    }
}
