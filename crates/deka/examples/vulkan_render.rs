use cosmic_text::FamilyOwned;
use deka::{DAL, PanelRef, TextStyle, WindowAttr};
use heka::{
    Style, align, border, clr,
    color::Shadow,
    flow, justify, pad, shadow, size,
    sizing::{Border, Padding},
};

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

    let mut count = 0;

    let (border_default, shadow_default, first_frame_pad) =
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_lowercase() == "gnome" {
                (
                    border!(1, 15, clr!(0xDDDDDDFF)),
                    shadow!(3., clr!(0x444444FF)),
                    pad!(20),
                )
            } else {
                (Border::default(), Shadow::default(), Padding::default())
            }
        } else {
            (Border::default(), Shadow::default(), Padding::default())
        };

    let outer_panel = dal.new_panel(
        None::<PanelRef>,
        Style {
            flow: flow!(column),
            padding: first_frame_pad,
            width: size!(100%),
            height: size!(100%),
            background_color: clr!(transparent),
            ..Default::default()
        },
    );

    let panel = dal.new_panel(
        Some(outer_panel),
        Style {
            flow: flow!(column),
            gap: 2,
            padding: pad!(20),
            width: size!(100%),
            height: size!(100%),
            justify_content: justify!(center),
            align_items: align!(center),
            shadow: shadow_default,
            border: border_default,
            background_color: clr!(white),
            ..Default::default()
        },
    );

    let label = dal.new_label(
        "Count = 0",
        Some(panel),
        Some(TextStyle {
            color: clr!(risd_blue),
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
