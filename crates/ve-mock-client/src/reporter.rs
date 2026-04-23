//! Test result reporting

use crate::flows::FlowResult;

/// Output format for test results
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Output formatter for test results
pub struct Reporter {
    format: OutputFormat,
}

impl Reporter {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn print(&self, results: &[FlowResult]) {
        match self.format {
            OutputFormat::Json => self.print_json(results),
            OutputFormat::Text => self.print_text(results),
        }
    }

    fn print_text(&self, results: &[FlowResult]) {
        println!();
        println!("=== Test Results ===");
        println!();

        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for r in results {
            let icon = match r.status.as_str() {
                "PASS" => {
                    passed += 1;
                    "✓"
                }
                "FAIL" => {
                    failed += 1;
                    "✗"
                }
                "SKIP" => {
                    skipped += 1;
                    "-"
                }
                _ => "?",
            };
            println!(
                "  {} {} ({:.2}s) {}",
                icon, r.id, r.duration_secs, r.message
            );
        }

        println!();
        println!(
            "Total: {} | Passed: {} | Failed: {} | Skipped: {}",
            results.len(),
            passed,
            failed,
            skipped
        );
    }

    fn print_json(&self, results: &[FlowResult]) {
        let json = serde_json::to_string_pretty(results).unwrap_or_default();
        println!("{json}");
    }
}
