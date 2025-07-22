#![doc = include_str!("../README.md")]

use std::collections::HashSet;

pub mod macros;

#[derive(Debug, Clone, Copy)]
struct Space {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Space {
    pub fn zero(id: usize) -> Self {
        Self {
            id,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }
}

#[derive(Debug, Clone, Copy)]
struct Capsule {
    space_ref: usize,
    parent_space_ref: usize,
    style_ref: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    capsule_ref: usize,
}

impl<'a> Frame {
    pub fn style_mut(&'a self, root: &'a mut Root) -> &'a mut Style {
        root.styles
            .get_mut(root.capsules[self.capsule_ref].style_ref)
            .unwrap()
    }

    pub fn set_dirty(&self, root: &mut Root) {
        root.set_dirty(self.capsule_ref);
    }

    pub fn get_ref(&self) -> usize {
        self.capsule_ref
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl std::fmt::Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

impl Color {
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const DODGER_BLUE: Color = Color {
        r: 30,
        g: 144,
        b: 255,
        a: 255,
    };

    /// FORMAT: RRGGBBAA
    #[allow(non_snake_case)]
    pub const fn Hex(hex: u32) -> Color {
        let r = ((hex >> (8 * 3)) & 0xFF) as u8;
        let g = ((hex >> (8 * 2)) & 0xFF) as u8;
        let b = ((hex >> (8 * 1)) & 0xFF) as u8;
        let a = ((hex >> (8 * 0)) & 0xFF) as u8;

        Color { r, g, b, a }
    }

    pub fn with_alpha(mut self, value: u8) -> Self {
        self.a = value;
        self
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Hex(0xFFFFFFFF)
    }
}

#[derive(Clone, Copy)]
pub enum SizeSpec {
    Fill,
    Fit,
    Pixel(u32),
    Percent(f32),
}

impl std::fmt::Debug for SizeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SizeSpec::Fill => write!(f, "fill"),
            SizeSpec::Fit => write!(f, "fit"),
            SizeSpec::Pixel(px) => write!(f, "{}px", px),
            SizeSpec::Percent(p) => write!(f, "{}%", p * 100.0),
        }
    }
}

impl SizeSpec {
    pub(crate) fn resolve_size(&self, parent_value: u32) -> u32 {
        match self {
            SizeSpec::Pixel(px) => *px,
            SizeSpec::Percent(pct) => (*pct * parent_value as f32) as u32,
            SizeSpec::Fill => parent_value, // basic fallback
            SizeSpec::Fit => 0,             // TODO: content-based later
        }
    }
}

impl Default for SizeSpec {
    fn default() -> Self {
        return Self::Pixel(0);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Style {
    pub background_color: Color,
    pub width: SizeSpec,
    pub height: SizeSpec,

    pub padding: u32,
}

#[derive(Debug)]
pub struct Root {
    pub(crate) spaces: Vec<Space>,
    pub(crate) styles: Vec<Style>,
    pub(crate) capsules: Vec<Capsule>,
    pub(crate) dirties: HashSet<usize>,
}

impl Root {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            spaces: vec![Space::zero(0).with_width(width).with_height(height)],
            styles: vec![],
            capsules: vec![],
            dirties: HashSet::new(),
        }
    }

    pub fn dbg(&self, caps_ref: usize, name: Option<&str>) {
        if let Some(name) = name {
            eprintln!("NAME  :: {name}");
        }

        let Some(capsule) = self.capsules.get(caps_ref) else {
            return;
        };

        let style = self.styles[capsule.style_ref];
        let space = self.spaces[capsule.space_ref];

        eprintln!("STYLE :: {style:#?}");
        eprintln!("SPACE :: {space:#?}");
    }

    #[cfg(feature = "debug")]
    pub fn debug_layout_tree_base(&self, start: usize, depth: usize) {
        use ansi_term::Style;

        let indent = "  ".repeat(depth);

        if let Some(capsule) = self.capsules.get(start) {
            let space = self.spaces[capsule.space_ref];
            let style = self.styles[capsule.style_ref];

            let is_child = depth != 0;

            let num_s = Style::new().dimmed().bold();
            let dim = Style::new().dimmed();
            let field = Style::new().fg(ansi_term::Color::Purple);
            let field_name = Style::new().bold();

            if !is_child {
                eprintln!("{indent}Capsule({})", num_s.paint(start.to_string()));
            } else {
                eprintln!(
                    "{indent}{}Capsule({})",
                    dim.paint("└"),
                    num_s.paint(start.to_string())
                );
            }

            eprintln!(
                "{indent}  {} {} .. {}={} {}={} {}={} {}={}",
                dim.paint("│"),
                dim.paint("Space"),
                field_name.paint("x"),
                field.paint(space.x.to_string()),
                field_name.paint("y"),
                field.paint(space.y.to_string()),
                field_name.paint("w"),
                field.paint(space.width.to_string()),
                field_name.paint("h"),
                field.paint(space.height.to_string())
            );
            eprintln!(
                "{indent}  {} {} .. {}={} {}={} {}={}",
                dim.paint("│"),
                dim.paint("Style"),
                field_name.paint("width"),
                field.paint(format!("{:?}", style.width)),
                field_name.paint("height"),
                field.paint(format!("{:?}", style.height)),
                field_name.paint("padding"),
                field.paint(format!("{}", style.padding))
            );

            for child in self.children_of(space.id) {
                self.debug_layout_tree_base(child, depth + 1);
            }
        } else {
            eprintln!("{indent}Capsule {start} not found.");
        }
    }

    #[cfg(feature = "debug")]
    pub fn debug_layout_tree(&self) {
        use ansi_term::Style;
        let s = Style::new().fg(ansi_term::Color::Yellow).bold();
        eprintln!(
            "{}",
            s.paint(format!(
                "R ┬ {}x{}",
                self.spaces[0].width, self.spaces[0].height
            ))
        );
        for (child, cap) in self.capsules.iter().enumerate() {
            if cap.parent_space_ref == self.spaces[0].id {
                self.debug_layout_tree_base(child, 1);
            }
        }
    }

    pub fn compute(&mut self) {
        for caps_ref in &self.dirties {
            let capsule = self.capsules[*caps_ref];
            let style = self.styles[capsule.style_ref];
            let parent = self.spaces[capsule.parent_space_ref];
            let space = &mut self.spaces[capsule.space_ref];

            space.width = style.width.resolve_size(parent.width);
            space.height = style.height.resolve_size(parent.height);

            space.x = parent.x + style.padding as i32;
            space.y = parent.y + style.padding as i32;
        }

        self.dirties.clear();
    }

    fn children_of(&self, parent_space_ref: usize) -> Vec<usize> {
        self.capsules
            .iter()
            .enumerate()
            .filter_map(move |(i, cap)| {
                if cap.parent_space_ref == parent_space_ref && cap.space_ref != parent_space_ref {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn internal_add_frame(&mut self, parent_id: usize) -> Frame {
        let new_id = self.spaces.len();
        let space = Space::zero(new_id);
        self.spaces.push(space);

        let new_style_idx = self.styles.len();
        self.styles.push(Style::default());

        let caps_ref = self.capsules.len();
        let caps = Capsule {
            space_ref: new_id,
            parent_space_ref: parent_id,
            style_ref: new_style_idx,
        };

        self.capsules.push(caps);

        Frame {
            capsule_ref: caps_ref,
        }
    }

    pub fn add_frame_child(&mut self, to: &Frame) -> Frame {
        self.internal_add_frame(self.capsules[to.capsule_ref].space_ref)
    }

    pub fn add_frame(&mut self) -> Frame {
        self.internal_add_frame(self.spaces[0].id)
    }

    fn set_dirty(&mut self, capsule_ref: usize) {
        let space_ref = self.capsules[capsule_ref].space_ref;
        if self.dirties.insert(capsule_ref) {
            for child in self.children_of(space_ref) {
                self.set_dirty(child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
