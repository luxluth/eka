use std::any::Any;

pub use label::Label;

mod label;

pub trait FrameElement {
    fn get_frame(&self) -> heka::Frame;
    fn data_ref(&self) -> Option<heka::DataRef>;
    fn name(&self) -> &str {
        "[NO_NAME]"
    }

    fn as_any(&self) -> &dyn Any;

    /// Returns this as a `&mut dyn Any`
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
