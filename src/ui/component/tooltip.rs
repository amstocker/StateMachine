use wgpu::Color;

use crate::ui::primitive::RendererController;


pub enum RelativePosition {
    Above,
    Below,
    Left,
    Right
}

pub struct Tooltip {
    base_position: (f32, f32),
    position: RelativePosition,
    color: Color,
    label: String,
    label_scale: f32,
    label_color: Color,
    label_padding: f32,
    // depth: Depth
}

impl Tooltip {
    pub fn draw(&self, renderer_controller: RendererController) {
        let label_width = renderer_controller.text_length_to_width(
            self.label.len(),
            self.label_scale
        );
    }
}