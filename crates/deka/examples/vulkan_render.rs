use deka::{DAL, TextStyle, WindowAttr};

fn main() -> Result<(), impl std::error::Error> {
    let mut dal = DAL::new(
        1000,
        700,
        WindowAttr {
            resizable: true,
            title: "Hello from Deka!".into(),
            ..WindowAttr::default()
        },
    );
    let _ = dal.new_label(
        "Hello from Deka!",
        None,
        Some(TextStyle {
            font_size: 32.0,
            font_family: cosmic_text::FamilyOwned::Monospace,
            ..Default::default()
        }),
    );

    dal.debug();
    dal.run()
}
