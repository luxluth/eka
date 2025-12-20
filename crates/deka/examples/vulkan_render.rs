use cosmic_text::FamilyOwned;
use deka::{Context, TextStyle, WindowAttr, eka};
use heka::{
    align, border, clr,
    color::Shadow,
    flow, justify, make_style, pad, shadow, size,
    sizing::{Border, Padding},
};

fn main() -> Result<(), impl std::error::Error> {
    let mut ctx = Context::new(
        1000,
        700,
        WindowAttr {
            resizable: false,
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

    eka! {
        ctx,
        Panel {
            style: make_style! {
                flow: flow!(column),
                padding: first_frame_pad,
                width: size!(100%),
                height: size!(100%),
                background_color: clr!(transparent),
            },
            children: [
                Panel {
                    style: make_style! {
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
                    },
                    children: [
                        count_label = Label {
                            text: "Count = 0",
                            style: TextStyle {
                                color: clr!(risd_blue),
                                font_size: 32.0,
                                font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
                                ..Default::default()
                            }
                        },
                        Button {
                            text: "increment +1",
                            on_click: move |ctx, _| {
                                count += 1;
                                ctx.set_label_text(count_label, format!("Count = {count}"));
                            },
                            style: TextStyle {
                                font_size: 14.0,
                                font_family: FamilyOwned::Name("Fantasque Sans Mono".into()),
                                ..Default::default()
                            }
                        }
                    ]
                }
            ]
        }
    };

    ctx.debug();
    ctx.run()
}
