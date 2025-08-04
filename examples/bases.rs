use std::time::Instant;

use heka::{BoxElement, Root, color, pad, size, style};

fn main() {
    let mut root = Root::new(800, 600);
    let frame1: BoxElement = root.add_frame(None);
    style!(frame1, &mut root, {
        background_color: color!(RED),
        width: size!(fill),
        height: size!(fill),
        padding: pad!(10, 20),
    });

    let frame: BoxElement = root.add_frame_child(&frame1, None);
    style!(frame, &mut root, {
        background_color: color!(RED),
        width: size!(fill),
        height: size!(fill),
    });

    let now = Instant::now();
    root.compute();
    let elapsed = now.elapsed();
    eprintln!("operation took {elapsed:?}");

    root.debug_layout_tree();
}
