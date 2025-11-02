use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Command {
    pub z_index: u32,
    pub desc: Drawing,
}

#[derive(Debug, Clone, Copy)]
pub enum Drawing {
    Rectangle {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: Color,
    },
}
