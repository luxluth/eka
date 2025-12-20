use super::FrameElement;
use heka::color::Color;

/// Checkbox component
pub struct Checkbox {
    pub(crate) frame: heka::Frame,
    pub checked: bool,
}

#[rustfmt::skip]
impl FrameElement for Checkbox {
    fn get_frame(&self) -> heka::Frame { self.frame }
    fn data_ref(&self) -> Option<heka::DataRef> { None }
    fn name(&self) -> &str { "[CHECKBOX]" }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Checkbox {
    pub(crate) fn new(
        root: &mut heka::Root,
        parent_frame: Option<&heka::Frame>,
        initial_checked: bool,
    ) -> Self {
        let frame = if let Some(parent) = parent_frame {
            root.add_frame_child(parent, None)
        } else {
            root.add_frame(None)
        };

        let checked_color = if initial_checked {
            Color::new(100, 100, 255, 255)
        } else {
            Color::new(200, 200, 200, 255)
        };

        frame.update_style(root, |style| {
            style.width = heka::sizing::SizeSpec::Pixel(20);
            style.height = heka::sizing::SizeSpec::Pixel(20);
            style.background_color = checked_color;
            style.border = heka::sizing::Border {
                size: 2,
                radius: 4,
                color: Color::new(50, 50, 50, 255),
            };
        });

        Self {
            frame,
            checked: initial_checked,
        }
    }

    pub fn toggle(&mut self, root: &mut heka::Root) {
        self.checked = !self.checked;
        let checked_color = if self.checked {
            Color::new(100, 100, 255, 255)
        } else {
            Color::new(200, 200, 200, 255)
        };

        self.frame.update_style(root, |style| {
            style.background_color = checked_color;
        });
        self.frame.set_dirty(root);
    }
}
