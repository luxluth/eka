use std::any::Any;

pub use button::Button;
pub use checkbox::Checkbox;
pub use label::Label;
pub use panel::Panel;
pub use text_input::TextInput;

mod button;
mod checkbox;
mod label;
mod panel;
mod text_input;

pub trait FrameElement: 'static {
    fn get_frame(&self) -> heka::Frame;
    fn data_ref(&self) -> Option<heka::DataRef>;
    fn name(&self) -> &str {
        "[NO_NAME]"
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
