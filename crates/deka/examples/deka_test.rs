use deka::{Context, Element};

fn main() {
    let mut ctx = Context::new(600, 800, Default::default());
    let label = ctx.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        None::<Element>,
        None,
    );
    println!("{label:?}");

    let button = ctx.new_button(
        "Click Me!".to_string(),
        None::<Element>,
        move |ctx, _event| {
            ctx.set_label_text(label, "You clicked the button!".to_string());
        },
        None,
    );

    println!("{button:?}");

    ctx.compute_layout();
    ctx.debug();
}
