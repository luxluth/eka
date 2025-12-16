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
        fill_color: Color,
        stroke_color: Color,
        z_index: u32,
        border_radius: u32,
        stroke_width: u32,
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
    ) -> [TVertex; 6] {
        let w = space.width.unwrap_or(0) as f32;
        let h = space.height.unwrap_or(0) as f32;
        let x = space.x as f32;
        let y = space.y as f32;

        let color_arr: [f32; 4] = (*color).into();

        let uv_tl = [0.0, 0.0];
        let uv_bl = [0.0, 1.0];
        let uv_tr = [1.0, 0.0];
        let uv_br = [1.0, 1.0];

        let size = [w, h];
        let r = radius as f32;
        let s = stroke_width as f32;

        [
            // Triangle 1 (Top-Left, Bottom-Left, Top-Right)
            TVertex {
                position: [x, y], // Top-Left
                color: color_arr,
                uv: uv_tl,
                size,
                radius: r,
                stroke_width: s,
            },
            TVertex {
                position: [x, y + h], // Bottom-Left
                color: color_arr,
                uv: uv_bl,
                size,
                radius: r,
                stroke_width: s,
            },
            TVertex {
                position: [x + w, y], // Top-Right
                color: color_arr,
                uv: uv_tr,
                size,
                radius: r,
                stroke_width: s,
            },
            // Triangle 2 (Top-Right, Bottom-Left, Bottom-Right)
            TVertex {
                position: [x + w, y], // Top-Right
                color: color_arr,
                uv: uv_tr,
                size,
                radius: r,
                stroke_width: s,
            },
            TVertex {
                position: [x, y + h], // Bottom-Left
                color: color_arr,
                uv: uv_bl,
                size,
                radius: r,
                stroke_width: s,
            },
            TVertex {
                position: [x + w, y + h], // Bottom-Right
                color: color_arr,
                uv: uv_br,
                size,
                radius: r,
                stroke_width: s,
            },
        ]
    }

    pub fn to_vertices(&self, dal: &mut DAL) -> Vec<TVertex> {
        match self {
            DrawCommand::Rect {
                space,
                fill_color,
                stroke_color,
                z_index: _,
                border_radius,
                stroke_width,
            } => {
                let mut vertices = Vec::new();

                // Draw Fill (if visible)
                if fill_color.a > 0 {
                    vertices.extend(Self::rect_vertices(
                        space,
                        fill_color,
                        *border_radius,
                        0, // Fill has 0 stroke width
                    ));
                }

                // Draw Stroke (if visible and has width)
                if stroke_color.a > 0 && *stroke_width > 0 {
                    vertices.extend(Self::rect_vertices(
                        space,
                        stroke_color,
                        *border_radius,
                        *stroke_width,
                    ));
                }

                vertices
            }
            DrawCommand::Text {
                buffer_ref,
                space,
                style,
                z_index: _,
            } => {
                let Some(buffer) = dal.get_buffer::<Buffer>(*buffer_ref) else {
                    return vec![];
                };

                let buffer = buffer.clone();

                let mut vertices = vec![];
                buffer.draw(
                    &mut dal.font_system,
                    &mut dal.swash_cache,
                    cosmic_text::Color(style.color.as_u32()),
                    |x, y, w, h, c| {
                        if c.a() == 0 {
                            return;
                        }

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
                        ));
                    },
                );

                vertices
            }
        }
    }
}
