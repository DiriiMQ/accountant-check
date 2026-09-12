"""PyInstaller entry point for the packaged desktop app. Run directly, or via
`python run_gui.py`; the built exe launches this with no console window."""

from invoice_validator.gui import main

if __name__ == "__main__":
    main()
