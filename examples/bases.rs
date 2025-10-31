use std::time::Instant;

use heka::{BoxElement, Root, color, pad, size, style};

fn main() {
    let mut root = Root::new(800, 600);
    let root_frame: BoxElement = root.add_frame(None);

    style!(root_frame, &mut root, {
        background_color: color!(red),
        width: size!(fill),
        height: size!(fill),
        padding: pad!(10, 20),
        gap: 10,
    });

    let frame: BoxElement = root.add_frame_child(&root_frame, None);
    style!(frame, &mut root, {
        background_color: color!(risd_blue),
        width: size!(fill),
    });

    let frame: BoxElement = root.add_frame_child(&root_frame, None);
    style!(frame, &mut root, {
        background_color: color!(dodger_blue),
        width: size!(fill),
    });

    root.remove_frame(frame.get_ref());

    let now = Instant::now();
    root.compute();
    let elapsed = now.elapsed();
    eprintln!("operation took {elapsed:?}");

    root.debug_layout_tree();

    println!("commands: {:?}", root.commands());
}
