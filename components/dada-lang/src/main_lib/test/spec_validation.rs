use std::collections::HashSet;
use std::fs;
use std::path::Path;

use dada_util::{Fallible, bail};
use regex::Regex;

/// Manages validation of spec references from test files
pub struct SpecValidator {
    /// All valid spec IDs extracted from the spec mdbook
    valid_spec_ids: HashSet<String>,
}

impl SpecValidator {
    /// Creates a new spec validator by scanning the spec mdbook
    pub fn new() -> Fallible<Self> {
        let mut validator = SpecValidator {
            valid_spec_ids: HashSet::new(),
        };
        validator.load_spec_ids()?;
        Ok(validator)
    }

    /// Loads all spec IDs from the spec mdbook source files
    fn load_spec_ids(&mut self) -> Fallible<()> {
        let spec_src_path = Path::new("spec/src");

        if !spec_src_path.exists() {
            bail!(
                "Spec source directory not found at {}. Make sure you're running from the dada project root.",
                spec_src_path.display()
            );
        }

        self.scan_directory(spec_src_path)?;
        Ok(())
    }

    /// Recursively scans a directory for markdown files and extracts spec IDs
    fn scan_directory(&mut self, dir: &Path) -> Fallible<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path)?;
            } else if let Some(extension) = path.extension()
                && extension == "md"
            {
                self.extract_spec_ids_from_file(&path)?;
            }
        }
        Ok(())
    }

    /// Extracts spec IDs from MyST directive syntax, resolving relative IDs
    /// using the file path and heading context.
    ///
    /// 💡 Uses the same resolution logic as the preprocessor (via `dada_spec_common`)
    /// to ensure test `#:spec` annotations match the IDs generated in the spec HTML.
    fn extract_spec_ids_from_file(&mut self, file_path: &Path) -> Fallible<()> {
        let content = fs::read_to_string(file_path)?;

        let spec_src = Path::new("spec/src");
        let relative_path = file_path.strip_prefix(spec_src).unwrap_or(file_path);
        let file_prefix = dada_spec_common::file_path_to_prefix(relative_path);

        let directive_start = Regex::new(r"^:::\{spec\}(.*)$")?;
        let directive_end = Regex::new(r"^:::$")?;
        let inline_re = Regex::new(r"\{spec\}`([^`]+)`")?;

        let mut heading_tracker = dada_spec_common::HeadingTracker::new();
        let mut in_directive = false;
        let mut current_parent_id = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if !in_directive {
                heading_tracker.process_line(trimmed);

                if let Some(captures) = directive_start.captures(trimmed) {
                    let rest = captures.get(1).map(|m| m.as_str()).unwrap_or("");
                    let (local_name, _tags) = dada_spec_common::parse_spec_tokens(rest);

                    let full_id = dada_spec_common::resolve_spec_id(
                        &file_prefix,
                        &heading_tracker.current_segments(),
                        local_name.as_deref().unwrap_or(""),
                    );
                    self.valid_spec_ids.insert(full_id.clone());
                    current_parent_id = full_id;
                    in_directive = true;
                }
            } else if directive_end.is_match(trimmed) {
                in_directive = false;
                current_parent_id.clear();
            } else {
                // Inside directive: check for inline sub-paragraphs.
                // Parse the backtick content to separate the name from tags
                // (e.g., `triple-quoted unimpl` → name="triple-quoted", tags=["unimpl"]).
                for cap in inline_re.captures_iter(trimmed) {
                    if let Some(content) = cap.get(1) {
                        let (name, _tags) =
                            dada_spec_common::parse_spec_tokens(content.as_str());
                        // For inline sub-paragraphs, the first token is always the name
                        let name =
                            name.unwrap_or_else(|| content.as_str().to_string());
                        let sub_id = format!("{}.{}", current_parent_id, name);
                        self.valid_spec_ids.insert(sub_id);
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates a list of spec references, returning any that are invalid
    pub fn validate_spec_refs(&self, spec_refs: &[String]) -> Vec<String> {
        spec_refs
            .iter()
            .filter(|spec_ref| !self.valid_spec_ids.contains(*spec_ref))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_validation() {
        // Mock validator for testing
        let mut validator = SpecValidator {
            valid_spec_ids: HashSet::new(),
        };
        validator
            .valid_spec_ids
            .insert("syntax.string-literals.delimiters.quoted".to_string());
        validator
            .valid_spec_ids
            .insert("permissions.lease.transfer".to_string());

        // Test batch validation
        let refs = vec![
            "syntax.string-literals.delimiters.quoted".to_string(),
            "invalid.spec.ref".to_string(),
            "permissions.lease.transfer".to_string(),
        ];
        let invalid_refs = validator.validate_spec_refs(&refs);
        assert_eq!(invalid_refs, vec!["invalid.spec.ref"]);
    }
}
