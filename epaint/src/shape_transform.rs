use crate::*;

pub fn adjust_colors(shape: &mut Shape, adjust_color: &impl Fn(&mut Color32)) {
    #![allow(clippy::match_same_arms)]
    match shape {
        Shape::Noop => {}
        Shape::Vec(shapes) => {
            for shape in shapes {
                adjust_colors(shape, adjust_color)
            }
        }
        Shape::Circle { fill, stroke, .. } => {
            adjust_color(fill);
            adjust_color(&mut stroke.color);
        }
        Shape::LineSegment { stroke, .. } => {
            adjust_color(&mut stroke.color);
        }
        Shape::Path { fill, stroke, .. } => {
            adjust_color(fill);
            adjust_color(&mut stroke.color);
        }
        Shape::Rect { fill, stroke, .. } => {
            adjust_color(fill);
            adjust_color(&mut stroke.color);
        }
        Shape::Text {
            default_color,
            color_map,
            ..
        } => {
            adjust_color(default_color);
            color_map.adjust(adjust_color);
        }
        Shape::Mesh(mesh) => {
            for v in &mut mesh.vertices {
                adjust_color(&mut v.color);
            }
        }
    }
}
