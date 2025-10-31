use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub enum DrawCommand {
    Rectangle {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Color,
        z_index: u32,
    },
}
