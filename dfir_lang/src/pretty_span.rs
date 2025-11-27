//! Pretty, human-readable printing of [`proc_macro2::Span`]s.

use std::path::Path;

extern crate proc_macro;

/// Helper struct which displays the span as `path:row:col` for human reading/IDE linking.
/// Example: `dfir\tests\surface_syntax.rs:42:18`.
pub struct PrettySpan(pub proc_macro2::Span);
impl std::fmt::Display for PrettySpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(nightly)]
        if proc_macro::is_available() {
            use std::path::MAIN_SEPARATOR;

            let span = self.0.unwrap();

            let path = span.file();
            let path = make_source_path_relative(&path);
            let mut path_str = path.display().to_string();
            if '/' != MAIN_SEPARATOR && path.is_relative() {
                // Display relative paths using unix-style separators for consistency.
                path_str = path_str.replace(MAIN_SEPARATOR, "/");
            }

            write!(
                f,
                "{}:{}:{}",
                path_str,
                span.start().line(),
                span.start().column(),
            )?;
            return Ok(());
        }

        write!(
            f,
            "unknown:{}:{}",
            self.0.start().line,
            self.0.start().column
        )
    }
}

/// Helper struct which displays the span as `row:col` for human reading.
pub struct PrettyRowCol(pub proc_macro2::Span);
impl std::fmt::Display for PrettyRowCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = self.0;
        write!(f, "{}:{}", span.start().line, span.start().column)
    }
}

/// Strip `DFIR_BASE_DIR` or `CARGO_MANIFEST_DIR` from the path prefix if possible.
pub fn make_source_path_relative(source_path: &impl AsRef<Path>) -> &Path {
    let source_path = source_path.as_ref();
    std::env::var_os("DFIR_BASE_DIR")
        .and_then(|base_dir| source_path.strip_prefix(base_dir).ok())
        .or_else(|| {
            let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")?;
            source_path.strip_prefix(manifest_dir).ok()
        })
        .unwrap_or(source_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use std::path::PathBuf;

    #[test]
    fn test_pretty_span_display() {
        let span = Span::call_site();
        let pretty = PrettySpan(span);
        let display_string = format!("{}", pretty);
        
        // Should contain line and column numbers
        assert!(display_string.contains(':'));
    }

    #[test]
    fn test_pretty_row_col_display() {
        let span = Span::call_site();
        let pretty = PrettyRowCol(span);
        let display_string = format!("{}", pretty);
        
        // Should be in format "row:col"
        assert!(display_string.contains(':'));
        // Should have at least line 1
        assert!(display_string.contains('1') || display_string.chars().any(|c| c.is_numeric()));
    }

    #[test]
    fn test_make_source_path_relative_absolute() {
        let path = PathBuf::from("/absolute/path/to/file.rs");
        let result = make_source_path_relative(&path);
        // Without env vars set, should return original path
        assert_eq!(result, path.as_path());
    }

    #[test]
    fn test_make_source_path_relative_with_env() {
        use std::env;
        
        let base_dir = "/tmp/test_base";
        let file_path = PathBuf::from("/tmp/test_base/src/file.rs");
        
        // Set environment variable temporarily
        env::set_var("DFIR_BASE_DIR", base_dir);
        
        let result = make_source_path_relative(&file_path);
        
        // Should strip the base directory
        assert_eq!(result, Path::new("src/file.rs"));
        
        // Clean up
        env::remove_var("DFIR_BASE_DIR");
    }

    #[test]
    fn test_make_source_path_relative_no_match() {
        let path = PathBuf::from("/other/path/file.rs");
        let result = make_source_path_relative(&path);
        // Should return original path when no env vars match
        assert_eq!(result, path.as_path());
    }

    #[test]
    fn test_make_source_path_relative_cargo_manifest() {
        use std::env;
        
        let manifest_dir = "/tmp/cargo_project";
        let file_path = PathBuf::from("/tmp/cargo_project/src/main.rs");
        
        // Ensure DFIR_BASE_DIR is not set
        env::remove_var("DFIR_BASE_DIR");
        // Set CARGO_MANIFEST_DIR
        env::set_var("CARGO_MANIFEST_DIR", manifest_dir);
        
        let result = make_source_path_relative(&file_path);
        
        // Should strip the manifest directory
        assert_eq!(result, Path::new("src/main.rs"));
        
        // Clean up
        env::remove_var("CARGO_MANIFEST_DIR");
    }

    #[test]
    fn test_make_source_path_relative_prefers_dfir_base() {
        use std::env;
        
        let base_dir = "/tmp/dfir_base";
        let manifest_dir = "/tmp/cargo_proj";
        let file_path = PathBuf::from("/tmp/dfir_base/test.rs");
        
        env::set_var("DFIR_BASE_DIR", base_dir);
        env::set_var("CARGO_MANIFEST_DIR", manifest_dir);
        
        let result = make_source_path_relative(&file_path);
        
        // Should prefer DFIR_BASE_DIR over CARGO_MANIFEST_DIR
        assert_eq!(result, Path::new("test.rs"));
        
        // Clean up
        env::remove_var("DFIR_BASE_DIR");
        env::remove_var("CARGO_MANIFEST_DIR");
    }
}
