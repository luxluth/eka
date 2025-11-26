use deka::{DAL, Element};

fn main() {
    let mut dal = DAL::new(600, 800, Default::default());
    let label = dal.new_label(
        "The Quick Brown Fox Jumps Over The Lazy Dog",
        None::<Element>,
        None,
    );
    println!("{label:?}");

    let button = dal.new_button(
        "Click Me!".to_string(),
        None::<Element>,
        move |dal, _event| {
            dal.set_label_text(label, "You clicked the button!".to_string());
        },
        None,
    );

    println!("{button:?}");

    dal.compute_layout();
    dal.debug();
}
