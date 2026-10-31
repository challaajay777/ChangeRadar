use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct IgnoreMatcher {
    glob_set: GlobSet,
    raw_patterns: Vec<String>,
}

impl IgnoreMatcher {
    pub fn new(patterns: &[String]) -> Self {
        let mut builder = GlobSetBuilder::new();
        let mut clean_patterns = Vec::new();

        for p in patterns {
            let trimmed = p.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Ok(glob) = GlobBuilder::new(trimmed)
                .case_insensitive(true)
                .literal_separator(false)
                .build()
            {
                builder.add(glob);
                clean_patterns.push(trimmed.to_string());
            }
        }

        let glob_set = builder.build().unwrap_or_else(|_| GlobSet::empty());
        Self {
            glob_set,
            raw_patterns: clean_patterns,
        }
    }

    pub fn is_ignored<P: AsRef<Path>>(&self, path: P) -> bool {
        let p = path.as_ref();
        
        // Fast checks for common hidden and temporary extensions
        if let Some(file_name) = p.file_name().and_then(|s| s.to_str()) {
            if file_name.starts_with("~$") || file_name.ends_with(".tmp") || file_name.ends_with(".crdownload") {
                return true;
            }
            if file_name == ".DS_Store" || file_name == "Thumbs.db" || file_name == "desktop.ini" {
                return true;
            }
        }

        self.glob_set.is_match(p)
    }

    pub fn patterns(&self) -> &[String] {
        &self.raw_patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_matcher() {
        let patterns = vec![
            ".git/**".to_string(),
            "node_modules/**".to_string(),
            "*.tmp".to_string(),
        ];
        let matcher = IgnoreMatcher::new(&patterns);

        assert!(matcher.is_ignored(".git/HEAD"));
        assert!(matcher.is_ignored("node_modules/react/index.js"));
        assert!(matcher.is_ignored("document.tmp"));
        assert!(!matcher.is_ignored("src/main.rs"));
        assert!(!matcher.is_ignored("docs/report.pdf"));
    }
}
