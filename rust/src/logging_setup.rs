//! File-based logging so a user hitting an issue can send back one file.
//!
//! The desktop GUI runs windowed (no console), so a crash there is otherwise
//! invisible to us. `configure_logging()` sends everything to a single rotating
//! log file under the user's home directory instead -- old activity rotates
//! into numbered backups (kept for a bit more context), but the *current*
//! `app_rCURRENT.log` (see `log_file()` for why it's not plain `app.log`) always
//! has the most recent run, so a user only needs to send that one file.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, LoggerHandle, Naming};
use sysinfo::System;

/// Kept alive for the process lifetime -- dropping a `LoggerHandle` stops logging.
/// Also doubles as the "already configured" guard, mirroring Python's module-level
/// `_configured` flag (`configure_logging()` is safe to call more than once).
static LOGGER_HANDLE: OnceLock<LoggerHandle> = OnceLock::new();

pub fn log_dir() -> PathBuf {
    // Override hook for tests only, so they don't write into the real user's home
    // directory; production callers never set this.
    if let Some(dir) = std::env::var_os("ACCOUNTANT_CHECK_LOG_DIR") {
        return PathBuf::from(dir);
    }
    dirs_home().join(".accountant_check").join("logs")
}

/// The active log file. Named `app_rCURRENT.log`, not `app.log` -- flexi_logger's
/// size-based rotation (`Naming::Numbers`) always keeps the active file under an
/// `_rCURRENT` infix so it can atomically rename it to `app_r00001.log` etc. on
/// rotation; there's no built-in mode that rotates while keeping a plain
/// `<basename>.<suffix>` active name the way Python's `RotatingFileHandler` does.
/// Still a single, predictable file to point a user at for support -- just this
/// literal name instead of `app.log`.
pub fn log_file() -> PathBuf {
    log_dir().join("app_rCURRENT.log")
}

fn dirs_home() -> PathBuf {
    // `Path::home()`'s Rust equivalent: $HOME on macOS/Linux, USERPROFILE on Windows.
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .expect("no home directory found")
}

/// Set up the rotating file handler. Safe to call more than once.
pub fn configure_logging() -> anyhow::Result<PathBuf> {
    if LOGGER_HANDLE.get().is_none() {
        let dir = log_dir();
        std::fs::create_dir_all(&dir)?;

        let handle = Logger::try_with_str("info")?
            .log_to_file(
                FileSpec::default()
                    .directory(&dir)
                    .basename("app")
                    .suffix("log")
                    .suppress_timestamp(),
            )
            .rotate(
                Criterion::Size(1_000_000),
                Naming::Numbers,
                Cleanup::KeepLogFiles(3),
            )
            .format(flexi_logger::detailed_format)
            .start()?;

        log::info!(
            "Logging started (rust={}, platform={})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS
        );
        // Another thread may have won the race and already set this; either way
        // logging is now configured, which is all callers care about.
        let _ = LOGGER_HANDLE.set(handle);
    }
    Ok(log_file())
}

/// Logs CPU/RAM usage on a background thread every `interval` seconds.
///
/// Runs on its own thread rather than blocking the caller, so it keeps reporting
/// even while the main thread is stuck in a long synchronous call -- if the app
/// appears to hang, the log should still show whether resources were tight right
/// before it stopped advancing. Use as a scope guard around the processing call
/// (it stops itself on `Drop`).
pub struct ResourceHeartbeat {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl ResourceHeartbeat {
    pub fn start(interval: Duration) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        log_once();
        let thread = std::thread::spawn(move || {
            while !stop_thread.load(Ordering::Relaxed) {
                std::thread::sleep(interval);
                if stop_thread.load(Ordering::Relaxed) {
                    break;
                }
                log_once();
            }
        });
        ResourceHeartbeat {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for ResourceHeartbeat {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn log_once() {
    let mut system = System::new_all();
    system.refresh_cpu_usage();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_cpu_usage();
    system.refresh_memory();

    let cpu = system.global_cpu_usage();
    // sysinfo (0.30+) reports memory in bytes.
    let available_mb = system.available_memory() as f64 / (1024.0 * 1024.0);
    let total = system.total_memory().max(1) as f64;
    let used_percent = (system.used_memory() as f64 / total) * 100.0;
    log::info!(
        "Resource check: cpu={:.0}% mem_available={:.0}MB mem_used={:.0}%",
        cpu,
        available_mb,
        used_percent
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    /// One combined test (rather than several `#[test]` fns) because
    /// `configure_logging()` latches onto a global logger for the whole test
    /// binary -- `cargo test` runs tests in parallel threads by default, so
    /// separate tests racing to set `ACCOUNTANT_CHECK_LOG_DIR` first would be
    /// flaky. This mirrors the file-creation, idempotency, and heartbeat checks
    /// from the original Python `test_logging_setup.py`.
    #[test]
    fn configure_logging_and_heartbeat() {
        let dir = tempfile::tempdir().unwrap();
        unsafe {
            std::env::set_var("ACCOUNTANT_CHECK_LOG_DIR", dir.path());
        }

        let path = configure_logging().unwrap();
        LOGGER_HANDLE.get().unwrap().flush();
        if !path.exists() {
            let listing: Vec<_> = std::fs::read_dir(dir.path())
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            panic!("expected {path:?} to exist; dir contains: {listing:?}");
        }
        assert_eq!(path, dir.path().join("app_rCURRENT.log"));

        // Safe to call more than once.
        let path_again = configure_logging().unwrap();
        assert_eq!(path, path_again);

        let before = std::fs::metadata(&path).unwrap().len();
        let start = Instant::now();
        let heartbeat = ResourceHeartbeat::start(Duration::from_millis(50));
        LOGGER_HANDLE.get().unwrap().flush();
        // The first heartbeat entry is written synchronously in `start()`.
        assert!(
            std::fs::metadata(&path).unwrap().len() > before,
            "should log immediately on start"
        );
        std::thread::sleep(Duration::from_millis(700));
        drop(heartbeat);
        LOGGER_HANDLE.get().unwrap().flush();
        let after = std::fs::metadata(&path).unwrap().len();
        assert!(after > before, "heartbeat should have logged periodically");
        // Sanity: this test shouldn't be silently skipped by an implausibly short run.
        assert!(start.elapsed() >= Duration::from_millis(700));
    }
}
