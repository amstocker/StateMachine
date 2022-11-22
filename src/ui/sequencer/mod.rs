mod background;

pub use background::GridBackground;
use wgpu::{Device, SurfaceConfiguration, RenderPass};

use crate::sequencer::{
    NUM_CHANNELS,
    Clip, MAX_CLIPS_PER_CHANNEL,
    Junction, MAX_JUNCTIONS_PER_CHANNEL, 
    SequencerSummary
};

use super::{quad::QuadDrawer, text::TextDrawer};

#[derive(Default)]
pub struct ClipInterface {
    model: Clip
}

#[derive(Default)]
pub struct JunctionInterface {
    model: Junction
}

#[derive(Default)]
pub struct ChannelInterface {
    clips: [ClipInterface; MAX_CLIPS_PER_CHANNEL],
    junctions: [JunctionInterface; MAX_JUNCTIONS_PER_CHANNEL]
}

pub struct SequencerInterface {
    channels: [ChannelInterface; NUM_CHANNELS],
    summary: SequencerSummary,
    background: GridBackground
}

impl SequencerInterface {
    pub fn init(device: &Device, config: &SurfaceConfiguration) -> Self {
        let background = GridBackground::init(device, config.format);

        // need global transform uniform
        Self {
            channels: Default::default(),
            summary: Default::default(),
            background
        }
    }

    pub fn handle_window_event(&mut self) {

    }

    pub fn draw(&self, quad_drawer: &QuadDrawer, text_drawer: &TextDrawer) {

    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.background.render(render_pass);
    }
}