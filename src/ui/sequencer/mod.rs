mod style;

use wgpu::Color;
use winit::{event::{WindowEvent, MouseButton, ElementState}, window::{Window, CursorIcon}};

use crate::sequencer::{
    NUM_CHANNELS,
    Clip, MAX_CLIPS_PER_CHANNEL,
    Junction, MAX_JUNCTIONS_PER_CHANNEL, 
    SequencerSummary, DEFAULT_CHANNEL_LENGTH, SequencerController, SequencerEvent, SequencerControlMessage,
    ChannelItemIndex, Playhead, PlayheadState, PlayheadDirection
};
use crate::ui::Depth;
use crate::ui::render::{RendererController, Primitive, Quad, Text};
use crate::ui::mouse::MousePosition;


#[derive(Debug, Default, Clone, Copy)]
enum Action {
    Channel {
        channel_action: ChannelAction,
        channel_index: usize,
        channel_location: u64
    },
    #[default] NoAction
}

impl Action {
    fn cursor_icon(&self) -> CursorIcon {
        match self {
            Action::Channel { channel_action: action, .. } => {
                match action {
                    ChannelAction::GrabClip { .. } => CursorIcon::Grab,
                    ChannelAction::CreateJunction => CursorIcon::Default,
                    ChannelAction::ModifyJunction => CursorIcon::Default,
                    ChannelAction::SetPlayhead => CursorIcon::Crosshair,
                }
            },
            Action::NoAction => CursorIcon::Default
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ChannelAction {
    GrabClip {
        clip_index: usize
    },
    CreateJunction,
    ModifyJunction,
    SetPlayhead
}

#[derive(Debug, Clone, Copy)]
enum State {
    GrabbingClip {
        channel_index: usize,
        clip_index: usize,
        relative_location: u64
    },
    PotentialAction {
        action: Action
    },
}

impl Default for State {
    fn default() -> Self {
        State::PotentialAction { action: Action::NoAction }
    }
}

impl State {
    fn cursor_icon(&self) -> CursorIcon {
        match self {
            State::GrabbingClip { .. } => CursorIcon::Grabbing,
            State::PotentialAction { action } => {
                action.cursor_icon()
            }
        }
    }
}


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
    channel_length: u64,
    mouse_position: MousePosition,
    state: State
}

impl SequencerInterface {
    pub fn init(controller: SequencerController) -> Self {
        // need global transform uniform
        Self {
            controller,
            channels: Default::default(),
            summary: Default::default(),
            channel_length: DEFAULT_CHANNEL_LENGTH,
            mouse_position: MousePosition::default(),
            state: State::default(),
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) {
        self.state = match event {
            WindowEvent::MouseInput { button, state: element_state, .. } => {
                match self.state {
                    State::GrabbingClip { .. } => {
                        match (button, element_state) {
                            (MouseButton::Left, ElementState::Released) => {
                                self.get_potential_action()
                            },
                            _ => self.state
                        }
                    },
                    State::PotentialAction { action } => {
                        self.handle_action(action, button, element_state)
                    },
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = MousePosition::from_physical(position, window.inner_size());
                match self.state {
                    State::GrabbingClip {
                        channel_index,
                        clip_index,
                        relative_location
                    } => {
                        self.handle_clip_move(channel_index, clip_index, relative_location);
                        self.state
                    },
                    _ => {
                        self.get_potential_action()
                    },
                }
            }
            _ => self.state
        };
        window.set_cursor_icon(self.state.cursor_icon());
    }

    fn get_potential_action(&self) -> State {
        let (channel_index, channel_location) = mouse_position_to_channel(
            self.mouse_position,
            self.channel_length
        );
        let channel = &self.channels[channel_index];
        for clip_index in 0..channel.active_clips {
            if channel.clips[clip_index].quad.contains(self.mouse_position) {
                return State::PotentialAction { 
                    action: Action::Channel {
                        channel_action: ChannelAction::GrabClip { 
                            clip_index
                        },
                        channel_index,
                        channel_location
                    }
                }
            }
        }
        State::PotentialAction { 
            action: Action::Channel {
                channel_action: ChannelAction::SetPlayhead,
                channel_index,
                channel_location
            }
        }
    }

    fn handle_action(&mut self, action: Action, button: &MouseButton, element_state: &ElementState) -> State {
        match action {
            Action::Channel {
                channel_action,
                channel_index,
                channel_location
            } => {
                match channel_action {
                    ChannelAction::GrabClip { clip_index } => {
                        match (button, element_state) {
                            (MouseButton::Left, ElementState::Pressed) => {
                                self.handle_clip_grab(channel_index, channel_location, clip_index)
                            },
                            _ => self.state
                        }
                    },
                    ChannelAction::CreateJunction => todo!(),
                    ChannelAction::ModifyJunction => todo!(),
                    ChannelAction::SetPlayhead => {
                        match element_state {
                            ElementState::Pressed => {
                                self.set_playhead(
                                    channel_index,
                                    Playhead {
                                        state: PlayheadState::Playing,
                                        location: channel_location,
                                        direction: match button {
                                            MouseButton::Left => PlayheadDirection::Right,
                                            _ => PlayheadDirection::Left
                                        },
                                    }
                                );
                            },
                            _ => {},
                        }
                        self.state
                    },
                }
            },
            _ => self.state,
        }
    }

    fn handle_clip_grab(&mut self, channel_index: usize, channel_location: u64, clip_index: usize) -> State {
        let clip = &mut self.channels[channel_index].clips[clip_index];
        State::GrabbingClip {
            relative_location: channel_location - clip.model.channel_location_start,
            channel_index,
            clip_index,
        }
    }

    pub fn handle_clip_move(&mut self, channel_index: usize, clip_index: usize, relative_location: u64) {
        let (_, channel_location) = mouse_position_to_channel(
            self.mouse_position,
            self.channel_length
        );
        let clip = &mut self.channels[channel_index].clips[clip_index];
        let width = clip.model.channel_location_end - clip.model.channel_location_start;
        let start = channel_location
            .saturating_sub(relative_location)
            .min(self.channel_length - width);
        clip.model.channel_location_start  = start;
        clip.model.channel_location_end = start + width;
        clip.quad = clip_to_quad(
            channel_index,
            self.channel_length,
            clip.model
        );
        self.controller.control_message_sender.push(
            SequencerControlMessage::SyncClip {
                index: ChannelItemIndex {
                    channel_index,
                    item_index: clip_index
                },
                clip: clip.model
            }
        ).unwrap();
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

    pub fn update(&mut self) {
        while let Ok(event) = self.controller.event_receiver.pop() {
            match event {
                SequencerEvent::Tick(summary) => { self.summary = summary; }
            }
        }
    }

    pub fn draw(&self, mut renderer_controller: RendererController) {
        for (channel_index, channel) in self.channels.iter().enumerate() {
            for clip in channel.clips.iter().filter(|clip| clip.model.enabled) {
                renderer_controller.draw(Primitive::Quad(clip.quad));
            }

            let playhead = self.summary.playheads[channel_index];
            match playhead.state {
                PlayheadState::Playing => {
                    renderer_controller.draw(playhead_to_primitive(
                        channel_index,
                        self.channel_length,
                        self.summary.playheads[channel_index]
                    ));
                },
                _ => {},
            }
            renderer_controller.draw(Primitive::Text(Text {
                text: format!("{:?}", playhead),
                position: (0.3, (NUM_CHANNELS as f32 - channel_index as f32) / NUM_CHANNELS as f32),
                scale: 30.0,
                color: Color::BLACK,
                depth: Depth::Top
            }))
        }
        renderer_controller.draw(Primitive::Text(Text {
            text: format!("Frames Processed: {}", self.summary.total_frames_processed),
            position: (0.0, 1.0),
            scale: 30.0,
            color: Color::BLACK,
            depth: Depth::Top
        }));
    }
}


fn mouse_position_to_channel(
    mouse_position: MousePosition,
    channel_length: u64,
) -> (usize, u64) {
    (
        (NUM_CHANNELS - 1 - (mouse_position.y * 4.0).floor() as usize)
            .clamp(0, NUM_CHANNELS - 1),
        (channel_length as f32 * mouse_position.x).floor() as u64
    )
}

fn clip_to_quad(channel_index: usize, channel_length: u64, clip: Clip) -> Quad {
    let w = (clip.channel_location_end as f32 - clip.channel_location_start as f32) / channel_length as f32;
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = clip.channel_location_start as f32 / channel_length as f32;
    let y = 1.0 - h - (channel_index as f32 / NUM_CHANNELS as f32);
    Quad {
        position: (x, y),
        size: (w, h),
        color: style::CLIP_COLOR,
        depth: Depth::Mid
    }
}

fn playhead_to_primitive(channel_index: usize, channel_length: u64, playhead: Playhead) -> Primitive {
    let w = 0.002;
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = playhead.location as f32 / channel_length as f32;
    let y = 1.0 - h - (channel_index as f32 / NUM_CHANNELS as f32);
    Primitive::Quad(Quad {
        position: (x, y),
        size: (w, h),
        color: Color::RED,
        depth: Depth::Front
    })
}