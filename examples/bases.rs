use heka::{Root, color, size, style};

fn main() {
    let mut root = Root::new(800, 600);
    let frame = root.add_frame();
    style!(frame, &mut root, {
        background_color: color!(RED),
        width: size!(12 px),
        height: size!(fill),
    });

    root.compute();
    root.dbg(frame.get_ref(), Some("Frame"));
}
