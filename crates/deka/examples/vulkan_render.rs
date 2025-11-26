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
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );

    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );

    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );
    dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        Some(panel),
        Some(TextStyle {
            color: Color::risd_blue,
            font_size: 32.0,
            font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
            ..Default::default()
        }),
    );

    let _ = dal.new_button(
        "Click Me!".to_string(),
        Some(panel),
        move |dal, _event| {
            dal.set_label_text(label, "You clicked the button!".to_string());
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
