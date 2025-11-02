pub use heka;
pub use text_style::TextStyle;

use crate::elements::Label;

use cosmic_text::{FontSystem, SwashCache};
use heka::{layout, size, style};

pub mod elements;
mod text_style;

/// Deka Abstraction Layer
pub struct DAL {
    root: heka::Root,
    root_frame: heka::Frame,
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
}

impl DAL {
    pub fn new(width: u32, height: u32) -> Self {
        let mut root = heka::Root::new(width, height);
        let root_frame = root.add_frame(None);

        style!(root_frame, &mut root, {
            width: size!(fill),
            height: size!(fill),
            layout: layout!(no_layout),
        });

        Self {
            root,
            root_frame,
            font_system: FontSystem::new(),
            swash_cache: SwashCache::new(),
        }
    }

    pub fn new_label<S: ToString>(
        &mut self,
        text: S,
        parent_frame: Option<&heka::Frame>,
        text_style: Option<TextStyle>,
    ) -> Label {
        Label::new(
            &mut self.root,
            Some(parent_frame.unwrap_or(&self.root_frame)),
            text.to_string(),
            text_style.unwrap_or(TextStyle::default()),
            &mut self.font_system,
        )
    }

    pub fn set_label_text(&mut self, label: &mut Label, new_text: String) {
        label.set_text(&mut self.root, &mut self.font_system, new_text);
    }

    pub fn set_label_style(&mut self, label: &mut Label, new_style: TextStyle) {
        label.set_style(&mut self.root, &mut self.font_system, new_style);
    }

    /// Compute inner layout
    pub fn compute_layout(&mut self) {
        self.root.compute();
    }

    /// Resizes the root window.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.root.resize(new_width, new_height);
    }

    #[cfg(feature = "debug")]
    pub fn debug(&self) {
        self.root.debug_layout_tree();
    }
}
