use deka::{DAL, WindowAttr};

fn main() -> Result<(), impl std::error::Error> {
    let mut dal = DAL::new(
        1000,
        700,
        WindowAttr {
            resizable: false,
            title: "Hello from Deka!".into(),
            ..WindowAttr::default()
        },
    );
    let _ = dal.new_label("Hello from Deka!", None, None);

    dal.compute_layout();
    dal.debug();
    dal.run()
}
