use winit::window::{Window, CursorIcon};
use winit::event::{WindowEvent, MouseButton};

use crate::ui::{Application, EventSender, State, MousePosition};
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

    fn init(config: InstrumentConfig, event_sender: EventSender<InstrumentEvent>) -> Instrument {
        let (sequencer_controller, sequencer) = Sequencer::new(event_sender);
        let mut output = Output::new(config.output);
        output.start(sequencer);

        Self {
            sequencer_controller,
            output,
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window, state: &mut State) {
        match event {
            WindowEvent::CursorMoved { .. } => {
                let (x, y) = state.mouse_position.get();
                state.clear_color = wgpu::Color {
                    r: x as f64,
                    g: y as f64,
                    b: 1.0,
                    a: 1.0,
                };
            },
            WindowEvent::MouseInput { button: MouseButton::Left, .. } => {
                window.set_cursor_icon(CursorIcon::Grabbing);
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

    fn draw(&self) {
    
    }
}
