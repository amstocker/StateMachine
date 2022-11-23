mod background;

pub use background::GridBackground;
use wgpu::{Device, SurfaceConfiguration, RenderPass, Color};
use winit::{event::{WindowEvent, MouseButton, ElementState}, window::{Window, CursorIcon}};

use crate::sequencer::{
    NUM_CHANNELS,
    Clip, MAX_CLIPS_PER_CHANNEL,
    Junction, MAX_JUNCTIONS_PER_CHANNEL, 
    SequencerSummary, DEFAULT_CHANNEL_LENGTH, SequencerController, SequencerEvent, SequencerControlMessage,
    ChannelItemIndex, Playhead, PlayheadState, PlayheadDirection
};
use crate::ui::primitive::{Quad, QuadDrawer, Text, TextDrawer};
use crate::ui::mouse::MousePosition;


#[derive(Debug, Default)]
pub struct ClipInterface {
    model: Clip,
    quad: Quad
}

#[derive(Debug, Default)]
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
    channel_length: u64,
    mouse_position: MousePosition,
    hover_channel_index: usize,
    hover_channel_location: u64,
    hover_clip_index: Option<usize>,
    grabbing: bool,
    grab_channel_index: usize,
    grab_clip_index: usize,
    grab_rel_location: u64
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
            channel_length: DEFAULT_CHANNEL_LENGTH,
            mouse_position: MousePosition::default(),
            hover_channel_index: 0,
            hover_channel_location: 0,
            hover_clip_index: None,
            grabbing: false,
            grab_channel_index: 0,
            grab_clip_index: 0,
            grab_rel_location: 0
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) {
        match event {
            WindowEvent::MouseInput { button, state, .. } => {
                if self.hover_clip_index.is_some() {
                    match (button, state) {
                        (MouseButton::Left, ElementState::Pressed) => {
                            self.handle_clip_grab();
                        },
                        (MouseButton::Left, ElementState::Released) => {
                            self.handle_clip_drop();
                        },
                        _ => {}
                    }
                } else {
                    match state {
                        ElementState::Pressed => {
                            self.set_playhead(self.hover_channel_index, Playhead {
                                state: PlayheadState::Playing,
                                location: self.hover_channel_location,
                                direction: match button {
                                    MouseButton::Left => PlayheadDirection::Right,
                                    _ => PlayheadDirection::Left
                                },
                            });
                        },
                        _ => {},
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = MousePosition::from_physical(position, window.inner_size());
                self.hover_channel_index = NUM_CHANNELS - 1 - (self.mouse_position.y * 4.0).floor() as usize;
                self.hover_channel_location = (self.channel_length as f32 * self.mouse_position.x).floor() as u64;
                if self.grabbing {
                    self.handle_clip_move();
                } else {
                    self.check_clip_hover();
                }
            }
            _ => {}
        };
        self.update_cursor_icon(window);
    }

    pub fn update_cursor_icon(&self, window: &Window) {
        if !self.grabbing {
            if self.hover_clip_index.is_some() {
                window.set_cursor_icon(CursorIcon::Grab);
            } else {
                window.set_cursor_icon(CursorIcon::Default);
            }
        } else {
            window.set_cursor_icon(CursorIcon::Grabbing);
        }
    }

    pub fn handle_clip_grab(&mut self) {
        self.grabbing = true;
        self.grab_channel_index = self.hover_channel_index;
        self.grab_clip_index = self.hover_clip_index.unwrap();
        
        let clip = &mut self.channels[self.grab_channel_index].clips[self.grab_clip_index];
        self.grab_rel_location = self.hover_channel_location - clip.model.channel_location_start;
    }

    pub fn handle_clip_move(&mut self) {
        let clip = &mut self.channels[self.grab_channel_index].clips[self.grab_clip_index];
        let width = clip.model.channel_location_end - clip.model.channel_location_start;
        let start = self.hover_channel_location
            .saturating_sub(self.grab_rel_location)
            .min(self.channel_length - width);
        clip.model.channel_location_start  = start;
        clip.model.channel_location_end = start + width;
        clip.quad = clip_to_quad(
            self.grab_channel_index,
            self.channel_length,
            clip.model
        );
        self.controller.control_message_sender.push(
            SequencerControlMessage::SyncClip {
                index: ChannelItemIndex {
                    channel_index: self.grab_channel_index,
                    item_index: self.grab_clip_index
                },
                clip: clip.model
            }
        ).unwrap();
    }

    pub fn handle_clip_drop(&mut self) {
        self.grabbing = false;
    }

    pub fn check_clip_hover(&mut self) {
        let channel = &self.channels[self.hover_channel_index];
        for clip_index in 0..channel.active_clips {
            if channel.clips[clip_index].quad.contains(self.mouse_position) {
                self.hover_clip_index = Some(clip_index);
                return;
            }
        }
        self.hover_clip_index = None;
    }

    pub fn add_clip(&mut self, channel_index: usize, model: Clip) {
        let channel = &mut self.channels[channel_index];
        channel.clips[channel.active_clips] = ClipInterface {
            model,
            quad: clip_to_quad(
                channel_index,
                self.channel_length,
                model
            )
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

    pub fn set_playhead(&mut self, channel_index: usize, playhead: Playhead) {
        self.controller.control_message_sender.push(
            SequencerControlMessage::SyncPlayhead {
                index: ChannelItemIndex {
                    channel_index,
                    item_index: 0
                },
                playhead
            }
        ).unwrap();
    }

    pub fn start_channel(&mut self, channel_index: usize) {
        self.set_playhead(
            channel_index,
            Playhead {
                state: PlayheadState::Playing,
                location: 0,
                direction: PlayheadDirection::Right,
            }
        );
    }

    pub fn update(&mut self) {
        while let Ok(event) = self.controller.event_receiver.pop() {
            match event {
                SequencerEvent::Tick(summary) => { self.summary = summary; }
            }
        }
    }

    pub fn draw(&self, quad_drawer: &mut QuadDrawer, text_drawer: &mut TextDrawer) {
        for (channel_index, channel) in self.channels.iter().enumerate() {
            for clip in channel.clips.iter().filter(|clip| clip.model.enabled) {
                quad_drawer.draw(clip.quad);
            }

            let playhead = self.summary.playheads[channel_index];
            match playhead.state {
                PlayheadState::Playing => {
                    quad_drawer.draw(playhead_to_quad(
                        channel_index,
                        self.channel_length,
                        self.summary.playheads[channel_index]
                    ));
                },
                _ => {},
            }
            text_drawer.draw(Text {
                text: &format!("{:?}", playhead),
                position: (0.3, (NUM_CHANNELS as f32 - channel_index as f32) / NUM_CHANNELS as f32),
                scale: 30.0,
                color: Color::BLACK
            })
        }
        text_drawer.draw(Text {
            text: &format!("Frames Processed: {}", self.summary.total_frames_processed),
            position: (0.0, 1.0),
            scale: 30.0,
            color: Color::BLACK,
        });
    }

    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.background.render(render_pass);
    }
}

fn clip_to_quad(channel_index: usize, channel_length: u64, clip: Clip) -> Quad {
    let w = (clip.channel_location_end as f32 - clip.channel_location_start as f32) / channel_length as f32;
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = clip.channel_location_start as f32 / channel_length as f32;
    let y = 1.0 - h - (channel_index as f32 / NUM_CHANNELS as f32);
    Quad {
        position: (x, y),
        size: (w, h),
        color: Color { r: 0.2, g: 0.4, b: 0.6, a: 1.0 },
        z: 0.0,
    }
}

fn playhead_to_quad(channel_index: usize, channel_length: u64, playhead: Playhead) -> Quad {
    // TODO: this should be a line
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