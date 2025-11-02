use deka::DAL;

fn main() {
    let mut dal = DAL::new(600, 800);
    let label = dal.new_label("Hello from deka", None, None);
    println!("{label:?}");
    dal.compute_layout();
    dal.debug();
}
