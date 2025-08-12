#![doc = include_str!("../README.md")]

use std::collections::HashSet;

use crate::arena::Arena;

mod arena;
pub mod macros;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Space {
    pub x: i32,
    pub y: i32,
    pub width: SizeSpec,
    pub height: SizeSpec,
}

impl Space {
    pub fn zero() -> Self {
        Self {
            x: 0,
            y: 0,
            width: SizeSpec::Pixel(0),
            height: SizeSpec::Pixel(0),
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = SizeSpec::Pixel(width);
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = SizeSpec::Pixel(height);
        self
    }
}

#[derive(Debug, Clone, Copy)]
struct Capsule {
    space_ref: usize,
    parent_ref: Option<usize>,
    style_ref: usize,
    data_ref: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    capsule_ref: usize,
}

pub type BoxElement = Frame;

impl<'a> Frame {
    pub fn style_mut(&'a self, root: &'a mut Root) -> &'a mut Style {
        unsafe {
            root.styles
                .get_unchecked_mut(root.capsules[self.capsule_ref].style_ref)
        }
    }

    pub fn set_dirty(&self, root: &mut Root) {
        root.set_dirty(self.capsule_ref);
    }

    pub fn get_ref(&self) -> usize {
        self.capsule_ref
    }
}

/// RGBA defined color values
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
    /// Red color
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    /// White color
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// DodgerBlue - a nice color
    pub const DODGER_BLUE: Color = Color {
        r: 30,
        g: 144,
        b: 255,
        a: 255,
    };

    /// Transform raw hex into RGBA componnent
    /// **FORMAT: RRGGBBAA**
    #[allow(non_snake_case)]
    pub const fn Hex(hex: u32) -> Color {
        let r = ((hex >> (8 * 3)) & 0xFF) as u8;
        let g = ((hex >> (8 * 2)) & 0xFF) as u8;
        let b = ((hex >> (8 * 1)) & 0xFF) as u8;
        let a = ((hex >> (8 * 0)) & 0xFF) as u8;

        Color { r, g, b, a }
    }

    /// Set an alpha value for the color
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

/// Define dimension specification for a given element.
/// These specification can either be dynamic or fixed.
/// fill | fit | ..px | ..%
#[derive(Clone, Copy, PartialEq)]
pub enum SizeSpec {
    /// **fill** represents the an element that wishes to fill up
    /// any remaining space in th parent
    Fill,
    /// **fit**, applyed to this element will grow or shrink to accomodate
    /// its children.
    Fit,
    /// **pixel** define a precise measure taken by this element
    /// this is the only precise unit of them all.
    Pixel(u32),
    /// **percent**, a value starting by 0..1 - 0.0 being 0% and 1.0 is 100%.
    /// It takes the size of the parent and multiplies it by the defined scalar
    Percent(f32),
}

impl std::ops::SubAssign for SizeSpec {
    fn sub_assign(&mut self, rhs: Self) {
        if self.is_pixel() && rhs.is_pixel() {
            *self = SizeSpec::Pixel(self.get() - rhs.get());
        } else if self.is_percent() && rhs.is_percent() {
            *self = SizeSpec::Percent(self.percent() - rhs.percent());
        }
    }
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
    pub(crate) fn resolve_size(&self, parent_value: u32) -> SizeSpec {
        match self {
            SizeSpec::Pixel(px) => Self::Pixel(*px),
            SizeSpec::Percent(pct) => Self::Pixel((*pct * parent_value as f32) as u32),
            SizeSpec::Fill => Self::Pixel(parent_value),
            SizeSpec::Fit => Self::Pixel(0),
        }
    }

    #[inline]
    fn is_fit(&self) -> bool {
        *self == SizeSpec::Fit
    }

    pub fn area(&self, other_spec: &SizeSpec) -> u32 {
        self.get() * other_spec.get()
    }

    pub fn get(&self) -> u32 {
        match &self {
            SizeSpec::Pixel(e) => *e,
            _ => 0,
        }
    }

    pub fn percent(&self) -> f32 {
        match &self {
            SizeSpec::Percent(e) => *e,
            _ => 0.0,
        }
    }

    #[inline]
    fn is_fill(&self) -> bool {
        *self == SizeSpec::Fill
    }

    fn is_pixel(&self) -> bool {
        match self {
            SizeSpec::Pixel(_) => true,
            _ => false,
        }
    }

    fn is_percent(&self) -> bool {
        match self {
            SizeSpec::Percent(_) => true,
            _ => false,
        }
    }
}

impl Default for SizeSpec {
    fn default() -> Self {
        return Self::Pixel(0);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Direction {
    #[default]
    Row,
    Column,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum LayoutStrategy {
    #[default]
    NoStrategy,
    Flex,
    // A later focus
    Grid,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum Position {
    Fixed {
        x: u32,
        y: u32,
    },
    #[default]
    Auto,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Padding {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Padding {
    pub fn new(left: u32, right: u32, top: u32, bottom: u32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn new_all(all: u32) -> Self {
        Self::new(all, all, all, all)
    }

    pub fn new_lr_tb(lr: u32, tb: u32) -> Self {
        Self::new(lr, lr, tb, tb)
    }
}

impl std::fmt::Display for Padding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pad(L{}, R{}, T{}, B{})",
            self.left, self.right, self.top, self.bottom
        )
    }
}

impl Padding {
    pub fn is_zero(&self) -> bool {
        self.left == 0 && self.right == 0 && self.top == 0 && self.bottom == 0
    }

    // #[inline]
    // pub fn apply_left(&self, left: &mut u32) {
    //     *left = self.left;
    // }
    //
    // #[inline]
    // pub fn apply_right(&self, right: &mut u32) {
    //     *right = self.right;
    // }
    //
    // #[inline]
    // pub fn apply_top(&self, top: &mut u32) {
    //     *top = self.top;
    // }
    //
    // #[inline]
    // pub fn apply_bottom(&self, bottom: &mut u32) {
    //     *bottom = self.bottom;
    // }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Style {
    /// Informative style only. Depending on the Frame
    /// type, this information may be taken into consideration for
    /// use. Like a Box like Frame
    pub background_color: Color,
    /// Width taken by a Frame
    pub width: SizeSpec,
    /// Height taken by a Frame
    pub height: SizeSpec,

    /// Padding setted for a Frame element
    pub padding: Padding,

    /// Define the layout to use for position children
    pub layout: LayoutStrategy,
    /// The direction of the layout. May be usless for the Grid layout
    pub flow: Direction,
    /// Set the gap between child elements
    pub gap: u32,

    /// Position relative to the parent element
    pub position: Position,

    /// Draw order change. Higher the later
    /// Note: If elements have the same z-index, will be
    /// drawn first the one that appears first in the tree.
    pub z_index: u32,
}

#[derive(Debug)]
pub struct Root {
    pub(crate) spaces: Vec<Space>,
    pub(crate) styles: Vec<Style>,
    pub(crate) capsules: Vec<Capsule>,
    pub(crate) dirties: HashSet<usize>,
    pub(crate) arena: Arena,
}

impl std::ops::Index<Capsule> for Vec<Style> {
    type Output = Style;

    fn index(&self, index: Capsule) -> &Self::Output {
        unsafe { self.get_unchecked(index.style_ref) }
    }
}

impl std::ops::Index<Capsule> for Vec<Space> {
    type Output = Space;

    fn index(&self, index: Capsule) -> &Self::Output {
        unsafe { self.get_unchecked(index.space_ref) }
    }
}

impl std::ops::IndexMut<Capsule> for Vec<Space> {
    fn index_mut(&mut self, index: Capsule) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index.space_ref) }
    }
}

impl Root {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            spaces: vec![Space::zero().with_width(width).with_height(height)],
            styles: vec![],
            capsules: vec![],
            dirties: HashSet::new(),
            arena: Arena::new(),
        }
    }

    /// This will take into account the layout strategy used by this specific
    /// parent to compute all the remaining children positions. And once it's
    /// complete, the compute will move on to each child element
    ///
    /// Since the parent is always defined before the children we'll make a
    /// lot of asumptions about the state of the code
    fn compute_frame(&mut self, frame: usize, _children: &Vec<usize>) {
        let parent_caps = self.capsules[frame];
        let style = self.styles[parent_caps];
        match style.layout {
            LayoutStrategy::NoStrategy => {}
            LayoutStrategy::Flex => {}
            s => todo!("impl {s:?}"),
        }
    }

    pub fn compute(&mut self) {
        while !self.dirties.is_empty() {
            let computing_frame = self.dirties.drain().next().unwrap();
            let children = self.children_of(computing_frame);
            for child in &children {
                self.dirties.remove(child);
            }

            self.compute_frame(computing_frame, &children);

            self.dirties.clear();
        }
    }

    fn children_of(&self, parent_ref: usize) -> Vec<usize> {
        self.capsules
            .iter()
            .enumerate()
            .filter_map(move |(i, cap)| {
                if cap.parent_ref.is_some() {
                    let cref = unsafe { cap.parent_ref.unwrap_unchecked() };
                    if cref == parent_ref && cap.space_ref != parent_ref {
                        Some(i)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_binding_for_frame<T>(&mut self, frame: &Frame) -> Option<&mut T> {
        if let Some(data_idx) = self.capsules[frame.capsule_ref].data_ref {
            return self.arena.get(data_idx);
        } else {
            None
        }
    }

    pub fn set_binding<T>(&mut self, data: T) -> usize {
        self.arena.alloc(data)
    }

    pub fn get_binding<T>(&self, index: usize) -> Option<&mut T> {
        self.arena.get(index)
    }

    fn internal_add_frame(&mut self, parent_ref: Option<usize>, data_ref: Option<usize>) -> Frame {
        let new_id = self.spaces.len();
        let space = Space::zero();
        self.spaces.push(space);

        let new_style_idx = self.styles.len();
        self.styles.push(Style::default());

        let caps_ref = self.capsules.len();
        let caps = Capsule {
            space_ref: new_id,
            parent_ref,
            style_ref: new_style_idx,
            data_ref,
        };

        self.capsules.push(caps);

        Frame {
            capsule_ref: caps_ref,
        }
    }

    pub fn add_frame_child(&mut self, to: &Frame, data_ref: Option<usize>) -> Frame {
        self.internal_add_frame(Some(to.capsule_ref), data_ref)
    }

    pub fn add_frame(&mut self, data_ref: Option<usize>) -> Frame {
        self.internal_add_frame(None, data_ref)
    }

    fn set_dirty(&mut self, capsule_ref: usize) {
        let space_ref = self.capsules[capsule_ref].space_ref;
        if self.dirties.insert(capsule_ref) {
            for child in self.children_of(space_ref) {
                self.set_dirty(child);
            }
        }
    }

    #[cfg(feature = "debug")]
    pub fn debug_layout_tree_base(&self, cref: usize, depth: usize) {
        use ansi_term::Style;

        let indent = "  ".repeat(depth);

        if let Some(capsule) = self.capsules.get(cref) {
            let space = self.spaces[capsule.space_ref];
            let style = self.styles[capsule.style_ref];

            let is_child = depth != 0;

            let num_s = Style::new().dimmed().bold();
            let dim = Style::new().dimmed();
            let field = Style::new().fg(ansi_term::Color::Purple);
            let field_name = Style::new().bold();

            if !is_child {
                eprintln!("{indent}Capsule({})", num_s.paint(cref.to_string()));
            } else {
                eprintln!(
                    "{indent}{}Capsule({})",
                    dim.paint("└"),
                    num_s.paint(cref.to_string())
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
                field.paint(format!("{:?}", space.width)),
                field_name.paint("h"),
                field.paint(format!("{:?}", space.height))
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

            for child in self.children_of(cref) {
                self.debug_layout_tree_base(child, depth + 1);
            }
        } else {
            eprintln!("{indent}Capsule {cref} not found.");
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
                self.spaces[0].width.get(),
                self.spaces[0].height.get()
            ))
        );
        for (child, cap) in self.capsules.iter().enumerate() {
            if cap.parent_ref.is_none() {
                self.debug_layout_tree_base(child, 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
