use crate::elements::FrameElement;
use std::any::Any;

/// A simple container component.
#[derive(Debug)]
pub struct Panel {
    pub(crate) frame: heka::Frame,
}

#[rustfmt::skip]
impl FrameElement for Panel {
    fn get_frame(&self) -> heka::Frame { self.frame }
    fn data_ref(&self) -> Option<heka::DataRef> { None }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn name(&self) -> &str { "[PANEL]" }
}
