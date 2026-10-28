use wgpu::Color;

use crate::{
    app::{App, AppState},
    sound::{Float, OutputConfig, Sound},
    ui::{ApplicationConfig, Style},
};

pub const TITLE: &str = "state_machine";

pub const CLEAR_COLOR: Color = Color {
    r: 1.0,
    g: 250.0 / 255.0,
    b: 235.0 / 255.0,
    a: 1.0,
};

#[derive(Default)]
pub struct AppConfig {
    pub output: OutputConfig,
    pub sounds: Vec<Sound<Float>>,
}

impl ApplicationConfig<App> for AppConfig {
    fn window_title(&self) -> &str {
        TITLE
    }

    fn init_state(&self) -> Option<AppState> {
        None
    }

    fn style(&self) -> Style {
        Style {
            clear_color: CLEAR_COLOR,
        }
    }
}
