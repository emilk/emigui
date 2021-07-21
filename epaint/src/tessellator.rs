//! Converts graphics primitives into textured triangles.
//!
//! This module converts lines, circles, text and more represented by [`Shape`]
//! into textured triangles represented by [`Mesh`].

#![allow(clippy::identity_op)]

use crate::{text::TextColorMap, *};
use emath::*;
use std::f32::consts::TAU;

// ----------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct PathPoint {
    pos: Pos2,

    /// For filled paths the normal is used for anti-aliasing (both strokes and filled areas).
    ///
    /// For strokes the normal is also used for giving thickness to the path
    /// (i.e. in what direction to expand).
    ///
    /// The normal could be estimated by differences between successive points,
    /// but that would be less accurate (and in some cases slower).
    ///
    /// Normals are normally unit-length.
    normal: Vec2,
}

/// A connected line (without thickness or gaps) which can be tessellated
/// to either to a stroke (with thickness) or a filled convex area.
/// Used as a scratch-pad during tessellation.
#[derive(Clone, Debug, Default)]
struct Path(Vec<PathPoint>);

impl Path {
    #[inline(always)]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    #[inline(always)]
    pub fn add_point(&mut self, pos: Pos2, normal: Vec2) {
        self.0.push(PathPoint { pos, normal });
    }

    pub fn add_circle(&mut self, center: Pos2, radius: f32) {
        let n = (radius * 4.0).round() as i32; // TODO: tweak a bit more
        let n = n.clamp(4, 64);
        self.reserve(n as usize);
        for i in 0..n {
            let angle = remap(i as f32, 0.0..=n as f32, 0.0..=TAU);
            let normal = vec2(angle.cos(), angle.sin());
            self.add_point(center + radius * normal, normal);
        }
    }

    pub fn add_line_segment(&mut self, points: [Pos2; 2]) {
        self.reserve(2);
        let normal = (points[1] - points[0]).normalized().rot90();
        self.add_point(points[0], normal);
        self.add_point(points[1], normal);
    }

    pub fn add_open_points(&mut self, points: &[Pos2]) {
        let n = points.len();
        assert!(n >= 2);

        if n == 2 {
            // Common case optimization:
            self.add_line_segment([points[0], points[1]]);
        } else {
            self.reserve(n);
            self.add_point(points[0], (points[1] - points[0]).normalized().rot90());
            let mut n0 = (points[1] - points[0]).normalized().rot90();
            for i in 1..n - 1 {
                let mut n1 = (points[i + 1] - points[i]).normalized().rot90();

                // Handle duplicated points (but not triplicated...):
                if n0 == Vec2::ZERO {
                    n0 = n1;
                } else if n1 == Vec2::ZERO {
                    n1 = n0;
                }

                let normal = (n0 + n1) / 2.0;
                let length_sq = normal.length_sq();
                let right_angle_length_sq = 0.5;
                let sharper_than_a_right_angle = length_sq < right_angle_length_sq;
                if sharper_than_a_right_angle {
                    // cut off the sharp corner
                    let center_normal = normal.normalized();
                    let n0c = (n0 + center_normal) / 2.0;
                    let n1c = (n1 + center_normal) / 2.0;
                    self.add_point(points[i], n0c / n0c.length_sq());
                    self.add_point(points[i], n1c / n1c.length_sq());
                } else {
                    // miter join
                    self.add_point(points[i], normal / length_sq);
                }

                n0 = n1;
            }
            self.add_point(
                points[n - 1],
                (points[n - 1] - points[n - 2]).normalized().rot90(),
            );
        }
    }

    pub fn add_line_loop(&mut self, points: &[Pos2]) {
        let n = points.len();
        assert!(n >= 2);
        self.reserve(n);

        let mut n0 = (points[0] - points[n - 1]).normalized().rot90();

        for i in 0..n {
            let next_i = if i + 1 == n { 0 } else { i + 1 };
            let mut n1 = (points[next_i] - points[i]).normalized().rot90();

            // Handle duplicated points (but not triplicated...):
            if n0 == Vec2::ZERO {
                n0 = n1;
            } else if n1 == Vec2::ZERO {
                n1 = n0;
            }

            let normal = (n0 + n1) / 2.0;
            let length_sq = normal.length_sq();
            let right_angle_length_sq = 0.5;
            let sharper_than_a_right_angle = length_sq < right_angle_length_sq;
            if sharper_than_a_right_angle {
                // cut off the sharp corner
                let center_normal = normal.normalized();
                let n0c = (n0 + center_normal) / 2.0;
                let n1c = (n1 + center_normal) / 2.0;
                self.add_point(points[i], n0c / n0c.length_sq());
                self.add_point(points[i], n1c / n1c.length_sq());
            } else {
                // miter join
                self.add_point(points[i], normal / length_sq);
            }

            n0 = n1;
        }
    }
}

pub mod path {
    //! Helpers for constructing paths
    use super::*;

    /// overwrites existing points
    pub fn rounded_rectangle(path: &mut Vec<Pos2>, rect: Rect, corner_radius: f32) {
        path.clear();

        let min = rect.min;
        let max = rect.max;

        let cr = corner_radius
            .min(rect.width() * 0.5)
            .min(rect.height() * 0.5);

        if cr <= 0.0 {
            let min = rect.min;
            let max = rect.max;
            path.reserve(4);
            path.push(pos2(min.x, min.y));
            path.push(pos2(max.x, min.y));
            path.push(pos2(max.x, max.y));
            path.push(pos2(min.x, max.y));
        } else {
            add_circle_quadrant(path, pos2(max.x - cr, max.y - cr), cr, 0.0);
            add_circle_quadrant(path, pos2(min.x + cr, max.y - cr), cr, 1.0);
            add_circle_quadrant(path, pos2(min.x + cr, min.y + cr), cr, 2.0);
            add_circle_quadrant(path, pos2(max.x - cr, min.y + cr), cr, 3.0);
        }
    }

    /// Add one quadrant of a circle
    ///
    /// * quadrant 0: right bottom
    /// * quadrant 1: left bottom
    /// * quadrant 2: left top
    /// * quadrant 3: right top
    //
    // Derivation:
    //
    // * angle 0 * TAU / 4 = right
    //   - quadrant 0: right bottom
    // * angle 1 * TAU / 4 = bottom
    //   - quadrant 1: left bottom
    // * angle 2 * TAU / 4 = left
    //   - quadrant 2: left top
    // * angle 3 * TAU / 4 = top
    //   - quadrant 3: right top
    // * angle 4 * TAU / 4 = right
    pub fn add_circle_quadrant(path: &mut Vec<Pos2>, center: Pos2, radius: f32, quadrant: f32) {
        // TODO: optimize with precalculated vertices for some radii ranges

        let n = (radius * 0.75).round() as i32; // TODO: tweak a bit more
        let n = n.clamp(2, 32);
        const RIGHT_ANGLE: f32 = TAU / 4.0;
        path.reserve(n as usize + 1);
        for i in 0..=n {
            let angle = remap(
                i as f32,
                0.0..=n as f32,
                quadrant * RIGHT_ANGLE..=(quadrant + 1.0) * RIGHT_ANGLE,
            );
            path.push(center + radius * Vec2::angled(angle));
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
pub enum PathType {
    Open,
    Closed,
}
use self::PathType::{Closed, Open};

/// Tessellation quality options
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct TessellationOptions {
    /// Size of a point in pixels, e.g. 2.0. Used to snap text to pixel boundaries.
    pub pixels_per_point: f32,
    /// Size of a pixel in points, e.g. 0.5, or larger if you want more blurry edges.
    pub aa_size: f32,
    /// Anti-aliasing makes shapes appear smoother, but requires more triangles and is therefore slower.
    /// By default this is enabled in release builds and disabled in debug builds.
    pub anti_alias: bool,
    /// If `true` (default) cull certain primitives before tessellating them
    pub coarse_tessellation_culling: bool,
    /// Output the clip rectangles to be painted?
    pub debug_paint_clip_rects: bool,
    /// Output the text-containing rectangles
    pub debug_paint_text_rects: bool,
    /// If true, no clipping will be done
    pub debug_ignore_clip_rects: bool,
}

impl Default for TessellationOptions {
    fn default() -> Self {
        Self {
            pixels_per_point: 1.0,
            aa_size: 1.0,
            anti_alias: true,
            coarse_tessellation_culling: true,
            debug_paint_text_rects: false,
            debug_paint_clip_rects: false,
            debug_ignore_clip_rects: false,
        }
    }
}

impl TessellationOptions {
    #[inline(always)]
    pub fn round_to_pixel(&self, point: f32) -> f32 {
        (point * self.pixels_per_point).round() / self.pixels_per_point
    }
}

/// Tessellate the given convex area into a polygon.
fn fill_closed_path(
    path: &[PathPoint],
    color: Color32,
    options: TessellationOptions,
    out: &mut Mesh,
) {
    if color == Color32::TRANSPARENT {
        return;
    }

    let n = path.len() as u32;
    if options.anti_alias {
        out.reserve_triangles(3 * n as usize);
        out.reserve_vertices(2 * n as usize);
        let color_outer = Color32::TRANSPARENT;
        let idx_inner = out.vertices.len() as u32;
        let idx_outer = idx_inner + 1;
        for i in 2..n {
            out.add_triangle(idx_inner + 2 * (i - 1), idx_inner, idx_inner + 2 * i);
        }
        let mut i0 = n - 1;
        for i1 in 0..n {
            let p1 = &path[i1 as usize];
            let dm = 0.5 * options.aa_size * p1.normal;
            out.colored_vertex(p1.pos - dm, color);
            out.colored_vertex(p1.pos + dm, color_outer);
            out.add_triangle(idx_inner + i1 * 2, idx_inner + i0 * 2, idx_outer + 2 * i0);
            out.add_triangle(idx_outer + i0 * 2, idx_outer + i1 * 2, idx_inner + 2 * i1);
            i0 = i1;
        }
    } else {
        out.reserve_triangles(n as usize);
        let idx = out.vertices.len() as u32;
        out.vertices.extend(path.iter().map(|p| Vertex {
            pos: p.pos,
            uv: WHITE_UV,
            color,
        }));
        for i in 2..n {
            out.add_triangle(idx, idx + i - 1, idx + i);
        }
    }
}

/// Tessellate the given path as a stroke with thickness.
fn stroke_path(
    path: &[PathPoint],
    path_type: PathType,
    stroke: Stroke,
    options: TessellationOptions,
    out: &mut Mesh,
) {
    if stroke.width <= 0.0 || stroke.color == Color32::TRANSPARENT {
        return;
    }

    let n = path.len() as u32;
    let idx = out.vertices.len() as u32;

    if options.anti_alias {
        let color_inner = stroke.color;
        let color_outer = Color32::TRANSPARENT;

        let thin_line = stroke.width <= options.aa_size;
        if thin_line {
            /*
            We paint the line using three edges: outer, inner, outer.

            .       o   i   o      outer, inner, outer
            .       |---|          aa_size (pixel width)
            */

            // Fade out as it gets thinner:
            let color_inner = mul_color(color_inner, stroke.width / options.aa_size);
            if color_inner == Color32::TRANSPARENT {
                return;
            }

            out.reserve_triangles(4 * n as usize);
            out.reserve_vertices(3 * n as usize);

            let mut i0 = n - 1;
            for i1 in 0..n {
                let connect_with_previous = path_type == PathType::Closed || i1 > 0;
                let p1 = &path[i1 as usize];
                let p = p1.pos;
                let n = p1.normal;
                out.colored_vertex(p + n * options.aa_size, color_outer);
                out.colored_vertex(p, color_inner);
                out.colored_vertex(p - n * options.aa_size, color_outer);

                if connect_with_previous {
                    out.add_triangle(idx + 3 * i0 + 0, idx + 3 * i0 + 1, idx + 3 * i1 + 0);
                    out.add_triangle(idx + 3 * i0 + 1, idx + 3 * i1 + 0, idx + 3 * i1 + 1);

                    out.add_triangle(idx + 3 * i0 + 1, idx + 3 * i0 + 2, idx + 3 * i1 + 1);
                    out.add_triangle(idx + 3 * i0 + 2, idx + 3 * i1 + 1, idx + 3 * i1 + 2);
                }
                i0 = i1;
            }
        } else {
            // thick line
            // TODO: line caps for really thick lines?

            /*
            We paint the line using four edges: outer, inner, inner, outer

            .       o   i     p    i   o   outer, inner, point, inner, outer
            .       |---|                  aa_size (pixel width)
            .         |--------------|     width
            .       |---------|            outer_rad
            .           |-----|            inner_rad
            */

            out.reserve_triangles(6 * n as usize);
            out.reserve_vertices(4 * n as usize);

            let mut i0 = n - 1;
            for i1 in 0..n {
                let connect_with_previous = path_type == PathType::Closed || i1 > 0;
                let inner_rad = 0.5 * (stroke.width - options.aa_size);
                let outer_rad = 0.5 * (stroke.width + options.aa_size);
                let p1 = &path[i1 as usize];
                let p = p1.pos;
                let n = p1.normal;
                out.colored_vertex(p + n * outer_rad, color_outer);
                out.colored_vertex(p + n * inner_rad, color_inner);
                out.colored_vertex(p - n * inner_rad, color_inner);
                out.colored_vertex(p - n * outer_rad, color_outer);

                if connect_with_previous {
                    out.add_triangle(idx + 4 * i0 + 0, idx + 4 * i0 + 1, idx + 4 * i1 + 0);
                    out.add_triangle(idx + 4 * i0 + 1, idx + 4 * i1 + 0, idx + 4 * i1 + 1);

                    out.add_triangle(idx + 4 * i0 + 1, idx + 4 * i0 + 2, idx + 4 * i1 + 1);
                    out.add_triangle(idx + 4 * i0 + 2, idx + 4 * i1 + 1, idx + 4 * i1 + 2);

                    out.add_triangle(idx + 4 * i0 + 2, idx + 4 * i0 + 3, idx + 4 * i1 + 2);
                    out.add_triangle(idx + 4 * i0 + 3, idx + 4 * i1 + 2, idx + 4 * i1 + 3);
                }
                i0 = i1;
            }
        }
    } else {
        out.reserve_triangles(2 * n as usize);
        out.reserve_vertices(2 * n as usize);

        let last_index = if path_type == Closed { n } else { n - 1 };
        for i in 0..last_index {
            out.add_triangle(
                idx + (2 * i + 0) % (2 * n),
                idx + (2 * i + 1) % (2 * n),
                idx + (2 * i + 2) % (2 * n),
            );
            out.add_triangle(
                idx + (2 * i + 2) % (2 * n),
                idx + (2 * i + 1) % (2 * n),
                idx + (2 * i + 3) % (2 * n),
            );
        }

        let thin_line = stroke.width <= options.aa_size;
        if thin_line {
            // Fade out thin lines rather than making them thinner
            let radius = options.aa_size / 2.0;
            let color = mul_color(stroke.color, stroke.width / options.aa_size);
            if color == Color32::TRANSPARENT {
                return;
            }
            for p in path {
                out.colored_vertex(p.pos + radius * p.normal, color);
                out.colored_vertex(p.pos - radius * p.normal, color);
            }
        } else {
            let radius = stroke.width / 2.0;
            for p in path {
                out.colored_vertex(p.pos + radius * p.normal, stroke.color);
                out.colored_vertex(p.pos - radius * p.normal, stroke.color);
            }
        }
    }
}

fn mul_color(color: Color32, factor: f32) -> Color32 {
    crate::epaint_assert!(0.0 <= factor && factor <= 1.0);
    // As an unfortunate side-effect of using premultiplied alpha
    // we need a somewhat expensive conversion to linear space and back.
    color.linear_multiply(factor)
}

// ----------------------------------------------------------------------------

/// Converts [`Shape`]s into [`Mesh`].
pub struct Tessellator {
    options: TessellationOptions,
    /// Only used for culling
    clip_rect: Rect,
    scratchpad_points: Vec<Pos2>,
    scratchpad_path: Path,
}

impl Tessellator {
    pub fn from_options(options: TessellationOptions) -> Self {
        Self {
            options,
            clip_rect: Rect::EVERYTHING,
            scratchpad_points: Default::default(),
            scratchpad_path: Default::default(),
        }
    }

    /// Tessellate a single [`Shape`] into a [`Mesh`].
    ///
    /// * `shape`: the shape to tessellate
    /// * `options`: tessellation quality
    /// * `tex_size`: size of the font texture (required to normalize glyph uv rectangles)
    /// * `out`: where the triangles are put
    /// * `scratchpad_path`: if you plan to run `tessellate_shape`
    ///    many times, pass it a reference to the same `Path` to avoid excessive allocations.
    pub fn tessellate_shape(&mut self, tex_size: [usize; 2], shape: Shape, out: &mut Mesh) {
        let clip_rect = self.clip_rect;
        let options = self.options;

        match shape {
            Shape::Noop => {}
            Shape::Vec(vec) => {
                for shape in vec {
                    self.tessellate_shape(tex_size, shape, out)
                }
            }
            Shape::Circle {
                center,
                radius,
                fill,
                stroke,
            } => {
                if radius <= 0.0 {
                    return;
                }

                if options.coarse_tessellation_culling
                    && !clip_rect.expand(radius + stroke.width).contains(center)
                {
                    return;
                }

                let path = &mut self.scratchpad_path;
                path.clear();
                path.add_circle(center, radius);
                fill_closed_path(&path.0, fill, options, out);
                stroke_path(&path.0, Closed, stroke, options, out);
            }
            Shape::Mesh(mesh) => {
                if mesh.is_valid() {
                    out.append(mesh);
                } else {
                    crate::epaint_assert!(false, "Invalid Mesh in Shape::Mesh");
                }
            }
            Shape::LineSegment { points, stroke } => {
                let path = &mut self.scratchpad_path;
                path.clear();
                path.add_line_segment(points);
                stroke_path(&path.0, Open, stroke, options, out);
            }
            Shape::Path {
                points,
                closed,
                fill,
                stroke,
            } => {
                if points.len() >= 2 {
                    let path = &mut self.scratchpad_path;
                    path.clear();
                    if closed {
                        path.add_line_loop(&points);
                    } else {
                        path.add_open_points(&points);
                    }

                    if fill != Color32::TRANSPARENT {
                        crate::epaint_assert!(
                            closed,
                            "You asked to fill a path that is not closed. That makes no sense."
                        );
                        fill_closed_path(&path.0, fill, options, out);
                    }
                    let typ = if closed { Closed } else { Open };
                    stroke_path(&path.0, typ, stroke, options, out);
                }
            }
            Shape::Rect {
                rect,
                corner_radius,
                fill,
                stroke,
            } => {
                let rect = PaintRect {
                    rect,
                    corner_radius,
                    fill,
                    stroke,
                };
                self.tessellate_rect(&rect, out);
            }
            Shape::Text {
                pos,
                galley,
                color_map,
                default_color,
                fake_italics,
            } => {
                if options.debug_paint_text_rects {
                    self.tessellate_rect(
                        &PaintRect {
                            rect: Rect::from_min_size(pos, galley.size).expand(0.5),
                            corner_radius: 2.0,
                            fill: Default::default(),
                            stroke: (0.5, default_color).into(),
                        },
                        out,
                    );
                }
                self.tessellate_text(
                    tex_size,
                    pos,
                    &galley,
                    default_color,
                    &color_map,
                    fake_italics,
                    out,
                );
            }
        }
    }

    pub(crate) fn tessellate_rect(&mut self, rect: &PaintRect, out: &mut Mesh) {
        let PaintRect {
            mut rect,
            corner_radius,
            fill,
            stroke,
        } = *rect;

        if self.options.coarse_tessellation_culling
            && !rect.expand(stroke.width).intersects(self.clip_rect)
        {
            return;
        }
        if rect.is_negative() {
            return;
        }

        // It is common to (sometimes accidentally) create an infinitely sized rectangle.
        // Make sure we can handle that:
        rect.min = rect.min.at_least(pos2(-1e7, -1e7));
        rect.max = rect.max.at_most(pos2(1e7, 1e7));

        let path = &mut self.scratchpad_path;
        path.clear();
        path::rounded_rectangle(&mut self.scratchpad_points, rect, corner_radius);
        path.add_line_loop(&self.scratchpad_points);
        fill_closed_path(&path.0, fill, self.options, out);
        stroke_path(&path.0, Closed, stroke, self.options, out);
    }

    pub fn tessellate_text(
        &mut self,
        tex_size: [usize; 2],
        pos: Pos2,
        galley: &super::Galley,
        default_color: Color32,
        color_map: &TextColorMap,
        fake_italics: bool,
        out: &mut Mesh,
    ) {
        if default_color == Color32::TRANSPARENT && color_map.is_empty() {
            return;
        }
        if cfg!(any(
            feature = "extra_asserts",
            all(feature = "extra_debug_asserts", debug_assertions),
        )) {
            galley.sanity_check();
        }

        // The contents of the galley is already snapped to pixel coordinates,
        // but we need to make sure the galley ends up on the start of a physical pixel:
        let pos = pos2(
            self.options.round_to_pixel(pos.x),
            self.options.round_to_pixel(pos.y),
        );

        let num_chars = galley.char_count_excluding_newlines();

        out.reserve_triangles(num_chars * 2);
        out.reserve_vertices(num_chars * 4);

        let inv_tex_w = 1.0 / tex_size[0] as f32;
        let inv_tex_h = 1.0 / tex_size[1] as f32;

        let clip_slack = 2.0; // Some fudge to handle letters that are slightly larger than expected.
        let clip_rect_min_y = self.clip_rect.min.y - clip_slack;
        let clip_rect_max_y = self.clip_rect.max.y + clip_slack;

        let mut char_pos = 0;
        let mut current_color = default_color;

        for row in &galley.rows {
            let row_min_y = pos.y + row.y_min;
            let row_max_y = pos.y + row.y_max;
            let is_line_visible = clip_rect_min_y <= row_max_y && row_min_y <= clip_rect_max_y;

            if self.options.coarse_tessellation_culling && !is_line_visible {
                // culling individual lines of text is important, since a single `Shape::Text`
                // can span hundreds of lines.
                char_pos += row.uv_rects.len();
                if row.ends_with_newline {
                    char_pos += 1;
                }
                continue;
            }

            for (x_offset, uv_rect) in row.x_offsets.iter().zip(&row.uv_rects) {
                if let Some(col) = color_map.color_change_at_index(char_pos) {
                    current_color = *col;
                }
                char_pos += 1;

                if let Some(glyph) = uv_rect {
                    let mut left_top = pos + glyph.offset + vec2(*x_offset, row.y_min);
                    left_top.x = self.options.round_to_pixel(left_top.x); // Pixel-perfection.
                    left_top.y = self.options.round_to_pixel(left_top.y); // Pixel-perfection.

                    let rect = Rect::from_min_max(left_top, left_top + glyph.size);
                    let uv = Rect::from_min_max(
                        pos2(
                            glyph.min.0 as f32 * inv_tex_w,
                            glyph.min.1 as f32 * inv_tex_h,
                        ),
                        pos2(
                            glyph.max.0 as f32 * inv_tex_w,
                            glyph.max.1 as f32 * inv_tex_h,
                        ),
                    );

                    if fake_italics {
                        let idx = out.vertices.len() as u32;
                        out.add_triangle(idx, idx + 1, idx + 2);
                        out.add_triangle(idx + 2, idx + 1, idx + 3);

                        let top_offset = rect.height() * 0.25 * Vec2::X;

                        out.vertices.push(Vertex {
                            pos: rect.left_top() + top_offset,
                            uv: uv.left_top(),
                            color: current_color,
                        });
                        out.vertices.push(Vertex {
                            pos: rect.right_top() + top_offset,
                            uv: uv.right_top(),
                            color: current_color,
                        });
                        out.vertices.push(Vertex {
                            pos: rect.left_bottom(),
                            uv: uv.left_bottom(),
                            color: current_color,
                        });
                        out.vertices.push(Vertex {
                            pos: rect.right_bottom(),
                            uv: uv.right_bottom(),
                            color: current_color,
                        });
                    } else {
                        out.add_rect_with_uv(rect, uv, current_color);
                    }
                }
            }
            if row.ends_with_newline {
                char_pos += 1;
            }
        }
    }
}

/// Turns [`Shape`]:s into sets of triangles.
///
/// The given shapes will be painted back-to-front (painters algorithm).
/// They will be batched together by clip rectangle.
///
/// * `shapes`: the shape to tessellate
/// * `options`: tessellation quality
/// * `tex_size`: size of the font texture (required to normalize glyph uv rectangles)
///
/// ## Returns
/// A list of clip rectangles with matching [`Mesh`].
pub fn tessellate_shapes(
    shapes: Vec<ClippedShape>,
    options: TessellationOptions,
    tex_size: [usize; 2],
) -> Vec<ClippedMesh> {
    let mut tessellator = Tessellator::from_options(options);

    let mut clipped_meshes: Vec<ClippedMesh> = Vec::default();

    for ClippedShape(clip_rect, shape) in shapes {
        if !clip_rect.is_positive() {
            continue; // skip empty clip rectangles
        }

        let start_new_mesh = match clipped_meshes.last() {
            None => true,
            Some(cm) => cm.0 != clip_rect || cm.1.texture_id != shape.texture_id(),
        };

        if start_new_mesh {
            clipped_meshes.push(ClippedMesh(clip_rect, Mesh::default()));
        }

        let out = &mut clipped_meshes.last_mut().unwrap().1;
        tessellator.clip_rect = clip_rect;
        tessellator.tessellate_shape(tex_size, shape, out);
    }

    if options.debug_paint_clip_rects {
        for ClippedMesh(clip_rect, mesh) in &mut clipped_meshes {
            tessellator.clip_rect = Rect::EVERYTHING;
            tessellator.tessellate_shape(
                tex_size,
                Shape::Rect {
                    rect: *clip_rect,
                    corner_radius: 0.0,
                    fill: Default::default(),
                    stroke: Stroke::new(2.0, Color32::from_rgb(150, 255, 150)),
                },
                mesh,
            )
        }
    }

    if options.debug_ignore_clip_rects {
        for ClippedMesh(clip_rect, _) in &mut clipped_meshes {
            *clip_rect = Rect::EVERYTHING;
        }
    }

    for ClippedMesh(_, mesh) in &clipped_meshes {
        crate::epaint_assert!(mesh.is_valid(), "Tessellator generated invalid Mesh");
    }

    clipped_meshes
}
