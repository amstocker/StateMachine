use std::sync::atomic::Ordering::SeqCst;

use eframe::egui;

use crate::sequencer::{SequencerController, GRID_SIZE};


pub struct UI {
    controller: SequencerController
}

impl UI {
    pub fn new(controller: SequencerController) -> Self {
        Self {
            controller
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
                ui.interact(title_bar_rect, Id::new("state_machine"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            ui.heading("Grid");
            for i in 0..GRID_SIZE {
                let node = self.controller.nodes.get(i).unwrap();
                ui.group(|ui| {
                    ui.label(format!("Sound: {}", node.sound_index.load(SeqCst)));
                    ui.label(format!("Frame index: {}", node.current_frame_index.load(SeqCst)));
                    ui.horizontal(|ui| {
                        if node.enabled.load(SeqCst) {
                            if ui.button("Disable").clicked() {
                                node.enabled.store(false, SeqCst);
                            }
                        } else {
                            if ui.button("Enable").clicked() {
                                node.enabled.store(true, SeqCst);
                            }
                        }
                        if ui.button("Play").clicked() {
                            node.current_frame_index.store(0, SeqCst);
                            node.is_playing.store(true, SeqCst);
                        }
                    });
                });
                
            }

            // for (id, sound) in self.app_state.sounds.read().iter() {
            //     ui.group(|ui| {
            //         ui.horizontal(|ui| {
            //             ui.label(format!("({})", id));
            //             ui.monospace(&sound.name);
            //         });
            //         ui.separator();
            //         ui.horizontal(|ui| {
            //             if ui.button("Trigger").clicked() {
            //             }
            //             if ui.button("Play").clicked() {
            //             }
            //             if ui.button("Pause").clicked() {
            //             }
            //         });
            //     });
            // }

            // ui.group(|ui| {
            //     if ui.button("Add Sample").clicked() {
            //         if let Some(filename) = rfd::FileDialog::new().pick_file() {
            //             App::add_sound_to_state(self.app_state.clone(), filename.display().to_string());
            //         }
            //     }
            // });
            
        });

        // necessary because egui cannot tell if atomic variables have been updated
        // TODO: better way?
        ctx.request_repaint();
    }
}