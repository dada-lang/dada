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
            } else if let Some(extension) = path.extension() {
                if extension == "md" {
                    self.extract_spec_ids_from_file(&path)?;
                }
            }
        }
        Ok(())
    }

    /// Extracts all r[...] spec IDs from a markdown file
    fn extract_spec_ids_from_file(&mut self, file_path: &Path) -> Fallible<()> {
        let content = fs::read_to_string(file_path)?;
        
        // Regex to match r[spec.id] patterns
        let re = Regex::new(r"r\[([^\]]+)\]")?;
        
        for cap in re.captures_iter(&content) {
            if let Some(spec_id) = cap.get(1) {
                self.valid_spec_ids.insert(spec_id.as_str().to_string());
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
        validator.valid_spec_ids.insert("syntax.string-literals.basic".to_string());
        validator.valid_spec_ids.insert("permissions.lease.transfer".to_string());
        
        // Test batch validation
        let refs = vec![
            "syntax.string-literals.basic".to_string(),
            "invalid.spec.ref".to_string(),
            "permissions.lease.transfer".to_string(),
        ];
        let invalid_refs = validator.validate_spec_refs(&refs);
        assert_eq!(invalid_refs, vec!["invalid.spec.ref"]);
    }
}