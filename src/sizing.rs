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
    pub(crate) fn resolve_size(&self, parent_value: u32) -> Option<u32> {
        match self {
            SizeSpec::Pixel(px) => Some(*px),
            SizeSpec::Percent(pct) => Some((*pct * parent_value as f32) as u32),
            SizeSpec::Fill => Some(parent_value),
            SizeSpec::Fit => None,
        }
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
    pub(crate) fn is_fit(&self) -> bool {
        *self == SizeSpec::Fit
    }

    #[inline]
    pub(crate) fn is_fill(&self) -> bool {
        *self == SizeSpec::Fill
    }

    pub(crate) fn is_pixel(&self) -> bool {
        match self {
            SizeSpec::Pixel(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_percent(&self) -> bool {
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
