use crate::{
    text::{Fonts, Galley, TextColorMap, TextStyle},
    Color32, Mesh, Stroke,
};
use emath::*;

/// A paint primitive such as a circle or a piece of text.
/// Coordinates are all screen space points (not physical pixels).
#[must_use = "Add a Shape to a Painter"]
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// Paint nothing. This can be useful as a placeholder.
    Noop,
    /// Recursively nest more shapes - sometimes a convenience to be able to do.
    /// For performance reasons it is better to avoid it.
    Vec(Vec<Shape>),
    Circle {
        center: Pos2,
        radius: f32,
        fill: Color32,
        stroke: Stroke,
    },
    LineSegment {
        points: [Pos2; 2],
        stroke: Stroke,
    },
    Path {
        points: Vec<Pos2>,
        /// If true, connect the first and last of the points together.
        /// This is required if `fill != TRANSPARENT`.
        closed: bool,
        /// Fill is only supported for convex polygons.
        fill: Color32,
        stroke: Stroke,
    },
    Rect {
        rect: Rect,
        /// How rounded the corners are. Use `0.0` for no rounding.
        corner_radius: f32,
        fill: Color32,
        stroke: Stroke,
    },
    Text {
        /// Top left corner of the first character..
        pos: Pos2,
        /// The layed out text
        galley: std::sync::Arc<Galley>,
        color_map: TextColorMap,
        default_color: Color32,
        /// If true, tilt the letters for an ugly italics effect
        fake_italics: bool,
    },
    Mesh(Mesh),
}

/// ## Constructors
impl Shape {
    /// A line between two points.
    /// More efficient than calling [`Self::line`].
    pub fn line_segment(points: [Pos2; 2], stroke: impl Into<Stroke>) -> Self {
        Self::LineSegment {
            points,
            stroke: stroke.into(),
        }
    }

    /// A line through many points.
    ///
    /// Use [`Self::line_segment`] instead if your line only connects two points.
    pub fn line(points: Vec<Pos2>, stroke: impl Into<Stroke>) -> Self {
        Self::Path {
            points,
            closed: false,
            fill: Default::default(),
            stroke: stroke.into(),
        }
    }

    /// A line that closes back to the start point again.
    pub fn closed_line(points: Vec<Pos2>, stroke: impl Into<Stroke>) -> Self {
        Self::Path {
            points,
            closed: true,
            fill: Default::default(),
            stroke: stroke.into(),
        }
    }

    /// Turn a line into equally spaced dots.
    pub fn dotted_line(
        points: &[Pos2],
        color: impl Into<Color32>,
        spacing: f32,
        radius: f32,
    ) -> Vec<Self> {
        let mut shapes = Vec::new();
        points_from_line(points, spacing, radius, color.into(), &mut shapes);
        shapes
    }

    /// Turn a line into dashes.
    pub fn dashed_line(
        points: &[Pos2],
        stroke: impl Into<Stroke>,
        dash_length: f32,
        gap_length: f32,
    ) -> Vec<Self> {
        let mut shapes = Vec::new();
        dashes_from_line(points, stroke.into(), dash_length, gap_length, &mut shapes);
        shapes
    }

    /// A convex polygon with a fill and optional stroke.
    pub fn convex_polygon(
        points: Vec<Pos2>,
        fill: impl Into<Color32>,
        stroke: impl Into<Stroke>,
    ) -> Self {
        Self::Path {
            points,
            closed: true,
            fill: fill.into(),
            stroke: stroke.into(),
        }
    }

    #[deprecated = "Renamed convex_polygon"]
    pub fn polygon(points: Vec<Pos2>, fill: impl Into<Color32>, stroke: impl Into<Stroke>) -> Self {
        Self::convex_polygon(points, fill, stroke)
    }

    pub fn circle_filled(center: Pos2, radius: f32, fill_color: impl Into<Color32>) -> Self {
        Self::Circle {
            center,
            radius,
            fill: fill_color.into(),
            stroke: Default::default(),
        }
    }

    pub fn circle_stroke(center: Pos2, radius: f32, stroke: impl Into<Stroke>) -> Self {
        Self::Circle {
            center,
            radius,
            fill: Default::default(),
            stroke: stroke.into(),
        }
    }

    pub fn rect_filled(rect: Rect, corner_radius: f32, fill_color: impl Into<Color32>) -> Self {
        Self::Rect {
            rect,
            corner_radius,
            fill: fill_color.into(),
            stroke: Default::default(),
        }
    }

    pub fn rect_stroke(rect: Rect, corner_radius: f32, stroke: impl Into<Stroke>) -> Self {
        Self::Rect {
            rect,
            corner_radius,
            fill: Default::default(),
            stroke: stroke.into(),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn text(
        fonts: &Fonts,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        text_style: TextStyle,
        color: Color32,
    ) -> Self {
        let galley = fonts.layout_multiline(text_style, text.to_string(), f32::INFINITY);
        let rect = anchor.anchor_rect(Rect::from_min_size(pos, galley.size));
        Self::Text {
            pos: rect.min,
            galley,
            default_color: color,
            color_map: TextColorMap::default(),
            fake_italics: false,
        }
    }
}

/// Creates equally spaced filled circles from a line.
fn points_from_line(
    line: &[Pos2],
    spacing: f32,
    radius: f32,
    color: Color32,
    shapes: &mut Vec<Shape>,
) {
    let mut position_on_segment = 0.0;
    line.windows(2).for_each(|window| {
        let start = window[0];
        let end = window[1];
        let vector = end - start;
        let segment_length = vector.length();
        while position_on_segment < segment_length {
            let new_point = start + vector * (position_on_segment / segment_length);
            shapes.push(Shape::circle_filled(new_point, radius, color));
            position_on_segment += spacing;
        }
        position_on_segment -= segment_length;
    });
}

/// Creates dashes from a line.
fn dashes_from_line(
    line: &[Pos2],
    stroke: Stroke,
    dash_length: f32,
    gap_length: f32,
    shapes: &mut Vec<Shape>,
) {
    let mut position_on_segment = 0.0;
    let mut drawing_dash = false;
    line.windows(2).for_each(|window| {
        let start = window[0];
        let end = window[1];
        let vector = end - start;
        let segment_length = vector.length();
        while position_on_segment < segment_length {
            let new_point = start + vector * (position_on_segment / segment_length);
            if drawing_dash {
                // This is the end point.
                if let Shape::Path { points, .. } = shapes.last_mut().unwrap() {
                    points.push(new_point);
                }
                position_on_segment += gap_length;
            } else {
                // Start a new dash.
                shapes.push(Shape::line(vec![new_point], stroke));
                position_on_segment += dash_length;
            }
            drawing_dash = !drawing_dash;
        }
        // If the segment ends and the dash is not finished, add the segment's end point.
        if drawing_dash {
            if let Shape::Path { points, .. } = shapes.last_mut().unwrap() {
                points.push(end);
            }
        }
        position_on_segment -= segment_length;
    });
}

/// ## Operations
impl Shape {
    pub fn mesh(mesh: Mesh) -> Self {
        crate::epaint_assert!(mesh.is_valid());
        Self::Mesh(mesh)
    }

    #[deprecated = "Renamed `mesh`"]
    pub fn triangles(mesh: Mesh) -> Self {
        Self::mesh(mesh)
    }

    #[inline(always)]
    pub fn texture_id(&self) -> super::TextureId {
        if let Shape::Mesh(mesh) = self {
            mesh.texture_id
        } else {
            super::TextureId::Egui
        }
    }

    /// Translate location by this much, in-place
    pub fn translate(&mut self, delta: Vec2) {
        match self {
            Shape::Noop => {}
            Shape::Vec(shapes) => {
                for shape in shapes {
                    shape.translate(delta);
                }
            }
            Shape::Circle { center, .. } => {
                *center += delta;
            }
            Shape::LineSegment { points, .. } => {
                for p in points {
                    *p += delta;
                }
            }
            Shape::Path { points, .. } => {
                for p in points {
                    *p += delta;
                }
            }
            Shape::Rect { rect, .. } => {
                *rect = rect.translate(delta);
            }
            Shape::Text { pos, .. } => {
                *pos += delta;
            }
            Shape::Mesh(mesh) => {
                mesh.translate(delta);
            }
        }
    }
}
