//! Shared filesystem helpers.

use std::path::Path;

use walkdir::{DirEntry, WalkDir};

/// Options for filesystem traversal.
#[derive(Debug, Clone, Copy, Default)]
pub struct WalkFilesOptions {
    /// Follow symlinks/junctions while walking.
    pub follow_symlinks: bool,
    /// Maximum traversal depth (same semantics as `walkdir::WalkDir::max_depth`).
    pub max_depth: Option<usize>,
}

/// Walk a directory tree and yield files only (directories are filtered out).
///
/// The iterator yields `walkdir::Error` entries for traversal failures.
pub fn walk_files(
    root: impl AsRef<Path>,
    options: WalkFilesOptions,
) -> impl Iterator<Item = Result<DirEntry, walkdir::Error>> {
    let mut walker = WalkDir::new(root).follow_links(options.follow_symlinks);
    if let Some(max_depth) = options.max_depth {
        walker = walker.max_depth(max_depth);
    }

    walker.into_iter().filter_map(|entry| match entry {
        Ok(entry) if entry.file_type().is_file() => Some(Ok(entry)),
        Ok(_) => None,
        Err(err) => Some(Err(err)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn walk_files_respects_max_depth() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        fs::write(root.join("a.txt"), "a").unwrap();
        fs::create_dir(root.join("nested")).unwrap();
        fs::write(root.join("nested").join("b.txt"), "b").unwrap();

        let options = WalkFilesOptions {
            follow_symlinks: false,
            max_depth: Some(1),
        };

        let files: Vec<_> = walk_files(root, options)
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .map(|e| e.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert!(files.contains(&"a.txt".to_string()));
        assert!(!files.contains(&"b.txt".to_string()));
    }
}
