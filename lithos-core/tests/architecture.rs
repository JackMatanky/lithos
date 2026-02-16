//! Architecture tests to enforce design patterns and constraints.

#[cfg(test)]
mod tests {
    use std::fs;

    use glob::glob;

    #[test]
    fn ports_must_not_import_std_fs() {
        // We search from the project root, but the test runs in the crate root.
        // Paths are relative to the crate root (lithos-core).
        let port_files =
            glob("src/**/ports.rs").expect("Failed to read glob pattern");

        for entry in port_files {
            let path = entry.expect("Glob entry error");
            let content =
                fs::read_to_string(&path).expect("Failed to read port file");

            assert!(
                !content.contains("std::fs"),
                "Port file {path:?} must not import std::fs. Domain ports \
                 must remain pure and storage-agnostic."
            );

            assert!(
                !content.contains("use std::path::PathBuf"),
                "Port file {path:?} must not use PathBuf in imports. Use \
                 &Path for arguments if necessary, but prefer database-native \
                 identifiers."
            );
        }
    }

    #[test]
    fn port_traits_must_not_have_file_io_methods() {
        let port_files =
            glob("src/**/ports.rs").expect("Failed to read glob pattern");

        for entry in port_files {
            let path = entry.expect("Glob entry error");
            let content =
                fs::read_to_string(&path).expect("Failed to read port file");

            assert!(
                !content.contains("fn load_from_file"),
                "Port trait in {path:?} has forbidden file I/O method \
                 'load_from_file'. Use Application Services + FileSource \
                 instead."
            );

            assert!(
                !content.contains("fn scan_directory"),
                "Port trait in {path:?} has forbidden file I/O method \
                 'scan_directory'. Use Application Services + FileSource \
                 instead."
            );

            assert!(
                !content.contains("fn write_to_file"),
                "Port trait in {path:?} has forbidden file I/O method \
                 'write_to_file'. Domain ports handle persistence, not \
                 filesystem directly."
            );
        }
    }

    #[test]
    fn contexts_must_not_import_each_other() {
        let contexts = ["config", "note", "schema", "template"];

        for &ctx in &contexts {
            let pattern = format!("src/{ctx}/**/*.rs");
            let files = glob(&pattern).expect("Failed to read glob pattern");

            for entry in files {
                let path = entry.expect("Glob entry error");
                let content =
                    fs::read_to_string(&path).expect("Failed to read file");

                check_imports(ctx, &content, &path, &contexts);
            }
        }
    }

    fn check_imports(
        ctx: &str,
        content: &str,
        path: &std::path::Path,
        contexts: &[&str],
    ) {
        for &other in contexts {
            if ctx == other || other == "config" {
                continue;
            }

            let import_pattern = format!("crate::{other}");
            assert!(
                !content.contains(&import_pattern),
                "Context '{ctx}' (file {path:?}) must not import context \
                 '{other}'. Contexts must be isolated."
            );
        }
    }
}
