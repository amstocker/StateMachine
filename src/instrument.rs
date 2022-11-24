use winit::window::{Window, CursorIcon};
use winit::event::{WindowEvent, MouseButton, ElementState};

use crate::ui::render::RendererController;
use crate::ui::sequencer::{SequencerInterface};
use crate::ui::mouse::MousePosition;
use crate::ui::Application;
use crate::config::InstrumentConfig;
use crate::sequencer::{SequencerController, Sequencer, SequencerEvent, Clip};
use crate::sound::{Output, SoundBankController, Float, SoundBank};


pub struct Instrument {
    sequencer_interface: SequencerInterface,
    sound_bank_controller: SoundBankController<Float>,
    _output: Output,
    mouse_position: MousePosition,
}

impl Application for Instrument {
    type Config = InstrumentConfig;

    fn init(config: InstrumentConfig) -> Instrument {
        let (
            sound_bank_controller,
            sound_bank
        ) = SoundBank::new(config.sounds);
        let (
            sequencer_controller,
            sequencer
        ) = Sequencer::new(sound_bank);
        
        let mut output = Output::new(config.output);
        output.start(sequencer);

        let mut sequencer_interface = SequencerInterface::init(sequencer_controller);

        let source_index: usize = 1;
        let metadata = sound_bank_controller.get(source_index).unwrap();
        sequencer_interface.add_clip(1, Clip {
            enabled: true,
            source_index,
            channel_location_start: 0,
            channel_location_end: metadata.length as u64,
            source_scale: 1.0,
            source_shift: 0,
        });

        let source_index: usize = 2;
        let metadata = sound_bank_controller.get(source_index).unwrap();
        sequencer_interface.add_clip(2, Clip {
            enabled: true,
            source_index,
            channel_location_start: 10_000,
            channel_location_end: 10_000 + metadata.length as u64,
            source_scale: 1.0,
            source_shift: 0,
        });

        let source_index: usize = 0;
        let metadata = sound_bank_controller.get(source_index).unwrap();
        sequencer_interface.add_clip(3, Clip {
            enabled: true,
            source_index,
            channel_location_start: 30_000,
            channel_location_end: 30_000 + metadata.length as u64,
            source_scale: 1.0,
            source_shift: 0,
        });

        Self {
            sequencer_interface,
            sound_bank_controller,
            _output: output,
            mouse_position: MousePosition::default(),
        }
    }

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) {
        self.sequencer_interface.handle_window_event(event, window);
    }

    fn update(&mut self) {
        self.sequencer_interface.update();
    }

    fn draw(&self, renderer_controller: RendererController) {
        self.sequencer_interface.draw(renderer_controller);
    }
}
