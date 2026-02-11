use std::path::Path;

use regex::Regex;

/// Converts a file path (relative to the spec book's `src/` directory) into
/// the dot-separated prefix for spec paragraph IDs.
///
/// # Examples
/// - `syntax/string-literals.md` → `syntax.string-literals`
/// - `syntax/README.md` → `syntax`
/// - `conventions.md` → `conventions`
/// - `README.md` → `` (empty)
pub fn file_path_to_prefix(source_path: &Path) -> String {
    let without_ext = source_path.with_extension("");

    // 💡 If the file is README.md, the prefix comes only from the parent directory.
    // This matches the mdbook convention where README.md is the index page for a directory.
    let effective_path = if without_ext.file_name().and_then(|f| f.to_str()) == Some("README") {
        without_ext
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default()
    } else {
        without_ext
    };

    effective_path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join(".")
}

/// Converts a heading text into a spec ID segment.
///
/// Lowercases, replaces spaces/underscores with hyphens, strips non-alphanumeric characters
/// (except hyphens).
pub fn heading_to_segment(heading_text: &str) -> String {
    heading_text
        .trim()
        .to_lowercase()
        .replace([' ', '_'], "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// Joins non-empty segments into a dot-separated spec ID.
///
/// Given a file prefix, the current heading segments, and an optional local name,
/// produces the fully-qualified spec paragraph ID.
pub fn resolve_spec_id(
    file_prefix: &str,
    heading_segments: &[String],
    local_name: &str,
) -> String {
    let mut parts: Vec<&str> = Vec::new();

    if !file_prefix.is_empty() {
        parts.push(file_prefix);
    }
    for seg in heading_segments {
        if !seg.is_empty() {
            parts.push(seg);
        }
    }
    if !local_name.is_empty() {
        parts.push(local_name);
    }

    parts.join(".")
}

/// Determines whether a token in a `:::{spec}` directive line is a tag rather than a local name.
///
/// Tags are: `rfcNNNN`, `!rfcNNNN`, `unimpl`.
pub fn is_tag(token: &str) -> bool {
    token.starts_with("rfc")
        || token.starts_with("!rfc")
        || token.starts_with('!')
        || token == "unimpl"
}

/// Parses the tokens after `:::{spec}` into an optional local name and a list of tags.
///
/// If the first token looks like a tag (starts with `rfc`, `!`, or is `unimpl`),
/// all tokens are treated as tags and there is no local name.
/// Otherwise the first token is the local name and the rest are tags.
pub fn parse_spec_tokens(rest: &str) -> (Option<String>, Vec<String>) {
    let tokens: Vec<&str> = rest.split_whitespace().filter(|s| !s.is_empty()).collect();
    if tokens.is_empty() {
        return (None, vec![]);
    }
    if is_tag(tokens[0]) {
        (None, tokens.iter().map(|s| s.to_string()).collect())
    } else {
        (
            Some(tokens[0].to_string()),
            tokens[1..].iter().map(|s| s.to_string()).collect(),
        )
    }
}

/// Tracks the current heading context while scanning a markdown file line-by-line.
///
/// 💡 H1 headings (`#`) are skipped because they correspond to the page title,
/// which is already captured in the file path prefix. Including H1 would double-count.
pub struct HeadingTracker {
    /// Stack of (heading level, segment) pairs.
    stack: Vec<(usize, String)>,
}

impl HeadingTracker {
    pub fn new() -> Self {
        HeadingTracker { stack: Vec::new() }
    }

    /// Processes a line of markdown. If it's a heading (level >= 2),
    /// updates the heading stack by popping entries at the same or deeper level,
    /// then pushing this heading.
    ///
    /// Returns `true` if the line was a heading.
    pub fn process_line(&mut self, line: &str) -> bool {
        let trimmed = line.trim();

        // Count leading `#` characters
        let hashes = trimmed.chars().take_while(|&c| c == '#').count();
        if hashes == 0 || hashes > 6 {
            return false;
        }

        // Must be followed by a space
        let rest = &trimmed[hashes..];
        if !rest.starts_with(' ') {
            return false;
        }

        let heading_text = rest.trim();
        if heading_text.is_empty() {
            return false;
        }

        // Skip H1 — it corresponds to the file/page title, already in file_prefix
        if hashes == 1 {
            return true;
        }

        let segment = heading_to_segment(heading_text);

        // Pop all entries at the same or deeper level
        self.stack.retain(|(level, _)| *level < hashes);
        self.stack.push((hashes, segment));

        true
    }

    /// Returns the current heading segments (just the segment strings, in order).
    pub fn current_segments(&self) -> Vec<String> {
        self.stack.iter().map(|(_, s)| s.clone()).collect()
    }
}

impl Default for HeadingTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Renders a list of spec tags (e.g., `rfc0001`, `unimpl`, `!rfc0001`) as HTML badge spans.
///
/// Returns an empty string if tags is empty, otherwise returns a space-prefixed
/// string of badge spans (matching the block-level directive badge format).
pub fn render_tag_badges(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }

    let badges: Vec<String> = tags
        .iter()
        .map(|tag| {
            if tag.starts_with('!') {
                format!("<span class=\"spec-rfc-badge spec-rfc-deleted\">{tag}</span>")
            } else if tag == "unimpl" {
                format!("<span class=\"spec-rfc-badge spec-rfc-unimpl\">{tag}</span>")
            } else {
                format!("<span class=\"spec-rfc-badge\">{tag}</span>")
            }
        })
        .collect();
    format!(" {}", badges.join(" "))
}

/// A parsed inline sub-paragraph marker found within a spec directive's content.
pub struct InlineSubParagraph {
    pub name: String,
    pub tags: Vec<String>,
    /// The index of the line within the directive content where this marker appears.
    pub line_index: usize,
}

/// Extracts inline `` {spec}`name [tags...]` `` markers from the content lines of a spec directive block.
///
/// The backtick content is parsed the same way as block directive arguments:
/// first token is the name, remaining tokens are tags (rfc, unimpl, etc.).
pub fn extract_inline_sub_paragraphs(content_lines: &[String]) -> Vec<InlineSubParagraph> {
    let re = Regex::new(r"\{spec\}`([^`]+)`").unwrap();
    let mut results = Vec::new();

    for (i, line) in content_lines.iter().enumerate() {
        for cap in re.captures_iter(line) {
            if let Some(content) = cap.get(1) {
                let (name, tags) = parse_spec_tokens(content.as_str());
                // 💡 For inline sub-paragraphs the first token is always the name,
                // even if it looks like a tag — unlike block directives where a
                // leading tag means "no local name". Inline markers must have a name.
                let name = name.unwrap_or_else(|| content.as_str().to_string());
                results.push(InlineSubParagraph {
                    name,
                    tags,
                    line_index: i,
                });
            }
        }
    }

    results
}

/// Replaces inline `` {spec}`name [tags...]` `` markers in content lines with HTML anchor spans
/// and tag badges.
///
/// Each marker becomes:
/// `<span id="parent_id.name" class="spec-sub-paragraph"><a href="#parent_id.name" class="spec-sub-label">.name</a> [badges]</span>`
pub fn transform_inline_sub_paragraphs(content_lines: &[String], parent_id: &str) -> Vec<String> {
    let re = Regex::new(r"\{spec\}`([^`]+)`").unwrap();

    content_lines
        .iter()
        .map(|line| {
            re.replace_all(line, |caps: &regex::Captures| {
                let content = &caps[1];
                let (name, tags) = parse_spec_tokens(content);
                let name = name.unwrap_or_else(|| content.to_string());
                let full_id = format!("{parent_id}.{name}");
                let badges = render_tag_badges(&tags);
                format!(
                    "<span id=\"{full_id}\" class=\"spec-sub-paragraph\">\
                     <a href=\"#{full_id}\" class=\"spec-sub-label\">.{name}</a>{badges}</span>"
                )
            })
            .into_owned()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_file_path_to_prefix() {
        assert_eq!(
            file_path_to_prefix(Path::new("syntax/string-literals.md")),
            "syntax.string-literals"
        );
        assert_eq!(
            file_path_to_prefix(Path::new("syntax/README.md")),
            "syntax"
        );
        assert_eq!(
            file_path_to_prefix(Path::new("conventions.md")),
            "conventions"
        );
        assert_eq!(file_path_to_prefix(Path::new("README.md")), "");
    }

    #[test]
    fn test_heading_to_segment() {
        assert_eq!(heading_to_segment("Delimiters"), "delimiters");
        assert_eq!(heading_to_segment("Escape Sequences"), "escape-sequences");
        assert_eq!(heading_to_segment("Type"), "type");
        assert_eq!(
            heading_to_segment("  Multiline Strings  "),
            "multiline-strings"
        );
    }

    #[test]
    fn test_resolve_spec_id() {
        assert_eq!(
            resolve_spec_id(
                "syntax.string-literals",
                &["delimiters".into()],
                "quoted"
            ),
            "syntax.string-literals.delimiters.quoted"
        );
        // No local name — ID is just context
        assert_eq!(
            resolve_spec_id("syntax.string-literals", &["delimiters".into()], ""),
            "syntax.string-literals.delimiters"
        );
        // Empty prefix (root file)
        assert_eq!(
            resolve_spec_id("", &["paragraph-references".into()], ""),
            "paragraph-references"
        );
        // Nested headings
        assert_eq!(
            resolve_spec_id(
                "syntax.string-literals",
                &["interpolation".into(), "advanced".into()],
                "nesting"
            ),
            "syntax.string-literals.interpolation.advanced.nesting"
        );
    }

    #[test]
    fn test_is_tag() {
        assert!(is_tag("rfc0001"));
        assert!(is_tag("!rfc0001"));
        assert!(is_tag("unimpl"));
        assert!(!is_tag("quoted"));
        assert!(!is_tag("delimiters"));
        assert!(!is_tag("escape-sequences"));
    }

    #[test]
    fn test_parse_spec_tokens() {
        // Local name + tags
        let (name, tags) = parse_spec_tokens("quoted rfc0001 unimpl");
        assert_eq!(name, Some("quoted".to_string()));
        assert_eq!(tags, vec!["rfc0001", "unimpl"]);

        // No local name — all tags
        let (name, tags) = parse_spec_tokens("rfc0001 unimpl");
        assert_eq!(name, None);
        assert_eq!(tags, vec!["rfc0001", "unimpl"]);

        // Empty
        let (name, tags) = parse_spec_tokens("");
        assert_eq!(name, None);
        assert!(tags.is_empty());

        // Local name only, no tags
        let (name, tags) = parse_spec_tokens("quoted");
        assert_eq!(name, Some("quoted".to_string()));
        assert!(tags.is_empty());
    }

    #[test]
    fn test_heading_tracker_basic() {
        let mut tracker = HeadingTracker::new();

        // H1 is skipped
        assert!(tracker.process_line("# String Literals"));
        assert!(tracker.current_segments().is_empty());

        // H2 adds a segment
        tracker.process_line("## Delimiters");
        assert_eq!(tracker.current_segments(), vec!["delimiters"]);

        // Same-level H2 replaces
        tracker.process_line("## Type");
        assert_eq!(tracker.current_segments(), vec!["type"]);

        // H2 then H3 nests
        tracker.process_line("## Escape Sequences");
        tracker.process_line("### Special Cases");
        assert_eq!(
            tracker.current_segments(),
            vec!["escape-sequences", "special-cases"]
        );

        // New H2 pops the H3
        tracker.process_line("## Interpolation");
        assert_eq!(tracker.current_segments(), vec!["interpolation"]);
    }

    #[test]
    fn test_heading_tracker_not_headings() {
        let mut tracker = HeadingTracker::new();

        // Not a heading — no space after #
        assert!(!tracker.process_line("#NotAHeading"));
        assert!(tracker.current_segments().is_empty());

        // Regular text
        assert!(!tracker.process_line("Some regular text"));

        // Empty line
        assert!(!tracker.process_line(""));

        // Code with hashes
        assert!(!tracker.process_line("####### Too many hashes"));
    }

    #[test]
    fn test_render_tag_badges() {
        assert_eq!(render_tag_badges(&[]), "");

        let result = render_tag_badges(&["rfc0001".to_string()]);
        assert!(result.contains("class=\"spec-rfc-badge\""));
        assert!(result.contains("rfc0001"));

        let result = render_tag_badges(&["unimpl".to_string()]);
        assert!(result.contains("spec-rfc-unimpl"));

        let result = render_tag_badges(&["!rfc0001".to_string()]);
        assert!(result.contains("spec-rfc-deleted"));

        let result = render_tag_badges(&["rfc0001".to_string(), "unimpl".to_string()]);
        assert!(result.contains("spec-rfc-badge\">rfc0001"));
        assert!(result.contains("spec-rfc-unimpl\">unimpl"));
    }

    #[test]
    fn test_extract_inline_sub_paragraphs() {
        let lines = vec![
            "There are multiple forms:".to_string(),
            "".to_string(),
            "* {spec}`quoted` Single quote...".to_string(),
            "* {spec}`triple-quoted` Triple quote...".to_string(),
            "Some other text".to_string(),
        ];
        let subs = extract_inline_sub_paragraphs(&lines);
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].name, "quoted");
        assert!(subs[0].tags.is_empty());
        assert_eq!(subs[0].line_index, 2);
        assert_eq!(subs[1].name, "triple-quoted");
        assert!(subs[1].tags.is_empty());
        assert_eq!(subs[1].line_index, 3);
    }

    #[test]
    fn test_extract_inline_sub_paragraphs_with_tags() {
        let lines = vec![
            "* {spec}`quoted` Implemented.".to_string(),
            "* {spec}`triple-quoted unimpl` Not yet.".to_string(),
            "* {spec}`raw rfc0002 unimpl` Future RFC.".to_string(),
        ];
        let subs = extract_inline_sub_paragraphs(&lines);
        assert_eq!(subs.len(), 3);

        assert_eq!(subs[0].name, "quoted");
        assert!(subs[0].tags.is_empty());

        assert_eq!(subs[1].name, "triple-quoted");
        assert_eq!(subs[1].tags, vec!["unimpl"]);

        assert_eq!(subs[2].name, "raw");
        assert_eq!(subs[2].tags, vec!["rfc0002", "unimpl"]);
    }

    #[test]
    fn test_transform_inline_sub_paragraphs() {
        let lines = vec![
            "There are multiple forms:".to_string(),
            "* {spec}`quoted` Single quote literals.".to_string(),
        ];
        let result = transform_inline_sub_paragraphs(&lines, "syntax.string-literals.delimiters");

        assert_eq!(result[0], "There are multiple forms:");
        assert!(result[1].contains("id=\"syntax.string-literals.delimiters.quoted\""));
        assert!(result[1].contains(">.quoted</a>"));
        assert!(result[1].contains("Single quote literals."));
    }

    #[test]
    fn test_transform_inline_sub_paragraphs_with_tags() {
        let lines = vec![
            "* {spec}`quoted` Implemented.".to_string(),
            "* {spec}`triple-quoted unimpl` Not yet.".to_string(),
        ];
        let result = transform_inline_sub_paragraphs(&lines, "s.delimiters");

        // quoted: no badges
        assert!(result[0].contains("id=\"s.delimiters.quoted\""));
        assert!(!result[0].contains("spec-rfc-badge"));

        // triple-quoted: unimpl badge, and ID uses only the name
        assert!(result[1].contains("id=\"s.delimiters.triple-quoted\""));
        assert!(result[1].contains(">.triple-quoted</a>"));
        assert!(result[1].contains("spec-rfc-unimpl"));
        assert!(result[1].contains("Not yet."));
    }
}
