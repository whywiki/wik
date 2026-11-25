//! The object database. Objects are stored inside `.git/objects`

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use flate2::Compression;
use flate2::write::ZlibEncoder;

use crate::error::IoResultExt;
use crate::{Commit, Error, HashAlgorithm, Object, ObjectId, Result};

/// Object database of one repository (`.git/objects`).
///
/// It owns the hash function so every object it stores gets its
/// id from the same implementation
pub struct Database {
    objects_dir: PathBuf,
    hasher: Box<dyn HashAlgorithm>,
}

impl Database {
    /// Creates a database for the objects in `git_dir/objects`.
    ///
    /// `git_dir` is the repository `.git` directory and `hasher`
    /// computes the id of every object this database stores.
    pub fn new(git_dir: &Path, hasher: Box<dyn HashAlgorithm>) -> Self {
        Self {
            objects_dir: git_dir.join("objects"),
            hasher,
        }
    }

    /// Checks if object with this id is in database
    pub fn contains(&self, id: ObjectId) -> bool {
        let (_, path) = self.object_location(id);

        path.exists()
    }

    /// Where the object with this id is stored:
    /// - folder - `objects/<first 2 hex chars>`
    /// - file - `object/<...>/<other 38 hex chars>`
    fn object_location(&self, id: ObjectId) -> (PathBuf, PathBuf) {
        let hex = id.to_string();

        let (dir_name, file_name) = hex.split_at(2);
        let dir = self.objects_dir.join(dir_name);
        let path = dir.join(file_name);

        (dir, path)
    }

    /// Stores `object` as loose object and returns its id
    ///
    /// The object is compressed with zlib and written to
    /// `objects/<first 2 hex chars>/<other 38 chars>` by using a temp file
    /// in the same directory so no reader ever sees a half-written object.
    /// If the object already exists nothing is written
    ///
    /// # Errors
    ///
    /// Returns [`Error::Fs`](crate::Error::Fs) if the object cannot be written
    pub fn store(&self, object: &Object) -> Result<ObjectId> {
        // 1. Serialize once (`<type> <len>\0<content>`). Same bytes are used
        //    getting id and compression for the file
        let content = object.serialize();
        let id = self.hasher.hash(&content);

        // 2. Path from the id: `objects/<first 2 hex chars>/<other 38>`
        let (dir, path) = self.object_location(id);

        // 3. Same id == same content. No need to write again
        if path.exists() {
            return Ok(id);
        }

        // 4. Write compressed bytes to a temp file in the same directory
        //    so the final step is an instant rename and not a copy. If
        //    anything fails here, the temp file is deleted when `temp`
        //    is dropped
        fs::create_dir_all(&dir).context("mkdir", &dir)?;
        let mut temp = tempfile::Builder::new()
            .prefix("tmp_obj_")
            // read-only for everyone (`-r--r--r--`) like Git objects
            .permissions(fs::Permissions::from_mode(0o444))
            .tempfile_in(&dir)
            .context("create a temporary file in", &dir)?;

        temp.write_all(&compress(&content)?)
            .context("write", temp.path())?;

        // 5. Give the temp file its final name without overwriting so
        //    no reader sees a half-written object
        match temp.persist_noclobber(&path) {
            Ok(_) => Ok(id),
            // Another process stored the same object at the same time
            Err(err) if err.error.kind() == io::ErrorKind::AlreadyExists => Ok(id),
            // Other error
            Err(err) => Err(err.error).context("write", &path),
        }
    }

    /// Stores `commit` and returns its id.
    ///
    /// The commit tree and parents must already be in the database.
    /// For now this only checks that they exist not that the tree is
    /// really a tree and the parents are really commits.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingObject`] if the tree or a parent is not in
    /// the database and [`Error::Fs`] if the commit cannot be written.
    pub fn store_commit(&self, commit: Commit) -> Result<ObjectId> {
        // The tree and every parent must already be stored
        if !self.contains(commit.tree) {
            return Err(Error::MissingObject { id: commit.tree });
        }
        for &parent in &commit.parents {
            if !self.contains(parent) {
                return Err(Error::MissingObject { id: parent });
            }
        }

        self.store(&Object::Commit(commit))
    }
}

/// Compresses `data` with zlib at the fastest level (Git default
/// for loose objects)
fn compress(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}
