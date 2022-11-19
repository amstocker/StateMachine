
use crate::sequencer::{SequencerController, Sequencer};
use crate::sound::Output;
use crate::ui::EventSender;

use crate::ui::Application;
use crate::config::InstrumentConfig;


pub enum InstrumentEvent {

}

pub struct Instrument {
    sequencer_controller: SequencerController,
    output: Output
}

impl Application for Instrument {
    type Event = InstrumentEvent;
    type Config = InstrumentConfig;

    fn init(config: InstrumentConfig, event_sender: EventSender<Self::Event>) -> Self {
        let (sequencer_controller, sequencer) = Sequencer::new(event_sender);
        let mut output = Output::new(config.output);
        output.start(sequencer);

        Self {
            sequencer_controller,
            output
        }
    }

    fn update(&mut self) {

    }

    fn draw(&self) {
    
    }

    fn handle(&mut self, event: Self::Event) {
    
    }
}
