"""Command-line entry point: validate an invoice file and write the report."""

import argparse
import logging
import time
from pathlib import Path

from .config import DEFAULT_OUTPUT_FILENAME
from .cross_row_rules import OverThresholdRule
from .engine import DEFAULT_RULES, run_rules
from .loader import load_invoices
from .logging_setup import ResourceHeartbeat, configure_logging
from .report import write_report

logger = logging.getLogger(__name__)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate invoice list and split invalid/flagged rows into a report file.",
    )
    parser.add_argument("input", type=Path, help="Path to input invoice Excel file")
    parser.add_argument(
        "-o", "--output", type=Path, default=None,
        help=f"Path to output report file (default: {DEFAULT_OUTPUT_FILENAME!r} next to input)",
    )
    parser.add_argument(
        "--threshold", type=int, default=None,
        help=(
            "Flat cash-payment threshold in VND, applied to every row regardless of date. "
            "Default: apply the threshold that was actually in effect on each invoice's date "
            "(20,000,000 before 2025-07-01, 5,000,000 from 2025-07-01) -- see config.THRESHOLD_SCHEDULE."
        ),
    )
    return parser


def run(input_path: Path, output_path: Path | None, threshold: int | None) -> Path:
    output = output_path or input_path.with_name(DEFAULT_OUTPUT_FILENAME)
    logger.info("Starting processing: input=%s output=%s threshold=%s", input_path, output, threshold)

    with ResourceHeartbeat(logger):
        t0 = time.perf_counter()
        df = load_invoices(input_path)
        logger.info("Loaded %d data rows (%.2fs)", len(df), time.perf_counter() - t0)

        t0 = time.perf_counter()
        rules = [*DEFAULT_RULES[:-1], OverThresholdRule(threshold)]
        violations = run_rules(df, rules)
        category_counts = {
            category: len({violation.excel_row for violation in violations if violation.category == category})
            for category in ("Du_lieu_loi", "Trung_lap", "Vuot_nguong")
        }
        logger.info("Rules finished (%.2fs): %s", time.perf_counter() - t0, category_counts)

        t0 = time.perf_counter()
        write_report(output, len(df), df, violations, threshold)
        logger.info("Report written (%.2fs): %s", time.perf_counter() - t0, output)

    threshold_desc = f"{threshold:,} VND (flat override)" if threshold is not None else "theo ngay hieu luc (xem nguong_ap_dung)"
    print(f"Tong so dong du lieu : {len(df)}")
    print(f"Du lieu loi          : {category_counts['Du_lieu_loi']}")
    print(f"Trung lap            : {category_counts['Trung_lap']}")
    print(f"Vuot nguong ({threshold_desc}): {category_counts['Vuot_nguong']}")
    print(f"Da ghi ket qua vao   : {output}")

    return output


def main() -> None:
    configure_logging()
    args = build_parser().parse_args()
    try:
        run(args.input, args.output, args.threshold)
    except Exception:
        logger.exception("Processing failed")
        raise


if __name__ == "__main__":
    main()
