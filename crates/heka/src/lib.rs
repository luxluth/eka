#![doc = include_str!("../README.md")]

use std::collections::{HashSet, VecDeque};

use crate::{
    boxalloc::Allocator,
    color::Color,
    position::{Direction, LayoutStrategy, Position},
    sizing::{Border, Margin, Padding, SizeSpec},
};

mod boxalloc;
pub mod color;
pub mod macros;
pub mod position;
pub mod sizing;

#[derive(Debug, Clone, Copy)]
pub struct Space {
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

/// A reference to an internal data element
pub type DataRef = usize;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapsuleRef {
    id: usize,
    generation: u32,
}

impl PartialOrd for CapsuleRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CapsuleRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then(self.generation.cmp(&other.generation))
    }
}

impl std::fmt::Debug for CapsuleRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.id, self.generation)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapsuleSlot {
    pub capsule: Option<Capsule>,
    generation: u32,
}

#[derive(Debug, Clone)]
pub struct Capsule {
    pub space_ref: usize,
    pub parent_ref: Option<CapsuleRef>,
    pub style_ref: usize,
    pub data_ref: Option<DataRef>,
    children: Vec<CapsuleRef>,
}

/// Describe a frame box element
#[derive(Debug, Clone, Copy)]
pub struct Frame {
    capsule_ref: CapsuleRef,
}

impl<'a> Frame {
    pub fn define(capsule_ref: CapsuleRef) -> Self {
        Self { capsule_ref }
    }
    pub fn update_style<F>(&self, root: &mut Root, applier: F)
    where
        F: FnOnce(&mut Style),
    {
        if let Some(style_mut) = self.get_style_mut(root) {
            applier(style_mut);
            self.set_dirty(root);
        }
    }

    fn get_style_mut(&self, root: &'a mut Root) -> Option<&'a mut Style> {
        let style_ref = if let Some(capsule) = root.get_capsule_mut(self.capsule_ref) {
            // We get the `usize`, and the borrow of `root` ends here.
            Some(capsule.style_ref)
        } else {
            // The handle was invalid
            return None;
        };

        if let Some(style_ref) = style_ref {
            root.styles
                .get_mut(style_ref)
                .and_then(|style_option| style_option.as_mut())
        } else {
            // This case is already handled, but we're explicit.
            None
        }
    }
}

impl Frame {
    pub fn get_ref(&self) -> CapsuleRef {
        self.capsule_ref
    }

    pub fn set_dirty(&self, root: &mut Root) {
        root.set_dirty(self.capsule_ref);
    }
}

#[derive(Debug, Clone, Copy)]
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

    /// Margin setted for a Frame element
    pub margin: Margin,

    pub border: Border,

    /// Defines how much a flex item will grow.
    /// Default is 0.0 (don't grow).
    pub flex_grow: f32,

    /// Defines how much a flex item will shrink.
    /// Default is 1.0 (shrink at a normal rate).
    pub flex_shrink: f32,

    /// Define the layout to use for position children
    pub layout: LayoutStrategy,
    /// The direction of the layout. May be usless for the Grid layout
    pub flow: Direction,
    /// Set the gap between child elements
    pub gap: u32,

    /// Position relative to the parent element
    pub position: Position,

    /// The intrinsic content width, as measured by a component.
    /// This is used by `SizeSpec::Fit`.
    pub intrinsic_width: Option<u32>,

    /// The intrinsic content height, as measured by a component.
    /// This is used by `SizeSpec::Fit`.
    pub intrinsic_height: Option<u32>,

    /// Draw order change. Higher the later
    /// Note: If elements have the same z-index, will be
    /// drawn first the one that appears first in the tree.
    pub z_index: u32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background_color: Color::default(),
            width: SizeSpec::default(),
            height: SizeSpec::default(),
            padding: Padding::default(),
            margin: Margin::default(),
            border: Border::default(),
            layout: LayoutStrategy::default(),
            flow: Direction::default(),
            position: Position::default(),
            gap: 0,
            z_index: 0,

            flex_grow: 0.0,
            flex_shrink: 1.0,

            intrinsic_width: None,
            intrinsic_height: None,
        }
    }
}

#[derive(Debug)]
pub struct Root {
    pub capsules: Vec<CapsuleSlot>,
    capsule_free_list: VecDeque<usize>,
    pub spaces: Vec<Option<Space>>,
    styles: Vec<Option<Style>>,

    dirties: HashSet<CapsuleRef>,
    allocator: Allocator,
}

impl Root {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            // NOTE: space[0] is the root space and should always be accessible
            spaces: vec![Some(Space::zero().with_width(width).with_height(height))],

            styles: vec![],
            capsules: vec![],
            dirties: HashSet::new(),
            capsule_free_list: VecDeque::new(),
            allocator: Allocator::new(),
        }
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        !self.dirties.is_empty()
    }

    pub fn get_binding_for_frame<T: 'static>(&mut self, frame: &Frame) -> Option<&T> {
        self.get_capsule(frame.capsule_ref)
            .and_then(|cap| cap.data_ref)
            .and_then(|data_idx| self.allocator.get(data_idx))
    }

    pub fn set_binding<T: 'static>(&mut self, data: T) -> DataRef {
        self.allocator.alloc(data)
    }

    pub fn get_binding<T: 'static>(&self, index: DataRef) -> Option<&T> {
        self.allocator.get(index)
    }

    pub fn get_binding_mut<T: 'static>(&mut self, index: DataRef) -> Option<&mut T> {
        self.allocator.get_mut(index)
    }

    pub fn unbind_data(&mut self, frame_ref: CapsuleRef) -> bool {
        if let Some(capsule) = self.get_capsule_mut(frame_ref) {
            if let Some(data_ref) = capsule.data_ref.take() {
                return self.allocator.dealloc(data_ref);
            }
        }
        false
    }

    pub fn set_parent(&mut self, child_frame: Frame, new_parent_frame: Frame) {
        let child_ref = child_frame.get_ref();

        // Remove child from its old parent's list
        let old_parent_ref = self.get_capsule(child_ref).and_then(|c| c.parent_ref);

        if let Some(old_parent_ref) = old_parent_ref {
            if let Some(old_parent_capsule) = self.get_capsule_mut(old_parent_ref) {
                // Remove the child from the old parent's children
                old_parent_capsule.children.retain(|&c| c != child_ref);
            }
            self.set_dirty(old_parent_ref); // Old parent's layout is now invalid
        }

        // Add child to new parent's list
        let new_parent_ref = new_parent_frame.get_ref();
        if let Some(new_parent_capsule) = self.get_capsule_mut(new_parent_ref) {
            new_parent_capsule.children.push(child_ref);
        }

        // Update the child's own parent reference
        if let Some(child_capsule) = self.get_capsule_mut(child_ref) {
            child_capsule.parent_ref = Some(new_parent_ref);
        }

        self.set_dirty(new_parent_ref);
    }

    fn internal_add_frame(
        &mut self,
        parent_ref: Option<CapsuleRef>,
        data: Option<DataRef>,
    ) -> Frame {
        let new_id = self.spaces.len();
        let space = Space::zero();

        self.spaces.push(Some(space));

        let new_style_idx = self.styles.len();
        self.styles.push(Some(Style::default()));

        let caps = Capsule {
            space_ref: new_id,
            parent_ref,
            style_ref: new_style_idx,
            data_ref: data,
            children: vec![],
        };

        let (new_id, new_generation) = {
            if let Some(recycled_id) = self.capsule_free_list.pop_front() {
                let slot = &mut self.capsules[recycled_id];
                slot.capsule = Some(caps);
                // The generation is already correct (it was incremented on removal)
                (recycled_id, slot.generation)
            } else {
                let new_id = self.capsules.len();
                let new_slot = CapsuleSlot {
                    capsule: Some(caps),
                    generation: 0, // Start at generation 0
                };
                self.capsules.push(new_slot);
                (new_id, 0)
            }
        };

        let new_ref = CapsuleRef {
            id: new_id,
            generation: new_generation,
        };

        if let Some(pref) = parent_ref {
            if let Some(parent_capsule) = self.get_capsule_mut(pref) {
                parent_capsule.children.push(new_ref);
            }
        }

        Frame {
            capsule_ref: new_ref,
        }
    }

    pub fn add_frame_child(&mut self, to: &Frame, data: Option<DataRef>) -> Frame {
        self.internal_add_frame(Some(to.capsule_ref), data)
    }

    pub fn add_frame(&mut self, data: Option<DataRef>) -> Frame {
        self.internal_add_frame(None, data)
    }
}

impl Root {
    pub fn hit_test(&self, x: i32, y: i32) -> Vec<CapsuleRef> {
        let mut hits = Vec::new();

        for (i, slot) in self.capsules.iter().enumerate() {
            if let Some(caps) = &slot.capsule {
                let space = self.spaces.get(caps.space_ref).and_then(|s| s.as_ref());
                if let Some(fs) = space {
                    let (w, h) = (fs.width.unwrap_or(0) as i32, fs.height.unwrap_or(0) as i32);

                    if x >= fs.x && x <= (fs.x + w) && y >= fs.y && y <= (fs.y + h) {
                        hits.push(CapsuleRef {
                            id: i,
                            generation: slot.generation,
                        });
                    }
                }
            }
        }

        hits
    }
}

impl Root {
    /// Safely gets an immutable reference to a capsule.
    pub fn get_capsule(&self, frame_ref: CapsuleRef) -> Option<&Capsule> {
        if let Some(slot) = self.capsules.get(frame_ref.id) {
            if slot.generation == frame_ref.generation {
                return slot.capsule.as_ref();
            }
        }

        None
    }

    /// Safely gets a mutable reference to a capsule.
    fn get_capsule_mut(&mut self, frame_ref: CapsuleRef) -> Option<&mut Capsule> {
        if let Some(slot) = self.capsules.get_mut(frame_ref.id) {
            if slot.generation == frame_ref.generation {
                return slot.capsule.as_mut();
            }
        }
        None
    }
}

impl Root {
    fn set_dirty(&mut self, capsule_ref: CapsuleRef) {
        if !self.dirties.insert(capsule_ref) {
            return;
        }

        let mut current = self.get_capsule(capsule_ref);
        while let Some(capsule) = current {
            if let Some(parent_ref) = capsule.parent_ref {
                if !self.dirties.insert(parent_ref) {
                    break; // Parent already dirty
                }
                current = self.get_capsule(parent_ref);
            } else {
                break; // Reached the top
            }
        }
    }
}

impl Root {
    pub fn compute(&mut self) {
        if self.dirties.is_empty() {
            return;
        }

        // We are going to re-compute everything
        self.dirties.clear();

        // 1. Get the screen's dimensions from the root space (space[0])
        let (root_w, root_h) = {
            let root_space = self.spaces[0].unwrap();
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
            .enumerate() // Gives us (i, slot)
            .filter_map(|(i, slot)| {
                slot.capsule.as_ref().and_then(|capsule_data| {
                    if capsule_data.parent_ref.is_none() {
                        Some(CapsuleRef {
                            id: i,
                            generation: slot.generation,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        // 3. Run Pass 1 (Measure) and Pass 2 (Layout) for each top-level frame.
        for capsule_ref in top_level_capsules {
            // Start Pass 1: This computes the "desired" size for all nodes
            // in this tree, storing it in their `Space`.
            self.compute_pass_1_measure(capsule_ref);

            // Start Pass 2: This gives each node its final position and size,
            // using the root dimensions as the available space.
            // A top-level node's "given" space is its own desired size,
            // but it's positioned at (0,0).
            // (Unless it's `Fill` or `Percent`, in which case it gets root_w/root_h)
            // Let's simplify and just pass the root size. Pass 2 will resolve it.
            self.compute_pass_2_layout(capsule_ref, 0, 0, root_w, root_h);
        }
    }
}

impl Root {
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let root_space = self.spaces[0]
            .as_mut()
            .expect("Root space [0] must always exist");

        root_space.width = Some(new_width);
        root_space.height = Some(new_height);

        let top_level_capsules = self
            .capsules
            .iter()
            .enumerate() // Gives us (i, slot)
            .filter_map(|(i, slot)| {
                slot.capsule.as_ref().and_then(|capsule_data| {
                    if capsule_data.parent_ref.is_none() {
                        Some(CapsuleRef {
                            id: i,
                            generation: slot.generation,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();

        for capsule_ref in top_level_capsules {
            self.set_dirty(capsule_ref);
        }
    }
}

impl Root {
    pub fn remove_frame(&mut self, frame_ref: CapsuleRef) {
        let capsule = match self.get_capsule_mut(frame_ref) {
            Some(cap) => cap.clone(), // We must clone it to release the `&mut self`
            None => return,           // Handle is old or invalid, do nothing
        };

        self.unbind_data(frame_ref);
        for child_ref in capsule.children.clone() {
            self.remove_frame(child_ref); // This call is now safe
        }

        if let Some(parent_ref) = capsule.parent_ref {
            if let Some(parent_capsule) = self.get_capsule_mut(parent_ref) {
                parent_capsule.children.retain(|&c| c != frame_ref);
                self.set_dirty(parent_ref);
            }
        }

        self.spaces[capsule.space_ref] = None;
        self.styles[capsule.style_ref] = None;
        self.dirties.remove(&frame_ref);

        // NOTE: Get the slot, `take()` the capsule, and increment the generation
        let slot = &mut self.capsules[frame_ref.id];
        slot.capsule = None; // The capsule is now gone
        slot.generation = slot.generation.wrapping_add(1); // Increment generation

        // Add the ID to the free list for recycling
        self.capsule_free_list.push_back(frame_ref.id);
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
        let (capsule, style, space_ref) = match self.get_capsule(frame_ref).and_then(|cap| {
            // Chain the getters. Get capsule, then its style.
            let style = self.styles[cap.style_ref].as_ref()?;
            Some((cap.clone(), style.clone(), cap.space_ref)) // Clone them
        }) {
            Some((cap, style, sref)) => (cap, style, sref),
            None => return, // Dead handle or missing style, skip.
        };

        let space = match self.spaces[space_ref].as_mut() {
            Some(s) => s,
            None => return, // This space was removed, skip.
        };

        // 1 - Determine My Final Size
        // Get my "desired" size from Pass 1
        let desired_w = space.width.unwrap();
        let desired_h = space.height.unwrap();

        // `Pixel`, `Percent`, `Fill` are resolved against `given_width`.
        // `Fit` returns `None`, so we `unwrap_or` our desired size from Pass 1.
        let final_w = style.width.resolve_size(given_width).unwrap_or(desired_w);
        let final_h = style.height.resolve_size(given_height).unwrap_or(desired_h);

        // 2 - Determine My Final Position
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

        // 3 - Store My Final Space
        space.x = final_x;
        space.y = final_y;
        space.width = Some(final_w);
        space.height = Some(final_h);

        // 4 - Calculate My "Content Box" for My Children
        let content_x = final_x + style.padding.left as i32 + style.border.size as i32;
        let content_y = final_y + style.padding.top as i32 + style.border.size as i32;
        let content_w = final_w
            .saturating_sub(style.padding.left + style.padding.right + style.border.size * 2);
        let content_h = final_h
            .saturating_sub(style.padding.top + style.padding.bottom + style.border.size * 2);

        // 5 - Pre-pass: Analyze In-Flow Children for Flex 'Fill'
        // We need to know how many `Fill` children we have to divide space.
        let mut in_flow_children = Vec::new();
        let mut total_base_w = 0.0;
        let mut total_base_h = 0.0;

        let mut total_grow_factor_w = 0.0;
        let mut total_grow_factor_h = 0.0;

        let mut total_weighted_shrink_w = 0.0;
        let mut total_weighted_shrink_h = 0.0;

        for &child_ref in &capsule.children {
            let (child_style, child_space) = match self.get_capsule(child_ref).and_then(|cap| {
                let style = self.styles[cap.style_ref].as_ref()?;
                let space = self.spaces[cap.space_ref].as_ref()?;
                Some((style, space))
            }) {
                Some((s, sp)) => (s, sp),
                None => continue, // Dead handle or missing data, skip
            };

            if child_style.position == Position::Auto {
                in_flow_children.push(child_ref);

                let base_w = child_space.width.unwrap() as f32;
                let base_h = child_space.height.unwrap() as f32;

                let (child_desired_w, child_desired_h) = (
                    child_space.width.unwrap() as f32,  // Use f32
                    child_space.height.unwrap() as f32, // Use f32
                );

                if style.flow == Direction::Row {
                    // Add to total base size (respecting Fill/Percent)
                    if !child_style.width.is_fill() && !child_style.width.is_percent() {
                        total_base_w += child_desired_w;
                    }
                    total_grow_factor_w += child_style.flex_grow;
                    total_weighted_shrink_w += child_style.flex_shrink * base_w;
                } else {
                    if !child_style.height.is_fill() && !child_style.height.is_percent() {
                        total_base_h += child_desired_h;
                    }
                    total_grow_factor_h += child_style.flex_grow;
                    total_weighted_shrink_h += child_style.flex_shrink * base_h;
                }
            }
        }

        // 7 - Calculate Space for 'Fill' Children
        let total_gap_w = if style.flow == Direction::Row && !in_flow_children.is_empty() {
            style.gap * (in_flow_children.len() as u32 - 1)
        } else {
            0
        } as f32;

        let total_gap_h = if style.flow == Direction::Column && !in_flow_children.is_empty() {
            style.gap * (in_flow_children.len() as u32 - 1)
        } else {
            0
        } as f32;

        let remaining_w = (content_w as f32) - total_base_w - total_gap_w;
        let remaining_h = (content_h as f32) - total_base_h - total_gap_h;

        // These will store our "per-point" ratios
        let mut grow_per_factor_w = 0.0;
        let mut grow_per_factor_h = 0.0;
        let mut shrink_ratio_w = 0.0;
        let mut shrink_ratio_h = 0.0;

        if remaining_w > 0.0 {
            // GROW LOGIC
            if total_grow_factor_w > 0.0 {
                grow_per_factor_w = remaining_w / total_grow_factor_w;
            }
        } else if remaining_w < 0.0 {
            // SHRINK LOGIC
            let overflow_amount = -remaining_w; // e.g., 100px overflow
            if total_weighted_shrink_w > 0.0 {
                // This is our "shrink multiplier"
                shrink_ratio_w = overflow_amount / total_weighted_shrink_w;
            }
        }

        if remaining_h > 0.0 {
            if total_grow_factor_h > 0.0 {
                grow_per_factor_h = remaining_h / total_grow_factor_h;
            }
        } else if remaining_h < 0.0 {
            let overflow_amount = -remaining_h;
            if total_weighted_shrink_h > 0.0 {
                shrink_ratio_h = overflow_amount / total_weighted_shrink_h;
            }
        }

        // 7 - Recurse and Arrange All Children
        let mut current_x = content_x;
        let mut current_y = content_y;
        let children_to_layout = capsule.children.clone();

        for child_ref in &children_to_layout {
            let (child_capsule, child_style, child_space) =
                match self.get_capsule(*child_ref).and_then(|cap| {
                    let style = self.styles[cap.style_ref].as_ref()?;
                    let space = self.spaces[cap.space_ref].as_ref()?;
                    Some((cap.clone(), style.clone(), space)) // Clone what we need
                }) {
                    Some((cap, style, space)) => (cap, style, space),
                    None => continue, // Dead handle
                };

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
                    let base_w = child_desired_w as f32;
                    let base_h = child_desired_h as f32;

                    let m_left = child_style.margin.left as i32;
                    let m_right = child_style.margin.right as i32;
                    let m_top = child_style.margin.top as i32;
                    let m_bottom = child_style.margin.bottom as i32;

                    match style.layout {
                        LayoutStrategy::Flex => match style.flow {
                            Direction::Row => {
                                child_given_x = current_x + m_left;
                                child_given_y = current_y + m_top; // Align top with margin

                                let final_child_w = if remaining_w > 0.0 {
                                    base_w + (child_style.flex_grow * grow_per_factor_w) // Grow
                                } else if remaining_w < 0.0 {
                                    let weighted_shrink = child_style.flex_shrink * base_w; // Shrink
                                    base_w - (weighted_shrink * shrink_ratio_w)
                                } else {
                                    base_w // Fits perfectly
                                };

                                child_given_w = match child_style.width {
                                    SizeSpec::Percent(_) => content_w,
                                    _ => final_child_w as u32,
                                };
                                child_given_h = content_h.saturating_sub((m_top + m_bottom) as u32); // Flex row items fill height minus margin
                            }
                            Direction::Column => {
                                child_given_x = current_x + m_left; // Align left with margin
                                child_given_y = current_y + m_top;
                                child_given_w = content_w.saturating_sub((m_left + m_right) as u32); // Flex col items fill width minus margin

                                let final_child_h = if remaining_h > 0.0 {
                                    base_h + (child_style.flex_grow * grow_per_factor_h) // Grow
                                } else if remaining_h < 0.0 {
                                    let weighted_shrink = child_style.flex_shrink * base_h; // Shrink
                                    base_h - (weighted_shrink * shrink_ratio_h)
                                } else {
                                    base_h // Fits perfectly
                                };

                                child_given_h = match child_style.height {
                                    SizeSpec::Percent(_) => content_h,
                                    _ => final_child_h as u32,
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

                    let child_space_mut = match self.spaces[child_capsule.space_ref].as_mut() {
                        Some(s) => s,
                        None => continue, // This child's space was removed
                    };

                    if style.layout == LayoutStrategy::Flex {
                        if style.flow == Direction::Row && child_style.height.is_auto() {
                            child_space_mut.height = Some(content_h);
                        }
                        if style.flow == Direction::Column && child_style.width.is_auto() {
                            child_space_mut.width = Some(content_w);
                        }
                    }

                    // Update cursor for next in-flow item
                    match style.layout {
                        LayoutStrategy::Flex => {
                            let (child_final_w, child_final_h) = {
                                (
                                    child_space_mut.width.unwrap(),
                                    child_space_mut.height.unwrap(),
                                )
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
    pub fn get_style(&self, frame_ref: CapsuleRef) -> Option<Style> {
        self.get_capsule(frame_ref).and_then(|cap| {
            // Chain the getters. Get capsule, then its style.
            let style = self.styles[cap.style_ref].as_ref()?;
            Some(style.clone())
        })
    }

    pub fn get_space(&self, frame_ref: CapsuleRef) -> Option<Space> {
        self.get_capsule(frame_ref).and_then(|cap| {
            // Chain the getters. Get capsule, then its style.
            let space = self.spaces[cap.space_ref].as_ref()?;
            Some(space.clone())
        })
    }
}

impl Root {
    /// PASS 1 (Bottom-Up): Measure desired content size.
    /// Returns (desired_width, desired_height)
    fn compute_pass_1_measure(&mut self, frame_ref: CapsuleRef) -> (u32, u32) {
        let (capsule, style) = match self.get_capsule(frame_ref).and_then(|cap| {
            // Chain the getters. Get capsule, then its style.
            let style = self.styles[cap.style_ref].as_ref()?;
            Some((cap.clone(), style.clone())) // Clone them
        }) {
            Some((cap, style)) => (cap, style),
            None => return (0, 0), // Dead handle or missing style, skip.
        };

        // 1 - Recurse and Measure "In-Flow" Children
        // Children with `Position::Fixed` are "out-of-flow" and do not
        // contribute to their parent's `FitContent` size.
        let mut in_flow_child_sizes = Vec::new();
        for &child_ref in &capsule.children {
            let child_style = match self
                .get_capsule(child_ref)
                .and_then(|cap| self.styles[cap.style_ref].as_ref())
            {
                Some(style) => style.clone(),
                None => continue, // Dead handle or missing style
            };

            // Recurse for all children
            let (child_w, child_h) = self.compute_pass_1_measure(child_ref);

            // Only "Auto" children participate in the parent's `Fit` sizing
            if child_style.position == Position::Auto {
                in_flow_child_sizes.push((child_w, child_h, child_style.margin));
            }
        }

        // 2 - Calculate This Node's "Content" Size
        let (mut content_w, mut content_h);

        if !capsule.children.is_empty() {
            // Calculate content size based on children (if we are `Fit`)
            match style.layout {
                LayoutStrategy::Flex => {
                    match style.flow {
                        Direction::Row => {
                            // Width is sum of child widths + gaps
                            content_w = in_flow_child_sizes
                                .iter()
                                .map(|(w, _, m)| *w + m.left + m.right)
                                .sum();
                            if !in_flow_child_sizes.is_empty() {
                                content_w += style.gap * (in_flow_child_sizes.len() as u32 - 1);
                            }
                            // Height is max of child heights
                            content_h = in_flow_child_sizes
                                .iter()
                                .map(|(_, h, m)| *h + m.top + m.bottom)
                                .max()
                                .unwrap_or(0);
                        }
                        Direction::Column => {
                            // Width is max of child widths
                            content_w = in_flow_child_sizes
                                .iter()
                                .map(|(w, _, m)| *w + m.left + m.right)
                                .max()
                                .unwrap_or(0);
                            // Height is sum of child heights + gaps
                            content_h = in_flow_child_sizes
                                .iter()
                                .map(|(_, h, m)| *h + m.top + m.bottom)
                                .sum();
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
                        .map(|(w, _, m)| *w + m.left + m.right)
                        .max()
                        .unwrap_or(0);
                    content_h = in_flow_child_sizes
                        .iter()
                        .map(|(_, h, m)| *h + m.top + m.bottom)
                        .max()
                        .unwrap_or(0);
                }
            }
        } else {
            content_w = style.intrinsic_width.unwrap_or(0);
            content_h = style.intrinsic_height.unwrap_or(0);
        }

        // 3 - Determine Final Desired Size Based on Style
        // `Fill` and `Percent` have 0 desired size in Pass 1. They expand in Pass 2.
        let desired_w = match style.width {
            SizeSpec::Pixel(w) => w,
            SizeSpec::Fit | SizeSpec::Auto => {
                content_w + style.padding.left + style.padding.right + style.border.size * 2
            }
            SizeSpec::Fill | SizeSpec::Percent(_) => 0,
        };

        let desired_h = match style.height {
            SizeSpec::Pixel(h) => h,
            SizeSpec::Fit | SizeSpec::Auto => {
                content_h + style.padding.top + style.padding.bottom + style.border.size * 2
            }
            SizeSpec::Fill | SizeSpec::Percent(_) => 0,
        };

        // 4 - Store Result in Space
        if let Some(space) = self.spaces[capsule.space_ref].as_mut() {
            space.width = Some(desired_w);
            space.height = Some(desired_h);
        }

        (desired_w, desired_h)
    }
}

#[cfg(feature = "debug")]
impl Root {
    /// Prints a debug representation of the entire layout tree.
    pub fn debug_layout_tree(&self) {
        use ansi_term::Style;
        let s = Style::new().fg(ansi_term::Color::Yellow).bold();

        // 1 - Safely get root space dimensions
        let (w, h) = self
            .spaces
            .get(0)
            .and_then(|s| s.as_ref())
            .map(|s| (s.width.unwrap_or(0), s.height.unwrap_or(0)))
            .unwrap_or((0, 0));

        eprintln!("{}", s.paint(format!("R ┬ {}x{}", w, h)));

        // 2 - Find all *valid* top-level nodes
        let mut top_level_nodes = Vec::new();
        for (i, slot) in self.capsules.iter().enumerate() {
            if let Some(cap) = &slot.capsule {
                if cap.parent_ref.is_none() {
                    top_level_nodes.push(CapsuleRef {
                        id: i,
                        generation: slot.generation,
                    });
                }
            }
        }

        // 3 - Print the tree for each top-level node
        let count = top_level_nodes.len();
        for (i, cref) in top_level_nodes.iter().enumerate() {
            let is_last = i == count - 1;
            // Start with an empty indent string, at depth 0
            self.debug_print_node(*cref, "", is_last);
        }
    }

    /// Recursively prints a single node and its children.
    /// `indent` is the string of `│ ` and `  ` characters.
    /// `is_last` determines if we use `└` or `├`.
    fn debug_print_node(&self, cref: CapsuleRef, indent: &str, is_last: bool) {
        use ansi_term::Style;

        // 1 - Setup Styles & Strings
        let num_s = Style::new().dimmed().bold();
        let dim = Style::new().dimmed();
        let field = Style::new().fg(ansi_term::Color::Purple);
        let _field_name = Style::new().bold();
        let error_s = Style::new().fg(ansi_term::Color::Red);

        // This is the new, more descriptive ref
        let cref_str = format!("{cref:?}");

        // Correctly determine tree-drawing characters
        let branch_char = if is_last { "└" } else { "├" };
        let continue_char = if is_last { " " } else { "│" }; // Note the space

        let branch_str = dim.paint(format!("{indent}{branch_char}─ "));
        let continue_str = format!("{indent}{continue_char}  "); // This is the indent for children

        // 2 - Safely Get All Data
        let (capsule, space, style) = {
            // Use our safe getter
            let Some(slot) = self.capsules.get(cref.id) else {
                eprintln!(
                    "{branch_str}{}",
                    error_s.paint(format!("Capsule {cref_str} [Invalid ID]"))
                );
                return;
            };
            if slot.generation != cref.generation {
                eprintln!(
                    "{branch_str}{}",
                    error_s.paint(format!("Capsule {cref_str} [Old Generation]"))
                );
                return;
            }

            // Now, get the inner data
            let Some(cap) = slot.capsule.as_ref() else {
                eprintln!(
                    "{branch_str}{}",
                    error_s.paint(format!("Capsule {cref_str} [Empty Slot]"))
                );
                return;
            };

            // Safely get space and style, allowing them to be None
            let sp = self.spaces.get(cap.space_ref).and_then(|s| s.as_ref());
            let st = self.styles.get(cap.style_ref).and_then(|s| s.as_ref());

            (cap.clone(), sp, st) // Clone capsule to release `self` borrow
        };

        // 3 - Print This Node's Info
        eprintln!("{branch_str}Capsule({})", num_s.paint(cref_str));

        let info_indent = dim.paint(format!("{continue_str}"));

        // Print Space (safely)
        if let Some(space) = space {
            eprintln!(
                "{info_indent}{dim_space} x={} y={} w={} h={}",
                field.paint(space.x.to_string()),
                field.paint(space.y.to_string()),
                field.paint(format!("{:?}", space.width)),
                field.paint(format!("{:?}", space.height)),
                dim_space = dim.paint("Space: ")
            );
        } else {
            eprintln!("{info_indent}{}", error_s.paint("Space: [None]"));
        }

        // Print Style (safely, with all new fields)
        if let Some(style) = style {
            eprintln!(
                "{info_indent}{dim_style} w={} h={} padding={}",
                field.paint(format!("{:?}", style.width)),
                field.paint(format!("{:?}", style.height)),
                field.paint(format!("{}", style.padding)),
                dim_style = dim.paint("Style: ")
            );
            // --- ADDING MORE INFO ---
            eprintln!(
                "{info_indent}{dim_layout} strategy={} flow={} gap={} pos={}",
                field.paint(format!("{:?}", style.layout)),
                field.paint(format!("{:?}", style.flow)),
                field.paint(style.gap.to_string()),
                field.paint(format!("{:?}", style.position)),
                dim_layout = dim.paint("Layout:")
            );
        } else {
            eprintln!("{info_indent}{}", error_s.paint("Style: [None]"));
        }

        // 4 - Recurse for Children
        let children_count = capsule.children.len();
        for (i, child_cref) in capsule.children.iter().enumerate() {
            let is_last_child = i == children_count - 1;
            self.debug_print_node(*child_cref, &continue_str, is_last_child);
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
