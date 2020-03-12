use std::path::{Path, PathBuf, Component};

/// Extension trait that provides the ability to resolve a path relative to a base directory
pub trait ResolvePath {
    /// Resolves a path relative to the given base directory. Returns an absolute path that is
    /// normalized (fixes any platform differences).
    ///
    /// The base directory path must be absolute
    fn resolve(&self, base_dir: &Path) -> PathBuf;
}

impl ResolvePath for Path {
    fn resolve(&self, base_dir: &Path) -> PathBuf {
        // Path resolution based on code found at
        // https://github.com/rust-lang/cargo/blob/9ef364a5507ef87843c5f37b11d3ccfbd8cbe478/src/cargo/util/paths.rs#L65-L90
        //
        // Resolution removes . and .. from the path, where . is removed without affecting the rest
        // of the path and .. will remove its parent from the path.
        //
        // This is needed because Windows extended-length paths disallow string parsing, so ".."
        // and "." aren't resolved in path names.
        //
        // Reference: https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file

        // Constraint: The base directory path (base_dir) should always be an absolute path.
        assert!(base_dir.is_absolute(), "Base directory path was not absolute.");

        // Create an absolute path, using base_dir only if necessary
        let path = if self.is_absolute() {
            self.to_path_buf()
        } else {
            base_dir.join(self)
        };

        // All slashes are converted to backslahes (for Windows) or to forward slashes (for every other OS)
        // so that PathBuf's components function can properly extract out the components of the path
        #[cfg(windows)]
        let path_str = path.to_str()
            .expect("Path was not valid Unicode")
            .replace("/", "\\");
        #[cfg(not(windows))]
        let path_str = path.to_str()
            .expect("Path was not valid Unicode")
            .replace("\\", "/");
        let path = Path::new(&path_str);

        let mut normalized_path = PathBuf::new();
        for component in path.components().peekable() {
            match component {
                Component::Prefix(_) |
                Component::RootDir => normalized_path.push(component.as_os_str()),
                Component::CurDir => {},
                Component::ParentDir => {
                    normalized_path.pop();
                },
                Component::Normal(c) => normalized_path.push(c),
            }
        }

        normalized_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! resolve_check {
        ($base:expr, $input:expr, $output:expr) => {
            let input_path = Path::new($input);
            let base_path = Path::new($base);
            assert_eq!(input_path.resolve(base_path), std::path::PathBuf::from($output));
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_absolute_path() {
        let input_path = "/home/yourname/spritec/sample/bigboi/test";

        let base_path = "/home/yourname/spritec/sample";
        let output_path = "/home/yourname/spritec/sample/bigboi/test";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_absolute_path() {
        let input_path = "C:\\user\\yourname\\spritec\\sample\\bigboi\\test";

        let base_path = "C:\\user\\yourname\\spritec\\sample";
        let output_path = "C:\\user\\yourname\\spritec\\sample\\bigboi\\test";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_relative_path_forward_slash() {
        let input_path = "../../../spritec/../src/./bin";

        let base_path = "/home/yourname/spritec";
        let output_path = "/src/bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_relative_path_forward_slash() {
        let input_path = "../../../spritec/../src/./bin";

        let base_path = "C:\\user\\yourname\\spritec";
        let output_path = "C:\\src\\bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(not(windows))]
    #[test]
    fn resolve_relative_path_backslash() {
        let input_path = "..\\..\\..\\spritec\\..\\src\\.\\bin";

        let base_path = "/home/yourname/spritec";
        let output_path = "/src/bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_relative_path_backslash() {
        let input_path = "..\\..\\..\\spritec\\..\\src\\.\\bin";

        let base_path = "C:\\user\\yourname\\spritec";
        let output_path = "C:\\src\\bin";
        resolve_check!(base_path, input_path, output_path);
    }

    #[cfg(windows)]
    #[test]
    fn resolve_relative_path_backslash_prefix() {
        let input_path = "..\\..\\..\\spritec\\..\\src\\.\\bin";

        // This special prefix is an old Windows feature that no one uses
        let base_path = "\\\\?\\C:\\user\\yourname\\spritec";
        let output_path = "\\\\?\\C:\\src\\bin";
        resolve_check!(base_path, input_path, output_path);
    }
}
