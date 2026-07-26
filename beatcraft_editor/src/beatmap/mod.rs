use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use crate::{DB_LOGIC, DB_MAIN};
use crate::editor::{App, RoutineAction};

pub mod event;
pub mod object;
pub mod data;
pub mod render;
#[cfg(test)]
pub mod tests;

impl App {
    pub fn draw_beatmap_editor(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame, shift: bool, ctrl: bool) {
        let gl = frame.gl().unwrap();

        egui::TopBottomPanel::top("menu_bar_beatmap_editor").show(ctx, |ui| {
            ui.add_space(2.);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open map\u{2026} \u{2502}").clicked() {
                        tracing::debug!(target: DB_LOGIC, "Spawning thread for opening beatmap project");
                        let (sx, rx) = mpsc::channel();
                        thread::spawn(move || {
                            let Some(map_folder) = rfd::FileDialog::new()
                                .set_title("Open Beatmap...")
                                .pick_folder() else {
                                    tracing::debug!(target: DB_LOGIC, "Canceled opening beatmap");
                                    return;
                                };
                            tracing::debug!(target: DB_LOGIC, ?map_folder, "Opening beatmap");
                            let _ = sx.send(map_folder);
                        });
                        self.add_routine(Box::new(move |s, gl| {
                            match rx.try_recv() {
                                Err(mpsc::TryRecvError::Empty) => RoutineAction::None,
                                Err(mpsc::TryRecvError::Disconnected) => {
                                    RoutineAction::Remove
                                }
                                Ok(folder) => {
                                    if let Err(e) = s.load_beatmap(folder) {
                                        s.set_status(None, "Failed to load beatmap", 2.);
                                        tracing::error!(target: DB_MAIN, "Failed to load beatmap: {e}");
                                    }
                                    RoutineAction::Remove
                                }
                            }
                        }));
                    }
                });
            });
        });
    }

    pub fn load_beatmap(&mut self, folder: PathBuf) -> anyhow::Result<()> {
        Ok(())
    }
}

