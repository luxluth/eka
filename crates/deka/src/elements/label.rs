use super::FrameElement;
use crate::TextStyle;
use cosmic_text::{Attrs, Buffer, FontSystem, Shaping};

/// Label component
pub struct Label {
    /// The handle to the layout node this component controls
    pub(crate) frame: heka::Frame,
    /// The actual text content
    pub(crate) text: String,
    /// The handle to the cosmic-text buffer, which is
    /// stored in heka's `Allocator`
    pub(crate) buffer_ref: heka::DataRef,

    /// Label Text style
    pub text_style: TextStyle,
}

#[rustfmt::skip]
impl FrameElement for Label {
    fn get_frame(&self) -> heka::Frame { self.frame }
    fn data_ref(&self) -> Option<heka::DataRef> { Some(self.buffer_ref) }
    fn name(&self) -> &str { "[LABEL]" }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Label {
    pub(crate) fn new(
        root: &mut heka::Root,
        parent_frame: Option<&heka::Frame>,
        text: String,
        text_style: TextStyle,
        font_system: &mut FontSystem,
    ) -> Self {
        let metrics = text_style.as_cosmic_metrics();
        let attrs = text_style.as_cosmic_attrs();

        let mut buffer = Buffer::new(font_system, metrics);
        buffer.set_text(
            font_system,
            &text,
            &attrs,
            Shaping::Advanced,
            Some(text_style.align),
        );

        buffer.shape_until_scroll(font_system, true);
        let (measured_width, measured_height) = Self::measure_buffer(&buffer);

        let buffer_ref = root.set_binding(buffer);
        let frame = if let Some(parent) = parent_frame {
            root.add_frame_child(parent, Some(buffer_ref))
        } else {
            root.add_frame(Some(buffer_ref))
        };

        frame.update_style(root, |style| {
            style.width = heka::sizing::SizeSpec::Fit;
            style.height = heka::sizing::SizeSpec::Fit;
            style.intrinsic_width = Some(measured_width);
            style.intrinsic_height = Some(measured_height);
        });

        Self {
            frame,
            text,
            buffer_ref,
            text_style,
        }
    }

    pub(crate) fn set_text(
        &mut self,
        root: &mut heka::Root,
        font_system: &mut FontSystem,
        new_text: String,
    ) {
        if self.text == new_text {
            return;
        }

        self.text = new_text;
        self.remeasure_and_push(root, font_system);
    }

    pub(crate) fn set_style(
        &mut self,
        root: &mut heka::Root,
        font_system: &mut FontSystem,
        new_style: TextStyle,
    ) {
        if self.text_style == new_style {
            return;
        }

        self.text_style = new_style;
        self.remeasure_and_push(root, font_system);
    }

    #[inline]
    pub fn get_text(&self) -> &str {
        &self.text
    }

    fn measure_buffer(buffer: &Buffer) -> (u32, u32) {
        let measured_width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
            .ceil() as u32;

        let metrics_line_height = buffer.metrics().line_height;
        let measured_height = if let Some(last_run) = buffer.layout_runs().last() {
            (last_run.line_y + metrics_line_height).ceil() as u32
        } else {
            0
        };

        (measured_width, measured_height)
    }

    pub(crate) fn remeasure_and_push(
        &mut self,
        root: &mut heka::Root,
        font_system: &mut FontSystem,
    ) {
        if let Some(buffer) = root.get_binding_mut::<Buffer>(self.buffer_ref) {
            buffer.set_text(
                font_system,
                &self.text,
                &Attrs::new(),
                Shaping::Advanced,
                None,
            );

            buffer.shape_until_scroll(font_system, true);

            let (measured_width, measured_height) = Self::measure_buffer(buffer);

            self.frame.update_style(root, |style| {
                style.intrinsic_width = Some(measured_width);
                style.intrinsic_height = Some(measured_height);
            });

            self.frame.set_dirty(root);
        }
    }
}
