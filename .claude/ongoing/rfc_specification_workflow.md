# RFC and Specification Workflow Design

## Status
Planning phase - All major design decisions resolved, ready for implementation

## Context
We want to establish a clear RFC and specification workflow for Dada language development. The goal is to:
- Make RFCs and specifications visible and accessible
- Use lightweight, focused tools (mdbook over Docusaurus)
- Separate concerns between RFCs, specs, and user documentation

## Decisions Made So Far

### Storage Structure
- `rfcs/` - mdbook for RFCs, deployed to `dada-lang.org/rfcs`
- `spec/` - mdbook for language specification, deployed to `dada-lang.org/spec` 
- `book/` - existing Docusaurus for tutorials/user docs at `dada-lang.org`
- Update `cargo xtask deploy` to handle all three sites

### Rationale
- mdbook is lighter weight and more approachable than Docusaurus
- RFCs and specs are distinct enough to warrant separation
- Keeps implementation docs (rustdoc) separate from language docs

## Resolved Decisions

1. **RFC lifecycle** - RFCs remain in the rfcs mdbook permanently as historical records, organized by status (Active, Accepted, Implemented, Rejected/Withdrawn)

2. **RFC numbering** - Simple sequential numbering (0001, 0002, etc.) with topic-based organization in SUMMARY.md

3. **RFC directory structure**:
   ```
   rfcs/
     src/
       SUMMARY.md
       0001-feature-name/
         README.md           # The main RFC document
         implementation.md   # Implementation progress tracking
         spec-draft.md      # Draft spec text (staging ground)
         examples/          # Example code if needed
   ```

4. **RFC to spec flow** - Mix of manual integration with spec drafts in RFC subdirectories:
   - RFCs can include draft spec text in `spec-draft.md` as a staging ground
   - When RFC is implemented, spec text is manually integrated into the main spec
   - Spec includes non-normative references to RFCs for motivation/context
   - Spec remains authoritative and factual; RFCs provide teaching/motivation

## Resolved Decisions (continued)

5. **RFC template**:
   ```markdown
   # RFC-0001: Title

   ## Summary
   Brief one-paragraph explanation

   ## Motivation
   Why are we doing this? What use cases does it support?

   ## Guide-level explanation
   Explain the proposal as if teaching it to another Dada programmer

   ## Reference-level explanation
   Technical details and edge cases

   ## Frequently asked questions
   Common questions and concerns about this design

   ## Future possibilities
   What future extensions or changes might this enable?
   ```

## Resolved Decisions (continued)

6. **Implementation tracking** - Combination approach:
   - SUMMARY.md organized by status sections (Active, Accepted, Implemented)
   - Each RFC README.md includes front matter with status:
     ```markdown
     ---
     status: active|accepted|implemented|rejected
     tracking-issue: #123  # optional
     implemented-version: 0.1.0  # optional, for implemented RFCs
     ---
     ```
   - Detailed implementation progress tracked in `implementation.md` within RFC directory

## Open Questions to Resolve

1. **Spec organization** - ✅ RESOLVED: Topic-based organization with Ferrocene-style paragraph identifiers
   - **Structure**: Organized by topic (syntax, type system, permissions, async, etc.) similar to Rust Reference
   - **Paragraph identifiers**: Each paragraph has a unique semantic identifier for cross-linking with tests
   - **Ferrocene inspiration**: Similar to Ferrocene Language Specification (influenced by Ada Reference Manual)
     - Each paragraph specifies one independent, testable rule/behavior  
     - Enables precise cross-referencing and requirement traceability
     - Format: `chapter:section:paragraph` using semantic names instead of numbers
     - Examples: `syntax:string-literals:escape-sequences`, `permissions:lease:transfer-rules`, `types:classes:field-access`
   - **Rationale for semantic IDs**: More stable during spec evolution and reorganization than numeric identifiers

2. **Cross-linking** - PARTIALLY RESOLVED
   - **Paragraph labeling**: Use `r[semantic.id]` syntax (like Rust Reference) to label spec paragraphs
   - **Test linking**: Tests annotated with `#:spec foo.bar.baz` to indicate which spec paragraph they test
   - **Markdown linking**: Still deciding between:
     a) Start with `/spec/...` URLs (works in production, broken locally)
     b) Add preprocessor later for environment-aware links
     c) Other options discussed: full URLs, link mapping file, shorthand syntax

3. **Decision authority** - ✅ RESOLVED: nikomatsakis acts as BDFL for RFC acceptance decisions

## Progress Update (2025-06-02)

### Recently Completed
- ✅ **mdbook structures** - Both `rfcs/` and `spec/` mdbooks created and working
- ✅ **Deploy script updated** - `cargo xtask deploy` now builds all three sites (Docusaurus, RFC mdbook, spec mdbook, rustdocs)
- ✅ **Paragraph labeling implemented** - `r[...]` syntax working in spec files
- ✅ **Markdown preprocessor created** - `dada-mdbook-preprocessor` processes `r[...]` labels into styled HTML anchors
- ✅ **Preprocessor applied to both books** - Both RFC and spec books use the preprocessor

### Current Task: CSS Consolidation ✅ COMPLETED

**Solution Implemented**: 
- Discovered that mdbook preprocessors cannot modify configuration dynamically
- Implemented inline CSS injection approach instead
- Preprocessor now:
  1. Scans each chapter for `r[...]` labels
  2. Processes labels into HTML with proper anchors
  3. Injects `<style>` tags at the end of chapters that contain labels
- Removed manual `additional-css` entries from both book.toml files
- Tested with both RFC and spec books - working correctly

### Recent Progress (2025-06-03)

**✅ RFC Template and Workflow Completed**
- Created RFC-0000 template as concise skeleton in RFC mdbook
- Implemented `cargo xtask rfc new` command that:
  - Auto-finds next RFC number
  - Creates RFC directory from template
  - Copies all template files with placeholder replacement
  - Updates SUMMARY.md automatically
- Updated `.development/rfc.md` workflow documentation
- Removed duplicate content and updated structure

### ✅ COMPLETED: Auto-populate SUMMARY.md (2025-06-03)

**Goal**: Modify the mdbook preprocessor to auto-populate the "All RFCs" section in SUMMARY.md

**Implementation Summary**:
- Modified `dada-mdbook-preprocessor` to scan for RFC directories and populate the book structure
- The preprocessor modifies the in-memory `Book` structure without touching files on disk
- Pattern inspired by rust-project-goals preprocessor

**Key Changes**:
1. Added `populate_rfc_sections` function that finds chapters to populate
2. Implemented `populate_all_rfcs_section` that:
   - Scans for directories matching `NNNN-*` pattern
   - Reads RFC titles from README.md files
   - Creates Chapter objects for each RFC and its sub-files
   - Extracts titles from first `#` heading in each file
3. Updated SUMMARY.md to use `- [All RFCs <!-- populate -->]()` format
4. Successfully tested with existing RFCs (0000-template and 0001-string-literals)

**Result**: The RFC book now automatically includes all RFCs with their sub-pages in the navigation

### ✅ COMPLETED: Dynamic RFC Population Enhancement (2025-06-13)

**Final Status**: All major RFC workflow features implemented and working

**What's Working**:
- ✅ SUMMARY.md simplified to just have `[All RFCs](./all.md)`
- ✅ Preprocessor successfully scans for RFC directories (`NNNN-*` pattern)
- ✅ Extracts RFC titles and creates chapter structure
- ✅ All RFCs appear in the book navigation with their sub-pages
- ✅ **NEW**: Beautiful "All RFCs" page with GitHub-inspired styling
- ✅ **NEW**: Status-based categorization (Active, Accepted, Implemented, Draft, Rejected)
- ✅ **NEW**: Shields.io status badges showing current RFC status
- ✅ **NEW**: Collapsible summary rows with disclosure triangles
- ✅ **NEW**: Proper markdown rendering in summary content
- ✅ **NEW**: Dark theme support for all components
- ✅ **NEW**: Strategic newlines for markdown processing in HTML

**Implementation Details Completed**:
- ✅ Extract status from RFC front matter with defaults
- ✅ Generate formatted HTML tables for all.md page
- ✅ GitHub-style table design with proper borders, padding, hover effects
- ✅ Disclosure triangles in number column for clean interaction
- ✅ Separate summary rows that expand below header rows
- ✅ CSS injection system for styling RFC tables
- ✅ AI memory comment system integrated into CLAUDE.md

### Remaining Lower Priority Tasks
- Set up test annotation system with `#:spec` comments
- Decide on markdown linking strategy
- Create example RFC using the new infrastructure

### Session Summary (2025-06-13)
Successfully completed the RFC All RFCs page implementation with a polished, GitHub-inspired interface that provides excellent UX for browsing RFCs by status with collapsible summaries.

## Progress Update: Test-Spec Linking Design (2025-06-13)

### ✅ COMPLETED: `#:spec` Comment System Design

**Design Decisions**:
1. **Syntax**: `#:spec topic.subtopic.detail` following existing `#:` configuration pattern
2. **Granularity**: Per-file (entire test file tests specified spec paragraphs)
3. **Location**: Must appear in file header like other `#:` configurations
4. **Multiple specs**: Each test file can reference multiple spec paragraphs (one per `#:spec` line)
5. **Validation**: Test runner will parse spec mdbook to validate spec IDs exist

**Implementation Plan**:
- Update `TestExpectations` struct in `expected.rs` to include `spec_refs: Vec<String>`
- Add parsing in `configuration()` method to handle `#:spec` lines
- Create spec validation module to parse spec mdbook and extract `r[...]` labels
- Generate spec coverage reports showing tested/untested paragraphs

### ✅ COMPLETED: Spec-to-Test Cross-linking Design

**Enhanced User Experience**:
1. **Visual indicator**: Test icon (🧪) next to each `r[...]` label in spec
2. **Dedicated test pages**: Each spec ID gets a page showing all related tests
3. **Interactive test viewer**:
   - Collapsible test content with disclosure triangles
   - "Expand All" / "Collapse All" buttons
   - Syntax-highlighted test code
   - GitHub links for each test file
   - Similar styling to RFC "All" page

**Implementation Approach**:
- Enhance mdbook preprocessor to:
  - Scan test files for `#:spec` annotations during build
  - Build index mapping spec IDs → test files
  - Modify `r[...]` rendering to add test icon links
  - Generate test pages for each spec ID
  - Include necessary CSS/JS for interactivity

**Example test page** (`spec/src/tests/syntax.string-literals.escape-sequences.md`):
- Lists all tests that reference this spec paragraph
- Shows test content in collapsible sections
- Links to GitHub for each test file
- Maintains consistent GitHub-inspired styling

### Implementation Plan

**Phase 1: Test Runner Foundation**
1. **Start with test runner** (`components/dada-lang/src/main_lib/test/expected.rs`)
   - Add `spec_refs: Vec<String>` to `TestExpectations` struct
   - Update `configuration()` method to parse `#:spec` lines
   - This gives us basic parsing without breaking anything

**Phase 2: Spec Validation**
2. **Create spec validation module** (`components/dada-lang/src/main_lib/test/spec_validation.rs`)
   - Parse spec mdbook to extract all `r[...]` labels
   - Validate `#:spec` references against actual spec paragraphs
   - Add validation to test runner

**Phase 3: Test Integration**
3. **Add spec validation to test execution** (update `test.rs`)
   - Call spec validation during test runs
   - Report invalid spec references as test failures

**Phase 4: mdbook Enhancement**
4. **Enhance mdbook preprocessor** (`components/dada-mdbook-preprocessor/src/lib.rs`)
   - Scan test files for `#:spec` annotations during build
   - Build spec-to-tests index
   - Modify `r[...]` rendering to add test icons

**Phase 5: Test Viewer Pages**
5. **Generate test viewer pages**
   - Create test pages showing related tests
   - Add collapsible content and GitHub links

**Phase 6: Polish**
6. **Add CSS/JS for interactivity**
   - Expand/collapse functionality
   - Styling consistent with RFC pages

## Progress Update: Test-Spec Linking Implementation (2025-06-15)

### ✅ COMPLETED: Phases 1-3 of Test-Spec Linking System

**Implementation Summary:**
Successfully implemented the core `#:spec` comment validation system that validates test file spec references against actual spec paragraphs.

**What's Working:**
- ✅ **Phase 1**: Test runner foundation - Added `spec_refs: Vec<String>` to `TestExpectations` struct and parsing for `#:spec` configuration comments
- ✅ **Phase 2**: Spec validation module - Created `SpecValidator` that scans `spec/src/` directory and extracts all `r[...]` labels using regex
- ✅ **Phase 3**: Test integration - Added `InvalidSpecReference` failure type and integrated validation into test execution flow
- ✅ **Error reporting**: Clear error messages in test reports when spec references are invalid
- ✅ **Validation logic**: Validates `#:spec topic.subtopic.detail` annotations against actual spec paragraph labels

**Technical Details:**
- `SpecValidator::new()` recursively scans spec mdbook source files
- Extracts spec IDs using pattern `r\[([^\]]+)\]` 
- Validation runs during `TestExpectations::compare()` method
- Invalid references reported as test failures with helpful guidance
- Added test file `tests/test_spec_parsing.dada` demonstrating usage

**Commit:** `319d9217` - "Implement #:spec comment validation system for test-spec linking"

### Remaining Tasks (Lower Priority)

**Phase 4-6: Enhanced mdbook Integration**
- **Phase 4**: Enhance mdbook preprocessor to scan test files and build spec-to-tests index
- **Phase 5**: Generate interactive test viewer pages showing tests for each spec paragraph  
- **Phase 6**: Add CSS/JS for expand/collapse functionality and GitHub-consistent styling

**Design for Future Phases:**
- Test icon (🧪) next to `r[...]` labels linking to test pages
- Dedicated pages like `spec/src/tests/syntax.string-literals.basic.md` showing related tests
- Collapsible test content with syntax highlighting and GitHub links
- "Expand All" / "Collapse All" buttons for test viewer pages

### Session Summary (2025-06-15)
Core `#:spec` validation system is production-ready. Test files can now reference spec paragraphs and get immediate validation feedback. The foundation is in place for future enhanced mdbook integration phases.