//! Command-line entry point: validate an invoice file and write the report.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use clap::Parser;

use invoice_validator::config::DEFAULT_OUTPUT_FILENAME;
use invoice_validator::engine::{default_rules_with_threshold, run_rules};
use invoice_validator::loader::load_invoices;
use invoice_validator::logging_setup::{ResourceHeartbeat, configure_logging};
use invoice_validator::report::write_report;
use invoice_validator::violations::Violation;

/// Validate invoice list and split invalid/flagged rows into a report file.
#[derive(Parser)]
struct Args {
    /// Path to input invoice Excel file
    input: PathBuf,

    /// Path to output report file (default: DEFAULT_OUTPUT_FILENAME next to input)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Flat cash-payment threshold in VND, applied to every row regardless of date.
    /// Default: apply the threshold that was actually in effect on each invoice's
    /// date (20,000,000 before 2025-07-01, 5,000,000 from 2025-07-01) -- see
    /// config::THRESHOLD_SCHEDULE.
    #[arg(long)]
    threshold: Option<i64>,
}

fn run(
    input_path: &Path,
    output_path: Option<&Path>,
    threshold: Option<i64>,
) -> anyhow::Result<PathBuf> {
    let output = output_path
        .map(PathBuf::from)
        .unwrap_or_else(|| input_path.with_file_name(DEFAULT_OUTPUT_FILENAME));
    log::info!(
        "Starting processing: input={} output={} threshold={:?}",
        input_path.display(),
        output.display(),
        threshold
    );

    let _heartbeat = ResourceHeartbeat::start(Duration::from_secs(5));

    let t0 = Instant::now();
    let rows = load_invoices(input_path)?;
    log::info!(
        "Loaded {} data rows ({:.2}s)",
        rows.len(),
        t0.elapsed().as_secs_f64()
    );

    let t0 = Instant::now();
    let rules = default_rules_with_threshold(threshold);
    let violations = run_rules(&rows, &rules);
    let category_counts = count_categories(&violations);
    log::info!(
        "Rules finished ({:.2}s): {:?}",
        t0.elapsed().as_secs_f64(),
        category_counts
    );

    let t0 = Instant::now();
    write_report(&output, rows.len(), &rows, &violations, threshold)?;
    log::info!(
        "Report written ({:.2}s): {}",
        t0.elapsed().as_secs_f64(),
        output.display()
    );

    let threshold_desc = match threshold {
        Some(t) => format!("{t} VND (flat override)"),
        None => "theo ngay hieu luc (xem nguong_ap_dung)".to_string(),
    };
    println!("Tong so dong du lieu : {}", rows.len());
    println!("Du lieu loi          : {}", category_counts.du_lieu_loi);
    println!("Trung lap            : {}", category_counts.trung_lap);
    println!(
        "Vuot nguong ({threshold_desc}): {}",
        category_counts.vuot_nguong
    );
    println!("Da ghi ket qua vao   : {}", output.display());

    Ok(output)
}

struct CategoryCounts {
    du_lieu_loi: usize,
    trung_lap: usize,
    vuot_nguong: usize,
}

impl std::fmt::Debug for CategoryCounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{'Du_lieu_loi': {}, 'Trung_lap': {}, 'Vuot_nguong': {}}}",
            self.du_lieu_loi, self.trung_lap, self.vuot_nguong
        )
    }
}

fn count_categories(violations: &[Violation]) -> CategoryCounts {
    use invoice_validator::config::{
        CATEGORY_DU_LIEU_LOI, CATEGORY_TRUNG_LAP, CATEGORY_VUOT_NGUONG,
    };
    use std::collections::HashSet;

    let count = |category: &str| -> usize {
        violations
            .iter()
            .filter(|v| v.category == category)
            .map(|v| v.excel_row)
            .collect::<HashSet<_>>()
            .len()
    };
    CategoryCounts {
        du_lieu_loi: count(CATEGORY_DU_LIEU_LOI),
        trung_lap: count(CATEGORY_TRUNG_LAP),
        vuot_nguong: count(CATEGORY_VUOT_NGUONG),
    }
}

fn main() -> anyhow::Result<()> {
    configure_logging()?;
    let args = Args::parse();
    match run(&args.input, args.output.as_deref(), args.threshold) {
        Ok(_) => Ok(()),
        Err(err) => {
            log::error!("Processing failed: {err:?}");
            Err(err)
        }
    }
}
