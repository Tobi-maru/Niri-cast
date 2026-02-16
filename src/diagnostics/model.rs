#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Ok,
    Warn,
    Error,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Ok => "OK",
            Severity::Warn => "WARN",
            Severity::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub title: String,
    pub severity: Severity,
    pub message: String,
    pub remediation: String,
}
