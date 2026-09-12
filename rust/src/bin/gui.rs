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
    /// eframe's default clear color is a near-black `rgba(12,12,12,180)`
    /// regardless of the egui style's visuals -- override it explicitly, or the
    /// window background stays dark even after `setup_style()` sets a light theme.
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

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

        let accent = egui::Color32::from_rgb(37, 99, 235);
        let subtle = egui::Color32::from_rgb(107, 114, 128);
        let card_bg = egui::Color32::from_rgb(243, 244, 246);

        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.heading(
                egui::RichText::new("Kiểm tra danh sách hóa đơn đầu vào")
                    .color(egui::Color32::from_rgb(17, 24, 39)),
            );
            ui.add_space(12.0);

            let file_label = self
                .selected_path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Chưa chọn file".to_string());
            ui.label(egui::RichText::new(file_label).color(subtle).italics());
            ui.add_space(16.0);

            let button = |ui: &mut egui::Ui, text: &str, enabled: bool, filled: bool| -> bool {
                let mut button = egui::Button::new(egui::RichText::new(text).size(16.0))
                    .min_size([240.0, 34.0].into());
                if filled && enabled {
                    button = button.fill(accent);
                }
                ui.add_enabled(enabled, button).clicked()
            };

            if button(ui, "Chọn file...", !self.processing, true) {
                self.choose_file();
            }
            ui.add_space(6.0);

            let submit_enabled = self.selected_path.is_some() && !self.processing;
            if button(ui, "Xử lý", submit_enabled, true) {
                self.submit();
            }
            ui.add_space(6.0);

            let open_enabled = self.output_path.is_some();
            if button(ui, "Mở file kết quả", open_enabled, false) {
                self.open_output();
            }

            ui.add_space(18.0);

            if self.processing {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(egui::RichText::new(&self.status).color(subtle));
                });
            } else if !self.status.is_empty() {
                egui::Frame::default()
                    .fill(card_bg)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::same(14))
                    .show(ui, |ui| {
                        ui.set_max_width(300.0);
                        ui.label(
                            egui::RichText::new(&self.status)
                                .color(egui::Color32::from_rgb(31, 41, 55)),
                        );
                    });
            }
            ui.add_space(20.0);
        });
    }
}

/// Registers Noto Sans as the primary font, front of both the proportional and
/// monospace family lists -- egui's bundled default font is missing the
/// precomposed Vietnamese glyphs (e.g. `ể`, `ậ`, `ữ`), which otherwise render as
/// tofu boxes. Noto Sans ("no tofu") explicitly covers the full Vietnamese
/// Latin Extended Additional block.
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../../assets/NotoSans.ttf"
        ))),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("noto_sans".to_owned());
    ctx.set_fonts(fonts);
}

fn setup_style(ctx: &egui::Context) {
    // Always light, regardless of the OS's dark/light setting -- the custom
    // card/accent colors below are tuned for a light background.
    ctx.set_theme(egui::ThemePreference::Light);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        for font_id in style.text_styles.values_mut() {
            font_id.size *= 1.15;
        }
        let rounding = egui::CornerRadius::same(8);
        style.visuals.widgets.inactive.corner_radius = rounding;
        style.visuals.widgets.hovered.corner_radius = rounding;
        style.visuals.widgets.active.corner_radius = rounding;
        style.visuals.window_fill = egui::Color32::WHITE;
        style.visuals.panel_fill = egui::Color32::WHITE;
    });
}

fn main() -> anyhow::Result<()> {
    configure_logging()?;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 420.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Accountant Check",
        native_options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            setup_style(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
    .map_err(|err| anyhow::anyhow!("eframe error: {err}"))
}
