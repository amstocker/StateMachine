use winit::window::{Window, CursorIcon};
use winit::event::{WindowEvent, MouseButton, ElementState};

use crate::ui::quad::{Quad, QuadDrawer};
use crate::ui::text::{Text, TextDrawer};
use crate::ui::mouse::MousePosition;
use crate::ui::Application;
use crate::config::InstrumentConfig;
use crate::sequencer::{SequencerController, Sequencer, SequencerEvent};
use crate::sound::Output;


pub struct Instrument {
    sequencer_controller: SequencerController,
    _output: Output,
    mouse_position: MousePosition,
    test_quad: Quad,
    grabbing: bool,
    relative: (f32, f32),
    total_frames_processed: u64
}

impl Application for Instrument {
    type Config = InstrumentConfig;

    fn init(config: InstrumentConfig) -> Instrument {
        let (
            sequencer_controller,
            sequencer
        ) = Sequencer::new();
        
        let mut output = Output::new(config.output);
        output.start(sequencer);

        Self {
            sequencer_controller,
            _output: output,
            mouse_position: MousePosition::default(),
            test_quad: Quad {
                position: (0.5, 0.5),
                size: (0.5, 0.5),
                color: wgpu::Color::GREEN,
                z: 0.0
            },
            grabbing: false,
            relative: (0.0, 0.0),
            total_frames_processed: 0
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) {
        match event {
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => {
                match state {
                    ElementState::Pressed => {
                        if self.test_quad.contains(self.mouse_position) { 
                            self.grabbing = true;
                            self.relative = (
                                self.mouse_position.x - self.test_quad.position.0,
                                self.mouse_position.y - self.test_quad.position.1
                            );
                        }
                    },
                    ElementState::Released => {
                        self.grabbing = false;
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = MousePosition::from_physical(position, window.inner_size());
                if !self.grabbing {
                    if self.test_quad.contains(self.mouse_position) {
                        window.set_cursor_icon(CursorIcon::Grab);
                    } else {
                        window.set_cursor_icon(CursorIcon::Default);
                    }
                } else {
                    window.set_cursor_icon(CursorIcon::Grabbing);
                    self.test_quad.position = (
                        self.mouse_position.x - self.relative.0,
                        self.mouse_position.y - self.relative.1
                    );
                }
            }
            _ => {}
        };
    }

    fn update(&mut self) {
        while let Ok(event) = self.sequencer_controller.event_receiver.pop() {
            match event {
                SequencerEvent::Tick(state) => {
                    self.total_frames_processed = state.total_frames_processed;
                }
            }
        }
    }

    fn draw(&self, quad_drawer: &mut QuadDrawer, text_drawer: &mut TextDrawer) {
        quad_drawer.draw(&self.test_quad);
        text_drawer.draw(Text {
            text: &format!("Frames Processed: {}", self.total_frames_processed),
            position: (0.0, 1.0),
            scale: 30.0,
            color: wgpu::Color::BLACK,
        });
    }
}
