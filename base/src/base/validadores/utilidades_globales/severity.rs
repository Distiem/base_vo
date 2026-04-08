// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Error    => "ERROR",
            Self::Warning  => "WARNING",
            Self::Info     => "INFO",
            Self::Critical => "CRITICAL"
        };
        write!(f, "{s}")
    }
}
