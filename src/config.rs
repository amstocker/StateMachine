use crate::{sound::{Sound, OutputConfig, Float}, ui::ApplicationConfig};


const TITLE: &str = "state_machine";

#[derive(Default)]
pub struct InstrumentConfig {
    pub output: OutputConfig,
    pub sounds: Vec<Sound<Float>>
}

impl ApplicationConfig for InstrumentConfig {
    fn window_title(&self) -> &str {
        TITLE
    }
}