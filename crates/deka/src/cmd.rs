use super::DAL;
use super::TextStyle;
use super::renderer::gui::utils::TVertex;
use cosmic_text::Buffer;
use heka::{Space, color::Color};

#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// A solid-color rectangle.
    Rect {
        space: Space,
        color: Color,
        z_index: u32,
        // border_radius, border_color, etc.
    },
    /// A block of text.
    Text {
        space: Space,
        buffer_ref: heka::DataRef,
        style: TextStyle,
        z_index: u32,
    },
    Border {
        space: heka::Space,
        border: heka::sizing::Border,
        z_index: u32,
    },
    // `Image { ... }`, `Svg { ... }`, etc.
}

impl DrawCommand {
    pub fn rect_vertices(z_index: u32, space: &Space, color: &Color) -> [TVertex; 6] {
        let w = space.width.unwrap_or(0) as f32;
        let h = space.height.unwrap_or(0) as f32;
        let x = space.x as f32;
        let y = space.y as f32;

        // NOTE: Vulkan depth range is usually [0.0, 1.0].
        // pipeline uses CompareOp::LessOrEqual:
        // 0.0 is NEAR (Top), 1.0 is FAR (Bottom).
        // We map z_index (0, 1, 2...) to (1.0, 0.99, 0.98...).
        // Higher z_index = Smaller Z value = Closer to camera.
        // We use a small step (0.0001) to allow for many layers.
        let z = (1.0 - (z_index as f32 * 0.0001)).max(0.0);

        let color_arr: [f32; 4] = (*color).into();

        let uv_tl = [0.0, 0.0];
        let uv_bl = [0.0, 1.0];
        let uv_tr = [1.0, 0.0];
        let uv_br = [1.0, 1.0];

        [
            // Triangle 1 (Top-Left, Bottom-Left, Top-Right)
            TVertex {
                position: [x, y, z], // Top-Left
                color: color_arr,
                uv: uv_tl,
            },
            TVertex {
                position: [x, y + h, z], // Bottom-Left
                color: color_arr,
                uv: uv_bl,
            },
            TVertex {
                position: [x + w, y, z], // Top-Right
                color: color_arr,
                uv: uv_tr,
            },
            // Triangle 2 (Top-Right, Bottom-Left, Bottom-Right)
            TVertex {
                position: [x + w, y, z], // Top-Right
                color: color_arr,
                uv: uv_tr,
            },
            TVertex {
                position: [x, y + h, z], // Bottom-Left
                color: color_arr,
                uv: uv_bl,
            },
            TVertex {
                position: [x + w, y + h, z], // Bottom-Right
                color: color_arr,
                uv: uv_br,
            },
        ]
    }

    pub fn to_vertices(&self, dal: &mut DAL) -> Vec<TVertex> {
        match self {
            DrawCommand::Rect {
                space,
                color,
                z_index,
            } => Self::rect_vertices(*z_index, space, color).to_vec(),
            DrawCommand::Text {
                buffer_ref,
                space,
                style,
                z_index,
            } => {
                let Some(buffer) = dal.get_buffer::<Buffer>(*buffer_ref) else {
                    return vec![];
                };

                let buffer = buffer.clone();

                // let total_text_height = buffer
                //     .layout_runs()
                //     .last()
                //     .map(|run| run.line_y + run.line_height)
                //     .unwrap_or(0.0);
                //
                // let container_h = space.height.unwrap_or(0) as f32;
                //
                // let center_offset_y = ((container_h - total_text_height) / 2.0).max(0.0);

                let mut vertices = vec![];
                buffer.draw(
                    &mut dal.font_system,
                    &mut dal.swash_cache,
                    cosmic_text::Color(style.color.as_u32()),
                    |x, y, w, h, c| {
                        if c.a() == 0 {
                            return;
                        }

                        // let final_y = y + (center_offset_y as i32);

                        vertices.extend(Self::rect_vertices(
                            *z_index,
                            &Space {
                                x: x + space.x,
                                y: y + space.y,
                                width: Some(w),
                                height: Some(h),
                            },
                            &Color::new(c.r(), c.g(), c.b(), c.a()),
                        ));
                    },
                );

                vertices
            }
            DrawCommand::Border {
                space,
                border,
                z_index,
            } => {
                let x = space.x;
                let y = space.y;
                let w = space.width.unwrap_or(0);
                let h = space.height.unwrap_or(0);
                let b = border.size;

                if w == 0 || h == 0 || b == 0 {
                    return vec![];
                }

                let mut vertices = Vec::with_capacity(24);

                let mut push_side = |sx: i32, sy: i32, sw: u32, sh: u32| {
                    vertices.extend(Self::rect_vertices(
                        *z_index,
                        &Space {
                            x: sx,
                            y: sy,
                            width: Some(sw),
                            height: Some(sh),
                        },
                        &border.color,
                    ));
                };

                // top
                // Full width, height is border size
                push_side(x, y, w, b);

                // bottom
                // Full width, sits at the bottom edge.
                // Check if height > border size to prevent drawing on top of the top border
                if h > b {
                    push_side(x, y + (h - b) as i32, w, b);
                }

                // left
                // Width is border size.
                // Height is (Total Height - 2 * Border Size) to fit between Top and Bottom.
                if h > 2 * b {
                    let side_h = h - (2 * b);
                    push_side(x, y + b as i32, b, side_h);
                }

                // right
                // Same vertical logic as Left Side, but positioned at the far right.
                if w > b && h > 2 * b {
                    let side_h = h - (2 * b);
                    push_side(x + (w - b) as i32, y + b as i32, b, side_h);
                }

                vertices
            }
        }
    }
}
