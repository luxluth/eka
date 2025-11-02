use std::collections::HashMap;

pub use heka;
pub use text_style::TextStyle;

use crate::elements::{FrameElement, Label};

use cosmic_text::{FontSystem, SwashCache};
use heka::{layout, size, style};

pub mod elements;
mod text_style;

/// Deka Abstraction Layer
pub struct DAL {
    root: heka::Root,
    root_frame: heka::Frame,
    elements: HashMap<heka::CapsuleRef, Box<dyn FrameElement>>,
    font_system: FontSystem,
    _swash_cache: SwashCache,
}

/// Represent UI element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element(pub(crate) heka::CapsuleRef);

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
            elements: HashMap::new(),
            font_system: FontSystem::new(),
            _swash_cache: SwashCache::new(),
        }
    }

    pub fn new_label<S: ToString>(
        &mut self,
        text: S,
        parent_frame: Option<&heka::Frame>,
        text_style: Option<TextStyle>,
    ) -> Element {
        let label = Label::new(
            &mut self.root,
            Some(parent_frame.unwrap_or(&self.root_frame)),
            text.to_string(),
            text_style.unwrap_or(TextStyle::default()),
            &mut self.font_system,
        );

        let label_ref = label.frame.get_ref();

        self.elements.insert(label_ref, Box::new(label));
        Element(label_ref)
    }

    pub fn set_label_text(&mut self, element: Element, new_text: String) {
        if let Some(frame_element) = self.elements.get_mut(&element.0) {
            if let Some(label) = frame_element.as_any_mut().downcast_mut::<Label>() {
                label.set_text(&mut self.root, &mut self.font_system, new_text);
            } else {
                eprintln!("[warning]: set_label_text called on an Element that is not a Label.");
            }
        }
    }

    pub fn set_label_style(&mut self, element: Element, new_style: TextStyle) {
        if let Some(frame_element) = self.elements.get_mut(&element.0) {
            if let Some(label) = frame_element.as_any_mut().downcast_mut::<Label>() {
                label.set_style(&mut self.root, &mut self.font_system, new_style);
            } else {
                eprintln!("[warning]: set_label_style called on an Element that is not a Label.");
            }
        }
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
