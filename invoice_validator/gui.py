"""Minimal desktop GUI: pick an input file, run validation, show a summary.

Uses tkinter (Python stdlib) rather than a web UI so the packaged app needs no
browser or local server -- just double-click and a native file-picker opens.
"""

from pathlib import Path
from tkinter import Tk, filedialog, messagebox

from .engine import DEFAULT_RULES, run_rules
from .loader import load_invoices
from .report import write_report

CATEGORIES = ("Du_lieu_loi", "Trung_lap", "Vuot_nguong")


def process(input_path: Path) -> tuple[Path, dict[str, int]]:
    output = input_path.with_name("invalid_data.xlsx")
    df = load_invoices(input_path)
    violations = run_rules(df, DEFAULT_RULES)
    write_report(output, len(df), df, violations, None)
    counts = {
        category: len({violation.excel_row for violation in violations if violation.category == category})
        for category in CATEGORIES
    }
    return output, counts


def main() -> None:
    root = Tk()
    root.withdraw()  # dialogs only -- no need for a full window

    chosen = filedialog.askopenfilename(
        title="Chon file danh sach hoa don (Excel)",
        filetypes=[("Excel files", "*.xlsx")],
    )
    if not chosen:
        return

    try:
        output, counts = process(Path(chosen))
    except Exception as exc:
        messagebox.showerror("Loi", f"Khong the xu ly file:\n{exc}")
        return

    messagebox.showinfo(
        "Hoan tat",
        (
            f"Du lieu loi: {counts['Du_lieu_loi']}\n"
            f"Trung lap: {counts['Trung_lap']}\n"
            f"Vuot nguong: {counts['Vuot_nguong']}\n\n"
            f"Da luu ket qua: {output}"
        ),
    )


if __name__ == "__main__":
    main()
