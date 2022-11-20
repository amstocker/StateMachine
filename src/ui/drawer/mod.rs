pub mod quad;
pub mod text;
pub mod util;

use quad::{QuadDrawer, Quad};
use text::{TextDrawer, Text};
use wgpu::{Device, SurfaceConfiguration};


pub struct Drawer {
    pub quad: QuadDrawer,
    pub text: TextDrawer
}

impl Drawer {
    pub fn init(device: &Device, config: &SurfaceConfiguration) -> Self {
        Self {
            quad: QuadDrawer::init(device, config.format),
            text: TextDrawer::init(device, (config.width, config.height), config.format)
        }
    }

    pub fn draw_quad(&mut self, quad: &Quad) {
        self.quad.draw(quad);
    }

    pub fn draw_text(&mut self, text: Text) {
        self.text.draw(text);
    }
}