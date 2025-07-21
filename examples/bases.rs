use heka::{Color, Root, style};

fn main() {
    let mut root = Root::new(800, 600);
    let frame = root.add_frame();
    style!(frame, &mut root, {
        background_color: Color::RED,
    });

    frame.update_style(&mut root);

    eprintln!("{frame:?}");
    eprintln!("{root:?}");
}
