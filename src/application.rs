use std::sync::atomic::Ordering::SeqCst;

use iced::{executor, Theme, Command};
use iced::widget::{Column, Text, Row, Button, Container, Space};
use iced::{Application, Element, Length};

use crate::config::Config;
use crate::engine::Engine;
use crate::sequencer::Node;
use crate::sound::*;
use crate::fonts::*;

use crate::{
    sequencer::{Sequencer, SequencerParameters, SequencerControlMessage, GRID_SIZE_ROOT},
    sound::{SoundBankMetadata},
};


pub type Float = f32;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Sequencer(SequencerControlMessage)
}

pub struct Torsion {
    sound_bank_metadata: SoundBankMetadata<Float>,
    sequencer_params: SequencerParameters,
    engine: Engine
}

impl Torsion {
    pub fn node_view<'a>(&'a self, index: usize, node: &'a Node) -> Element<'a, Message> {
        use SequencerControlMessage::*;

        let sound_index = node.sound_index.load(SeqCst);
        let sound_metadata = self.sound_bank_metadata.get(sound_index).unwrap();
        
        let mut column = Column::new();
        let mut sound_info = Row::new();
        sound_info = sound_info.push(
            Text::new(&sound_metadata.name)
                .font(JETBRAINS_MONO_BOLD)
        );
        sound_info = sound_info.push(
            Space::with_width(Length::Fill)
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("-")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(DecrSoundIndex(index)))
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("+")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(IncrSoundIndex(index)))
        );
        sound_info = sound_info.push(
            Space::with_width(Length::Units(3))
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("Play")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(PlaySound(index)))
        );
        column = column.push(sound_info);
        column = column.push(Space::with_height(Length::Units(3)));
    
        let mut enable = Row::new();
        enable = enable.push(Space::with_width(Length::Fill));
        enable = enable.push(
            if node.enabled.load(SeqCst) {
                Button::new(
                    Text::new("Disable")
                        .font(JETBRAINS_MONO)
                ).on_press(Message::Sequencer(DisableSound(index)))
            } else {
                Button::new(
                    Text::new("Enable")
                        .font(JETBRAINS_MONO)
                ).on_press(Message::Sequencer(EnableSound(index)))
            }
        );
        column = column.push(enable);

        Container::new(column)
            .width(Length::Units(150))
            .padding(5)
            .into()
    }
}

impl Application for Torsion {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Config;

    fn new(config: Config) -> (Self, Command<Message>) {
        let (mut sound_bank_metadata, sound_bank) = SoundBank::new();
        for sound in config.init_sounds {
            sound_bank_metadata.add_sound(sound).unwrap();
        }
    
        let (sequencer_params, sequencer) = Sequencer::new(sound_bank);

        let mut engine = Engine::new(config.output);
        engine.run(sequencer);

        (
            Self { 
                sound_bank_metadata,
                sequencer_params,
                engine
            },
            Command::none()
        )

    }

    fn title(&self) -> String {
        String::from("State Machine")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Sequencer(control) => self.sequencer_params.handle_sequencer_control(control)
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut column = Column::new();
        for j in 0..GRID_SIZE_ROOT {
            let mut row = Row::new();
            for i in 0..GRID_SIZE_ROOT {
                let index = j * GRID_SIZE_ROOT + i;
                let node = self.sequencer_params.nodes.get(index).unwrap();
                row = row.push(self.node_view(index, node));
            }
            column = column.push(row);
        }
        Container::new(column).into()
    }
}