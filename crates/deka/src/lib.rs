use std::collections::HashMap;

pub use heka;
use heka::Frame;
use heka::Style;
use log::warn;
pub use text_style::AsCosmicColor;
pub use text_style::TextStyle;

use crate::elements::{Button, FrameElement, Label, Panel};

use cosmic_text::{FontSystem, SwashCache};
use events::*;
use heka::{layout, size, style};

mod app;
mod cmd;
pub mod elements;
pub mod renderer;
mod text_style;

/// Deka Abstraction Layer
pub struct DAL {
    root: heka::Root,
    root_frame: heka::Frame,
    elements: HashMap<heka::CapsuleRef, Box<dyn FrameElement>>,
    callbacks: HashMap<heka::CapsuleRef, Box<dyn FnMut(&mut DAL, &ClickEvent)>>,

    attr: WindowAttr,

    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
}

pub mod events {
    #[derive(Debug, Clone, Copy)]
    pub struct ClickEvent {
        pub x: i32,
        pub y: i32,
        // pub button: MouseButton
    }
}

/// Represent UI element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element(pub(crate) heka::CapsuleRef);

impl Element {
    pub fn raw(&self) -> heka::CapsuleRef {
        self.0
    }

    pub fn frame(&self) -> heka::Frame {
        Frame::define(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct WindowAttr {
    pub resizable: bool,
    pub title: String,
    pub size: (u32, u32),
}

impl Default for WindowAttr {
    fn default() -> Self {
        Self {
            resizable: false,
            title: String::from("heka, deka, heka, eve"),
            size: (800, 600),
        }
    }
}

impl DAL {
    pub fn new(width: u32, height: u32, mut attr: WindowAttr) -> Self {
        let mut root = heka::Root::new(width, height);
        attr.size = (width, height);
        let root_frame = root.add_frame(None);
        let root_panel = Panel { frame: root_frame };

        style!(root_frame, &mut root, {
            width: size!(fill),
            height: size!(fill),
            layout: layout!(no_layout),
        });

        let mut elements: HashMap<heka::CapsuleRef, Box<dyn FrameElement>> = HashMap::new();
        elements.insert(root_frame.get_ref(), Box::new(root_panel));

        let mut ft_sys = FontSystem::new();
        ft_sys.db_mut().load_system_fonts();

        Self {
            root,
            root_frame,
            elements,
            callbacks: HashMap::new(),
            font_system: ft_sys,
            swash_cache: SwashCache::new(),

            attr,
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

    pub fn new_panel(&mut self, parent_frame: Option<&heka::Frame>, style: Style) -> Element {
        let new_frame = if let Some(parent) = parent_frame {
            self.root.add_frame_child(parent, None)
        } else {
            self.root.add_frame(None)
        };

        let panel = Panel { frame: new_frame };

        new_frame.update_style(&mut self.root, |s| {
            *s = style;
        });

        self.elements.insert(panel.frame.get_ref(), Box::new(panel));
        Element(new_frame.get_ref())
    }

    pub fn set_label_text(&mut self, element: Element, new_text: String) {
        if let Some(frame_element) = self.elements.get_mut(&element.0) {
            if let Some(label) = frame_element.as_any_mut().downcast_mut::<Label>() {
                label.set_text(&mut self.root, &mut self.font_system, new_text);
            } else {
                warn!("set_label_text called on an Element that is not a Label.");
            }
        }
    }

    pub fn set_label_style(&mut self, element: Element, new_style: TextStyle) {
        if let Some(frame_element) = self.elements.get_mut(&element.0) {
            if let Some(label) = frame_element.as_any_mut().downcast_mut::<Label>() {
                label.set_style(&mut self.root, &mut self.font_system, new_style);
            } else {
                warn!("set_label_style called on an Element that is not a Label.");
            }
        }
    }

    /// Creates a new `Button` component with text.
    pub fn new_button<S: ToString, F>(
        &mut self,
        text: S,
        parent_frame: Option<&heka::Frame>,
        on_click: F,
    ) -> Element
    where
        F: FnMut(&mut DAL, &ClickEvent) + 'static,
    {
        let parent = parent_frame.unwrap_or(&self.root_frame);
        let button_frame = self.root.add_frame_child(parent, None);
        let button_ref = button_frame.get_ref();

        style!(button_frame, &mut self.root, {
            width: size!(fit),
            height: size!(fit),
            padding: heka::sizing::Padding::new_all(8),
            background_color: heka::color::Color::new(200, 200, 200, 255),
        });

        let label_style = TextStyle::default();
        let label_element = self.new_label(text, Some(&button_frame), Some(label_style));

        let button_component = Button {
            frame: button_frame,
            child_label: label_element,
        };

        self.callbacks.insert(button_ref, Box::new(on_click));
        self.elements.insert(button_ref, Box::new(button_component));

        Element(button_ref)
    }

    pub fn render(&self) -> Vec<cmd::DrawCommand> {
        // Tuple: (Z-Index, Priority, CapsuleRef, Command)
        // Priority: 0 for Rects, 1 for Text. Ensures Text is always ON TOP of Rects for same Z.
        // CapsuleRef: Used as a stable tie-breaker to prevent HashMap-induced flickering.

        let mut commands = Vec::new();

        for (capsule_ref, element) in &self.elements {
            // Get the computed layout and style
            if let (Some(space), Some(style)) = (
                self.root.get_space(*capsule_ref),
                self.root.get_style(*capsule_ref),
            ) {
                if style.background_color.a > 0 {
                    commands.push((
                        style.z_index,
                        0,
                        *capsule_ref,
                        cmd::DrawCommand::Rect {
                            space,
                            color: style.background_color,
                            z_index: style.z_index,
                        },
                    ));
                }

                if let Some(label) = element.as_any().downcast_ref::<Label>() {
                    if let Some(data_ref) = element.data_ref() {
                        commands.push((
                            style.z_index,
                            1,
                            *capsule_ref,
                            cmd::DrawCommand::Text {
                                space,
                                buffer_ref: data_ref,
                                style: label.text_style.clone(),
                                z_index: style.z_index,
                            },
                        ));
                    }
                }
            }
        }

        // Z-Index (Logic)
        // Priority (Text > Rect)
        // CapsuleRef (Stability)
        commands.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        commands.into_iter().map(|(_, _, _, cmd)| cmd).collect()
    }

    pub fn run(self) -> Result<(), impl std::error::Error> {
        use winit::event_loop::EventLoop;
        let _ = env_logger::try_init();

        let event_loop = EventLoop::new().unwrap();
        let mut application = app::Application::new(&event_loop, self);

        event_loop.run_app(&mut application)
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.root.is_dirty()
    }

    /// Compute inner layout
    pub fn compute_layout(&mut self) {
        self.root.compute();
    }

    /// Resizes the root window.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.root.resize(new_width, new_height);
    }

    pub fn on_click(&mut self, x: i32, y: i32) {
        let Some(hitted) = self.root.hit_test(x, y) else {
            return;
        };
        let Some(mut callback) = self.callbacks.remove(&hitted) else {
            return;
        };
        let event = ClickEvent { x, y };
        callback(self, &event);
        self.callbacks.insert(hitted, callback);
    }

    pub fn get_buffer<T: 'static>(&self, buffer_ref: usize) -> Option<&T> {
        self.root.get_binding(buffer_ref)
    }

    pub fn get_buffer_mut<T: 'static>(&mut self, buffer_ref: usize) -> Option<&mut T> {
        self.root.get_binding_mut(buffer_ref)
    }

    #[cfg(feature = "debug")]
    pub fn debug(&self) {
        self.root.debug_layout_tree();
    }
}
