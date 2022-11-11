mod ui;
mod fonts;
mod utils;
mod sound;
mod interpolator;
mod sequencer;
mod output;

use assert_no_alloc::*;

use iced::{Sandbox, Settings};
use ui::Application;

// assert_no_alloc
#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


fn main() -> iced::Result {
    Application::<f32>::run(Settings::default())
}


