//! Minimal desktop GUI: native file-picker -> process -> native result dialog.
//!
//! Uses `rfd` (native OS dialogs only, no window/widget toolkit) rather than a
//! full GUI framework, matching the scope of the original tkinter app's actual
//! job -- picking a file and reporting back counts -- with the smallest possible
//! new dependency surface for this rewrite. Unlike the Python version's
//! persistent window, this app runs one file through per launch and exits; the
//! "open result file" action from that window becomes a Yes/No prompt at the
//! end instead of a standing button.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

fn main() -> anyhow::Result<()> {
    configure_logging()?;

    let Some(input_path) = rfd::FileDialog::new()
        .set_title("Chọn file danh sách hóa đơn (Excel)")
        .add_filter("Excel files", &["xlsx"])
        .pick_file()
    else {
        return Ok(());
    };

    match process(&input_path) {
        Ok((output, counts)) => {
            let message = format!(
                "Dữ liệu lỗi: {}\nTrùng lặp: {}\nVượt ngưỡng: {}\n\nĐã lưu kết quả: {}\n\nMở file kết quả?",
                counts.du_lieu_loi,
                counts.trung_lap,
                counts.vuot_nguong,
                output.display()
            );
            let open_it = MessageDialog::new()
                .set_title("Hoàn tất")
                .set_description(message)
                .set_level(MessageLevel::Info)
                .set_buttons(MessageButtons::YesNo)
                .show();
            if open_it == rfd::MessageDialogResult::Yes
                && let Err(err) = open_file(&output)
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
        Err(err) => {
            log::error!("Processing failed: {err:?}");
            MessageDialog::new()
                .set_title("Lỗi")
                .set_description(format!(
                    "Không thể xử lý file:\n{err}\n\nChi tiết đã được lưu vào file log, vui lòng gửi file này để được hỗ trợ:\n{}",
                    log_file().display()
                ))
                .set_level(MessageLevel::Error)
                .set_buttons(MessageButtons::Ok)
                .show();
        }
    }

    Ok(())
}
