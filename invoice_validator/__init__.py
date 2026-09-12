"""Validate an invoice list Excel file and split invalid/flagged rows into a report file.

See config.py for the input column layout and the business-rule thresholds, and
CLAUDE.md / vietnam-e_invoice_knowledge_base.md for where those rules come from.
"""

from .config import DEFAULT_THRESHOLD

__all__ = ["DEFAULT_THRESHOLD"]
