use cosmic_text::FamilyOwned;
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

    let mut count = 0;

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
        "Count = 0",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );

    let _ = dal.new_button(
        "increment +1".to_string(),
        Some(panel),
        move |dal, _event| {
            count += 1;
            dal.set_label_text(label, format!("Count = {count}").to_string());
        },
        Some(TextStyle {
            font_size: 14.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );

    dal.debug();
    dal.run()
}
