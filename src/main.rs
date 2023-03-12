use std::{fs, thread};
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use eframe::Frame;
use egui::{Context, FontData, FontDefinitions, FontFamily, ProgressBar, ScrollArea};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::file_cache::FilesCache;

mod paragraph;
mod file_cache;
mod polly;
mod audio;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Note Reader",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )?;

    // Close all audio as soon as the window closed
    exit(0);
}

#[derive(Deserialize, Serialize)]
struct App {
    // only serde path, so the files_cache can be updated
    path: Option<PathBuf>,
    #[serde(skip)]
    files_cache: Option<FilesCache>,
    #[serde(skip)]
    play_progress: Arc<AtomicU8>,
    #[serde(skip)]
    para: String,
    #[serde(skip)]
    chosen_file: Option<PathBuf>,

    #[serde(skip)]
    sound: &'static str,
}

impl Default for App {
    fn default() -> Self {
        Self {
            path: None,
            files_cache: None,
            /// 0 to 100
            play_progress: Arc::new(AtomicU8::new(0)),
            para: String::new(),
            chosen_file: None,
            sound: "none",
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        let mut fonts = FontDefinitions::default();
        // TODO: is this font legal?
        fonts.font_data.insert("pingfang".to_owned(), FontData::from_static(include_bytes!("/System/Library/Fonts/PingFang.ttc")));
        fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(0, "pingfang".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn display_path(&self) -> String {
        match &self.path {
            None => { "Not Chosen".to_string() }
            Some(p) => { p.as_path().display().to_string() }
        }
    }

    fn display_paragraph(&self) -> String {
        self.para.clone()
    }

    fn display_chosen_file(&self) -> String {
        match &self.chosen_file {
            None => "Press 'play' to random choose a file to play".to_string(),
            Some(f) => f.strip_prefix(self.path.clone().unwrap().as_path()).unwrap().display().to_string(),
        }
    }


    async fn play(path: PathBuf, progress: Arc<AtomicU8>) {
        println!("{:?}", path);
        let paragraphs = paragraph::divide_note_content(&path);
        let para_len = paragraphs.len();
        // use channel to pre-process the content, but not eagerly processing all
        let (tx, mut rx) = mpsc::channel(2);
        tokio::spawn(async move {
            for p in paragraphs {
                let processed = polly::process(p).await;
                if (tx.send(processed).await).is_err() {
                    println!("receiver dropped");
                    return;
                }
            }
        });

        // update progress
        let mut played_count = 0u8;
        while let Some(p) = rx.recv().await {
            audio::play(p);
            played_count += 1;
            progress.store((played_count as usize * 100 / para_len) as u8, Ordering::Release);
            println!("set progress to {}", (played_count as usize * 100 / para_len));
            // sleep some seconds between paragraphs
            thread::sleep(Duration::from_secs(1));
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("load path").clicked() {
                        self.path = FileDialog::new().pick_folder();
                        if let Some(files) = self.path.clone() {
                            self.files_cache = Some(FilesCache::new(&files));
                        }
                    }
                    ui.label(self.display_path());
                });

                egui::ComboBox::from_id_source("combo")
                    .selected_text(self.sound)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.sound, "a", "1");
                        ui.selectable_value(&mut self.sound, "b", "2");
                        ui.selectable_value(&mut self.sound, "c", "3");
                    })
                ;


                ui.horizontal(|ui| {
                    if ui.button("play").clicked() {
                        // reset progress to 0
                        self.play_progress.store(0, Ordering::Release);
                        // random choose path and play
                        if self.files_cache.is_none() {
                            match &self.path {
                                None => self.para = "please first load a path".to_string(),
                                Some(p) => {
                                    println!("reload file cache");
                                    self.files_cache = Some(FilesCache::new(p));
                                }
                            }
                        }

                        if self.files_cache.is_some() {
                            let path = self.files_cache.as_ref().unwrap().get_random().unwrap().clone();
                            self.chosen_file = Some(path.clone());
                            self.para = fs::read_to_string(path.clone()).unwrap();
                            let progress = self.play_progress.clone();
                            tokio::spawn(async {
                                Self::play(path, progress).await;
                            });
                        }
                    };

                    ui.label(self.display_chosen_file());
                });


                ui.add(ProgressBar::new(self.play_progress.load(Ordering::Acquire) as f32 / 100.0));

                // show paragraph in scroll area
                ui.add_space(10.0);
                ui.label("Content:");
                ui.separator();

                ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.label(self.display_paragraph())
                });
            })
        });

        // refresh UI so the progress bar won't stall when cursor not focus on the app
        ctx.request_repaint_after(Duration::from_secs(1));
    }


    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
