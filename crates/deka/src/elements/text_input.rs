use super::FrameElement;
use crate::events::KeyEvent;
use crate::{DAL, Element, ElementRef, LabelRef};

/// TextInput component
pub struct TextInput {
    pub(crate) frame: heka::Frame,
    pub(crate) label: LabelRef,
}

#[rustfmt::skip]
impl FrameElement for TextInput {
    fn get_frame(&self) -> heka::Frame { self.frame }
    fn data_ref(&self) -> Option<heka::DataRef> { None }
    fn name(&self) -> &str { "[TEXT_INPUT]" }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl TextInput {
    pub(crate) fn new(
        dal: &mut DAL,
        parent_frame: Option<impl ElementRef>,
        initial_text: String,
    ) -> Self {
        let parent = if let Some(pf) = parent_frame {
            &heka::Frame::define(pf.raw())
        } else {
            &dal.root_frame
        };

        let input_frame = dal.root.add_frame_child(parent, None);

        input_frame.update_style(&mut dal.root, |style| {
            style.width = heka::sizing::SizeSpec::Pixel(200);
            style.height = heka::sizing::SizeSpec::Pixel(30);
            style.padding = heka::sizing::Padding::all(5);
            style.background_color = heka::color::Color::new(255, 255, 255, 255);
            style.border = heka::sizing::Border {
                size: 1,
                radius: 2,
                color: heka::color::Color::new(150, 150, 150, 255),
            };
            style.layout = heka::position::LayoutStrategy::Flex;
        });

        let label = dal.new_label(initial_text, Some(Element(input_frame.get_ref())), None);

        Self {
            frame: input_frame,
            label,
        }
    }

    pub fn handle_key(&mut self, dal: &mut DAL, event: &KeyEvent) {
        if !event.pressed {
            return;
        }

        use winit::keyboard::Key;
        match &event.logical_key {
            Key::Named(winit::keyboard::NamedKey::Backspace) => {
                let mut text = dal.get_label_text(self.label).to_string();
                text.pop();
                dal.set_label_text(self.label, text);
            }
            _ => {
                if let Some(text_to_append) = &event.text {
                    let mut text = dal.get_label_text(self.label).to_string();
                    text.push_str(text_to_append.as_str());
                    dal.set_label_text(self.label, text);
                }
            }
        }
    }
}
