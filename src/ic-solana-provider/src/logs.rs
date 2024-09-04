#![allow(dead_code)]

use ic_canister_log::{declare_log_buffer, GlobalBuffer, Sink};

// High-priority messages.
declare_log_buffer!(name = INFO_BUF, capacity = 1000);

// Low-priority info messages.
declare_log_buffer!(name = DEBUG_BUF, capacity = 1000);

// Trace of HTTP requests and responses.
declare_log_buffer!(name = TRACE_HTTP_BUF, capacity = 1000);

// Error messages.
declare_log_buffer!(name = ERROR_BUF, capacity = 1000);

pub const INFO: PrintProxySink = PrintProxySink("INFO", &INFO_BUF);
pub const DEBUG: PrintProxySink = PrintProxySink("DEBUG", &DEBUG_BUF);
pub const TRACE_HTTP: PrintProxySink = PrintProxySink("TRACE_HTTP", &TRACE_HTTP_BUF);
pub const ERROR: PrintProxySink = PrintProxySink("ERROR", &ERROR_BUF);

pub struct PrintProxySink(&'static str, &'static GlobalBuffer);

impl Sink for PrintProxySink {
    fn append(&self, entry: ic_canister_log::LogEntry) {
        ic_cdk::println!(
            "IS-SOLANA-PROVIDER: {} {}:{} {}",
            self.0,
            entry.file,
            entry.line,
            entry.message
        );
        self.1.append(entry)
    }
}
