use winit::window::{Window, CursorIcon};
use winit::event::{WindowEvent, MouseButton, ElementState};

use crate::ui::drawer::{Drawer, quad::Quad, text::Text};
use crate::ui::mouse::MousePosition;
use crate::ui::{Application, EventSender, GPUState};
use crate::config::InstrumentConfig;
use crate::sequencer::{SequencerController, Sequencer};
use crate::sound::Output;


pub enum InstrumentEvent {

}

pub struct Instrument {
    sequencer_controller: SequencerController,
    output: Output,
    mouse_position: MousePosition,
    test_quad: Quad,
    grabbing: bool,
    relative: (f32, f32)
}

impl Application for Instrument {
    type Event = InstrumentEvent;
    type Config = InstrumentConfig;

    fn init(
        config: InstrumentConfig,
        event_sender: EventSender<InstrumentEvent>
    ) -> Instrument {
        let (
            sequencer_controller,
            sequencer
        ) = Sequencer::new(event_sender);
        
        let mut output = Output::new(config.output);
        output.start(sequencer);

        Self {
            sequencer_controller,
            output,
            mouse_position: MousePosition::default(),
            test_quad: Quad {
                position: (0.5, 0.5),
                size: (0.5, 0.5),
                color: wgpu::Color::GREEN,
                z: 0.0
            },
            grabbing: false,
            relative: (0.0, 0.0)
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window, state: &mut GPUState) {
        match event {
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => {
                match state {
                    ElementState::Pressed => {
                        window.set_cursor_icon(CursorIcon::Grabbing);
                        if self.test_quad.contains(self.mouse_position) { 
                            self.grabbing = true;
                            self.relative = (
                                self.mouse_position.x - self.test_quad.position.0,
                                self.mouse_position.y - self.test_quad.position.1
                            );
                        }
                    },
                    ElementState::Released => {
                        window.set_cursor_icon(CursorIcon::Default);
                        self.grabbing = false;
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = MousePosition::from_physical(position, state.size);
                if !self.grabbing {
                    if self.test_quad.contains(self.mouse_position) {
                        window.set_cursor_icon(CursorIcon::Grab);
                    } else {
                        window.set_cursor_icon(CursorIcon::Default);
                    }
                }
                if self.grabbing {
                    self.test_quad.position = (
                        self.mouse_position.x - self.relative.0,
                        self.mouse_position.y - self.relative.1
                    );
                }
            }
            _ => {}
        };
    }

    fn handle_application_event(&mut self, event: InstrumentEvent) {
        match event {

        }
    }

    fn update(&mut self) {
    }

    fn draw(&self, drawer: &mut Drawer) {
        drawer.draw_quad(&self.test_quad);
        drawer.draw_text(Text {
            text: "Hello".into(),
            position: (0.0, 0.1),
            scale: 40.0,
            color: wgpu::Color::BLACK,
        });
    }
}
