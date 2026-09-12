//! Rust re-implementation of the Vietnamese invoice-compliance validator
//! (formerly the `invoice_validator` Python package). See `../../CLAUDE.md`
//! for the business-rule background and the migration plan.

pub mod config;
pub mod cross_row_rules;
pub mod engine;
pub mod field_rules;
pub mod helpers;
pub mod loader;
pub mod logging_setup;
pub mod report;
pub mod row;
pub mod violations;
