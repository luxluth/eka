use std::time::Instant;

use heka::{Root, color, size, style};

fn main() {
    let mut root = Root::new(800, 600);
    let frame1 = root.add_frame();
    style!(frame1, &mut root, {
        background_color: color!(RED),
        width: size!(fit),
        height: size!(fit),
        padding: 5,
    });

    let frame = root.add_frame_child(&frame1);
    style!(frame, &mut root, {
        background_color: color!(RED),
        width: size!(fill),
        height: size!(100),
    });

    let now = Instant::now();
    root.compute();
    let elapsed = now.elapsed();
    eprintln!("operation took {elapsed:?}");

    root.debug_layout_tree();
}
