use time::OffsetDateTime;

use crate::diagnostics::{DiagnosticItem, Severity};

#[derive(Debug, Clone)]
pub struct TroubleshootReport {
    pub generated_at: OffsetDateTime,
    pub items: Vec<DiagnosticItem>,
}

impl TroubleshootReport {
    pub fn new(items: Vec<DiagnosticItem>) -> Self {
        Self {
            generated_at: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
            items,
        }
    }

    pub fn ok_count(&self) -> usize {
        self.items.iter().filter(|x| x.severity == Severity::Ok).count()
    }

    pub fn warn_count(&self) -> usize {
        self.items
            .iter()
            .filter(|x| x.severity == Severity::Warn)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.items
            .iter()
            .filter(|x| x.severity == Severity::Error)
            .count()
    }
}
