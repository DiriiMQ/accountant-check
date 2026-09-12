"""Minimal desktop GUI: a welcome window with a file picker and submit button.

Uses tkinter (Python stdlib) rather than a web UI so the packaged app needs no
browser or local server -- just double-click and a window opens.
"""

import logging
from pathlib import Path
from tkinter import Button, Label, StringVar, Tk, filedialog, messagebox

from .engine import DEFAULT_RULES, run_rules
from .loader import load_invoices
from .logging_setup import LOG_FILE, configure_logging
from .report import write_report

logger = logging.getLogger(__name__)

CATEGORIES = ("Du_lieu_loi", "Trung_lap", "Vuot_nguong")


def process(input_path: Path) -> tuple[Path, dict[str, int]]:
    output = input_path.with_name("invalid_data.xlsx")
    logger.info("Bat dau xu ly: input=%s", input_path)
    df = load_invoices(input_path)
    logger.info("Da doc %d dong du lieu", len(df))
    violations = run_rules(df, DEFAULT_RULES)
    write_report(output, len(df), df, violations, None)
    counts = {
        category: len({violation.excel_row for violation in violations if violation.category == category})
        for category in CATEGORIES
    }
    logger.info("Ket qua: %s -> %s", counts, output)
    return output, counts


class App:
    def __init__(self, root: Tk):
        self.root = root
        self.selected_path: Path | None = None

        root.title("Accountant Check")
        root.resizable(False, False)

        Label(
            root, text="Kiểm tra danh sách hóa đơn đầu vào",
            font=("TkDefaultFont", 12, "bold"),
        ).pack(padx=24, pady=(20, 10))

        self.file_var = StringVar(value="Chưa chọn file")
        Label(root, textvariable=self.file_var, fg="gray").pack(padx=24, pady=(0, 14))

        Button(root, text="Chọn file...", width=22, command=self.choose_file).pack(pady=4)

        self.submit_button = Button(root, text="Xử lý", width=22, state="disabled", command=self.submit)
        self.submit_button.pack(pady=4)

        self.status_var = StringVar(value="")
        Label(root, textvariable=self.status_var, justify="left", anchor="w").pack(
            padx=24, pady=(16, 20), fill="x"
        )

    def choose_file(self) -> None:
        chosen = filedialog.askopenfilename(
            title="Chọn file danh sách hóa đơn (Excel)",
            filetypes=[("Excel files", "*.xlsx")],
        )
        if not chosen:
            return
        self.selected_path = Path(chosen)
        self.file_var.set(self.selected_path.name)
        self.submit_button.config(state="normal")
        self.status_var.set("")

    def submit(self) -> None:
        if self.selected_path is None:
            return
        self.submit_button.config(state="disabled")
        self.status_var.set("Đang xử lý...")
        self.root.update()
        try:
            output, counts = process(self.selected_path)
        except Exception as exc:
            logger.exception("Xu ly bi loi")
            messagebox.showerror(
                "Lỗi",
                f"Không thể xử lý file:\n{exc}\n\n"
                f"Chi tiết đã được lưu vào file log, vui lòng gửi file này để được hỗ trợ:\n{LOG_FILE}",
            )
            return
        finally:
            self.submit_button.config(state="normal")
        self.status_var.set(
            f"Dữ liệu lỗi: {counts['Du_lieu_loi']}\n"
            f"Trùng lặp: {counts['Trung_lap']}\n"
            f"Vượt ngưỡng: {counts['Vuot_nguong']}\n\n"
            f"Đã lưu kết quả: {output}"
        )


def main() -> None:
    configure_logging()
    root = Tk()
    App(root)
    root.mainloop()


if __name__ == "__main__":
    main()
