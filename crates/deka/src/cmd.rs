use super::TextStyle;
use super::renderer::gui::utils::TVertex;
use heka::{Space, color::Color};

#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// A solid-color rectangle.
    Rect {
        space: Space,
        color: Color,
        // border_radius, border_color, etc.
    },
    /// A block of text.
    Text {
        space: Space,
        buffer_ref: heka::DataRef,
        style: TextStyle,
    },
    // `Image { ... }`, `Svg { ... }`, etc.
}

impl DrawCommand {
    pub fn to_vertices(&self, screen_size: [f32; 2]) -> Vec<TVertex> {
        match self {
            DrawCommand::Rect { space, color } => {
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

                let color_arr: [f32; 4] = (*color).into();

                // Create the vertices for the rectangle (two triangles)
                vec![
                    // Top-left
                    TVertex {
                        position: [nx, ny],
                        color: color_arr,
                    },
                    // Top-right
                    TVertex {
                        position: [nx + nw, ny],
                        color: color_arr,
                    },
                    // Bottom-right
                    TVertex {
                        position: [nx + nw, ny + nh],
                        color: color_arr,
                    },
                    // Top-left
                    TVertex {
                        position: [nx, ny],
                        color: color_arr,
                    },
                    // Bottom-right
                    TVertex {
                        position: [nx + nw, ny + nh],
                        color: color_arr,
                    },
                    // Bottom-left
                    TVertex {
                        position: [nx, ny + nh],
                        color: color_arr,
                    },
                ]
            }
            DrawCommand::Text { .. } => {
                // For now, we don't render text with this renderer
                vec![]
            }
        }
    }
}
