use deka::DAL;

fn main() -> Result<(), impl std::error::Error> {
    let mut dal = DAL::new(800, 600, Default::default());
    let _ = dal.new_label("Hello from Deka!", None, None);

    dal.compute_layout();
    dal.debug();
    dal.run()
}
