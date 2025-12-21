use winit::{dpi::PhysicalPosition, event::MouseButton, keyboard::SmolStr};

#[derive(Debug, Clone, Copy)]
pub struct ClickEvent {
    pub pos: PhysicalPosition<f64>,
    pub button: MouseButton,
    pub double_click: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct HoverEvent {
    pub hovered: bool,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub logical_key: winit::keyboard::Key,
    pub text: Option<SmolStr>,
    pub pressed: bool,
}

#[derive(Debug, Clone)]
pub enum WindowCommand {
    SetTitle(String),
    SetSize(u32, u32),
    SetResizable(bool),
    SetDecorations(bool),
    Maximize,
    Minimize,
    DragWindow,
    Quit,
}

#[derive(Debug, Clone)]
pub enum SystemEvent {
    Click {
        pos: PhysicalPosition<f64>,
        button: MouseButton,
        pressed: bool,
        double_click: bool,
    },
    CursorMoved(PhysicalPosition<f64>),
    Keyboard {
        logical_key: winit::keyboard::Key,
        text: Option<SmolStr>,
        pressed: bool,
    },
    Resize(u32, u32),
    RequestRedraw,
}
