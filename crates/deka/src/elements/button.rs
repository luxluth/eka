use super::FrameElement;
use crate::Element;

pub struct Button {
    /// The button's main frame (the clickable background)
    pub(crate) frame: heka::Frame,
    /// The handle to the child label
    pub child_label: Element,
}

#[rustfmt::skip]
impl FrameElement for Button {
    fn get_frame(&self) -> heka::Frame { self.frame }
    fn data_ref(&self) -> Option<heka::DataRef> { None } // The frame has no content
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any  { self }
}

impl Button {
    pub fn child(&self) -> Element {
        return self.child_label;
    }
}
