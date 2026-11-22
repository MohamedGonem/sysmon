//! Filtering processes by a literal string or a regular expression.

use std::fmt;

use crate::regex::{Regex, RegexError};
use crate::types::ProcessStat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Literal,
    Regex,
}

#[derive(Debug)]
pub struct ProcessFilter {
    pattern: String,
    matcher: Matcher,
}

#[derive(Debug)]
enum Matcher {
    Literal(String),
    Regex(Regex),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchError {
    message: String,
}

impl SearchError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SearchError {}

impl From<RegexError> for SearchError {
    fn from(error: RegexError) -> Self {
        Self::new(error.to_string())
    }
}

impl ProcessFilter {
    /// Build a filter that matches PID, name, user, or command line.
    ///
    /// Literal matching is a case-insensitive substring search. Regex
    /// matching uses the in-tree regex engine.
    pub fn new(pattern: &str, mode: SearchMode) -> Result<Self, SearchError> {
        let matcher = match mode {
            SearchMode::Literal => Matcher::Literal(pattern.to_ascii_lowercase()),
            SearchMode::Regex => Matcher::Regex(Regex::new(pattern)?),
        };

        Ok(Self {
            pattern: pattern.to_string(),
            matcher,
        })
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn is_match(&self, process: &ProcessStat) -> bool {
        self.matches_text(&process.pid.to_string())
            || self.matches_text(&process.name)
            || self.matches_text(&process.user)
            || self.matches_text(&process.command_line)
    }

    fn matches_text(&self, text: &str) -> bool {
        match &self.matcher {
            Matcher::Literal(pattern) => text.to_ascii_lowercase().contains(pattern),
            Matcher::Regex(regex) => regex.is_match(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessFilter, SearchMode};
    use crate::types::ProcessStat;

    fn process() -> ProcessStat {
        ProcessStat {
            pid: 4242,
            name: "firefox".to_string(),
            state: 'S',
            cpu_usage_percentage: 1.5,
            memory_rss_bytes: 1024,
            memory_percentage: 0.1,
            virtual_memory_bytes: 2048,
            user: "crimson".to_string(),
            threads: 4,
            open_fds: Some(10),
            command_line: "/usr/lib/firefox/firefox --profile default".to_string(),
            parent_pid: 1,
            nice: 0,
            start_time: 0,
        }
    }

    fn literal(pattern: &str) -> ProcessFilter {
        ProcessFilter::new(pattern, SearchMode::Literal).expect("literal filter should build")
    }

    fn regex(pattern: &str) -> ProcessFilter {
        ProcessFilter::new(pattern, SearchMode::Regex).expect("regex filter should build")
    }

    #[test]
    fn literal_matches_every_searchable_field() {
        let process = process();

        assert!(literal("FIRE").is_match(&process));
        assert!(literal("crimson").is_match(&process));
        assert!(literal("profile").is_match(&process));
        assert!(literal("4242").is_match(&process));
    }

    #[test]
    fn literal_rejects_unrelated_text() {
        assert!(!literal("chrome").is_match(&process()));
    }

    #[test]
    fn regex_matches_with_anchors_and_fields() {
        let process = process();

        assert!(regex("^firefox$").is_match(&process));
        assert!(regex("^crimson$").is_match(&process));
        assert!(regex("^4242$").is_match(&process));
        assert!(regex(r"^/usr/lib/.+ --profile \w+$").is_match(&process));
    }

    #[test]
    fn regex_is_case_insensitive() {
        assert!(regex("^FIREFOX$").is_match(&process()));
    }

    #[test]
    fn regex_rejects_unrelated_text() {
        assert!(!regex("^chrome$").is_match(&process()));
    }

    #[test]
    fn invalid_regex_is_reported() {
        let error = ProcessFilter::new("(", SearchMode::Regex).expect_err("should fail");

        assert!(error.to_string().contains("missing ')'"), "{error}");
    }

    #[test]
    fn empty_patterns_match_everything() {
        let process = process();

        assert!(literal("").is_match(&process));
        assert!(regex("").is_match(&process));
    }
}
