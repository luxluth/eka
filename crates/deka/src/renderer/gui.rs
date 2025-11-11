use std::sync::Arc;

use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::GraphicsPipeline,
};

use crate::cmd::DrawCommand;

pub mod utils {
    use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

    #[derive(BufferContents, Vertex, Debug, Clone, Copy)]
    #[repr(C)]
    pub struct TVertex {
        #[format(R32G32_SFLOAT)]
        pub position: [f32; 2],
        #[format(R32G32B32A32_SFLOAT)]
        pub color: [f32; 4],
    }
}

pub struct GuiRenderer {
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub vertex_buffer: Option<Subbuffer<[utils::TVertex]>>,
}

impl GuiRenderer {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        Self {
            memory_allocator,
            vertex_buffer: None,
        }
    }

    /// Converts draw commands to vertices and uploads them.
    pub fn upload_draw_commands(&mut self, draw_commands: &[DrawCommand], screen_size: [f32; 2]) {
        let vertices: Vec<utils::TVertex> = draw_commands
            .iter()
            .flat_map(|cmd| cmd.to_vertices(screen_size))
            .collect();

        if vertices.is_empty() {
            self.vertex_buffer = None;
            return;
        }

        let buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )
        .expect("[VULKAN::GUI] Failed to create GUI vertex buffer");

        self.vertex_buffer = Some(buffer);
    }

    pub fn render<'a>(
        &'a self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipeline: Arc<GraphicsPipeline>,
    ) {
        if let Some(ref vb) = self.vertex_buffer {
            let builder = builder
                .bind_pipeline_graphics(pipeline)
                .unwrap()
                .bind_vertex_buffers(0, vb.clone())
                .unwrap();

            unsafe {
                builder.draw(vb.len() as u32, 1, 0, 0).unwrap();
            }
        }
    }
}
