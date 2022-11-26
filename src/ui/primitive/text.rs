use wgpu::*;
use wgpu::util::StagingBelt;
use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder};
use wgpu_glyph::{ab_glyph::FontArc, Section};
use winit::dpi::PhysicalSize;

use crate::ui::{Depth, Transform};
use crate::ui::fonts::*;
use crate::util::color_to_f32_array;


pub struct Text {
    pub label: String,
    pub position: (f32, f32),
    pub scale: f32,
    pub color: Color,
    pub depth: Depth
}

impl Text {
    fn into_section_with_transform(&self, bounds: (f32, f32), transform: Transform) -> Section {
        let x = self.position.0 + transform.translate.0;
        let y = self.position.1 + transform.translate.1;
        Section {
            screen_position: (
                x * bounds.0,
                (1.0 - y) * bounds.1
            ),
            bounds,
            text: vec![
                wgpu_glyph::Text::new(&self.label)
                    .with_color(color_to_f32_array(self.color))
                    .with_scale(self.scale)
                    .with_z(self.depth.z())
            ],
            ..Section::default()
        }
    }
}

pub struct TextHandler {
    glyph_brush: GlyphBrush<DepthStencilState>,
    bounds: (f32, f32),
    font_character_ratio: f32
}

impl TextHandler {
    pub fn init(
        device: &Device,
        format: TextureFormat,
        depth_stencil_state: DepthStencilState,
        size: PhysicalSize<u32>
    ) -> Self {
        let font: FontArc = JETBRAINS_MONO.into();
        let glyph_brush =
            GlyphBrushBuilder::using_font(font.clone())
                .depth_stencil_state(depth_stencil_state)
                .build(device, format);

        let mut handler = Self {
            glyph_brush,
            bounds: (0.0, 0.0),
            font_character_ratio: measure_monospace_font_character_ratio(font)
        };
        handler.resize(size);
        handler
    }

    pub fn text_length_to_width(&self, length: usize, scale: f32) -> f32 {
        (length as f32) * self.font_character_ratio * scale
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.bounds = (size.width as f32, size.height as f32);
    }

    pub fn write(&mut self, text: &Text, transform: Transform) {
        self.glyph_brush.queue(text.into_section_with_transform(self.bounds, transform));
    }

    pub fn render(
        &mut self,
        device: &Device,
        staging_belt: &mut StagingBelt,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
        width: u32,
        height: u32
    ) {
        self.glyph_brush
            .draw_queued(
                device,
                staging_belt,
                encoder,
                view,
                RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                },
                width,
                height,
            )
            .expect("Draw queued");
    }
}