use wgpu::{Device, TextureFormat, CommandEncoder, TextureView, Color};
use wgpu_glyph::{GlyphBrushBuilder, ab_glyph::FontArc, GlyphBrush, Section};

use crate::ui::{fonts::*, State};
use crate::ui::util::color_to_f32_array;


pub struct Text {
    pub text: String,
    pub position: (f32, f32),
    pub scale: f32,
    pub color: Color
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
                .with_scale(self.scale)],
            ..Section::default()
        }
    }
}

pub struct TextDrawer {
    glyph_brush: GlyphBrush<()>,
    bounds: (f32, f32)
}

impl TextDrawer {
    pub fn init(device: &Device, size: (u32, u32), format: TextureFormat) -> Self {
        let font: FontArc = JETBRAINS_MONO.into();
        let glyph_brush = GlyphBrushBuilder::using_font(font.clone()).build(device, format);

        let mut drawer = Self {
            glyph_brush,
            bounds: (0.0, 0.0)
        };
        drawer.resize(size);

        drawer
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        self.bounds = (size.0 as f32, size.1 as f32);
    }

    pub fn draw(&mut self, text: &Text) {
        self.glyph_brush.queue(text.into_section(self.bounds));
    }

    pub fn render(&mut self, encoder: &mut CommandEncoder, view: TextureView, gpu_state: &mut State) {
        self.glyph_brush
            .draw_queued(
                &gpu_state.device,
                &mut gpu_state.staging_belt,
                encoder,
                &view,
                gpu_state.size.width,
                gpu_state.size.height,
            )
            .expect("Draw queued");
    }
}