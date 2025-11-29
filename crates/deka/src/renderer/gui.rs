use crate::{DAL, cmd::DrawCommand};
use log::debug;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

pub mod utils {
    use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

    #[derive(BufferContents, Vertex, Debug, Clone, Copy)]
    #[repr(C)]
    pub struct TVertex {
        #[format(R32G32B32_SFLOAT)]
        pub position: [f32; 3], // x, y, z
        #[format(R32G32B32A32_SFLOAT)]
        pub color: [f32; 4],
        #[format(R32G32_SFLOAT)]
        pub uv: [f32; 2],
    }
}

pub struct GuiRenderer {
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    // Change: Store Option so we can easily replace the whole buffer
    pub vertex_buffers: Vec<Option<Subbuffer<[utils::TVertex]>>>,
    pub vertex_counts: Vec<u32>,
}

impl GuiRenderer {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        Self {
            memory_allocator,
            vertex_buffers: Vec::new(),
            vertex_counts: Vec::new(),
        }
    }

    pub fn resize(&mut self, num_buffers: usize) {
        self.vertex_buffers.clear();
        self.vertex_counts.clear();

        // Fill with None initially
        for _ in 0..num_buffers {
            self.vertex_buffers.push(None);
            self.vertex_counts.push(0);
        }
    }

    pub fn upload_draw_commands(
        &mut self,
        image_index: usize,
        draw_commands: &[DrawCommand],
        dal: &mut DAL,
    ) {
        let vertices: Vec<utils::TVertex> = draw_commands
            .iter()
            .flat_map(|cmd| cmd.to_vertices(dal))
            .collect();

        let vertex_count = vertices.len();
        self.vertex_counts[image_index] = vertex_count as u32;

        if vertex_count == 0 {
            return;
        }

        debug!(
            "Allocating new buffer for image {} with {} vertices",
            image_index, vertex_count
        );

        // This bypasses the lock check because we aren't touching the old memory.
        let new_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices.into_iter(),
        )
        .expect("Failed to create vertex buffer");

        // If the GPU is still using the OLD buffer at this index, `vulkano` keeps
        // that old memory alive until the GPU is done, then drops it automatically.
        self.vertex_buffers[image_index] = Some(new_buffer);
    }

    pub fn render<'a>(
        &'a self,
        image_index: usize,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let vertex_count = self.vertex_counts[image_index];
        if vertex_count == 0 {
            return;
        }

        if let Some(vb) = &self.vertex_buffers[image_index] {
            builder.bind_vertex_buffers(0, vb.clone()).unwrap();
            unsafe {
                builder.draw(vertex_count, 1, 0, 0).unwrap();
            }
        }
    }
}
