use cosmic_text::Buffer;
use cosmic_text::FontSystem;
use cosmic_text::SwashCache;
use deka::DAL;
use deka::DrawCommand;
use deka::TextStyle;
use tiny_skia::{Color as TinySkiaColor, Paint, Pixmap, Rect, Transform};
use winit::dpi::PhysicalPosition;
use winit::window::Window;

#[derive(Default)]
struct DekaRenderer;

impl DekaRenderer {
    pub fn paint(
        &self,
        pixmap: &mut Pixmap,
        dal: &DAL,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        commands: Vec<DrawCommand>,
    ) {
        pixmap.fill(TinySkiaColor::TRANSPARENT);

        for cmd in commands {
            match cmd {
                DrawCommand::Rect { space, color } => {
                    let mut paint = Paint::default();
                    paint.set_color_rgba8(color.r, color.g, color.b, color.a);
                    if let Some(rect) = Rect::from_xywh(
                        space.x as f32,
                        space.y as f32,
                        space.width.unwrap_or(0) as f32,
                        space.height.unwrap_or(0) as f32,
                    ) {
                        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                }
                DrawCommand::Text {
                    space,
                    buffer_ref,
                    style,
                } => {
                    let Some(buffer) = dal.get_buffer::<Buffer>(buffer_ref) else {
                        continue;
                    };

                    draw_text(
                        pixmap,
                        buffer,
                        style,
                        font_system,
                        swash_cache,
                        space.x as f32,
                        space.y as f32,
                    );
                }
            }
        }
    }
}

fn draw_text(
    pixmap: &mut Pixmap,
    buffer: &Buffer,
    style: TextStyle,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    x_offset: f32,
    y_offset: f32,
) {
    buffer.draw(
        font_system,
        swash_cache,
        cosmic_text::Color(style.color.as_u32()),
        |px, py, w, h, color| {
            if color.a() == 0 || w != 1 || h != 1 {
                return;
            }

            let final_x = x_offset as i32 + px;
            let final_y = y_offset as i32 + py;
            let paint_color = cosmic_to_tiny_skia(color);

            let Some(rect) = Rect::from_xywh(final_x as f32, final_y as f32, w as f32, h as f32)
            else {
                return;
            };

            let mut paint = Paint {
                blend_mode: tiny_skia::BlendMode::SourceOver,
                ..Paint::default()
            };
            paint.set_color(paint_color);

            pixmap.fill_rect(
                rect,
                &paint,
                Transform::identity(),
                None, // No mask
            );
        },
    );
}

fn cosmic_to_tiny_skia(c: cosmic_text::Color) -> TinySkiaColor {
    let a = c.a() as u16;

    // Perform pre-multiplication
    let r = ((c.r() as u16 * a) / 255) as u8;
    let g = ((c.g() as u16 * a) / 255) as u8;
    let b = ((c.b() as u16 * a) / 255) as u8;

    TinySkiaColor::from_rgba8(r, g, b, c.a())
}

struct AppState<'a> {
    window: Window,

    // WGPU
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,

    // CPU Canvas
    pixmap: Pixmap,

    // GPU Texture
    texture: wgpu::Texture,
    texture_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,

    // EKA: The Deka Abstraction Layer!
    dal: DAL,
    renderer: DekaRenderer,
    cursor_pos: PhysicalPosition<f64>,
}

impl<'a> AppState<'a> {
    async fn new(window: Window) -> Self {
        todo!()
    }
}

fn main() {}
