use traces_fs::{FileWriter, WriteTarget};

use crate::error::TemplateArtifactError;

#[inline]
pub fn resolve_target(raw: &str) -> Result<WriteTarget, TemplateArtifactError> {
    WriteTarget::try_new(raw).map_err(TemplateArtifactError::Path)
}

#[inline]
pub fn commit(
    target: &WriteTarget,
    content: &str,
    writer: &impl FileWriter,
) -> Result<(), TemplateArtifactError> {
    writer.create_new(target, content.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod resolve_target {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn resolves_valid_relative_path() {
        let target = resolve_target("notes/out.md").unwrap();
        assert_eq!(target.as_path(), PathBuf::from("notes/out.md"));
    }

    #[test]
    fn rejects_absolute_path() {
        let err = resolve_target("/abs/out.md").unwrap_err();
        assert!(matches!(err, TemplateArtifactError::Path(_)));
    }

    #[test]
    fn rejects_traversal_path() {
        let err = resolve_target("../escape.md").unwrap_err();
        assert!(matches!(err, TemplateArtifactError::Path(_)));
    }
}

#[cfg(test)]
mod commit {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn writes_content_through_writer() {
        let dir = tempfile::tempdir().unwrap();
        let writer = traces_fs::FsWriter::new(dir.path());
        let target = resolve_target("out.md").unwrap();

        commit(&target, "hello", &writer).unwrap();

        let content =
            std::fs::read_to_string(dir.path().join("out.md")).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn rejects_commit_to_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("out.md"), "existing").unwrap();
        let writer = traces_fs::FsWriter::new(dir.path());
        let target = resolve_target("out.md").unwrap();

        let err = commit(&target, "new", &writer).unwrap_err();

        assert!(matches!(err, TemplateArtifactError::Write(_)));
    }

    /// Verifies that committing to a nested path like `subdir/out.md` creates
    /// the intermediate directory and writes the file at the expected location.
    #[test]
    fn creates_intermediate_directory() {
        let dir = tempfile::tempdir().unwrap();
        let writer = traces_fs::FsWriter::new(dir.path());
        let target = resolve_target("subdir/out.md").unwrap();

        commit(&target, "content", &writer).unwrap();

        let path = dir.path().join("subdir/out.md");
        assert!(path.exists());
        let content = std::fs::read_to_string(path).unwrap();
        assert_eq!(content, "content");
    }
}
