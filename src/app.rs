use wgpu::Color;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

use crate::config::AppConfig;
use crate::sequencer::interface::SequencerInterface;
use crate::sequencer::{self, Clip, Sequencer};
use crate::sound::{Float, Output, SoundBank, SoundBankController};
use crate::ui::input::{Input, InputHandler, MousePosition};
use crate::ui::layout::{ThreePanelLayout, ThreePanelPosition};
use crate::ui::primitive::{Draw, Drawable, Line};
use crate::ui::{Application, Depth, Transform};

#[derive(Debug, Default, Clone, Copy)]
pub enum AppState {
    Sequencer(sequencer::interface::State),
    #[default]
    NotDoingAnything,
}

pub struct App {
    global_layout: ThreePanelLayout,
    sequencer_interface: SequencerInterface,
    sequencer_transform: Transform,
    sound_bank_controller: SoundBankController<Float>,
    _output: Output,
    mouse_position: MousePosition,
}

impl Application for App {
    type Config = AppConfig;
    type State = AppState;

    fn init(config: AppConfig) -> App {
        let (sound_bank_controller, sound_bank) = SoundBank::new(config.sounds)
            .expect("demo sample bank must fit the prototype capacity");
        let (sequencer_controller, sequencer) = Sequencer::new(sound_bank);

        let mut output = Output::new(config.output);
        output.start(sequencer);

        let mut sequencer_interface = SequencerInterface::init(sequencer_controller);

        for (channel_index, source_index) in [(0, 1), (1, 3), (2, 0), (3, 2)] {
            let Some(metadata) = sound_bank_controller.get(source_index) else {
                continue;
            };
            if metadata.length == 0 {
                continue;
            }
            sequencer_interface
                .add_clip(
                    channel_index,
                    Clip {
                        enabled: true,
                        source_index,
                        channel_location_start: 0,
                        channel_location_end: (metadata.length as u64)
                            .min(sequencer::DEFAULT_CHANNEL_LENGTH),
                        source_scale: 1.0,
                        source_shift: 0,
                    },
                )
                .expect("demo clips must fit their channels");
        }

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

    fn handle_resize(&mut self, _size: PhysicalSize<u32>) {
        self.sequencer_interface
            .set_transform(self.sequencer_transform);
    }

    fn update(&mut self, state: AppState) -> AppState {
        self.sequencer_interface.update();
        state
    }
}

impl InputHandler<App> for App {
    fn handle(&mut self, input: Input, state: AppState) -> AppState {
        let window = input.window;
        let event = input.event;
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = MousePosition::from_physical(position, window.inner_size());
                self.sequencer_interface.set_mouse_position(
                    self.mouse_position
                        .transform(self.sequencer_transform.inverse()),
                )
            }
            _ => {}
        }
        self.sequencer_interface.handle_window_event(event, window);
        state
    }
}

impl Drawable for App {
    fn draw(&self, draw: &mut Draw) {
        draw.line(Line {
            from: (0.0, self.global_layout.vertical.divide),
            to: (1.0, self.global_layout.vertical.divide),
            color: Color::BLACK,
            depth: Depth::Mid,
        });
        draw.line(Line {
            from: (self.global_layout.horizontal.divide, 0.0),
            to: (
                self.global_layout.horizontal.divide,
                self.global_layout.vertical.divide,
            ),
            color: Color::BLACK,
            depth: Depth::Mid,
        });
        draw.with(&self.sequencer_interface);
    }
}
