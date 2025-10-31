#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Position {
    Fixed {
        x: u32,
        y: u32,
    },
    #[default]
    Auto,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Direction {
    #[default]
    Row,
    Column,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum LayoutStrategy {
    NoStrategy,
    #[default]
    Flex,
    // A later focus
    Grid,
}
