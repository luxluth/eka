use std::collections::HashMap;

pub use edl_macro::eka;
pub use heka;
use heka::Frame;
use heka::Style;
use heka::align;
use heka::clr;
use heka::justify;
use heka::margin;
use heka::pad;
use log::warn;
pub use text_style::AsCosmicColor;
pub use text_style::TextStyle;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

use crate::elements::{Button, Checkbox, FrameElement, Label, Panel, TextInput};

use cosmic_text::{FontSystem, SwashCache};
use events::*;
use heka::{layout, size, style};

mod al;
mod cmd;
pub mod elements;
pub mod renderer;
mod text_style;

/// Deka UI Context
pub struct Context {
    root: heka::Root,
    root_frame: heka::Frame,
    elements: HashMap<heka::CapsuleRef, Box<dyn FrameElement>>,
    click_callbacks: HashMap<heka::CapsuleRef, Box<dyn FnMut(&mut Context, &ClickEvent)>>,
    hover_callbacks: HashMap<heka::CapsuleRef, Box<dyn FnMut(&mut Context, &HoverEvent)>>,

    pub(crate) attr: WindowAttr,

    pub(crate) font_system: FontSystem,
    pub(crate) swash_cache: SwashCache,

    pub(crate) mouse_pos: PhysicalPosition<f64>,
    pub(crate) mouse_pressed: bool,
    pub(crate) hovered_element: Option<heka::CapsuleRef>,
    pub(crate) focused_element: Option<heka::CapsuleRef>,

    pub(crate) keyboard_callbacks:
        HashMap<heka::CapsuleRef, Box<dyn FnMut(&mut Context, &KeyEvent)>>,
}

pub mod events {
    use winit::{dpi::PhysicalPosition, event::MouseButton};

    #[derive(Debug, Clone, Copy)]
    pub struct ClickEvent {
        pub pos: PhysicalPosition<f64>,
        pub button: MouseButton,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct HoverEvent {
        pub hovered: bool,
    }

    #[derive(Debug, Clone)]
    pub struct KeyEvent {
        pub logical_key: winit::keyboard::Key,
        pub text: Option<winit::keyboard::SmolStr>,
        pub pressed: bool,
    }
}

pub trait ElementRef: Copy + Into<Element> {
    fn raw(&self) -> heka::CapsuleRef;
}

/// Represent UI element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Element(pub(crate) heka::CapsuleRef);

impl ElementRef for Element {
    fn raw(&self) -> heka::CapsuleRef {
        self.0
    }
}

impl Element {
    pub fn frame(&self) -> heka::Frame {
        Frame::define(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelRef(pub(crate) heka::CapsuleRef);
impl From<LabelRef> for Element {
    fn from(v: LabelRef) -> Self {
        Element(v.0)
    }
}
impl ElementRef for LabelRef {
    fn raw(&self) -> heka::CapsuleRef {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PanelRef(pub(crate) heka::CapsuleRef);
impl From<PanelRef> for Element {
    fn from(v: PanelRef) -> Self {
        Element(v.0)
    }
}
impl ElementRef for PanelRef {
    fn raw(&self) -> heka::CapsuleRef {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ButtonRef(pub(crate) heka::CapsuleRef);
impl From<ButtonRef> for Element {
    fn from(v: ButtonRef) -> Self {
        Element(v.0)
    }
}
impl ElementRef for ButtonRef {
    fn raw(&self) -> heka::CapsuleRef {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CheckboxRef(pub(crate) heka::CapsuleRef);
impl From<CheckboxRef> for Element {
    fn from(v: CheckboxRef) -> Self {
        Element(v.0)
    }
}
impl ElementRef for CheckboxRef {
    fn raw(&self) -> heka::CapsuleRef {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextInputRef(pub(crate) heka::CapsuleRef);
impl From<TextInputRef> for Element {
    fn from(v: TextInputRef) -> Self {
        Element(v.0)
    }
}
impl ElementRef for TextInputRef {
    fn raw(&self) -> heka::CapsuleRef {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct WindowAttr {
    pub resizable: bool,
    pub title: String,
    pub size: (u32, u32),
    pub app_id: String,
}

impl Default for WindowAttr {
    fn default() -> Self {
        Self {
            resizable: true,
            title: String::from("heka, deka, heka, eve"),
            size: (800, 600),
            app_id: String::from("org.deka.app"),
        }
    }
}

impl Context {
    pub fn new(width: u32, height: u32, mut attr: WindowAttr) -> Self {
        let mut root = heka::Root::new(width, height);
        attr.size = (width, height);
        let root_frame = root.add_frame(None);
        let root_panel = Panel { frame: root_frame };

        style!(root_frame, &mut root, {
            width: size!(fill),
            height: size!(fill),
            layout: layout!(no_layout),
            background_color: clr!(transparent),
        });

        let mut elements: HashMap<heka::CapsuleRef, Box<dyn FrameElement>> = HashMap::new();
        elements.insert(root_frame.get_ref(), Box::new(root_panel));

        let mut ft_sys = FontSystem::new();
        ft_sys.db_mut().load_system_fonts();

        Self {
            root,
            root_frame,
            elements,
            click_callbacks: HashMap::new(),
            hover_callbacks: HashMap::new(),
            font_system: ft_sys,
            swash_cache: SwashCache::new(),

            attr,
            mouse_pos: PhysicalPosition::default(),
            mouse_pressed: false,
            hovered_element: None,
            focused_element: None,
            keyboard_callbacks: HashMap::new(),
        }
    }
}

impl Context {
    pub fn new_label<S: ToString>(
        &mut self,
        text: S,
        parent_frame: Option<impl ElementRef>,
        text_style: Option<TextStyle>,
    ) -> LabelRef {
        let parent_frame = if let Some(pf) = parent_frame {
            &Frame::define(pf.raw())
        } else {
            &self.root_frame
        };

        let label = Label::new(
            &mut self.root,
            Some(parent_frame),
            text.to_string(),
            text_style.unwrap_or(TextStyle::default()),
            &mut self.font_system,
        );

        let label_ref = label.frame.get_ref();

        self.elements.insert(label_ref, Box::new(label));
        LabelRef(label_ref)
    }

    pub fn new_panel(&mut self, parent_frame: Option<impl ElementRef>, style: Style) -> PanelRef {
        let parent = if let Some(pf) = parent_frame {
            &Frame::define(pf.raw())
        } else {
            &self.root_frame
        };

        let new_frame = self.root.add_frame_child(parent, None);
        let panel = Panel { frame: new_frame };

        new_frame.update_style(&mut self.root, |s| {
            *s = style;
        });

        self.elements.insert(panel.frame.get_ref(), Box::new(panel));
        PanelRef(new_frame.get_ref())
    }

    pub fn new_checkbox(
        &mut self,
        parent_frame: Option<impl ElementRef>,
        initial_checked: bool,
    ) -> CheckboxRef {
        let parent = if let Some(pf) = parent_frame {
            &Frame::define(pf.raw())
        } else {
            &self.root_frame
        };

        let checkbox = Checkbox::new(&mut self.root, Some(parent), initial_checked);
        let checkbox_ref = checkbox.frame.get_ref();

        self.elements.insert(checkbox_ref, Box::new(checkbox));
        CheckboxRef(checkbox_ref)
    }

    pub fn toggle_checkbox(&mut self, element: CheckboxRef) {
        self.with_component_mut::<Checkbox>(element.0, |checkbox, ctx| {
            checkbox.toggle(&mut ctx.root);
        });
    }

    pub fn new_text_input(
        &mut self,
        parent_frame: Option<impl ElementRef>,
        initial_text: String,
    ) -> TextInputRef {
        let text_input = TextInput::new(self, parent_frame, initial_text);
        let text_input_ref = text_input.frame.get_ref();

        self.keyboard_callbacks.insert(
            text_input_ref,
            Box::new(move |ctx, event| {
                ctx.with_component_mut::<TextInput>(text_input_ref, |input, ctx| {
                    input.handle_key(ctx, event);
                });
            }),
        );

        // focusable on click
        self.on_click(Element(text_input_ref), move |ctx, _| {
            ctx.set_focus(Element(text_input_ref));
        });

        self.elements.insert(text_input_ref, Box::new(text_input));
        TextInputRef(text_input_ref)
    }

    pub fn set_label_text<S: ToString>(&mut self, element: LabelRef, new_text: S) {
        self.with_component_mut::<Label>(element.0, |label, ctx| {
            label.set_text(&mut ctx.root, &mut ctx.font_system, new_text.to_string());
        });
    }

    pub fn get_label_text(&self, element: LabelRef) -> &str {
        if let Some(el) = self.elements.get(&element.0) {
            if let Some(label) = el.as_any().downcast_ref::<Label>() {
                return label.get_text();
            }
        }
        ""
    }

    pub fn set_label_style(&mut self, element: LabelRef, new_style: TextStyle) {
        self.with_component_mut::<Label>(element.0, |label, ctx| {
            label.set_style(&mut ctx.root, &mut ctx.font_system, new_style);
        });
    }

    /// Helper to safely downcast and modify a component.
    /// Reduces boilerplate in set_* methods.
    fn with_component_mut<T: FrameElement + 'static>(
        &mut self,
        capsule_ref: heka::CapsuleRef,
        op: impl FnOnce(&mut T, &mut Context),
    ) {
        if let Some(mut frame_element) = self.elements.remove(&capsule_ref) {
            if let Some(component) = frame_element.as_any_mut().downcast_mut::<T>() {
                op(component, self);
            } else {
                warn!(
                    "Element type mismatch: Expected {}",
                    std::any::type_name::<T>()
                );
            }
            // Put the element back into the map
            self.elements.insert(capsule_ref, frame_element);
        } else {
            warn!("Element not found or invalid reference: {:?}", capsule_ref);
        }
    }

    /// Creates a new `Button` component with text.
    pub fn new_button<S: ToString, F>(
        &mut self,
        text: S,
        parent_frame: Option<impl ElementRef>,
        on_click: F,
        label_style: Option<TextStyle>,
    ) -> ButtonRef
    where
        F: FnMut(&mut Context, &ClickEvent) + 'static,
    {
        let parent = if let Some(pf) = parent_frame {
            &Frame::define(pf.raw())
        } else {
            &self.root_frame
        };

        let button_frame = self.root.add_frame_child(parent, None);
        let button_ref = button_frame.get_ref();

        style!(button_frame, &mut self.root, {
            width: size!(fit),
            height: size!(fit),
            padding: pad!(6, 2),
            margin: margin!(0, 4),
            border: heka::sizing::Border {
                size: 2,
                radius: 5,
                color: clr!(0x8f8f9dFF),
            },
            justify_content: justify!(center),
            align_items: align!(center),
            background_color: clr!(0xe9e9edFF),
            layout: layout!(flex),
        });

        let label_style = label_style.unwrap_or(TextStyle::default());
        let label_element = self.new_label(
            text,
            Some(Element(button_frame.get_ref())),
            Some(label_style),
        );

        let button_component = Button {
            frame: button_frame,
            child_label: label_element.into(),
        };

        self.click_callbacks.insert(button_ref, Box::new(on_click));
        self.elements.insert(button_ref, Box::new(button_component));

        ButtonRef(button_ref)
    }
}

impl Context {
    pub fn on_hover<F>(&mut self, element: impl ElementRef, callback: F)
    where
        F: FnMut(&mut Context, &HoverEvent) + 'static,
    {
        self.hover_callbacks
            .insert(element.raw(), Box::new(callback));
    }

    pub fn on_click<F>(&mut self, element: impl ElementRef, callback: F)
    where
        F: FnMut(&mut Context, &ClickEvent) + 'static,
    {
        self.click_callbacks
            .insert(element.raw(), Box::new(callback));
    }
}

impl Context {
    pub fn run(self) -> Result<(), impl std::error::Error> {
        use winit::event_loop::EventLoop;
        let _ = env_logger::try_init();

        let event_loop = EventLoop::new().unwrap();
        let mut application = al::Application::new(&event_loop, self);

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
    pub(crate) fn resize(&mut self, new_width: u32, new_height: u32) {
        self.root.resize(new_width, new_height);
    }
}

impl Context {
    pub(crate) fn click(&mut self, mouse_button: MouseButton, pressed: bool) {
        if pressed {
            self.mouse_pressed = true;
            return;
        }

        if self.mouse_pressed && !pressed {
            self.mouse_pressed = false;
            let hits = self.root.hit_test(
                self.mouse_pos.x.ceil() as i32,
                self.mouse_pos.y.ceil() as i32,
            );

            if hits.is_empty() {
                return;
            }

            let mut hit_candidates: Vec<(heka::CapsuleRef, u32)> = hits
                .into_iter()
                .filter_map(|cref| {
                    let style = self.root.get_style(cref)?;
                    Some((cref, style.z_index))
                })
                .collect();

            hit_candidates.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

            let event = ClickEvent {
                pos: self.mouse_pos,
                button: mouse_button,
            };

            for (cref, _) in hit_candidates {
                if let Some(mut callback) = self.click_callbacks.remove(&cref) {
                    callback(self, &event);
                    self.click_callbacks.insert(cref, callback);

                    return;
                }
            }
        }
    }

    pub(crate) fn update_hover(&mut self) {
        let hits = self.root.hit_test(
            self.mouse_pos.x.ceil() as i32,
            self.mouse_pos.y.ceil() as i32,
        );

        let mut hit_candidates: Vec<(heka::CapsuleRef, u32)> = hits
            .into_iter()
            .filter_map(|cref| {
                let style = self.root.get_style(cref)?;
                Some((cref, style.z_index))
            })
            .collect();

        hit_candidates.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

        // Find the topmost candidate that has a hover callback
        let best_cref = hit_candidates
            .iter()
            .find(|(cref, _)| self.hover_callbacks.contains_key(cref))
            .map(|(cref, _)| *cref);

        if best_cref != self.hovered_element {
            // Leave previous
            if let Some(prev_cref) = self.hovered_element {
                if let Some(mut callback) = self.hover_callbacks.remove(&prev_cref) {
                    callback(self, &HoverEvent { hovered: false });
                    self.hover_callbacks.insert(prev_cref, callback);
                }
            }

            // Enter new
            if let Some(new_cref) = best_cref {
                if let Some(mut callback) = self.hover_callbacks.remove(&new_cref) {
                    callback(self, &HoverEvent { hovered: true });
                    self.hover_callbacks.insert(new_cref, callback);
                }
            }

            self.hovered_element = best_cref;
        }
    }

    pub(crate) fn key_event(&mut self, event: KeyEvent) {
        if let Some(focused) = self.focused_element {
            if let Some(mut callback) = self.keyboard_callbacks.remove(&focused) {
                callback(self, &event);
                self.keyboard_callbacks.insert(focused, callback);
            }
        }
    }

    pub fn set_focus(&mut self, element: impl ElementRef) {
        self.focused_element = Some(element.raw());
    }
}

impl Context {
    pub fn render(&self) -> Vec<cmd::DrawCommand> {
        // Tuple: (Z-Index, Priority, CapsuleRef, Command)
        // Priority: 0 for Rects, 1 for Text. Ensures Text is always ON TOP of Rects for same Z.
        // CapsuleRef: Used as a stable tie-breaker to prevent HashMap-induced flickering.

        let mut commands = Vec::with_capacity(self.elements.len());

        for (capsule_ref, element) in &self.elements {
            // Get the computed layout and style
            if let (Some(space), Some(style)) = (
                self.root.get_space(*capsule_ref),
                self.root.get_style(*capsule_ref),
            ) {
                commands.push((
                    style.z_index,
                    0,
                    *capsule_ref,
                    cmd::DrawCommand::Rect {
                        space,
                        fill_color: style.background_color,
                        stroke_color: style.border.color,
                        z_index: style.z_index,
                        border_radius: style.border.radius,
                        stroke_width: style.border.size,
                        shadow_color: style.shadow.color,
                        shadow_blur: style.shadow.blur,
                    },
                ));

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

        // Z-Index (Logic) -> Priority (Text > Rect) -> CapsuleRef (Stability)
        commands.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        commands.into_iter().map(|(_, _, _, cmd)| cmd).collect()
    }
}

impl Context {
    pub fn get_buffer<T: 'static>(&self, buffer_ref: usize) -> Option<&T> {
        self.root.get_binding(buffer_ref)
    }

    pub fn get_buffer_mut<T: 'static>(&mut self, buffer_ref: usize) -> Option<&mut T> {
        self.root.get_binding_mut(buffer_ref)
    }
}

#[cfg(feature = "debug")]
impl Context {
    pub fn debug(&self) {
        self.root.debug_layout_tree();
    }
}
