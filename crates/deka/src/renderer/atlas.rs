use std::{collections::HashMap, sync::Arc};

use cosmic_text::CacheKey;
use vulkano::{
    format::Format,
    image::{Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

pub struct TextureUpdate {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct Atlas {
    pub texture: Arc<Image>,
    pub width: u32,
    pub height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    // key -> (u, v, width, height) in normalized coords? No, pixel coords for now.
    pub cache: HashMap<CacheKey, (u32, u32, u32, u32)>,
}

impl Atlas {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        let width = 1024;
        let height = 1024;

        let texture = Image::new(
            memory_allocator,
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8_UNORM, // Single channel for glyphs
                extent: [width, height, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .expect("Failed to create atlas texture");

        Self {
            texture,
            width,
            height,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            cache: HashMap::new(),
        }
    }

    /// Returns (x, y, is_new_allocation).
    pub fn allocate(&mut self, key: CacheKey, width: u32, height: u32) -> Option<(u32, u32, bool)> {
        if let Some(&(x, y, _, _)) = self.cache.get(&key) {
            return Some((x, y, false));
        }

        // 1px padding
        let padding = 1;
        let w = width + padding;
        let h = height + padding;

        if self.cursor_x + w > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }

        if self.cursor_y + h > self.height {
            // Atlas full
            return None;
        }

        let x = self.cursor_x;
        let y = self.cursor_y;

        self.cursor_x += w;
        if h > self.row_height {
            self.row_height = h;
        }

        self.cache.insert(key, (x, y, width, height));
        Some((x, y, true))
    }
}
