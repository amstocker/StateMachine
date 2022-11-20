use winit::window::{Window, CursorIcon};
use winit::event::{WindowEvent, MouseButton, ElementState};

use crate::ui::drawer::{Drawer, quad::Quad, text::Text};
use crate::ui::{Application, EventSender, GPUState};
use crate::config::InstrumentConfig;
use crate::sequencer::{SequencerController, Sequencer};
use crate::sound::Output;


pub enum InstrumentEvent {

}

pub struct Instrument {
    sequencer_controller: SequencerController,
    output: Output,
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
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window, state: &mut GPUState) {
        match event {
            WindowEvent::MouseInput { button: MouseButton::Left, state, .. } => {
                match state {
                    ElementState::Pressed => {
                        window.set_cursor_icon(CursorIcon::Grabbing);
                    },
                    ElementState::Released => {
                        window.set_cursor_icon(CursorIcon::Default);
                    }
                }
            },
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
        drawer.draw_quad(Quad {
            position: (0.0, 0.0),
            size: (0.5, 0.5),
            color: wgpu::Color::RED,
            z: 0.0
        });
        drawer.draw_quad(Quad {
            position: (0.5, 0.5),
            size: (0.5, 0.5),
            color: wgpu::Color::GREEN,
            z: 0.0
        });
        drawer.draw_text(Text {
            text: "Hello".into(),
            position: (0.0, 1.0),
            scale: 40.0,
            color: wgpu::Color::BLACK,
        });
    }
}
