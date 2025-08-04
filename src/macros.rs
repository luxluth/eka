#[macro_export]
macro_rules! style {
    ($elem:expr, $root:expr, {
    $($field:ident : $value:expr),* $(,)?
    }) => {{
        {
            let style_mut = $elem.style_mut($root);
            $(
                style_mut.$field = $value;
            )*
            $elem.set_dirty($root);
        }
    }};
}

#[macro_export]
macro_rules! size {
    (fill) => {
        $crate::SizeSpec::Fill
    };
    (fit) => {
        $crate::SizeSpec::Fit
    };
    ($val:literal %) => {
        $crate::SizeSpec::Percent($val as f32 / 100f32)
    };
    ($val:literal px) => {
        $crate::SizeSpec::Pixel($val)
    };
    ($val:literal) => {
        $crate::SizeSpec::Pixel($val)
    };
}

#[macro_export]
macro_rules! layout {
    (flex) => {
        $crate::LayoutStrategy::Flex
    };
    (grid) => {
        $crate::LayoutStrategy::Grid
    };
}

#[macro_export]
macro_rules! flow {
    (row) => {
        $crate::Direction::Row
    };
    (column) => {
        $crate::Direction::Column
    };
}

#[macro_export]
macro_rules! pos {
    (auto) => {
        $crate::Position::Auto
    };
    ($x:expr, $y:expr) => {
        $crate::Position::Fixed { x: $x, y: $y }
    };
}

#[macro_export]
macro_rules! px {
    ($e:expr) => {
        $crate::SizeSpec::Pixel($e)
    };
}

#[macro_export]
macro_rules! color {
    ($name:ident) => {
        $crate::Color::$name
    };
    ($hex:literal) => {
        $crate::Color::Hex($hex)
    };
}

#[macro_export]
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        $crate::Color {
            r: $r,
            g: $g,
            b: $b,
            a: 255,
        }
    };
}

#[macro_export]
macro_rules! rgba {
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        $crate::Color {
            r: $r,
            g: $g,
            b: $b,
            a: $a,
        }
    };
}

#[macro_export]
macro_rules! pad {
    ($value:expr) => {
        $crate::Padding::new_all($value)
    };
    ($lr:expr, $tb:expr) => {
        $crate::Padding::new_lr_tb($lr, $tb)
    };
    ($l:expr, $r:expr, $t:expr, $b:expr) => {
        $crate::Padding::new($l, $r, $t, $b)
    };
}
