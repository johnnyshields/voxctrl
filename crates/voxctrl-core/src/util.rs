use std::path::{Path, PathBuf};

/// Join a `/`-separated relative path onto `base`, using the platform's native
/// path separator for each component.
///
/// ```text
/// repo_path(Path::new("D:\\models"), "mistralai/Voxtral-Mini-4B")
///   → "D:\\models\\mistralai\\Voxtral-Mini-4B"   (Windows)
///   → "D:\\models/mistralai/Voxtral-Mini-4B"      (Unix — unlikely base)
/// ```
pub fn repo_path(base: &Path, repo: &str) -> PathBuf {
    repo.split('/').fold(base.to_path_buf(), |p, c| p.join(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_path_splits_on_slash() {
        let base = Path::new("/models");
        let result = repo_path(base, "org/name");
        assert_eq!(result, PathBuf::from("/models").join("org").join("name"));
    }

    #[test]
    fn repo_path_single_component() {
        let base = Path::new("/models");
        let result = repo_path(base, "simple");
        assert_eq!(result, PathBuf::from("/models/simple"));
    }
}
