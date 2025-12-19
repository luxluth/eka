use crate::renderer::atlas::Atlas;
use crate::{DAL, cmd::DrawCommand};
use log::debug;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, BufferImageCopy, CopyBufferToImageInfo, PrimaryAutoCommandBuffer,
    },
    descriptor_set::DescriptorSet,
    image::{ImageAspects, ImageSubresourceLayers},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::PipelineLayout,
};

pub mod utils {
    use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

    #[derive(BufferContents, Vertex, Debug, Clone, Copy)]
    #[repr(C)]
    pub struct TVertex {
        #[format(R32G32_SFLOAT)]
        pub position: [f32; 2], // x, y
        #[format(R32G32B32A32_SFLOAT)]
        pub color: [f32; 4],
        #[format(R32G32_SFLOAT)]
        pub uv: [f32; 2],
        #[format(R32G32_SFLOAT)]
        pub size: [f32; 2],
        #[format(R32_SFLOAT)]
        pub radius: f32,
        #[format(R32_SFLOAT)]
        pub stroke_width: f32,
        #[format(R32_SFLOAT)]
        pub blur: f32,
        #[format(R32_UINT)]
        pub obj_type: u32,
    }
}

pub struct GuiRenderer {
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub atlas: Atlas,
    // Change: Store Option so we can easily replace the whole buffer
    pub vertex_buffers: Vec<Option<Subbuffer<[utils::TVertex]>>>,
    pub vertex_counts: Vec<u32>,
    pub index_buffers: Vec<Option<Subbuffer<[u32]>>>,
    pub index_counts: Vec<u32>,
}

impl GuiRenderer {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        Self {
            atlas: Atlas::new(memory_allocator.clone()),
            memory_allocator,
            vertex_buffers: Vec::new(),
            vertex_counts: Vec::new(),
            index_buffers: Vec::new(),
            index_counts: Vec::new(),
        }
    }

    pub fn resize(&mut self, num_buffers: usize) {
        self.vertex_buffers.clear();
        self.vertex_counts.clear();
        self.index_buffers.clear();
        self.index_counts.clear();

        // Fill with None initially
        for _ in 0..num_buffers {
            self.vertex_buffers.push(None);
            self.vertex_counts.push(0);
            self.index_buffers.push(None);
            self.index_counts.push(0);
        }
    }

    pub fn upload_draw_commands(
        &mut self,
        image_index: usize,
        draw_commands: &[DrawCommand],
        dal: &mut DAL,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        let mut all_vertices: Vec<utils::TVertex> = Vec::new();
        let mut all_indices: Vec<u32> = Vec::new();
        let mut uploads = Vec::new();

        for cmd in draw_commands {
            let (vertices, indices) = cmd.to_geometry(dal, &mut self.atlas, &mut uploads);
            let offset = all_vertices.len() as u32;

            all_vertices.extend(vertices);
            all_indices.extend(indices.iter().map(|i| i + offset));
        }

        let mut all_data = Vec::new();
        let mut regions = Vec::new();
        let mut current_offset = 0;

        for upload in uploads {
            if upload.data.is_empty() {
                continue;
            }

            // Align to 4 bytes
            let padding = (4 - (current_offset % 4)) % 4;
            for _ in 0..padding {
                all_data.push(0);
                current_offset += 1;
            }

            regions.push(BufferImageCopy {
                buffer_offset: current_offset,
                image_offset: [upload.x, upload.y, 0],
                image_extent: [upload.width, upload.height, 1],
                image_subresource: ImageSubresourceLayers {
                    aspects: ImageAspects::COLOR,
                    mip_level: 0,
                    array_layers: 0..1,
                },
                ..Default::default()
            });

            all_data.extend_from_slice(&upload.data);
            current_offset += upload.data.len() as u64;
        }

        if !all_data.is_empty() {
            let staging_buffer = Buffer::from_iter(
                self.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::TRANSFER_SRC,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                all_data.into_iter(),
            )
            .expect("Failed to create staging buffer");

            builder
                .copy_buffer_to_image(CopyBufferToImageInfo {
                    regions: regions.into_iter().collect(),
                    ..CopyBufferToImageInfo::buffer_image(
                        staging_buffer,
                        self.atlas.texture.clone(),
                    )
                })
                .expect("Failed to copy buffer to image");
        }

        let vertex_count = all_vertices.len();
        let index_count = all_indices.len();

        self.vertex_counts[image_index] = vertex_count as u32;
        self.index_counts[image_index] = index_count as u32;

        if vertex_count == 0 || index_count == 0 {
            return;
        }

        debug!(
            "Allocating new buffer for image {} with {} vertices and {} indices",
            image_index, vertex_count, index_count
        );

        // This bypasses the lock check because we aren't touching the old memory.
        let new_vertex_buffer = Buffer::from_iter(
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
            all_vertices.into_iter(),
        )
        .expect("Failed to create vertex buffer");

        let new_index_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            all_indices.into_iter(),
        )
        .expect("Failed to create index buffer");

        // If the GPU is still using the OLD buffer at this index, `vulkano` keeps
        // that old memory alive until the GPU is done, then drops it automatically.
        self.vertex_buffers[image_index] = Some(new_vertex_buffer);
        self.index_buffers[image_index] = Some(new_index_buffer);
    }

    pub fn render<'a>(
        &'a self,
        image_index: usize,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipeline_layout: &Arc<PipelineLayout>,
        descriptor_set: &Arc<DescriptorSet>,
    ) {
        let index_count = self.index_counts[image_index];
        if index_count == 0 {
            return;
        }

        if let (Some(vb), Some(ib)) = (
            &self.vertex_buffers[image_index],
            &self.index_buffers[image_index],
        ) {
            builder
                .bind_descriptor_sets(
                    vulkano::pipeline::PipelineBindPoint::Graphics,
                    pipeline_layout.clone(),
                    0,
                    descriptor_set.clone(),
                )
                .unwrap();

            builder.bind_vertex_buffers(0, vb.clone()).unwrap();
            builder.bind_index_buffer(ib.clone()).unwrap();
            unsafe {
                builder.draw_indexed(index_count, 1, 0, 0, 0).unwrap();
            }
        }
    }
}
