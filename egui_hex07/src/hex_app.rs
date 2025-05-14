use crate::diff::{self, HexCell};
use arb_comp06::{bpe::Bpe, matcher, test_utils};
use egui::{Color32, RichText, Ui};
use egui_extras::{Column, TableBody, TableBuilder, TableRow};
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, PartialEq)]
enum WhichFile {
    File0,
    File1,
}
fn drop_select_text(selected: bool) -> &'static str {
    if selected {
        "⬇ Loading dropped files here ⬇"
    } else {
        "⬇ Load dropped files here ⬇"
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum DiffMethod {
    ByIndex,
    BpeGreedy00,
}

pub struct HexApp {
    source_name0: Option<String>,
    source_name1: Option<String>,
    pattern0: Arc<Mutex<Option<Vec<u8>>>>,
    pattern1: Arc<Mutex<Option<Vec<u8>>>>,
    diffs0: Arc<Mutex<Vec<HexCell>>>,
    diffs1: Arc<Mutex<Vec<HexCell>>>,
    file_drop_target: WhichFile,
    diff_method: DiffMethod,
    update_diffs_handle: Option<thread::JoinHandle<()>>,
}

fn random_pattern() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..2000).map(|_| rng.gen_range(0..=255)).collect()
}

impl HexApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut result = Self {
            source_name0: Some("zeroes0".to_string()),
            source_name1: Some("zeroes1".to_string()),
            pattern0: Arc::new(Mutex::new(Some(vec![0; 1000]))),
            pattern1: Arc::new(Mutex::new(Some(vec![0; 1000]))),
            diffs0: Arc::new(Mutex::new(vec![])),
            diffs1: Arc::new(Mutex::new(vec![])),
            file_drop_target: WhichFile::File0,
            diff_method: DiffMethod::ByIndex,
            update_diffs_handle: None,
        };

        result.update_diffs();
        result
    }

    fn update_diffs(&mut self) {
        if self.update_diffs_handle.is_some() {
            if let Some(handle) = self.update_diffs_handle.take_if(|x| x.is_finished()) {
                handle.join().unwrap();
                log::info!("update_diffs handle joined");
            } else {
                log::info!("update_diffs handle is not finished");
                return;
            }
        }

        let pattern0 = self.pattern0.clone();
        let pattern1 = self.pattern1.clone();

        let diffs0 = self.diffs0.clone();
        let diffs1 = self.diffs1.clone();

        let diff_method = self.diff_method;

        self.update_diffs_handle = Some(thread::spawn(move || {
            let pattern0 = pattern0.lock().unwrap();
            let pattern1 = pattern1.lock().unwrap();

            let (new_diffs0, new_diffs1) =
                if let (Some(pattern0), Some(pattern1)) = (&*pattern0, &*pattern1) {
                    let len = std::cmp::max(pattern0.len(), pattern1.len());
                    match diff_method {
                        DiffMethod::ByIndex => diff::get_diffs(pattern0, pattern1, 0..len),
                        DiffMethod::BpeGreedy00 => {
                            let bpe = Bpe::new(&[pattern0, pattern1]);

                            let pattern0 = bpe.encode(pattern0);
                            let pattern1 = bpe.encode(pattern1);

                            let matches = matcher::greedy00(&pattern0, &pattern1);
                            test_utils::matches_to_cells(&matches, |x| bpe.decode(x.clone()))
                        }
                    }
                } else {
                    (vec![], vec![])
                };
            log::info!("started updating diffs");   
            {
                let mut diffs0 = diffs0.lock().unwrap();
                *diffs0 = new_diffs0;
            }
            {
                let mut diffs1 = diffs1.lock().unwrap();
                *diffs1 = new_diffs1;
            }
            log::info!("finished updating diffs");
        }));
    }

    fn add_header_row(&mut self, mut header: TableRow<'_, '_>) {
        let no_pattern = "[none]".to_string();

        header.col(|ui| {
            ui.heading("address");
        });
        header.col(|ui| {
            ui.vertical(|ui| {
                ui.heading(self.source_name0.as_ref().unwrap_or(&no_pattern));
                ui.horizontal(|ui| {
                    let text = drop_select_text(self.file_drop_target == WhichFile::File0);
                    ui.selectable_value(&mut self.file_drop_target, WhichFile::File0, text)
                        .highlight();
                    if ui.button("randomize").clicked() {
                        {
                            let mut pattern0 = self.pattern0.lock().unwrap();
                            *pattern0 = Some(random_pattern());
                        }
                        self.source_name0 = Some("random".to_string());
                        self.update_diffs();
                    }
                });
            });
        });
        header.col(|_| {});
        header.col(|ui| {
            ui.vertical(|ui| {
                ui.heading(self.source_name1.as_ref().unwrap_or(&no_pattern));
                ui.horizontal(|ui| {
                    let text = drop_select_text(self.file_drop_target == WhichFile::File1);
                    ui.selectable_value(&mut self.file_drop_target, WhichFile::File1, text)
                        .highlight();
                    if ui.button("randomize").clicked() {
                        {
                            let mut pattern1 = self.pattern1.lock().unwrap();
                            *pattern1 = Some(random_pattern());
                        }
                        self.source_name1 = Some("random".to_string());
                        self.update_diffs();
                    }
                });
            });
        });
    }

    fn add_body_contents(&self, body: TableBody<'_>) {
        fn color(c: usize) -> Color32 {
            let hi: u8 = 255;
            let lo: u8 = 128;
            match c % 6 {
                0 => Color32::from_rgb(hi, lo, lo),
                1 => Color32::from_rgb(hi, hi, lo),
                2 => Color32::from_rgb(lo, hi, lo),
                3 => Color32::from_rgb(lo, hi, hi),
                4 => Color32::from_rgb(lo, lo, hi),
                5 => Color32::from_rgb(hi, lo, hi),
                _ => unreachable!(),
            }
        }
        fn contrast(color: Color32) -> Color32 {
            Color32::from_rgb(
                u8::wrapping_add(color.r(), 128),
                u8::wrapping_add(color.g(), 128),
                u8::wrapping_add(color.b(), 128),
            )
        }

        let diffs0 = self.diffs0.lock().unwrap();
        let diffs1 = self.diffs1.lock().unwrap();

        let hex_grid_width = 16;

        let row_height = 18.0;
        let num_rows = 1 + std::cmp::max(diffs0.len(), diffs1.len()) / hex_grid_width;

        body.rows(row_height, num_rows, |mut row| {
            let row_index = row.index();

            let add_hex_row = |ui: &mut Ui, diffs: &Vec<HexCell>| {
                (0..hex_grid_width).for_each(|i| {
                    let cell = diffs.get(i + row_index * hex_grid_width);

                    match cell {
                        Some(&HexCell::Same { value, source_id }) => ui.label(
                            RichText::new(format!("{value:02X}"))
                                .color(color(source_id))
                                .monospace(),
                        ),
                        Some(&HexCell::Diff { value, source_id }) => {
                            let color = color(source_id);
                            let contrast = contrast(color);
                            ui.label(
                                RichText::new(format!("{value:02X}"))
                                    .color(contrast)
                                    .background_color(color)
                                    .monospace(),
                            )
                        }

                        Some(&HexCell::Blank) => ui.monospace("__"),
                        None => ui.monospace("xx"),
                    };
                });
            };

            let add_ascii_row = |ui: &mut Ui, diffs: &Vec<HexCell>| {
                (0..hex_grid_width).for_each(|i| {
                    let cell = diffs.get(i + row_index * hex_grid_width);

                    match cell {
                        Some(&HexCell::Same { value, source_id }) => ui.label(
                            RichText::new(format!("{}", value as char))
                                .color(color(source_id))
                                .monospace(),
                        ),
                        Some(&HexCell::Diff { value, source_id }) => {
                            let color = color(source_id);
                            let contrast = contrast(color);

                            ui.label(
                                RichText::new(format!("{}", value as char))
                                    .color(contrast)
                                    .background_color(color)
                                    .monospace(),
                            )
                        }
                        Some(&HexCell::Blank) => ui.monospace("_"),
                        None => ui.monospace("x"),
                    };
                });
            };

            row.col(|ui| {
                ui.label(RichText::new(format!("{:08X}", row_index * hex_grid_width)).monospace());
            });
            row.col(|ui| add_hex_row(ui, &diffs0));
            row.col(|ui| add_ascii_row(ui, &diffs0));
            row.col(|ui| add_hex_row(ui, &diffs1));
            row.col(|ui| add_ascii_row(ui, &diffs1));
        });
    }
}

impl eframe::App for HexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if let Some(dropped_file) = i.raw.dropped_files.first() {
                // This should only be Some when running as a native app.
                if let Some(path) = &dropped_file.path {
                    match self.file_drop_target {
                        WhichFile::File0 => {
                            self.source_name0 = Some(path.to_string_lossy().to_string());
                            {
                                let mut pattern0 = self.pattern0.lock().unwrap();
                                *pattern0 = std::fs::read(path).ok();
                                if pattern0.is_none() {
                                    log::error!("failed to read file: {:?}", path);
                                }
                            }
                        }
                        WhichFile::File1 => {
                            self.source_name1 = Some(path.to_string_lossy().to_string());
                            {
                                let mut pattern1 = self.pattern1.lock().unwrap();
                                *pattern1 = std::fs::read(path).ok();
                                if pattern1.is_none() {
                                    log::error!("failed to read file: {:?}", path);
                                }
                            }
                        }
                    }
                }
                // This should only be Some when running as a web app.
                else if let Some(bytes) = &dropped_file.bytes {
                    match self.file_drop_target {
                        WhichFile::File0 => {
                            {
                                let mut pattern0 = self.pattern0.lock().unwrap();
                                *pattern0 = Some(bytes.to_vec());
                            }
                            self.source_name0 = Some(dropped_file.name.clone());
                        }
                        WhichFile::File1 => {
                            {
                                let mut pattern1 = self.pattern1.lock().unwrap();
                                *pattern1 = Some(bytes.to_vec());
                            }
                            self.source_name1 = Some(dropped_file.name.clone());
                        }
                    }
                }
                self.update_diffs();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("hex diff test (egui UI)");

                ui.label("diff method:");
                use DiffMethod::*;
                if ui
                    .selectable_value(&mut self.diff_method, ByIndex, "By Index")
                    .clicked()
                {
                    self.update_diffs();
                }

                if ui
                    .selectable_value(&mut self.diff_method, BpeGreedy00, "BPE Greedy 00")
                    .clicked()
                {
                    self.update_diffs();
                }
            });

            TableBuilder::new(ui)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .striped(true)
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .header(20.0, |header| self.add_header_row(header))
                .body(|body| self.add_body_contents(body));
        });
    }
}
