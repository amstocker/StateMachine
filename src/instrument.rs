use wgpu::Color;
use winit::dpi::PhysicalSize;
use winit::window::{Window, CursorIcon};
use winit::event::{WindowEvent, MouseButton, ElementState};

use crate::ui::layout::{ThreePanelLayout, ThreePanelLayoutTransform};
use crate::ui::primitive::{Draw, Primitive, Line};
use crate::ui::mouse::MousePosition;
use crate::ui::{Application, Transform, Depth};
use crate::config::InstrumentConfig;
use crate::sequencer::{SequencerController, Sequencer, SequencerEvent, Clip};
use crate::sequencer::interface::{SequencerInterface};
use crate::sound::{Output, SoundBankController, Float, SoundBank};


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

        let global_layout = ThreePanelLayout {
            vertical_divide: 0.3,
            horizontal_divide: 0.2
        };

        let ThreePanelLayoutTransform {
            main_panel_transform: sequencer_transform,
            ..
        } = global_layout.transform();
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

    fn handle_window_event(&mut self, event: &WindowEvent, window: &Window) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = MousePosition::from_physical(position, window.inner_size());
                self.sequencer_interface.set_mouse_position(
                    self.mouse_position.transform(self.sequencer_transform.inverse())
                )
            },
            _ => {}
        }
        self.sequencer_interface.handle_window_event(event, window);
    }

    fn handle_resize(&mut self, size: PhysicalSize<u32>) {
        self.sequencer_interface.set_transform(self.sequencer_transform);
    }

    fn update(&mut self) {
        self.sequencer_interface.update();
    }

    fn draw(&self, mut draw: Draw) {
        draw.line(Line {
            from: (0.0, self.global_layout.vertical_divide),
            to: (1.0, self.global_layout.vertical_divide),
            color: Color::BLACK,
            depth: Depth::Mid,
        });
        draw.line(Line {
            from: (self.global_layout.horizontal_divide, self.global_layout.vertical_divide),
            to: (self.global_layout.horizontal_divide, 1.0),
            color: Color::BLACK,
            depth: Depth::Mid,
        });
        draw.with(&self.sequencer_interface);
    }
}
