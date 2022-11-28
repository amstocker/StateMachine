use wgpu::Color;
use winit::dpi::PhysicalSize;
use winit::window::{Window, CursorIcon};
use winit::event::{WindowEvent, MouseButton, ElementState};

use crate::ui::layout::{ThreePanelLayout, ThreePanelPosition};
use crate::ui::primitive::{Draw, Line, Quad, Text, Drawable};
use crate::ui::input::{MousePosition, Input, InputHandler};
use crate::ui::{Application, Transform, Depth, Position, Transformable};
use crate::config::InstrumentConfig;
use crate::sequencer::{SequencerController, Sequencer, SequencerEvent, Clip, self};
use crate::sequencer::{interface::{SequencerInterface}};
use crate::sound::{Output, SoundBankController, Float, SoundBank};


#[derive(Debug, Default, Clone, Copy)]
pub enum InstrumentState {
    Sequencer(sequencer::interface::State),
    #[default] NotDoingAnything
}

pub struct Instrument {
    global_layout: ThreePanelLayout,
    sequencer_interface: SequencerInterface,
    sequencer_transform: Transform,
    sound_bank_controller: SoundBankController<Float>,
    _output: Output,
    mouse_position: MousePosition,
}

impl Application for Instrument {
    type Config = InstrumentConfig;
    type State = InstrumentState;

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

        let source_index: usize = 3;
        let metadata = sound_bank_controller.get(source_index).unwrap();
        sequencer_interface.add_clip(1, Clip {
            enabled: true,
            source_index,
            channel_location_start: 0,
            channel_location_end: metadata.length as u64,
            source_scale: 1.0,
            source_shift: 0,
        });

        let source_index: usize = 0;
        let metadata = sound_bank_controller.get(source_index).unwrap();
        sequencer_interface.add_clip(2, Clip {
            enabled: true,
            source_index,
            channel_location_start: 0,
            channel_location_end: metadata.length as u64,
            source_scale: 1.0,
            source_shift: 0,
        });

        let source_index: usize = 2;
        let metadata = sound_bank_controller.get(source_index).unwrap();
        sequencer_interface.add_clip(3, Clip {
            enabled: true,
            source_index,
            channel_location_start: 0,
            channel_location_end: metadata.length as u64,
            source_scale: 1.0,
            source_shift: 0,
        });

        let source_index: usize = 1;
        let metadata = sound_bank_controller.get(source_index).unwrap();
        sequencer_interface.add_clip(0, Clip {
            enabled: true,
            source_index,
            channel_location_start: 0,
            channel_location_end: metadata.length as u64,
            source_scale: 1.0,
            source_shift: 0,
        });

        let global_layout = ThreePanelLayout::new(0.8, 0.3);
        
        let sequencer_transform = global_layout.get(ThreePanelPosition::Main);
        sequencer_interface.set_transform(sequencer_transform);

        Self {
            global_layout,
            sequencer_interface,
            sequencer_transform,
            sound_bank_controller,
            _output: output,
            mouse_position: MousePosition::default(),
        }
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.sequencer_interface.set_transform(self.sequencer_transform);
    }

    fn update(&mut self, state: InstrumentState) -> InstrumentState {
        self.sequencer_interface.update();
        state
    }
}

impl InputHandler<Instrument> for Instrument {
    fn handle(&mut self, input: Input, state: InstrumentState) -> InstrumentState {
        let window = input.window;
        let event = input.event;
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = MousePosition::from_physical(position, window.inner_size());
                let position = Position(self.mouse_position.x, self.mouse_position.y);

                self.sequencer_interface.set_mouse_position(
                    self.mouse_position.transform(self.sequencer_transform.inverse())
                )
            },
            _ => {}
        }
        self.sequencer_interface.handle_window_event(event, window);
        state
    }
}

impl Drawable for Instrument {
    fn draw(&self, draw: &mut Draw) {
        draw.line(Line {
            from: (0.0, self.global_layout.vertical.divide),
            to: (1.0, self.global_layout.vertical.divide),
            color: Color::BLACK,
            depth: Depth::Mid,
        });
        draw.line(Line {
            from: (self.global_layout.horizontal.divide, 0.0),
            to: (self.global_layout.horizontal.divide, self.global_layout.vertical.divide),
            color: Color::BLACK,
            depth: Depth::Mid,
        });
        draw.with(&self.sequencer_interface);
    }
}
