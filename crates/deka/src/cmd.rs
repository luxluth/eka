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
    // `Image { ... }`, `Svg { ... }`, etc.
}

impl DrawCommand {
    pub fn rect_vertices(
        screen_size: [f32; 2],
        z_index: u32,
        space: &Space,
        color: &Color,
    ) -> [TVertex; 6] {
        let w = space.width.unwrap_or(0) as f32;
        let h = space.height.unwrap_or(0) as f32;
        let x = space.x as f32;
        let y = space.y as f32;

        let screen_w = screen_size[0];
        let screen_h = screen_size[1];

        // Normalize coordinates from pixel space to NDC space [-1.0, 1.0]
        let nx = (x / screen_w) * 2.0 - 1.0;
        let ny = (y / screen_h) * 2.0 - 1.0;
        let nw = (w / screen_w) * 2.0;
        let nh = (h / screen_h) * 2.0;

        // NOTE: Vulkan depth range is usually [0.0, 1.0].
        // pipeline uses CompareOp::LessOrEqual:
        // 0.0 is NEAR (Top), 1.0 is FAR (Bottom).
        // We map z_index (0, 1, 2...) to (1.0, 0.99, 0.98...).
        // Higher z_index = Smaller Z value = Closer to camera.
        // We use a small step (0.0001) to allow for many layers.
        let z = (1.0 - (z_index as f32 * 0.0001)).max(0.0);

        let color_arr: [f32; 4] = (*color).into();
        [
            // Triangle 1 (Top-Left, Bottom-Left, Top-Right)
            TVertex {
                position: [nx, ny, z], // Top-Left
                color: color_arr,
            },
            TVertex {
                position: [nx, ny + nh, z], // Bottom-Left
                color: color_arr,
            },
            TVertex {
                position: [nx + nw, ny, z], // Top-Right
                color: color_arr,
            },
            // Triangle 2 (Top-Right, Bottom-Left, Bottom-Right)
            TVertex {
                position: [nx + nw, ny, z], // Top-Right
                color: color_arr,
            },
            TVertex {
                position: [nx, ny + nh, z], // Bottom-Left
                color: color_arr,
            },
            TVertex {
                position: [nx + nw, ny + nh, z], // Bottom-Right
                color: color_arr,
            },
        ]
    }

    pub fn to_vertices(&self, screen_size: [f32; 2], dal: &mut DAL) -> Vec<TVertex> {
        match self {
            DrawCommand::Rect {
                space,
                color,
                z_index,
            } => Self::rect_vertices(screen_size, *z_index, space, color).to_vec(),
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
                            screen_size,
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
        }
    }
}
