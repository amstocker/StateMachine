use wgpu::Color;

use crate::ui::{primitive::RendererController, Depth, layout::RelativePosition};


const DEPTH: Depth = Depth::Modal;



pub struct PopupMenu {
    base_position: (f32, f32),
    relative_position: RelativePosition,
    background_color: Color,
    label: String,
    label_scale: f32,
    label_color: Color,
    label_padding: f32
}

impl PopupMenu {
    pub fn draw(&self, renderer_controller: RendererController) {
        let label_width = renderer_controller.text_length_to_width(
            self.label.len(),
            self.label_scale
        );
    }
}