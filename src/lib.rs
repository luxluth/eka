#![doc = include_str!("../README.md")]

use std::collections::{HashSet, VecDeque};

use crate::{
    arena::Arena,
    color::Color,
    position::{Direction, LayoutStrategy, Position},
    sizing::{Padding, SizeSpec},
};
use smallvec::SmallVec;

mod arena;
pub mod color;
pub mod macros;
pub mod position;
pub mod sizing;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Space {
    pub x: i32,
    pub y: i32,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Space {
    pub fn zero() -> Self {
        Self {
            x: 0,
            y: 0,
            width: None,
            height: None,
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }
}

pub type CapsuleRef = usize;
pub type DataRef = usize;

#[derive(Debug, Clone)]
struct Capsule {
    space_ref: usize,
    parent_ref: Option<CapsuleRef>,
    style_ref: usize,
    data_ref: Option<DataRef>,
    children: Vec<CapsuleRef>,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    capsule_ref: CapsuleRef,
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

impl std::ops::Index<&Capsule> for Vec<Style> {
    type Output = Style;

    fn index(&self, index: &Capsule) -> &Self::Output {
        unsafe { self.get_unchecked(index.style_ref) }
    }
}

impl std::ops::Index<Capsule> for Vec<Space> {
    type Output = Space;

    fn index(&self, index: Capsule) -> &Self::Output {
        unsafe { self.get_unchecked(index.space_ref) }
    }
}

impl std::ops::Index<&Capsule> for Vec<Space> {
    type Output = Space;

    fn index(&self, index: &Capsule) -> &Self::Output {
        unsafe { self.get_unchecked(index.space_ref) }
    }
}

impl std::ops::IndexMut<Capsule> for Vec<Space> {
    fn index_mut(&mut self, index: Capsule) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index.space_ref) }
    }
}

impl std::ops::IndexMut<&Capsule> for Vec<Space> {
    fn index_mut(&mut self, index: &Capsule) -> &mut Self::Output {
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

    fn children_of(&self, parent_ref: CapsuleRef) -> &Vec<CapsuleRef> {
        &self.capsules[parent_ref].children
    }

    pub fn get_binding_for_frame<T>(&mut self, frame: &Frame) -> Option<&mut T> {
        if let Some(data_idx) = self.capsules[frame.capsule_ref].data_ref {
            return self.arena.get(data_idx);
        } else {
            None
        }
    }

    pub fn set_binding<T>(&mut self, data: T) -> DataRef {
        self.arena.alloc(data)
    }

    pub fn get_binding<T>(&self, index: DataRef) -> Option<&mut T> {
        self.arena.get(index)
    }

    fn internal_add_frame(
        &mut self,
        parent_ref: Option<CapsuleRef>,
        data: Option<DataRef>,
    ) -> Frame {
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
            data_ref: data,
            children: vec![],
        };

        self.capsules.push(caps);

        if let Some(parent) = parent_ref {
            self.capsules[parent].children.push(caps_ref);
        }

        Frame {
            capsule_ref: caps_ref,
        }
    }

    pub fn add_frame_child(&mut self, to: &Frame, data: Option<DataRef>) -> Frame {
        self.internal_add_frame(Some(to.capsule_ref), data)
    }

    pub fn add_frame(&mut self, data: Option<DataRef>) -> Frame {
        self.internal_add_frame(None, data)
    }

    fn set_dirty(&mut self, capsule: CapsuleRef) {
        if self.dirties.insert(capsule) {
            // Use the efficient version from above
            let children = self.capsules[capsule].children.clone();
            for child in children {
                self.set_dirty(child);
            }
        }
    }
}

impl Root {
    pub fn compute(&mut self) {
        // For now, we just compute the whole tree.
        // We can re-integrate the `dirties` logic later once this works.
        self.dirties.clear();

        // 1. Get the screen's dimensions from the root space (space[0])
        let (root_w, root_h) = {
            let root_space = self.spaces[0];
            (
                root_space.width.unwrap_or(0),
                root_space.height.unwrap_or(0),
            )
        };

        // 2. Find all top-level capsules (those with no parent)
        // We must collect them first to avoid borrow-checker issues.
        let top_level_capsules = self
            .capsules
            .iter()
            .enumerate()
            .filter_map(|(i, cap)| {
                if cap.parent_ref.is_none() {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // 3. Run Pass 1 (Measure) and Pass 2 (Layout) for each top-level frame.
        for capsule_ref in top_level_capsules {
            // Start Pass 1: This computes the "desired" size for all nodes
            // in this tree, storing it in their `Space`.
            self.compute_pass_1_measure(capsule_ref);

            // Start Pass 2: This gives each node its final position and size,
            // using the root dimensions as the available space.
            // Note: We use the `space` from Pass 1, which holds the *desired* size.
            let desired_space = self.spaces[self.capsules[capsule_ref].space_ref];
            let _given_width = desired_space.width.unwrap_or(0);
            let _given_height = desired_space.height.unwrap_or(0);

            // A top-level node's "given" space is its own desired size,
            // but it's positioned at (0,0).
            // (Unless it's `Fill` or `Percent`, in which case it gets root_w/root_h)
            // Let's simplify and just pass the root size. Pass 2 will resolve it.
            self.compute_pass_2_layout(capsule_ref, 0, 0, root_w, root_h);
        }
    }
}

impl Root {
    fn compute_pass_2_layout(
        &mut self,
        frame_ref: CapsuleRef,
        given_x: i32,
        given_y: i32,
        given_width: u32,
        given_height: u32,
    ) {
        let capsule = self.capsules[frame_ref].clone();
        let style = self.styles[capsule.style_ref];
        let space = &mut self.spaces[capsule.space_ref];

        // --- 1. Determine My Final Size ---
        // Get my "desired" size from Pass 1
        let desired_w = space.width.unwrap();
        let desired_h = space.height.unwrap();

        // `Pixel`, `Percent`, `Fill` are resolved against `given_width`.
        // `Fit` returns `None`, so we `unwrap_or` our desired size from Pass 1.
        let final_w = style.width.resolve_size(given_width).unwrap_or(desired_w);
        let final_h = style.height.resolve_size(given_height).unwrap_or(desired_h);

        // --- 2. Determine My Final Position ---
        // This is determined by *my* `Position` style.
        // The `given_x/y` are from my parent's layout flow.
        let (final_x, final_y) = match style.position {
            Position::Auto => (given_x, given_y),
            Position::Fixed { x, y } => {
                // `Position::Fixed` is relative to the *parent's content box*,
                // which is what `given_x/y` represent (for the *start* of the flow).
                (given_x + x as i32, given_y + y as i32)
            }
        };

        // --- 3. Store My Final Space ---
        space.x = final_x;
        space.y = final_y;
        space.width = Some(final_w);
        space.height = Some(final_h);

        // --- 4. Calculate My "Content Box" for My Children ---
        let content_x = final_x + style.padding.left as i32;
        let content_y = final_y + style.padding.top as i32;
        let content_w = final_w.saturating_sub(style.padding.left + style.padding.right);
        let content_h = final_h.saturating_sub(style.padding.top + style.padding.bottom);

        // --- 5. Pre-pass: Analyze In-Flow Children for Flex 'Fill' ---
        // We need to know how many `Fill` children we have to divide space.
        let mut in_flow_children = Vec::new();
        let mut total_non_fill_w = 0;
        let mut total_non_fill_h = 0;
        let mut fill_w_count = 0;
        let mut fill_h_count = 0;

        for &child_ref in &capsule.children {
            let child_style = self.styles[self.capsules[child_ref].style_ref];
            if child_style.position == Position::Auto {
                in_flow_children.push(child_ref);
                let child_space = self.spaces[self.capsules[child_ref].space_ref];
                let (child_desired_w, child_desired_h) =
                    (child_space.width.unwrap(), child_space.height.unwrap());

                if style.flow == Direction::Row {
                    if child_style.width.is_fill() {
                        fill_w_count += 1;
                    } else {
                        total_non_fill_w += child_desired_w;
                    }
                } else {
                    // Direction::Column
                    if child_style.height.is_fill() {
                        fill_h_count += 1;
                    } else {
                        total_non_fill_h += child_desired_h;
                    }
                }
            }
        }

        // --- 6. Calculate Space for 'Fill' Children ---
        let total_gap_w = if style.flow == Direction::Row && !in_flow_children.is_empty() {
            style.gap * (in_flow_children.len() as u32 - 1)
        } else {
            0
        };

        let total_gap_h = if style.flow == Direction::Column && !in_flow_children.is_empty() {
            style.gap * (in_flow_children.len() as u32 - 1)
        } else {
            0
        };

        let remaining_w = content_w.saturating_sub(total_non_fill_w + total_gap_w);
        let remaining_h = content_h.saturating_sub(total_non_fill_h + total_gap_h);

        let fill_w_each = if fill_w_count > 0 {
            remaining_w / fill_w_count
        } else {
            0
        };
        let fill_h_each = if fill_h_count > 0 {
            remaining_h / fill_h_count
        } else {
            0
        };

        // --- 7. Recurse and Arrange All Children ---
        let mut current_x = content_x;
        let mut current_y = content_y;

        for child_ref in &capsule.children {
            let child_capsule = self.capsules[*child_ref].clone();
            let child_style = self.styles[child_capsule.style_ref];
            let child_space = self.spaces[child_capsule.space_ref];
            let (child_desired_w, child_desired_h) =
                (child_space.width.unwrap(), child_space.height.unwrap());

            match child_style.position {
                Position::Fixed { .. } => {
                    // This child is "out-of-flow". It's positioned relative
                    // to our content box, but doesn't affect `current_x/y`.
                    // Its `given_width/height` is our content box.
                    self.compute_pass_2_layout(
                        *child_ref, content_x, // Base for fixed positioning
                        content_y, // Base for fixed positioning
                        content_w, content_h,
                    );
                }
                Position::Auto => {
                    // This child is "in-flow".
                    let (child_given_x, child_given_y, child_given_w, child_given_h);
                    match style.layout {
                        LayoutStrategy::Flex => match style.flow {
                            Direction::Row => {
                                child_given_x = current_x;
                                child_given_y = current_y;
                                child_given_w = if child_style.width.is_fill() {
                                    fill_w_each
                                } else {
                                    child_desired_w
                                };
                                child_given_h = content_h; // Flex row items fill height
                            }
                            Direction::Column => {
                                child_given_x = current_x;
                                child_given_y = current_y;
                                child_given_w = content_w; // Flex col items fill width
                                child_given_h = if child_style.height.is_fill() {
                                    fill_h_each
                                } else {
                                    child_desired_h
                                };
                            }
                        },
                        _ => {
                            // NoStrategy
                            child_given_x = current_x;
                            child_given_y = current_y;
                            child_given_w = content_w; // Default: fill width
                            child_given_h = child_desired_h; // Default: use desired height
                        }
                    }

                    self.compute_pass_2_layout(
                        *child_ref,
                        child_given_x,
                        child_given_y,
                        child_given_w,
                        child_given_h,
                    );

                    // Update cursor for next in-flow item
                    match style.layout {
                        LayoutStrategy::Flex => {
                            // After recursion, get the child's *final* size
                            let (child_final_w, child_final_h) = {
                                let final_space = self.spaces[child_capsule.space_ref];
                                (final_space.width.unwrap(), final_space.height.unwrap())
                            };

                            match style.flow {
                                Direction::Row => {
                                    current_x += child_final_w as i32 + style.gap as i32
                                }
                                Direction::Column => {
                                    current_y += child_final_h as i32 + style.gap as i32
                                }
                            }
                        }
                        _ => {} // NoStrategy: all items stack at the same x,y
                    }
                }
            }
        }
    }
}

impl Root {
    /// PASS 1 (Bottom-Up): Measure desired content size.
    /// Returns (desired_width, desired_height)
    fn compute_pass_1_measure(&mut self, frame_ref: CapsuleRef) -> (u32, u32) {
        let capsule = self.capsules[frame_ref].clone(); // Clone to avoid borrow issues
        let style = self.styles[capsule.style_ref];

        // --- 1. Recurse and Measure "In-Flow" Children ---
        // Children with `Position::Fixed` are "out-of-flow" and do not
        // contribute to their parent's `FitContent` size.
        let mut in_flow_child_sizes = Vec::new();
        for &child_ref in &capsule.children {
            let child_style = self.styles[self.capsules[child_ref].style_ref];

            // Recurse for all children
            let (child_w, child_h) = self.compute_pass_1_measure(child_ref);

            // Only "Auto" children participate in the parent's `Fit` sizing
            if child_style.position == Position::Auto {
                in_flow_child_sizes.push((child_w, child_h));
            }
        }

        // --- 2. Calculate This Node's "Content" Size ---
        let (mut content_w, mut content_h) = (0, 0);

        // Calculate content size based on children (if we are `Fit`)
        match style.layout {
            LayoutStrategy::Flex => {
                match style.flow {
                    Direction::Row => {
                        // Width is sum of child widths + gaps
                        content_w = in_flow_child_sizes.iter().map(|(w, _)| *w).sum();
                        if !in_flow_child_sizes.is_empty() {
                            content_w += style.gap * (in_flow_child_sizes.len() as u32 - 1);
                        }
                        // Height is max of child heights
                        content_h = in_flow_child_sizes
                            .iter()
                            .map(|(_, h)| *h)
                            .max()
                            .unwrap_or(0);
                    }
                    Direction::Column => {
                        // Width is max of child widths
                        content_w = in_flow_child_sizes
                            .iter()
                            .map(|(w, _)| *w)
                            .max()
                            .unwrap_or(0);
                        // Height is sum of child heights + gaps
                        content_h = in_flow_child_sizes.iter().map(|(_, h)| *h).sum();
                        if !in_flow_child_sizes.is_empty() {
                            content_h += style.gap * (in_flow_child_sizes.len() as u32 - 1);
                        }
                    }
                }
            }
            LayoutStrategy::NoStrategy | LayoutStrategy::Grid => {
                // Default: size is the max of any child
                content_w = in_flow_child_sizes
                    .iter()
                    .map(|(w, _)| *w)
                    .max()
                    .unwrap_or(0);
                content_h = in_flow_child_sizes
                    .iter()
                    .map(|(_, h)| *h)
                    .max()
                    .unwrap_or(0);
            }
        }

        // --- 3. Determine Final Desired Size Based on Style ---
        // `Fill` and `Percent` have 0 desired size in Pass 1. They expand in Pass 2.
        let desired_w = match style.width {
            SizeSpec::Pixel(w) => w,
            SizeSpec::Fit => content_w + style.padding.left + style.padding.right,
            SizeSpec::Fill | SizeSpec::Percent(_) => 0,
        };

        let desired_h = match style.height {
            SizeSpec::Pixel(h) => h,
            SizeSpec::Fit => content_h + style.padding.top + style.padding.bottom,
            SizeSpec::Fill | SizeSpec::Percent(_) => 0,
        };

        // --- 4. Store Result in Space ---
        let space = &mut self.spaces[capsule.space_ref];
        space.width = Some(desired_w);
        space.height = Some(desired_h);

        (desired_w, desired_h)
    }
}

#[cfg(feature = "debug")]
impl Root {
    pub fn debug_layout_tree_base(&self, cref: CapsuleRef, depth: usize) {
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
                self.debug_layout_tree_base(*child, depth + 1);
            }
        } else {
            eprintln!("{indent}Capsule {cref} not found.");
        }
    }

    pub fn debug_layout_tree(&self) {
        use ansi_term::Style;
        let s = Style::new().fg(ansi_term::Color::Yellow).bold();
        eprintln!(
            "{}",
            s.paint(format!(
                "R ┬ {}x{}",
                self.spaces[0].width.unwrap_or(0),
                self.spaces[0].height.unwrap_or(0)
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
