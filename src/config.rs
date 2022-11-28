use wgpu::Color;

use crate::{sound::{Sound, OutputConfig, Float}, ui::{ApplicationConfig, Style}, instrument::{InstrumentState, Instrument}};


pub const TITLE: &str = "state_machine";

pub const CLEAR_COLOR: Color = Color {
    r: 255.0 / 255.0,
    g: 250.0 / 255.0,
    b: 235.0 / 255.0,
    a: 1.0
};


#[derive(Default)]
pub struct InstrumentConfig {
    pub output: OutputConfig,
    pub sounds: Vec<Sound<Float>>
}

impl ApplicationConfig<Instrument> for InstrumentConfig where {
    fn window_title(&self) -> &str {
        TITLE
    }

    fn init_state(&self) -> Option<InstrumentState> {
        None
    }

    fn style(&self) -> Style {
        Style {
            clear_color: CLEAR_COLOR
        }
    }
}