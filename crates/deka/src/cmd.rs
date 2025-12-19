use super::DAL;
use super::TextStyle;
use super::renderer::gui::utils::TVertex;
use cosmic_text::Buffer;
use heka::{Space, color::Color};

#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// A rectangle with optional fill and stroke.
    Rect {
        space: Space,
        z_index: u32,
        fill_color: Color,
        border_radius: u32,
        stroke_color: Color,
        stroke_width: u32,
        shadow_color: Color,
        shadow_blur: f32,
    },
    /// A block of text.
    Text {
        space: Space,
        buffer_ref: heka::DataRef,
        style: TextStyle,
        z_index: u32,
    },
    // `Image { ... }`, `Svg { ... }`, etc.
}

impl DrawCommand {
    pub fn rect_vertices(
        space: &Space,
        color: &Color,
        radius: u32,
        stroke_width: u32,
        blur: f32,
    ) -> [TVertex; 4] {
        let mut w = space.width.unwrap_or(0) as f32;
        let mut h = space.height.unwrap_or(0) as f32;
        let mut x = space.x as f32;
        let mut y = space.y as f32;

        if blur > 0.0 {
            let expansion = blur * 2.0;
            x -= blur;
            y -= blur;
            w += expansion;
            h += expansion;
        }

        let color_arr: [f32; 4] = (*color).into();

        let uv_tl = [0.0, 0.0];
        let uv_bl = [0.0, 1.0];
        let uv_tr = [1.0, 0.0];
        let uv_br = [1.0, 1.0];

        let size = [w, h];
        let r = radius as f32;
        let s = stroke_width as f32;

        [
            // Top-Left
            TVertex {
                position: [x, y],
                color: color_arr,
                uv: uv_tl,
                size,
                radius: r,
                stroke_width: s,
                blur,
            },
            // Bottom-Left
            TVertex {
                position: [x, y + h],
                color: color_arr,
                uv: uv_bl,
                size,
                radius: r,
                stroke_width: s,
                blur,
            },
            // Top-Right
            TVertex {
                position: [x + w, y],
                color: color_arr,
                uv: uv_tr,
                size,
                radius: r,
                stroke_width: s,
                blur,
            },
            // Bottom-Right
            TVertex {
                position: [x + w, y + h],
                color: color_arr,
                uv: uv_br,
                size,
                radius: r,
                stroke_width: s,
                blur,
            },
        ]
    }

    pub fn to_geometry(&self, dal: &mut DAL) -> (Vec<TVertex>, Vec<u32>) {
        match self {
            DrawCommand::Rect {
                space,
                fill_color,
                stroke_color,
                z_index: _,
                border_radius,
                stroke_width,
                shadow_color,
                shadow_blur,
            } => {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();

                let mut add_quad = |quad_vertices: [TVertex; 4]| {
                    let start_v = vertices.len() as u32;
                    vertices.extend(quad_vertices);
                    indices.extend([
                        start_v,
                        start_v + 1,
                        start_v + 2,
                        start_v + 2,
                        start_v + 1,
                        start_v + 3,
                    ]);
                };

                // Draw Shadow (if visible)
                if shadow_color.a > 0 && *shadow_blur > 0.0 {
                    add_quad(Self::rect_vertices(
                        space,
                        shadow_color,
                        *border_radius,
                        0,
                        *shadow_blur,
                    ));
                }

                // Draw Fill (if visible)
                if fill_color.a > 0 {
                    add_quad(Self::rect_vertices(
                        space,
                        fill_color,
                        *border_radius,
                        0, // Fill has 0 stroke width
                        0.0,
                    ));
                }

                // Draw Stroke (if visible and has width)
                if stroke_color.a > 0 && *stroke_width > 0 {
                    add_quad(Self::rect_vertices(
                        space,
                        stroke_color,
                        *border_radius,
                        *stroke_width,
                        0.0,
                    ));
                }

                (vertices, indices)
            }
            DrawCommand::Text {
                buffer_ref,
                space,
                style,
                z_index: _,
            } => {
                let Some(buffer) = dal.get_buffer::<Buffer>(*buffer_ref) else {
                    return (vec![], vec![]);
                };

                let buffer = buffer.clone();

                let mut vertices = vec![];
                let mut indices = vec![];

                buffer.draw(
                    &mut dal.font_system,
                    &mut dal.swash_cache,
                    cosmic_text::Color(style.color.as_u32()),
                    |x, y, w, h, c| {
                        if c.a() == 0 {
                            return;
                        }

                        let start_v = vertices.len() as u32;

                        vertices.extend(Self::rect_vertices(
                            &Space {
                                x: x + space.x,
                                y: y + space.y,
                                width: Some(w),
                                height: Some(h),
                            },
                            &Color::new(c.r(), c.g(), c.b(), c.a()),
                            0, // Text currently has 0 radius
                            0, // Text currently has 0 stroke width
                            0.0,
                        ));

                        indices.extend([
                            start_v,
                            start_v + 1,
                            start_v + 2,
                            start_v + 2,
                            start_v + 1,
                            start_v + 3,
                        ]);
                    },
                );

                (vertices, indices)
            }
        }
    }
}
