"""Minimal desktop GUI: a welcome window with a file picker and submit button.

Uses tkinter (Python stdlib) rather than a web UI so the packaged app needs no
browser or local server -- just double-click and a window opens.
"""

from pathlib import Path
from tkinter import Button, Label, StringVar, Tk, filedialog, messagebox

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


class App:
    def __init__(self, root: Tk):
        self.root = root
        self.selected_path: Path | None = None

        root.title("Accountant Check")
        root.resizable(False, False)

        Label(
            root, text="Kiem tra danh sach hoa don dau vao",
            font=("TkDefaultFont", 12, "bold"),
        ).pack(padx=24, pady=(20, 10))

        self.file_var = StringVar(value="Chua chon file")
        Label(root, textvariable=self.file_var, fg="gray").pack(padx=24, pady=(0, 14))

        Button(root, text="Chon file...", width=22, command=self.choose_file).pack(pady=4)

        self.submit_button = Button(root, text="Xu ly", width=22, state="disabled", command=self.submit)
        self.submit_button.pack(pady=4)

        self.status_var = StringVar(value="")
        Label(root, textvariable=self.status_var, justify="left", anchor="w").pack(
            padx=24, pady=(16, 20), fill="x"
        )

    def choose_file(self) -> None:
        chosen = filedialog.askopenfilename(
            title="Chon file danh sach hoa don (Excel)",
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
        self.status_var.set("Dang xu ly...")
        self.root.update()
        try:
            output, counts = process(self.selected_path)
        except Exception as exc:
            messagebox.showerror("Loi", f"Khong the xu ly file:\n{exc}")
            return
        finally:
            self.submit_button.config(state="normal")
        self.status_var.set(
            f"Du lieu loi: {counts['Du_lieu_loi']}\n"
            f"Trung lap: {counts['Trung_lap']}\n"
            f"Vuot nguong: {counts['Vuot_nguong']}\n\n"
            f"Da luu ket qua: {output}"
        )


def main() -> None:
    root = Tk()
    App(root)
    root.mainloop()


if __name__ == "__main__":
    main()
