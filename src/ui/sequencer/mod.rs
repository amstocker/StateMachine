mod background;

pub use background::GridBackground;
use wgpu::{Device, SurfaceConfiguration, RenderPass, Color};

use crate::sequencer::{
    NUM_CHANNELS,
    Clip, MAX_CLIPS_PER_CHANNEL,
    Junction, MAX_JUNCTIONS_PER_CHANNEL, 
    SequencerSummary, DEFAULT_CHANNEL_LENGTH, SequencerController, SequencerEvent, SequencerControlMessage,
    ChannelItemIndex, Playhead, PlayheadState, PlayheadDirection
};

use super::{quad::{QuadDrawer, Quad}, text::{TextDrawer, Text}};

#[derive(Default)]
pub struct ClipInterface {
    model: Clip,
    quad: Quad
}

#[derive(Default)]
pub struct JunctionInterface {
    model: Junction
}

#[derive(Default)]
pub struct ChannelInterface {
    clips: [ClipInterface; MAX_CLIPS_PER_CHANNEL],
    junctions: [JunctionInterface; MAX_JUNCTIONS_PER_CHANNEL],
    active_clips: usize,
    active_junctions: usize,
}

pub struct SequencerInterface {
    controller: SequencerController,
    channels: [ChannelInterface; NUM_CHANNELS],
    summary: SequencerSummary,
    background: GridBackground,
    channel_length: u64
}

impl SequencerInterface {
    pub fn init(device: &Device, config: &SurfaceConfiguration, controller: SequencerController) -> Self {
        let background = GridBackground::init(device, config.format);

        // need global transform uniform
        Self {
            controller,
            channels: Default::default(),
            summary: Default::default(),
            background,
            channel_length: DEFAULT_CHANNEL_LENGTH
        }
    }

    pub fn handle_window_event(&mut self) {

    }

    pub fn add_clip(&mut self, channel_index: usize, model: Clip) {
        let channel = &mut self.channels[channel_index];
        let w = (model.channel_location_end as f32 - model.channel_location_start as f32) / self.channel_length as f32;
        let h = 1.0 / NUM_CHANNELS as f32;
        let x = model.channel_location_start as f32 / self.channel_length as f32;
        let y = 1.0 - h - (channel_index as f32 / NUM_CHANNELS as f32);
        channel.clips[channel.active_clips] = ClipInterface {
            model,
            quad: Quad {
                position: (x, y),
                size: (w, h),
                color: Color::BLUE,
                z: 0.0,
            }
        };
        self.controller.control_message_sender.push(
            SequencerControlMessage::SyncClip {
                index: ChannelItemIndex {
                    channel_index,
                    item_index: channel.active_clips
                },
                clip: model
            }
        ).unwrap();
        channel.active_clips += 1;
        
    }

    pub fn start_channel(&mut self, channel_index: usize) {
        self.controller.control_message_sender.push(
            SequencerControlMessage::SyncPlayhead {
                index: ChannelItemIndex { channel_index, item_index: 0 },
                playhead: Playhead {
                    state: PlayheadState::Playing,
                    location: 0,
                    direction: PlayheadDirection::Right,
                }
            }
        ).unwrap();
    }

    pub fn update(&mut self) {
        while let Ok(event) = self.controller.event_receiver.pop() {
            match event {
                SequencerEvent::Tick(summary) => { self.summary = summary; }
            }
        }
    }

    pub fn draw(&self, quad_drawer: &mut QuadDrawer, text_drawer: &mut TextDrawer) {
        for (index, channel) in self.channels.iter().enumerate() {
            for clip in channel.clips.iter().filter(|clip| clip.model.enabled) {
                quad_drawer.draw(clip.quad);
            }

            let playhead = self.summary.playheads[index];
            match playhead.state {
                PlayheadState::Playing => {
                    quad_drawer.draw(playhead_to_quad(
                        index,
                        self.channel_length,
                        self.summary.playheads[index]
                    ));
                },
                PlayheadState::Stopped => {},
            }
            
        }
        text_drawer.draw(Text {
            text: &format!("Frames Processed: {}", self.summary.total_frames_processed),
            position: (0.0, 1.0),
            scale: 30.0,
            color: wgpu::Color::BLACK,
        });
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.background.render(render_pass);
    }
}

fn playhead_to_quad(channel_index: usize, channel_length: u64, playhead: Playhead) -> Quad {
    let w = 0.002;
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = playhead.location as f32 / channel_length as f32;
    let y = 1.0 - h - (channel_index as f32 / NUM_CHANNELS as f32);
    Quad {
        position: (x, y),
        size: (w, h),
        color: Color::RED,
        z: 0.0,
    }
}