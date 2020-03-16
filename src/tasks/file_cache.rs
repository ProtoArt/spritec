use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::query3d::{File, FileError};

/// A cache for Rc<File> that will only keep weak references.
///
/// If the file has not been opened or if the current reference to the file has been dropped,
/// a new Rc<File> will be created. The purpose of using weak references explicitly is to ensure
/// that files are dropped in a timely manner, even if this cache is accidentally left around.
#[derive(Debug, Default)]
pub struct WeakFileCache {
    // HACK: Turns out that dropping large amounts of vertex buffers is enormously expensive. This
    //   should be storing `Weak<Mutex<File>>`, not `Arc<Mutex<File>>`.
    // TODO: See if we can fix the glium bug that led to this hack. Check this back to `Weak` and
    //   generate a flamegraph. You'll see exactly where the slowness occurs.
    cache: HashMap<PathBuf, Arc<Mutex<File>>>,
}

impl WeakFileCache {
    /// Attempt to get a file from the cache
    ///
    /// Returns None if the file was never opened or if it has since been closed
    pub fn get(&self, path: &Path) -> Option<Arc<Mutex<File>>> {
        // HACK: Restore this to being the following line:
        //self.cache.get(path).and_then(|f| f.upgrade())
        self.cache.get(path).cloned()
    }

    /// Opens a 3D file based on its extension
    pub fn open(&mut self, path: &Path) -> Result<Arc<Mutex<File>>, FileError> {
        self.open_with(path, File::open)
    }

    /// Opens a glTF file
    pub fn open_gltf(&mut self, path: &Path) -> Result<Arc<Mutex<File>>, FileError> {
        self.open_with(path, File::open_gltf)
    }

    /// Attempts to retrieve the file from the cache, or opens the file with the given function if
    /// it was never opened or has since been closed.
    fn open_with<F>(&mut self, path: &Path, open: F) -> Result<Arc<Mutex<File>>, FileError>
        where F: FnOnce(&Path) -> Result<File, FileError>,
    {
        match self.get(path) {
            Some(file) => Ok(file),
            None => {
                let file = Arc::new(Mutex::new(open(path)?));
                // HACK: Restore this to being the following line:
                //self.cache.insert(path.to_path_buf(), Arc::downgrade(&file));
                self.cache.insert(path.to_path_buf(), file.clone());
                Ok(file)
            },
        }
    }
}
