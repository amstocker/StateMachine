mod state;
mod style;

#[cfg(test)]
mod tests;

use wgpu::Color;
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    window::Window,
};

use crate::config::CLEAR_COLOR;
use crate::sequencer::{compatibility::PrototypeSession, *};
use crate::ui::input::MousePosition;
use crate::ui::primitive::Drawable;
use crate::ui::primitive::{Draw, Line, Primitive, Quad, Text};
use crate::ui::Depth;
use crate::ui::{Transform, Transformable};
pub use state::*;

pub struct SequencerInterface {
    session: PrototypeSession,
    summary: SequencerSummary,
    channel_length: u64,
    mouse_position: MousePosition,
    state: State,
    last_error: Option<EditError>,
    transform: Transform,
}

impl SequencerInterface {
    pub fn init(controller: SequencerController) -> Self {
        Self {
            session: PrototypeSession::new(controller),
            summary: Default::default(),
            channel_length: DEFAULT_CHANNEL_LENGTH,
            mouse_position: MousePosition::default(),
            state: State::default(),
            last_error: None,
            transform: Transform::identity(),
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    fn get_potential_action(&self) -> Action {
        let channel_index = mouse_position_to_channel_index(self.mouse_position);
        let channel_location =
            mouse_position_to_channel_location(self.mouse_position, self.channel_length);
        if mouse_position_is_on_junction_lane(self.mouse_position, channel_index) {
            return Action::Channel {
                channel_action: ChannelAction::CreateJunction,
                channel_index,
                channel_location,
            };
        }
        let channel = &self.session.document().tracks()[channel_index];
        for (clip_index, &clip) in channel.clips().iter().enumerate() {
            if clip.enabled
                && clip_to_quad(channel_index, self.channel_length, clip)
                    .contains(self.mouse_position)
            {
                return Action::Channel {
                    channel_action: ChannelAction::GrabClip { clip_index },
                    channel_index,
                    channel_location,
                };
            }
        }
        Action::Channel {
            channel_action: ChannelAction::SetPlayhead,
            channel_index,
            channel_location,
        }
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) {
        self.state = match event {
            WindowEvent::MouseInput {
                button,
                state: element_state,
                ..
            } => self.handle_mouse_input(button, element_state),
            WindowEvent::CursorMoved { .. } => self.handle_cursor_move(),
            _ => self.state,
        };
        window.set_cursor_icon(self.state.cursor_icon());
    }

    pub fn set_mouse_position(&mut self, mouse_position: MousePosition) {
        self.mouse_position = mouse_position;
    }

    fn handle_mouse_input(&mut self, button: &MouseButton, element_state: &ElementState) -> State {
        match self.state {
            State::GrabbingClip { .. } => match (button, element_state) {
                (MouseButton::Left, ElementState::Released) => State::default(),
                _ => self.state,
            },
            State::Hovering { potential_action } => {
                self.handle_action(potential_action, button, element_state)
            }
            State::CreatingJunction {
                source_channel_index,
                source_channel_location,
            } => match (button, element_state) {
                (MouseButton::Left, ElementState::Released) => {
                    let index = mouse_position_to_channel_index(self.mouse_position);
                    let location = mouse_position_to_channel_location(
                        self.mouse_position,
                        self.channel_length,
                    );
                    let result = self.handle_create_junction(
                        source_channel_index,
                        if index == source_channel_index {
                            Junction {
                                enabled: true,
                                location,
                                junction_type: JunctionType::Reflect,
                            }
                        } else {
                            Junction {
                                enabled: true,
                                location: source_channel_location,
                                junction_type: JunctionType::Jump {
                                    destination_channel_index: index,
                                    destination_location: location,
                                    split: true,
                                },
                            }
                        },
                    );
                    self.last_error = result.err();
                    State::default()
                }
                _ => self.state,
            },
        }
    }

    fn handle_cursor_move(&mut self) -> State {
        match self.state {
            State::GrabbingClip {
                channel_index,
                clip_index,
                relative_location,
            } => {
                let result = self.handle_clip_move(channel_index, clip_index, relative_location);
                self.last_error = result.err();
                self.state
            }
            State::Hovering { .. } => State::Hovering {
                potential_action: self.get_potential_action(),
            },
            State::CreatingJunction { .. } => self.state,
        }
    }

    fn handle_action(
        &mut self,
        action: Action,
        button: &MouseButton,
        element_state: &ElementState,
    ) -> State {
        match action {
            Action::Channel {
                channel_action,
                channel_index,
                channel_location,
            } => match channel_action {
                ChannelAction::GrabClip { clip_index } => match (button, element_state) {
                    (MouseButton::Left, ElementState::Pressed) => {
                        let clip =
                            &self.session.document().tracks()[channel_index].clips()[clip_index];
                        State::GrabbingClip {
                            relative_location: channel_location
                                .saturating_sub(clip.channel_location_start),
                            channel_index,
                            clip_index,
                        }
                    }
                    _ => self.state,
                },
                ChannelAction::CreateJunction => match (button, element_state) {
                    (MouseButton::Left, ElementState::Pressed) => State::CreatingJunction {
                        source_channel_index: channel_index,
                        source_channel_location: channel_location,
                    },
                    _ => self.state,
                },
                ChannelAction::ModifyJunction => todo!(),
                ChannelAction::SetPlayhead => {
                    match element_state {
                        ElementState::Pressed => {
                            let result = self.handle_set_playhead(
                                channel_index,
                                Playhead {
                                    state: PlayheadState::Playing,
                                    location: channel_location,
                                    direction: match button {
                                        MouseButton::Left => PlayheadDirection::Right,
                                        _ => PlayheadDirection::Left,
                                    },
                                },
                            );
                            self.last_error = result.err();
                        }
                        _ => {}
                    }
                    self.state
                }
            },
            _ => self.state,
        }
    }

    pub fn handle_clip_move(
        &mut self,
        channel_index: usize,
        clip_index: usize,
        relative_location: u64,
    ) -> Result<(), EditError> {
        let start = mouse_position_to_channel_location(self.mouse_position, self.channel_length)
            .saturating_sub(relative_location);
        self.session.move_clip(channel_index, clip_index, start)
    }

    pub fn handle_create_junction(
        &mut self,
        channel_index: usize,
        model: Junction,
    ) -> Result<(), EditError> {
        self.session.add_junction(channel_index, model)
    }

    pub fn add_clip(&mut self, channel_index: usize, model: Clip) -> Result<(), EditError> {
        self.session.add_clip(channel_index, model)
    }

    pub fn handle_set_playhead(
        &mut self,
        channel_index: usize,
        playhead: Playhead,
    ) -> Result<(), EditError> {
        self.session.set_playhead(channel_index, playhead)
    }

    pub fn update(&mut self) {
        if let Err(error) = self.session.flush_pending() {
            self.last_error = Some(error);
        }
        if let Some(summary) = self.session.latest_summary() {
            self.summary = summary;
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
        for (channel_index, channel) in self.session.document().tracks().iter().enumerate() {
            for &clip in channel.clips().iter().filter(|clip| clip.enabled) {
                draw.quad(clip_to_quad(channel_index, self.channel_length, clip));
            }
            for junction in channel
                .junctions()
                .iter()
                .filter(|junction| junction.enabled)
            {
                match junction.junction_type {
                    JunctionType::Jump {
                        destination_channel_index,
                        destination_location,
                        ..
                    } => {
                        let (source_marker, dest_marker) = jump_junction_to_primitives(
                            channel_index,
                            junction.location,
                            destination_channel_index,
                            destination_location,
                            self.channel_length,
                        );
                        draw.primitive(source_marker);
                        draw.primitive(dest_marker);
                    }
                    JunctionType::Reflect => {
                        let marker = reflect_junction_to_primitive(
                            channel_index,
                            junction.location,
                            self.channel_length,
                        );
                        draw.primitive(marker);
                    }
                    JunctionType::Stop => todo!(),
                }
            }

            let playhead = self.summary.playheads[channel_index];
            match playhead.state {
                PlayheadState::Playing => {
                    draw.primitive(playhead_to_primitive(
                        channel_index,
                        self.channel_length,
                        self.summary.playheads[channel_index],
                    ));
                }
                _ => {}
            }

            let inv = 1.0 / NUM_CHANNELS as f32;
            let y = inv * channel_index as f32;
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
                position: (0.0, y),
                size: (1.0, dy),
                color: Color {
                    r: s * r,
                    g: s * g,
                    b: s * b,
                    a,
                },
                depth: Depth::Back,
            });
        }

        draw.primitive_absolute(Primitive::Text(Text {
            label: match self.last_error {
                Some(error) => format!("Edit rejected: {error}"),
                None => format!("{} frames processed", self.summary.total_frames_processed),
            },
            position: (0.0, 0.0),
            scale: 30.0,
            color: Color::BLACK,
            depth: Depth::Top,
        }));
    }
}

fn mouse_position_to_channel_index(mouse_position: MousePosition) -> usize {
    ((mouse_position.y * NUM_CHANNELS as f32).floor() as usize).clamp(0, NUM_CHANNELS - 1)
}

fn mouse_position_to_channel_location(mouse_position: MousePosition, channel_length: u64) -> u64 {
    (channel_length as f32 * mouse_position.x).floor() as u64
}

fn mouse_position_is_on_junction_lane(mouse_position: MousePosition, channel_index: usize) -> bool {
    let inv = 1.0 / NUM_CHANNELS as f32;
    let y = inv * channel_index as f32;

    mouse_position.y - y < inv * style::JUNCTION_LANE_PROPORTION
}

fn clip_to_quad(channel_index: usize, channel_length: u64, clip: Clip) -> Quad {
    let w = (clip.channel_location_end as f32 - clip.channel_location_start as f32)
        / channel_length as f32;
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = clip.channel_location_start as f32 / channel_length as f32;
    let y = channel_index as f32 / NUM_CHANNELS as f32;
    Quad {
        position: (x, y),
        size: (w, h),
        color: style::CLIP_COLOR,
        depth: Depth::Mid,
    }
}

fn playhead_to_primitive(
    channel_index: usize,
    channel_length: u64,
    playhead: Playhead,
) -> Primitive {
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = playhead.location as f32 / channel_length as f32;
    let y = channel_index as f32 / NUM_CHANNELS as f32;
    Primitive::Quad(Quad {
        position: (x, y),
        size: (style::MARKER_LINE_WIDTH, h),
        color: Color::RED,
        depth: Depth::Front,
    })
}

fn reflect_junction_to_primitive(
    channel_index: usize,
    channel_location: u64,
    channel_length: u64,
) -> Primitive {
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = channel_location as f32 / channel_length as f32;
    let y = h * channel_index as f32;
    Primitive::Quad(Quad {
        position: (x, y),
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
    channel_length: u64,
) -> (Primitive, Primitive) {
    let h = 1.0 / NUM_CHANNELS as f32;
    let x = source_channel_location as f32 / channel_length as f32;
    let y = h * source_channel_index as f32;
    let x_dest = destination_location as f32 / channel_length as f32;
    let y_dest = h * destination_channel_index as f32;
    (
        Primitive::Quad(Quad {
            position: (x, y),
            size: (style::MARKER_LINE_WIDTH, h),
            color: Color::BLUE,
            depth: Depth::Front,
        }),
        Primitive::Quad(Quad {
            position: (x_dest, y_dest),
            size: (style::MARKER_LINE_WIDTH, h),
            color: Color::BLUE,
            depth: Depth::Front,
        }),
    )
}
