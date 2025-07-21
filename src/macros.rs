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
            $elem.update_style($root);
        }
    }};
}
