/// A convenient macro to create a style.
/// ```rust,ignore
/// let s = make_style!({
///     background_color: clr!(RED),
///     width: size!(fill),
///     ...
/// });
/// ```
#[macro_export]
macro_rules! make_style {
    ($($field:ident : $value:expr),* $(,)?) => {{
        let mut style = $crate::Style::default();
        $(
            style.$field = $value;
        )*
        style
    }};
}

/// A convinient macro to modify a frame style.
/// It can be call multiple time in a row for
/// a same frame element
/// ```rust,ignore
/// style!(frame1, &mut root, {
///     background_color: clr!(RED),
///     width: size!(fill),
///     ...
/// });
/// ```
#[macro_export]
macro_rules! style {
    ($elem:expr, $root:expr, {
    $($field:ident : $value:expr),* $(,)?
    }) => {{
        $elem.update_style($root, |style_mut| {
            $(
                style_mut.$field = $value;
            )*
        });
    }};
}

/// A concise way to specify size constraints for width or height.
///
/// This macro supports several modes:
/// * **Fixed pixels:** `size!(100 px)` or `size!(100)`
/// * **Percentage:** `size!(50 %)` (Relative to parent content box)
/// * **Fill:** `size!(fill)` (Takes all remaining space, like `flex-grow: 1`)
/// * **Fit:** `size!(fit)` (Shrinks to fit the content size)
/// * **Auto:** `size!(auto)` (Natural size, usually equivalent to fit)
///
/// # Examples
/// ```rust,ignore
/// let w = size!(fill);
/// let h = size!(50 px);
/// let w2 = size!(25 %);
/// ```
#[macro_export]
macro_rules! size {
    (fill) => {
        $crate::sizing::SizeSpec::Fill
    };
    (fit) => {
        $crate::sizing::SizeSpec::Fit
    };
    (auto) => {
        $crate::sizing::SizeSpec::Auto
    };
    ($val:literal %) => {
        $crate::sizing::SizeSpec::Percent($val as f32 / 100f32)
    };
    ($val:literal px) => {
        $crate::sizing::SizeSpec::Pixel($val)
    };
    ($val:literal) => {
        $crate::sizing::SizeSpec::Pixel($val)
    };
}

/// explicit constructor for a pixel size specification.
///
/// Unlike `size!`, this macro accepts expressions, allowing for calculations.
///
/// # Example
/// ```rust,ignore
/// let width = px!(100 + 20); // Result: 120px
/// ```
#[macro_export]
macro_rules! px {
    ($e:expr) => {
        $crate::sizing::SizeSpec::Pixel($e)
    };
}

/// Defines the layout strategy for a container.
///
/// * `flex` - Lays out children in a row or column (see `flow!`).
/// * `grid` - **(Experimental)** Lays out children in a grid.
/// * `no_layout` - Children are positioned absolutely using `pos!(x, y)`.
#[macro_export]
macro_rules! layout {
    (flex) => {
        $crate::position::LayoutStrategy::Flex
    };
    (grid) => {
        $crate::position::LayoutStrategy::Grid
    };
    (no_layout) => {
        $crate::position::LayoutStrategy::NoStrategy
    };
}

/// Sets the primary axis direction for a Flex layout.
///
/// * `row` - Children are arranged horizontally (Left to Right).
/// * `column` - Children are arranged vertically (Top to Bottom).
#[macro_export]
macro_rules! flow {
    (row) => {
        $crate::position::Direction::Row
    };
    (column) => {
        $crate::position::Direction::Column
    };
}

/// Sets the position of an element within its parent.
///
/// * `auto` - The element is part of the standard layout flow.
/// * `x, y` - The element is removed from the flow and positioned relative
///   to the parent's top-left corner (content box).
///
/// # Examples
/// ```rust,ignore
/// pos!(auto);       // Standard flow
/// pos!(10, 50);     // Fixed at x:10, y:50
/// ```
#[macro_export]
macro_rules! pos {
    (auto) => {
        $crate::position::Position::Auto
    };
    ($x:expr, $y:expr) => {
        $crate::position::Position::Fixed { x: $x, y: $y }
    };
}

/// Defines a color using a named preset or a Hex literal.
///
/// # Hex Format
/// When using a literal, the format must be **0xRRGGBBAA**.
/// * **RR**: Red (00-FF)
/// * **GG**: Green (00-FF)
/// * **BB**: Blue (00-FF)
/// * **AA**: Alpha (00-FF, where FF is opaque)
///
/// # Examples
/// ```rust,ignore
/// clr!(red);           // Named color
/// clr!(0xFF0000FF);    // Opaque Red
/// clr!(0x00FF0080);    // Semi-transparent Green
/// ```
#[macro_export]
macro_rules! clr {
    ($name:ident) => {
        $crate::color::Color::$name
    };
    ($hex:literal) => {
        $crate::color::Color::Hex($hex)
    };
}

/// Creates a solid opaque color from RGB components.
///
/// Arguments should be `u8` (0-255). Alpha is set to 255 (Opaque).
///
/// # Example
/// ```rust,ignore
/// rgb!(255, 0, 0) // Red
/// ```
#[macro_export]
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        $crate::color::Color {
            r: $r,
            g: $g,
            b: $b,
            a: 255,
        }
    };
}

/// Creates a color from RGBA components.
///
/// Arguments should be `u8` (0-255).
///
/// # Example
/// ```rust,ignore
/// rgba!(255, 0, 0, 128) // 50% transparent Red
/// ```
#[macro_export]
macro_rules! rgba {
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        $crate::color::Color {
            r: $r,
            g: $g,
            b: $b,
            a: $a,
        }
    };
}

/// Creates a color from Hue, Saturation, and Lightness.
///
/// # Arguments
/// * `h` - Hue in degrees (0 - 360)
/// * `s` - Saturation (0.0 - 1.0)
/// * `l` - Lightness (0.0 - 1.0)
///
/// # Example
/// ```rust,ignore
/// let red = hsl!(0, 1.0, 0.5);
/// let pastel_blue = hsl!(200, 0.7, 0.8);
/// ```
#[macro_export]
macro_rules! hsl {
    ($h:expr, $s:expr, $l:expr) => {
        $crate::color::Color::from_hsl($h as f32, $s as f32, $l as f32)
    };
}

/// Creates a color from Hue, Saturation, Lightness, and Alpha.
///
/// # Arguments
/// * `h` - Hue in degrees (0 - 360)
/// * `s` - Saturation (0.0 - 1.0)
/// * `l` - Lightness (0.0 - 1.0)
/// * `a` - Alpha (0.0 - 1.0)
///
/// # Example
/// ```rust,ignore
/// let transparent_red = hsla!(0, 1.0, 0.5, 0.5);
/// ```
#[macro_export]
macro_rules! hsla {
    ($h:expr, $s:expr, $l:expr, $a:expr) => {
        $crate::color::Color::from_hsla($h as f32, $s as f32, $l as f32, $a as f32)
    };
}

/// A convenient macro to specify padding.
///
/// # Argument Order
/// 1. `pad!(all)` - Applies to all sides.
/// 2. `pad!(lr, tb)` - Left/Right, Top/Bottom.
/// 3. `pad!(left, right, top, bottom)` - **Note:** This order differs from CSS (TRBL).
///
/// # Examples
/// ```rust,ignore
/// pad!(10);             // 10px all around
/// pad!(10, 20);         // 10px horizontal, 20px vertical
/// pad!(5, 10, 15, 20);  // Left:5, Right:10, Top:15, Bottom:20
/// ```
#[macro_export]
macro_rules! pad {
    ($value:expr) => {
        $crate::sizing::Padding::all($value)
    };
    ($lr:expr, $tb:expr) => {
        $crate::sizing::Padding::lr_tb($lr, $tb)
    };
    ($l:expr, $r:expr, $t:expr, $b:expr) => {
        $crate::sizing::Padding::new($l, $r, $t, $b)
    };
}

/// A convenient macro to specify margin.
///
/// # Argument Order
/// 1. `margin!(all)` - Applies to all sides.
/// 2. `margin!(lr, tb)` - Left/Right, Top/Bottom.
/// 3. `margin!(left, right, top, bottom)` - **Note:** This order differs from CSS (TRBL).
///
/// # Examples
/// ```rust,ignore
/// margin!(10);             // 10px all around
/// margin!(10, 20);         // 10px horizontal, 20px vertical
/// margin!(5, 10, 15, 20);  // Left:5, Right:10, Top:15, Bottom:20
/// ```
#[macro_export]
macro_rules! margin {
    ($value:expr) => {
        $crate::sizing::Margin::all($value)
    };
    ($lr:expr, $tb:expr) => {
        $crate::sizing::Margin::lr_tb($lr, $tb)
    };
    ($l:expr, $r:expr, $t:expr, $b:expr) => {
        $crate::sizing::Margin::new($l, $r, $t, $b)
    };
}

/// Specifies a border for an element.
///
/// # Examples
/// ```rust,ignore
/// border!(2);                 // 2px wide, default color (Black/Transparent)
/// border!(2, clr!(red));    // 2px wide, Red
/// border!(2, 5, clr!(red)); // 2px wide, 5px radius, Red
/// ```
#[macro_export]
macro_rules! border {
    ($size:expr) => {
        $crate::sizing::Border {
            size: $size,
            ..Default::default()
        }
    };
    ($size:expr, $color:expr) => {
        $crate::sizing::Border {
            size: $size,
            color: $color,
            ..Default::default()
        }
    };
    ($size:expr, $radius:expr, $color:expr) => {
        $crate::sizing::Border {
            size: $size,
            radius: $radius,
            color: $color,
        }
    }; // Ambiguity resolution: If 2 args are numbers, assume size and radius.
       // If 2 args are number and Color (expr), it's handled by the macro matcher if types were checked,
       // but macro matching is purely syntactic.
       // We can't easily distinguish `border!(2, 5)` vs `border!(2, clr!(RED))`.
       // So we rely on explicit usage or differing syntax if needed.
       // However, since `clr!` expands to `Color::...`, we can try to add a variant for 3 args.
}

/// Specifies a shadow for an element.
///
/// # Examples
/// ```rust,ignore
/// shadow!(10.0);                 // 10px blur, default color (Black)
/// shadow!(10.0, clr!(red));    // 10px blur, Red
/// ```
#[macro_export]
macro_rules! shadow {
    ($blur:expr) => {
        $crate::color::Shadow {
            blur: $blur,
            ..Default::default()
        }
    };
    ($blur:expr, $color:expr) => {
        $crate::color::Shadow {
            blur: $blur,
            color: $color,
        }
    };
}

/// Sets the distribution of children along the **main axis**.
///
/// This macro corresponds to the CSS `justify-content` property. It determines how
/// remaining free space is distributed between items when the items do not occupy
/// the total available size.
///
/// The behavior changes based on the parent's flow direction:
///
/// * **Row context:** Controls horizontal distribution (Left to Right).
/// * **Column context:** Controls vertical distribution (Top to Bottom).
///
/// # Note on Flex Grow
/// This property is only effective if there is **remaining space**. If children are
/// configured to grow (e.g., `flex_grow: 1.0`) and they consume all available space,
/// this setting will have no visible effect.
///
/// # Arguments
/// * `start` - Packs items toward the start of the main axis (Default).
/// * `center` - Packs items tightly around the center of the main axis.
/// * `end` - Packs items toward the end of the main axis.
/// * `space-between` - Distributes items evenly. The first item is flush with the start, the last is flush with the end.
/// * `space-around` - Distributes items with equal space around them. Visually, the space between items is twice as wide as the space at the edges.
/// * `space-evenly` - Distributes items so that the spacing between any two items (and the edges) is exactly the same.
#[macro_export]
macro_rules! justify {
    (start) => {
        $crate::position::JustifyContent::Start
    };
    (center) => {
        $crate::position::JustifyContent::Center
    };
    (end) => {
        $crate::position::JustifyContent::End
    };
    (space-between) => {
        $crate::position::JustifyContent::SpaceBetween
    };
    (space-around) => {
        $crate::position::JustifyContent::SpaceAround
    };
    (space-evenly) => {
        $crate::position::JustifyContent::SpaceEvenly
    };
}

/// Sets the alignment of children along the **cross axis**.
///
/// This macro corresponds to the CSS `align-items` property. The behavior changes
/// based on the parent's flow direction:
///
/// * **Row context:** Controls vertical alignment (Top, Middle, Bottom).
/// * **Column context:** Controls horizontal alignment (Left, Center, Right).
///
/// # Arguments
/// * `start` - Aligns items to the start of the cross axis (Top or Left).
/// * `center` - Aligns items to the center of the cross axis.
/// * `end` - Aligns items to the end of the cross axis (Bottom or Right).
#[macro_export]
macro_rules! align {
    (start) => {
        $crate::position::AlignItems::Start
    };
    (center) => {
        $crate::position::AlignItems::Center
    };
    (end) => {
        $crate::position::AlignItems::End
    };
}
