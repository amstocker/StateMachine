use std::sync::atomic::Ordering::SeqCst;

use iced::{executor, Theme, Command};
use iced::widget::{Column, Text, Row, Button, Container, Space};
use iced::{Application, Element, Length};

use crate::{
    config::Config,
    output::Output,
    sequencer::{Node, Sequencer, SequencerParameters, SequencerControlMessage, MAX_NODES},
    sound::*,
    fonts::*
};


pub type Float = f32;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Sequencer(SequencerControlMessage)
}

pub struct Instrument {
    sound_bank_metadata: SoundBankMetadata<Float>,
    sequencer_parameters: SequencerParameters,
    output: Output
}

impl Instrument {
    pub fn node_view<'a>(&'a self, node_index: usize, node: &'a Node) -> Element<'a, Message> {
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
            ).on_press(Message::Sequencer(DecrSoundIndex(node_index)))
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("+")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(IncrSoundIndex(node_index)))
        );
        sound_info = sound_info.push(
            Space::with_width(Length::Units(3))
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("Play")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(PlaySoundOnce(node_index)))
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
                ).on_press(Message::Sequencer(DisableSound(node_index)))
            } else {
                Button::new(
                    Text::new("Enable")
                        .font(JETBRAINS_MONO)
                ).on_press(Message::Sequencer(EnableSound(node_index)))
            }
        );
        column = column.push(enable);

        Container::new(column)
            .width(Length::Units(150))
            .padding(5)
            .into()
    }
}

impl Application for Instrument {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Config;

    fn title(&self) -> String {
        String::from("Instrument")
    }

    fn new(config: Config) -> (Self, Command<Message>) {
        let (
            sound_bank_metadata,
            sound_bank
        ) = SoundBank::new(config.sounds);
        let (
            sequencer_parameters,
            sequencer
        ) = Sequencer::new(sound_bank);

        let mut output = Output::new(config.output);
        output.start(sequencer);
        (
            Self { 
                sound_bank_metadata,
                sequencer_parameters,
                output
            },
            Command::none()
        )
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Sequencer(control) => self.sequencer_parameters.handle_message(control)
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut row = Row::new();
        for index in 0..MAX_NODES {
            let node = self.sequencer_parameters.nodes.get(index);
            row = row.push(self.node_view(index, node));
        }
        Container::new(row).into()
    }
}