mod application;
mod fonts;
mod utils;
mod sound;
mod interpolator;
mod sequencer;
mod output;
mod config;
mod engine;

use iced::{Application, Settings};

use crate::application::Torsion;


fn main() -> iced::Result {
    Torsion::run(Settings {
        flags: Default::default(),
        ..Settings::default()
    })
}


