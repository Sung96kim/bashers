use regex::Regex;

pub mod kmg;
pub mod track;

pub fn pod_pattern_regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|_| {
        let escaped = regex::escape(pattern);
        Regex::new(&format!("(?i){}", escaped)).expect("escaped pattern must be valid")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_regex_pattern() {
        let re = pod_pattern_regex("api-.*");
        assert!(re.is_match("api-server"));
        assert!(re.is_match("api-worker-123"));
        assert!(!re.is_match("frontend"));
    }

    #[test]
    fn test_invalid_regex_falls_back_to_escaped() {
        let re = pod_pattern_regex("[invalid");
        assert!(re.is_match("[invalid"));
        assert!(re.is_match("[INVALID"));
    }

    #[test]
    fn test_valid_regex_with_groups() {
        let re = pod_pattern_regex("pod(1)");
        assert!(re.is_match("pod1"));
    }

    #[test]
    fn test_invalid_regex_falls_back_literal_parens() {
        let re = pod_pattern_regex("pod(");
        assert!(re.is_match("pod("));
        assert!(re.is_match("POD("));
    }

    #[test]
    fn test_case_sensitive_valid_regex() {
        let re = pod_pattern_regex("MyPod");
        assert!(re.is_match("MyPod"));
        assert!(!re.is_match("mypod"));
    }

    #[test]
    fn test_fallback_is_case_insensitive() {
        let re = pod_pattern_regex("[MyPod");
        assert!(re.is_match("[MyPod"));
        assert!(re.is_match("[mypod"));
    }

    #[test]
    fn test_empty_pattern_matches_everything() {
        let re = pod_pattern_regex("");
        assert!(re.is_match("anything"));
        assert!(re.is_match(""));
    }
}
