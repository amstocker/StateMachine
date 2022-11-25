use wgpu::util::StagingBelt;
use wgpu::{Device, TextureFormat, CommandEncoder, TextureView, Color, DepthStencilState};
use wgpu_glyph::{GlyphBrushBuilder, ab_glyph::FontArc, GlyphBrush, Section};
use winit::dpi::PhysicalSize;

use crate::ui::Depth;
use crate::ui::{fonts::*, render::Renderer};
use crate::ui::util::color_to_f32_array;


pub struct Text {
    pub text: String,
    pub position: (f32, f32),
    pub scale: f32,
    pub color: Color,
    pub depth: Depth
}

impl Text {
    fn into_section(&self, bounds: (f32, f32)) -> Section {
        Section {
            screen_position: (
                self.position.0 * bounds.0,
                (1.0 - self.position.1) * bounds.1
            ),
            bounds,
            text: vec![wgpu_glyph::Text::new(&self.text)
                .with_color(color_to_f32_array(self.color))
                .with_scale(self.scale)
                .with_z(self.depth.z())],
            ..Section::default()
        }
    }
}

pub struct TextHandler {
    glyph_brush: GlyphBrush<DepthStencilState>,
    bounds: (f32, f32)
}

impl TextHandler {
    pub fn init(
        device: &Device,
        format: TextureFormat,
        depth_stencil_state: DepthStencilState,
        size: PhysicalSize<u32>
    ) -> Self {
        let font: FontArc = JETBRAINS_MONO.into();
        let glyph_brush = GlyphBrushBuilder::using_font(font.clone())
            .depth_stencil_state(depth_stencil_state)
            .build(device, format);

        let mut drawer = Self {
            glyph_brush,
            bounds: (0.0, 0.0)
        };
        drawer.resize(size);

        drawer
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.bounds = (size.width as f32, size.height as f32);
    }

    pub fn write(&mut self, text: &Text) {
        self.glyph_brush.queue(text.into_section(self.bounds));
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
                wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
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