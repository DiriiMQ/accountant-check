//! Desktop GUI: a persistent welcome window with a file picker and submit button,
//! matching the original Python tkinter app's layout and flow.
//!
//! Uses `egui`/`eframe` (a pure-Rust immediate-mode GUI) rather than a web UI, for
//! the same reason the Python version chose tkinter: the packaged app needs no
//! browser or local server, just double-click and a window opens.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui;
use rfd::{MessageButtons, MessageDialog, MessageLevel};

use invoice_validator::config::{
    CATEGORY_DU_LIEU_LOI, CATEGORY_TRUNG_LAP, CATEGORY_VUOT_NGUONG, DEFAULT_OUTPUT_FILENAME,
};
use invoice_validator::engine::{default_rules, run_rules};
use invoice_validator::loader::load_invoices;
use invoice_validator::logging_setup::{ResourceHeartbeat, configure_logging, log_file};
use invoice_validator::report::write_report;
use invoice_validator::violations::Violation;

struct Counts {
    du_lieu_loi: usize,
    trung_lap: usize,
    vuot_nguong: usize,
}

fn count(violations: &[Violation], category: &str) -> usize {
    use std::collections::HashSet;
    violations
        .iter()
        .filter(|v| v.category == category)
        .map(|v| v.excel_row)
        .collect::<HashSet<_>>()
        .len()
}

fn process(input_path: &Path) -> anyhow::Result<(PathBuf, Counts)> {
    let output = input_path.with_file_name(DEFAULT_OUTPUT_FILENAME);
    log::info!("Starting processing: input={}", input_path.display());

    let _heartbeat = ResourceHeartbeat::start(Duration::from_secs(5));
    let t0 = Instant::now();
    let rows = load_invoices(input_path)?;
    log::info!(
        "Loaded {} data rows ({:.2}s)",
        rows.len(),
        t0.elapsed().as_secs_f64()
    );

    let t0 = Instant::now();
    let violations = run_rules(&rows, &default_rules());
    write_report(&output, rows.len(), &rows, &violations, None)?;
    let counts = Counts {
        du_lieu_loi: count(&violations, CATEGORY_DU_LIEU_LOI),
        trung_lap: count(&violations, CATEGORY_TRUNG_LAP),
        vuot_nguong: count(&violations, CATEGORY_VUOT_NGUONG),
    };
    log::info!(
        "Rules + report finished ({:.2}s): Du_lieu_loi={} Trung_lap={} Vuot_nguong={} -> {}",
        t0.elapsed().as_secs_f64(),
        counts.du_lieu_loi,
        counts.trung_lap,
        counts.vuot_nguong,
        output.display()
    );
    Ok((output, counts))
}

fn open_file(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", "/B"])
            .arg(path)
            .status()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).status()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(path).status()?;
    }
    Ok(())
}

/// Outcome of a background `process()` run, sent back to the UI thread.
enum ProcessOutcome {
    Success { output: PathBuf, counts: Counts },
    Failure(String),
}

#[derive(Default)]
struct App {
    selected_path: Option<PathBuf>,
    output_path: Option<PathBuf>,
    status: String,
    processing: bool,
    result_rx: Option<mpsc::Receiver<ProcessOutcome>>,
}

impl App {
    fn choose_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Chọn file danh sách hóa đơn (Excel)")
            .add_filter("Excel files", &["xlsx"])
            .pick_file()
        {
            self.selected_path = Some(path);
            self.output_path = None;
            self.status.clear();
        }
    }

    fn submit(&mut self) {
        let Some(path) = self.selected_path.clone() else {
            return;
        };
        self.processing = true;
        self.status = "Đang xử lý...".to_string();
        let (tx, rx) = mpsc::channel();
        self.result_rx = Some(rx);
        std::thread::spawn(move || {
            let outcome = match process(&path) {
                Ok((output, counts)) => ProcessOutcome::Success { output, counts },
                Err(err) => {
                    log::error!("Processing failed: {err:?}");
                    ProcessOutcome::Failure(err.to_string())
                }
            };
            let _ = tx.send(outcome);
        });
    }

    fn open_output(&self) {
        if let Some(output) = &self.output_path
            && let Err(err) = open_file(output)
        {
            log::error!("Failed to open result file: {err}");
            MessageDialog::new()
                .set_title("Lỗi")
                .set_description(format!("Không thể mở file:\n{err}"))
                .set_level(MessageLevel::Error)
                .set_buttons(MessageButtons::Ok)
                .show();
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.result_rx {
            if let Ok(outcome) = rx.try_recv() {
                self.processing = false;
                self.result_rx = None;
                match outcome {
                    ProcessOutcome::Success { output, counts } => {
                        self.output_path = Some(output.clone());
                        self.status = format!(
                            "Dữ liệu lỗi: {}\nTrùng lặp: {}\nVượt ngưỡng: {}\n\nĐã lưu kết quả: {}",
                            counts.du_lieu_loi,
                            counts.trung_lap,
                            counts.vuot_nguong,
                            output.display()
                        );
                    }
                    ProcessOutcome::Failure(message) => {
                        self.status.clear();
                        MessageDialog::new()
                            .set_title("Lỗi")
                            .set_description(format!(
                                "Không thể xử lý file:\n{message}\n\nChi tiết đã được lưu vào file log, vui lòng gửi file này để được hỗ trợ:\n{}",
                                log_file().display()
                            ))
                            .set_level(MessageLevel::Error)
                            .set_buttons(MessageButtons::Ok)
                            .show();
                    }
                }
            } else {
                // Still running -- keep polling for the background thread's result.
                ui.ctx().request_repaint_after(Duration::from_millis(100));
            }
        }

        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.heading("Kiểm tra danh sách hóa đơn đầu vào");
            ui.add_space(10.0);

            let file_label = self
                .selected_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Chưa chọn file".to_string());
            ui.label(egui::RichText::new(file_label).color(egui::Color32::GRAY));
            ui.add_space(14.0);

            if ui
                .add_sized([220.0, 28.0], egui::Button::new("Chọn file..."))
                .clicked()
                && !self.processing
            {
                self.choose_file();
            }
            ui.add_space(4.0);

            let submit_enabled = self.selected_path.is_some() && !self.processing;
            if ui
                .add_enabled(
                    submit_enabled,
                    egui::Button::new("Xử lý").min_size([220.0, 28.0].into()),
                )
                .clicked()
            {
                self.submit();
            }
            ui.add_space(4.0);

            let open_enabled = self.output_path.is_some();
            if ui
                .add_enabled(
                    open_enabled,
                    egui::Button::new("Mở file kết quả").min_size([220.0, 28.0].into()),
                )
                .clicked()
            {
                self.open_output();
            }

            ui.add_space(16.0);
            ui.label(&self.status);
            ui.add_space(20.0);
        });
    }
}

fn main() -> anyhow::Result<()> {
    configure_logging()?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([360.0, 340.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Accountant Check",
        native_options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
    .map_err(|err| anyhow::anyhow!("eframe error: {err}"))
}
