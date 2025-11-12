use cosmic_text::{FamilyOwned, Weight};
use deka::{DAL, TextStyle, WindowAttr};
use heka::{
    Style,
    color::Color,
    position::Direction,
    sizing::{Padding, SizeSpec},
};

fn main() -> Result<(), impl std::error::Error> {
    let mut dal = DAL::new(
        1000,
        700,
        WindowAttr {
            resizable: false,
            title: "Hello from Deka!".into(),
            ..WindowAttr::default()
        },
    );

    let panel = dal.new_panel(
        None,
        Style {
            flow: Direction::Column,
            gap: 2,
            padding: Padding::new_all(20),
            width: SizeSpec::Fill,
            height: SizeSpec::Fill,
            ..Default::default()
        },
    );

    let label = dal.new_label(
        "Hello, Eka!",
        Some(&panel.frame()),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            weight: Weight::BOLD,
            font_family: FamilyOwned::Name("Inter".into()),
            ..Default::default()
        }),
    );

    let _ = dal.new_button(
        "Click Me!".to_string(),
        Some(&panel.frame()),
        move |dal, _event| {
            dal.set_label_text(label, "You clicked the button!".to_string());
        },
    );

    dal.debug();
    dal.run()
}
