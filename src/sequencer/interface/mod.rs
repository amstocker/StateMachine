mod style;
mod state;

use wgpu::Color;
use winit::{event::{WindowEvent, MouseButton, ElementState}, window::Window};

use state::*;
use crate::sequencer::*;
use crate::ui::Depth;
use crate::ui::primitive::{Draw, Primitive, Quad, Text, Line};
use crate::ui::mouse::MousePosition;
use crate::ui::CLEAR_COLOR;
use crate::ui::{Transform, Transformable};
use crate::ui::primitive::Drawable;


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
    state: State,
    transform: Transform
}

impl SequencerInterface {
    pub fn init(controller: SequencerController) -> Self {
        Self {
            controller,
            channels: Default::default(),
            summary: Default::default(),
            channel_length: DEFAULT_CHANNEL_LENGTH,
            mouse_position: MousePosition::default(),
            state: State::default(),
            transform: Transform::identity()
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    fn get_potential_action(&self) -> Action {
        let channel_index = mouse_position_to_channel_index(self.mouse_position);
        let channel_location = mouse_position_to_channel_location(self.mouse_position, self.channel_length);
        if mouse_position_is_on_junction_lane(self.mouse_position, channel_index) {
            return Action::Channel {
                channel_action: ChannelAction::CreateJunction,
                channel_index,
                channel_location
            }
        }
        let channel = &self.channels[channel_index];
        for clip_index in 0..channel.active_clips {
            if channel.clips[clip_index].quad.contains(self.mouse_position) {
                return Action::Channel {
                    channel_action: ChannelAction::GrabClip { 
                        clip_index
                    },
                    channel_index,
                    channel_location
                }
            }
        }
        Action::Channel {
            channel_action: ChannelAction::SetPlayhead,
            channel_index,
            channel_location
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) {
        self.state = match event {
            WindowEvent::MouseInput { button, state: element_state, .. } => {
                self.handle_mouse_input(button, element_state)
            }
            WindowEvent::CursorMoved { .. } => {
                self.handle_cursor_move()
            }
            _ => self.state
        };
        window.set_cursor_icon(self.state.cursor_icon());
    }

    pub fn set_mouse_position(&mut self, mouse_position: MousePosition) {
        self.mouse_position = mouse_position;
    }

    fn handle_mouse_input(&mut self, button: &MouseButton, element_state: &ElementState) -> State {
        match self.state {
            State::GrabbingClip { .. } => {
                match (button, element_state) {
                    (MouseButton::Left, ElementState::Released) => {
                        State::default()
                    },
                    _ => self.state
                }
            },
            State::Hovering { potential_action } => {
                self.handle_action(
                    potential_action,
                    button,
                    element_state
                )
            },
            State::CreatingJunction { 
                source_channel_index,
                source_channel_location
            } => {
                match (button, element_state) {
                    (MouseButton::Left, ElementState::Released) => {
                        let index = mouse_position_to_channel_index(self.mouse_position);
                        let location = mouse_position_to_channel_location(self.mouse_position, self.channel_length);
                        self.handle_create_junction(
                            source_channel_index,
                            if index == source_channel_index {
                                Junction {
                                    enabled: true,
                                    location,
                                    junction_type: JunctionType::Reflect
                                }
                            } else {
                                Junction {
                                    enabled: true,
                                    location: source_channel_location,
                                    junction_type: JunctionType::Jump {
                                        destination_channel_index: index,
                                        destination_location: location,
                                        split: true
                                    }
                                }
                            }
                        );
                        State::default()
                    },
                    _ => self.state
                }
            },
        }
    }

    fn handle_cursor_move(&mut self) -> State {
        match self.state {
            State::GrabbingClip {
                channel_index,
                clip_index,
                relative_location
            } => {
                self.handle_clip_move(channel_index, clip_index, relative_location);
                self.state
            },
            State::Hovering { .. } => State::Hovering {
                    potential_action: self.get_potential_action()
                },
            State::CreatingJunction { .. } => {
                self.state
            },
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
                                let clip = &self.channels[channel_index].clips[clip_index];
                                State::GrabbingClip {
                                    relative_location: channel_location - clip.model.channel_location_start,
                                    channel_index,
                                    clip_index,
                                }
                            },
                            _ => self.state
                        }
                    },
                    ChannelAction::CreateJunction => {
                        match (button, element_state) {
                            (MouseButton::Left, ElementState::Pressed) => State::CreatingJunction {
                                source_channel_index: channel_index,
                                source_channel_location: channel_location
                            },
                            _ => self.state
                        }
                    },
                    ChannelAction::ModifyJunction => todo!(),
                    ChannelAction::SetPlayhead => {
                        match element_state {
                            ElementState::Pressed => {
                                self.handle_set_playhead(
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

    pub fn handle_clip_move(&mut self, channel_index: usize, clip_index: usize, relative_location: u64) {
        let clip = &mut self.channels[channel_index].clips[clip_index];
        let width = clip.model.channel_location_end - clip.model.channel_location_start;
        let start = mouse_position_to_channel_location(self.mouse_position, self.channel_length)
            .saturating_sub(relative_location)
            .min(self.channel_length - width);
        
        clip.model.channel_location_start = start;
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

    pub fn handle_create_junction(&mut self, channel_index: usize, model: Junction) {
        let channel = &mut self.channels[channel_index];
        channel.junctions[channel.active_junctions] = JunctionInterface {
            model
        };
        self.controller.control_message_sender.push(
            SequencerControlMessage::SyncJunction {
                index: ChannelItemIndex {
                    channel_index,
                    item_index: channel.active_junctions
                },
                junction: model
            }
        ).unwrap();

        channel.active_junctions += 1;
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

    pub fn handle_set_playhead(&mut self, channel_index: usize, playhead: Playhead) {
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

}

impl Transformable for SequencerInterface {
    fn transform(&self) -> Transform {
        self.transform
    }
}

impl Drawable for SequencerInterface {
    fn draw(&self, draw: &mut Draw) {
        for (channel_index, channel) in self.channels.iter().enumerate() {
            let inv = 1.0 / NUM_CHANNELS as f32;
            let y = inv * (NUM_CHANNELS as f32 - channel_index as f32);
            
            for clip in channel.clips.iter().filter(|clip| clip.model.enabled) {
                draw.quad(clip.quad);
            }
            for junction in channel.junctions.iter().filter(|junction| junction.model.enabled) {
                match junction.model.junction_type {
                    JunctionType::Jump {
                        destination_channel_index,
                        destination_location,
                        ..
                    } => {
                        let (
                            source_marker,
                            dest_marker
                        ) = jump_junction_to_primitives(
                            channel_index,
                            junction.model.location,
                            destination_channel_index,
                            destination_location,
                            self.channel_length
                        );
                        draw.primitive(source_marker);
                        draw.primitive(dest_marker);
                    },
                    JunctionType::Reflect => {
                        let marker = reflect_junction_to_primitive(
                            channel_index,
                            junction.model.location,
                            self.channel_length
                        );
                        draw.primitive(marker);
                    },
                    JunctionType::Stop => todo!()
                }
            }

            let playhead = self.summary.playheads[channel_index];
            match playhead.state {
                PlayheadState::Playing => {
                    draw.primitive(playhead_to_primitive(
                        channel_index,
                        self.channel_length,
                        self.summary.playheads[channel_index]
                    ));
                },
                _ => {},
            }

            draw.line(Line {
                from: (0.0, y),
                to: (1.0, y),
                color: Color::BLACK,
                depth: Depth::Mid,
            });

            let dy = inv * style::JUNCTION_LANE_PROPORTION;
            let Color { r, g, b, a } = CLEAR_COLOR;
            let s = 0.9;
            draw.quad(Quad {
                position: (0.0, y - dy),
                size: (1.0, dy),
                color: Color { r: s * r, g: s * g, b: s * b, a },
                depth: Depth::Back,
            });
        }
    }
}


fn mouse_position_to_channel_index(mouse_position: MousePosition) -> usize {
    (NUM_CHANNELS - 1 - (mouse_position.y * NUM_CHANNELS as f32).floor() as usize)
        .clamp(0, NUM_CHANNELS - 1)
}

fn mouse_position_to_channel_location(mouse_position: MousePosition, channel_length: u64) -> u64 {
    (channel_length as f32 * mouse_position.x).floor() as u64
}

fn mouse_position_is_on_junction_lane(mouse_position: MousePosition, channel_index: usize) -> bool {
    let inv = 1.0 / NUM_CHANNELS as f32;
    let y = 1.0 - inv * channel_index as f32;
    
    y - mouse_position.y < inv * style::JUNCTION_LANE_PROPORTION
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
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = playhead.location as f32 / channel_length as f32;
    let y = 1.0 - h - (channel_index as f32 / NUM_CHANNELS as f32);
    Primitive::Quad(Quad {
        position: (x, y),
        size: (style::MARKER_LINE_WIDTH, h),
        color: Color::RED,
        depth: Depth::Front
    })
}

fn reflect_junction_to_primitive(
    channel_index: usize,
    channel_location: u64,
    channel_length: u64
) -> Primitive {
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = channel_location as f32 / channel_length as f32;
    let y = h * (NUM_CHANNELS as f32 - channel_index as f32);
    Primitive::Quad(Quad {
        position: (x, y - h),
        size: (style::MARKER_LINE_WIDTH, h),
        color: Color::GREEN,
        depth: Depth::Front,
    })
}

fn jump_junction_to_primitives(
    source_channel_index: usize,
    source_channel_location: u64,
    destination_channel_index: usize,
    destination_location: u64,
    channel_length: u64
) -> (Primitive, Primitive) {
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = source_channel_location as f32 / channel_length as f32;
    let y = h * (NUM_CHANNELS as f32 - source_channel_index as f32);
    let x_dest = destination_location as f32 / channel_length as f32;
    let y_dest = h * (NUM_CHANNELS as f32 - destination_channel_index as f32);
    (
        Primitive::Quad(Quad {
            position: (x, y - h),
            size: (style::MARKER_LINE_WIDTH, h),
            color: Color::BLUE,
            depth: Depth::Front,
        }),
        Primitive::Quad(Quad {
            position: (x_dest, y_dest - h),
            size: (style::MARKER_LINE_WIDTH, h),
            color: Color::BLUE,
            depth: Depth::Front,
        })
    )
}