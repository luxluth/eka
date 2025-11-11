use deka::DAL;

fn main() {
    let mut dal = DAL::new(600, 800);
    let label = dal.new_label("Hello, Eka!", None, None);
    println!("{label:?}");

    let button = dal.new_button("Click Me!".to_string(), None, move |dal, _event| {
        dal.set_label_text(label, "You clicked the button!".to_string());
    });

    println!("{button:?}");

    dal.compute_layout();
    dal.debug();
}
