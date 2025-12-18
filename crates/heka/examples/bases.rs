use std::time::Instant;

use heka::{Frame, Root, clr, pad, size, style};

fn main() {
    let mut root = Root::new(800, 600);
    let root_frame: Frame = root.add_frame(None);

    style!(root_frame, &mut root, {
        background_color: clr!(red),
        width: size!(fill),
        height: size!(fill),
        padding: pad!(10, 20),
        gap: 10,
    });

    let frame: Frame = root.add_frame_child(&root_frame, None);
    style!(frame, &mut root, {
        background_color: clr!(risd_blue),
        width: size!(fill),
        flex_grow: 1.0,
    });

    let frame: Frame = root.add_frame_child(&root_frame, None);
    style!(frame, &mut root, {
        background_color: clr!(dodger_blue),
        width: size!(fill),
        flex_grow: 1.0,
    });

    root.remove_frame(frame.get_ref());

    let frame: Frame = root.add_frame_child(&root_frame, None);
    style!(frame, &mut root, {
        background_color: clr!(dodger_blue),
        width: size!(fill),
        flex_grow: 1.0,
    });

    root.remove_frame(frame.get_ref());

    let frame: Frame = root.add_frame_child(&root_frame, None);
    style!(frame, &mut root, {
        background_color: clr!(dodger_blue),
        width: size!(fill),
        flex_grow: 2.0,
    });

    let now = Instant::now();
    root.compute();
    let elapsed = now.elapsed();
    eprintln!("operation took {elapsed:?}");

    let now_fast = Instant::now();
    root.compute(); // <-- This one will be ~nanoseconds
    let elapsed_fast = now_fast.elapsed();
    eprintln!("'Do nothing' compute took {elapsed_fast:?}");

    #[cfg(feature = "debug")]
    root.debug_layout_tree();
}
