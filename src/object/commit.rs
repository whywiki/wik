//! Commit objects. Commit saves a snapshot of:
//! - files (tree)
//! - previous commits
//! - who made it
//! - when they made it
//! - message

use std::fmt;

use crate::ObjectId;

/// Who made a change, and when
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// The person name.
    pub name: String,
    /// The person email
    pub email: String,
    /// When: in seconds since 1970-01-01 UTC (Unix time).
    pub time: i64,
    /// The time zone as minutes away from UTC: `60` is `+0100`
    pub offset_minutes: i32,
}

impl fmt::Display for Signature {
    /// Writes the signature as Git does: `Wiki <wiki@example.com> 1700000000 +0100`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let minutes = self.offset_minutes.abs();
        write!(
            f,
            "{} <{}> {} {sign}{:02}{:02}",
            self.name,
            self.email,
            self.time,
            minutes / 60,
            minutes % 60
        )
    }
}

/// Commit object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    /// id of the root tree: snapshot of the files
    pub tree: ObjectId,
    /// ids of the commits this one comes after. A first commit has
    /// none and merge has two or more
    pub parents: Vec<ObjectId>,
    /// Who wrote the change
    pub author: Signature,
    /// Who made the commit. Usually the same as the author
    pub committer: Signature,
    /// The commit message
    pub message: String,
}

impl Commit {
    /// The commit's content in Git format, without the object header:
    ///
    /// ```text
    /// tree <id>
    /// parent <id> (one line per parent)
    /// author <signature>
    /// committer <signature>
    ///
    /// <message>
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut text = format!("tree {}\n", self.tree);
        for parent in &self.parents {
            text.push_str(&format!("parent {parent}\n"));
        }
        text.push_str(&format!("author {}\n", self.author));
        text.push_str(&format!("committer {}\n", self.committer));
        // An empty line ends the header lines
        text.push('\n');
        text.push_str(&self.message);
        text.into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Object, Sha1Crate};

    /// Wiki signature
    fn wiki() -> Signature {
        Signature {
            name: "Wiki".to_string(),
            email: "wiki@example.com".to_string(),
            time: 1_700_000_000,
            offset_minutes: 60,
        }
    }

    /// The id of the empty tree which is the same in every Git repository
    fn empty_tree() -> ObjectId {
        "4b825dc642cb6eb9a060e54bf8d69288fbee4904".parse().unwrap()
    }

    /// A first commit of the empty tree made by Wiki
    fn first_commit() -> Commit {
        Commit {
            tree: empty_tree(),
            parents: Vec::new(),
            author: wiki(),
            committer: wiki(),
            message: "first commit\n".to_string(),
        }
    }

    #[test]
    fn signature_writes_the_offset_as_sign_hours_minutes() {
        let mut signature = wiki();
        assert_eq!(
            signature.to_string(),
            "Wiki <wiki@example.com> 1700000000 +0100"
        );

        signature.offset_minutes = -330;
        assert_eq!(
            signature.to_string(),
            "Wiki <wiki@example.com> 1700000000 -0530"
        );

        signature.offset_minutes = 0;
        assert_eq!(
            signature.to_string(),
            "Wiki <wiki@example.com> 1700000000 +0000"
        );
    }

    #[test]
    fn first_commit_is_written_like_git() {
        let expected = "tree 4b825dc642cb6eb9a060e54bf8d69288fbee4904\n\
                        author Wiki <wiki@example.com> 1700000000 +0100\n\
                        committer Wiki <wiki@example.com> 1700000000 +0100\n\
                        \n\
                        first commit\n";

        assert_eq!(first_commit().to_bytes(), expected.as_bytes());
    }

    // The ids below are from `git commit-tree` with the same tree, people,
    // times and messages

    #[test]
    fn first_commit_id_matches_git() {
        let id = Object::Commit(first_commit()).id(&Sha1Crate);

        assert_eq!(id.to_string(), "4cb646982ce6a8bd909378b40747de7a32c02630");
    }

    #[test]
    fn commit_with_a_parent_matches_git() {
        let first = Object::Commit(first_commit()).id(&Sha1Crate);
        let second = Commit {
            tree: empty_tree(),
            parents: vec![first],
            author: wiki(),
            committer: Signature {
                name: "Bongo".to_string(),
                email: "bongo@example.com".to_string(),
                time: 1_700_003_600,
                offset_minutes: -330,
            },
            message: "second commit\n".to_string(),
        };

        let id = Object::Commit(second).id(&Sha1Crate);

        assert_eq!(id.to_string(), "f2655e8975fd341852f7554a2249023d2ccfc783");
    }
}
