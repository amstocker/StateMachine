use std::sync::Arc;

use crossbeam_channel::Sender;
use eframe::egui;

use crate::app::*;
use crate::playback::Message;


// how to properly abstract UI?

pub struct UI {
    app_state: Arc<State>
}

impl UI {
    pub fn new(app_state: Arc<State>, sender: Sender<Message>) -> Self {
        Self {
            app_state
        }
    }

    pub fn run(self) {
        let options = eframe::NativeOptions {
            // Hide the OS-specific "chrome" around the window:
            decorated: false,
            // To have rounded corners we need transparency:
            //transparent: true,
            min_window_size: Some(egui::vec2(320.0, 100.0)),
            ..Default::default()
        };
        eframe::run_native(
            "State Machine",
            options,
            Box::new(|_cc| Box::new(self)),
        );
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            use eframe::egui::*;

            //let text_color = ctx.style().visuals.text_color();
            let height = 28.0;
            let rect = ui.max_rect();

            // Interact with the title bar (drag to move window):
            let title_bar_rect = {
                let mut rect = rect;
                rect.max.y = rect.min.y + height;
                rect
            };
            let title_bar_response =
                ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            ui.heading("Samples");
            //for sound in &self.state.graph.sounds {
            //}

            ui.group(|ui| {
                if ui.button("Add Sample").clicked() {
                    if let Some(filename) = rfd::FileDialog::new().pick_file() {
                        //self.add_sound(filename.display().to_string());
                    }
                }
            });
            
        });
    }
}