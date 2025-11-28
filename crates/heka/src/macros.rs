/// A convinient macro to modify a frame style.
/// It can be call multiple time in a row for
/// a same frame element
/// ```rs
/// style!(frame1, &mut root, {
///     background_color: color!(RED),
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

/// A macro to shorten the way size is specified
/// `size!(fill)` | `size!(fit)` | `size!(100%)` | `size!(12 px)` | `size!(2)`
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

/// Denoting a pixel size specificatioin value
#[macro_export]
macro_rules! px {
    ($e:expr) => {
        $crate::sizing::SizeSpec::Pixel($e)
    };
}

/// Macro to define layout strategy (flex, grid)
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

/// Indicates a layout direction (row, column)
#[macro_export]
macro_rules! flow {
    (row) => {
        $crate::position::Direction::Row
    };
    (column) => {
        $crate::position::Direction::Column
    };
}

/// Set an element position `pos!(auto)` | `pos!(x, y)`
#[macro_export]
macro_rules! pos {
    (auto) => {
        $crate::position::Position::Auto
    };
    ($x:expr, $y:expr) => {
        $crate::position::Position::Fixed { x: $x, y: $y }
    };
}

/// A color definition. Can either be a known named color in the library
/// or an hex literal that represent the color.
/// Note: An hex literal must follow the following format - **RRGGBBAA**, the
/// color might be diplayed incorectly if not
#[macro_export]
macro_rules! color {
    ($name:ident) => {
        $crate::color::Color::$name
    };
    ($hex:literal) => {
        $crate::color::Color::Hex($hex)
    };
}

/// Define simple rgba componnent
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

/// Define a simple rgba componnent
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

/// Convinient macro to specify padding
/// `pad!(10)` -> 10 pixel unit on 4 sides
/// `pad!(10, 20)` -> 10 pixel on left and right sides and 10 for the top and bottom sides
/// `pad!(1, 2, 3, 4)` -> single side padding spec - left, right, top, bottom
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

/// Convinient macro to specify margin
/// `margin!(10)` -> 10 pixel unit on 4 sides
/// `margin!(10, 20)` -> 10 pixel on left and right sides and 10 for the top and bottom sides
/// `margin!(1, 2, 3, 4)` -> single side margin spec - left, right, top, bottom
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

/// Convinient macro to specify border
/// `border!(12)` `border(12, Color)`
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
        }
    };
}
