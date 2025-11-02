pub use label::Label;

mod label;

pub trait FrameElement {
    fn get_frame(&self) -> heka::Frame;
    fn data_ref(&self) -> Option<heka::DataRef>;
    fn name(&self) -> &str {
        "[NO_NAME]"
    }
}
