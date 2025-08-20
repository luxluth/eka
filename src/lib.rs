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

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
enum RequestBending { W, H, T, L }

#[derive(Debug, Clone, Copy)]
enum RequestReason {
    Fit,
    Fill,
}

#[derive(Debug, Clone, Copy)]
enum Request {
    Parent(RequestBending, RequestReason),
    Child(RequestBending, RequestReason),
}

#[derive(Debug)]
struct Action {
    requester: CapsuleRef,
    requests: SmallVec<[Request; 4]>,
}

#[derive(Debug)]
struct RequestSolver {
    actions: VecDeque<Action>,
}

impl RequestSolver {
    pub fn new() -> Self {
        RequestSolver {
            actions: VecDeque::new(),
        }
    }

    pub fn resolve(&mut self, root: &mut Root) {
        while !self.actions.is_empty() {
            let this_action = self.actions.pop_front().unwrap();
            let this_requester = root.capsules[this_action.requester];

            for req in &this_action.requests {
                // The current children is asking for it parent w
                let parent_cap = {
                    if let Some(parent_ref) = this_requester.parent_ref {
                        root.spaces[root.capsules[parent_ref].space_ref]
                    } else {
                        root.spaces[0]
                    }
                };
                let parent_style = {
                    if let Some(parent_ref) = this_requester.parent_ref {
                        Some(root.styles[root.capsules[parent_ref].space_ref])
                    } else {
                        None
                    }
                };
                match req {
                    Request::Parent(request_bending, reason) => match request_bending {
                        RequestBending::W => {
                            let requester_space = &mut root.spaces[this_requester];

                            match reason {
                                RequestReason::Fit => {
                                    unreachable!("A child should not fit its parent")
                                }
                                RequestReason::Fill => {
                                    requester_space.width = Some(parent_cap.width.unwrap());
                                }
                            }

                            if let Some(parent_style) = parent_style {
                                let mut base_height = requester_space.height.unwrap_or(0);
                                base_height -= parent_style.padding.top;
                                base_height -= parent_style.padding.bottom;
                                requester_space.height = Some(base_height);
                            }
                        }
                        RequestBending::H => {
                            let requester_space = &mut root.spaces[this_requester];

                            match reason {
                                RequestReason::Fit => {
                                    unreachable!("A child should not fit its parent")
                                }
                                RequestReason::Fill => {
                                    requester_space.height = Some(parent_cap.height.unwrap());
                                }
                            }

                            if let Some(parent_style) = parent_style {
                                let mut base_height = requester_space.height.unwrap_or(0);
                                base_height -= parent_style.padding.top;
                                base_height -= parent_style.padding.bottom;
                                requester_space.height = Some(base_height);
                            }
                        }
                        RequestBending::T => {
                            if let Some(parent_style) = parent_style {
                                let requester_space = &mut root.spaces[this_requester];
                                requester_space.y += parent_style.padding.top as i32;
                            }
                        }
                        RequestBending::L => {
                            if let Some(parent_style) = parent_style {
                                let requester_space = &mut root.spaces[this_requester];
                                requester_space.x += parent_style.padding.top as i32;
                            }
                        }
                    },
                    Request::Child(request_bending, _reason) => match request_bending {
                        RequestBending::W => {
                            // The current requester is a parent and is
                            // waiting for the child width
                        }
                        RequestBending::H => {}
                        RequestBending::T => {}
                        RequestBending::L => {}
                    },
                }
            }
        }
    }
}

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

#[derive(Debug, Clone, Copy)]
struct Capsule {
    space_ref: usize,
    parent_ref: Option<usize>,
    style_ref: usize,
    data_ref: Option<usize>,
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

    pub(crate) solver: RequestSolver,
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
            solver: RequestSolver::new(),
            arena: Arena::new(),
        }
    }

    /// This will take into account the layout strategy used by this specific
    /// parent to compute all the remaining children positions. And once it's
    /// complete, the compute will move on to each child element
    ///
    /// Since the parent is always defined before the children we'll make a
    /// lot of asumptions about the state of the code
    fn compute_frame(&mut self, frame: CapsuleRef, _children: &Vec<CapsuleRef>) {
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

    fn children_of(&self, parent_ref: CapsuleRef) -> Vec<CapsuleRef> {
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
        };

        self.capsules.push(caps);

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
        let space_ref = self.capsules[capsule].space_ref;
        if self.dirties.insert(capsule) {
            for child in self.children_of(space_ref) {
                self.set_dirty(child);
            }
        }
    }

    #[cfg(feature = "debug")]
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
