//! Builds nested trees from a flat list of files.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{Database, Entry, Mode, Object, ObjectId, Result, Tree};

/// folder in memory while its tree is being built.
#[derive(Default)]
struct Folder {
    /// files directly in this folder
    files: Vec<Entry>,
    /// subfolders by name
    folders: BTreeMap<OsString, Folder>,
}

/// Stores one tree per folder and returns the id of the root tree.
///
/// Each item in `files` is a path relative to the root, the file mode,
/// and the id of its blob. The blobs must already be stored.
///
/// It works in two steps:
/// 1. Put every file into a [`Folder`] structure in memory.
/// 2. Store the trees, the deepest folders first.
///
/// Folders without files get no tree like in Git.
///
/// # Errors
///
/// Returns [`Error::Fs`](crate::Error::Fs) if a tree cannot be written.
pub(crate) fn store_tree(
    files: Vec<(PathBuf, Mode, ObjectId)>,
    database: &Database,
) -> Result<ObjectId> {
    let mut root = Folder::default();

    // Step 1: build the folders in memory
    for (path, mode, id) in files {
        root.insert(&path, mode, id);
    }

    // Step 2: store a tree for every folder
    root.store(database)
}

impl Folder {
    /// Puts file into the right subfolder. Missing folders are created and
    /// stored in [Folder].
    ///
    /// For example, `src/hash/sha1.rs` goes into the folder `src` then into
    /// `hash`, and `sha1.rs` is added.
    fn insert(&mut self, path: &Path, mode: Mode, id: ObjectId) {
        // folder we are in now. Every call start at self, which is always
        // root when called from store_tree
        let mut folder = self;

        // Go down through the folder names in the path. For `src/hash/sha1.rs`
        // the parent is `src/hash`
        if let Some(parent) = path.parent() {
            // First we take `src` and then `hash`
            for part in parent {
                // Go into the subfolder with this name (set new folder). If it
                // doesn't exist create an empty one
                folder = folder.folders.entry(part.to_os_string()).or_default();
            }
        }

        let name = path.file_name().expect("a listed file always has a name");

        // Now we are in the right folder. Add the file to it
        folder.files.push(Entry {
            name: name.to_os_string(),
            mode,
            id,
        });
    }

    /// Stores the tree of this folder and returns its id.
    ///
    /// Folder tree needs the ids of its subfolders trees, so the
    /// subfolders are stored first. This function calls itself for them
    /// so the deepest folders are stored first.
    fn store(self, database: &Database) -> Result<ObjectId> {
        // start with the files of this folder
        let mut entries = self.files;

        // store each subfolder tree then add an entry that points to it
        for (name, folder) in self.folders {
            let id = folder.store(database)?;
            entries.push(Entry {
                name,
                mode: Mode::Directory,
                id,
            });
        }

        // all entries are ready. Tree::new sorts them the way Git does
        database.store(&Object::Tree(Tree::new(entries)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Sha1Crate;
    use tempfile::TempDir;

    /// A file at `path` holding `content` as `store_tree` expects it
    fn file(path: &str, content: &[u8]) -> (PathBuf, Mode, ObjectId) {
        let id = Object::Blob(content.to_vec()).id(&Sha1Crate);
        (PathBuf::from(path), Mode::File, id)
    }

    /// A new repository in `tmp` and its object database
    fn database(tmp: &TempDir) -> Database {
        crate::init(tmp.path()).unwrap();
        Database::new(&tmp.path().join(".git"), Box::new(Sha1Crate))
    }

    #[test]
    fn nested_folders_match_git() {
        let tmp = TempDir::new().unwrap();
        let database = database(&tmp);
        // Not sorted on purpose: the order of the list must not matter.
        let files = vec![
            file("src/hash/sha1.rs", b"pub fn hash() {}\n"),
            file("README.md", b"read me\n"),
            file("src/main.rs", b"fn main() {}\n"),
        ];

        let id = store_tree(files, &database).unwrap();

        // `git write-tree` for a folder with exactly these files.
        assert_eq!(id.to_string(), "3768cc1578ecefc20a9f116369d801ce152a344e");
    }

    #[test]
    fn every_folder_gets_its_own_tree() {
        let tmp = TempDir::new().unwrap();
        let database = database(&tmp);
        let files = vec![
            file("src/hash/sha1.rs", b"pub fn hash() {}\n"),
            file("README.md", b"read me\n"),
            file("src/main.rs", b"fn main() {}\n"),
        ];

        store_tree(files, &database).unwrap();

        // The trees for `src` and `src/hash` with Git's ids
        let objects = tmp.path().join(".git/objects");
        assert!(
            objects
                .join("8b/e372e391225c80c78cea24a73f0aad2936668c")
                .is_file()
        );
        assert!(
            objects
                .join("35/5648fa532af3baedbcc408c074117e4963b93c")
                .is_file()
        );
    }

    #[test]
    fn no_files_give_the_empty_tree() {
        let tmp = TempDir::new().unwrap();
        let database = database(&tmp);

        let id = store_tree(Vec::new(), &database).unwrap();

        // `git write-tree` in a repository without files
        assert_eq!(id.to_string(), "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
    }
}
