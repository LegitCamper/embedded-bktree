#![no_std]
#[cfg(feature = "std")]
extern crate std;
// BK Tree for no_std enviroments using Levenshtein for the diff

#[cfg(feature = "read")]
pub use read::{Node, NodeIntoIterator};
#[cfg(feature = "write")]
pub use write::write_bktree;

// this is the lenght of the children array in Node
// corresponds to the number of top level words with a diff
// equal to or lower than the root/parent node
const CHILDREN_LENGTH: usize = 15;

const ROOT_WORD: &str = "the";

/// write is explicitly for creating the bktree during compile time
/// it is intended to be used in your build.rs file:
///
///
/// Ensure word list is sorted by most frequently used words
/// to ensure lookup speeds are fast
///
#[cfg(feature = "write")]
mod write {
    use super::{CHILDREN_LENGTH, ROOT_WORD};
    use levenshtein::levenshtein;
    use std::{
        boxed::Box,
        env::var,
        fmt, format,
        fs::File,
        io::Write,
        path::{Path, PathBuf},
        string::String,
        vec::Vec,
    };

    #[derive(Debug, Clone)]
    pub struct Node<'a> {
        pub word: &'a str,
        pub children: [Option<Box<Node<'a>>>; CHILDREN_LENGTH],
    }

    impl<'a> Node<'a> {
        fn new(word: &'a str) -> Self {
            Self {
                word,
                children: [const { None }; CHILDREN_LENGTH],
            }
        }
        fn add(&mut self, word: &'a str) {
            let diff = levenshtein(self.word, word);
            if diff < CHILDREN_LENGTH {
                if let Some(node) = self.children[diff].as_mut() {
                    node.add(word);
                } else {
                    self.children[diff] = Some(Box::new(Node::new(word)));
                }
            }
        }
        pub fn as_string(&self) -> String {
            assert_eq!(ROOT_WORD, self.word);
            let string = format!("static TREE: Node = {:?};", self);
            // ensuring children are refs
            string.replace("Some(", "Some(&")
        }
    }

    /// Write word list to bk tree file
    /// You can specify a specific path, otherwise 'OUT_DIR' is used.
    /// the default file name is tree.rs -
    /// #example:
    /// ```
    /// // build.rs file
    /// // include!(concat!(env!("OUT_DIR"), "/tree.rs"));
    /// ```
    pub fn write_bktree<'a>(file_path: Option<PathBuf>, word_list: &mut Vec<&'a str>) {
        let mut tree = Node::new(ROOT_WORD); // root node
        let index = word_list
            .iter()
            .position(|x| *x == ROOT_WORD)
            .expect(format!("{} was not found in word_list", ROOT_WORD).as_str());
        word_list.remove(index); // remove root node word
        word_list.dedup();
        word_list.iter().for_each(|w| tree.add(w));

        // write the tree to cargo out's directory
        let mut buffer = File::create(match file_path {
            Some(path) => path,
            None => Path::new(&var("OUT_DIR").unwrap()).join("tree.rs"),
        })
        .unwrap();
        buffer.write_all(tree.as_string().as_bytes()).unwrap();
    }
}

/// read is explicitly for reading the contents of the tree
/// during runtime.
///
/// use embedded_bktree::read::*;
/// include!(concat!(env!("OUT_DIR"), "tree.rs"));
/// let corrections = TREE.corrections("foo");
///
#[cfg(feature = "read")]
mod read {
    use super::CHILDREN_LENGTH;

    use std::{vec, vec::Vec};

    #[derive(Debug, Clone)]
    pub struct Node {
        pub word: &'static str,
        pub children: [Option<&'static Node>; CHILDREN_LENGTH],
    }

    pub struct NodeIntoIterator {
        stack: Vec<(u8, &'static Node)>,
        first: bool,
    }
    impl NodeIntoIterator {
        pub fn new(node: &'static Node) -> Self {
            let stack = vec![(0, node)];
            Self { stack, first: true }
        }
    }
    impl Iterator for NodeIntoIterator {
        type Item = &'static Node;

        fn next(&mut self) -> Option<Self::Item> {
            if self.first {
                self.first = false;
                return Some(self.stack.first().unwrap().1);
            }
            loop {
                for (i, node) in self
                    .stack
                    .last()
                    .unwrap()
                    .1
                    .children
                    .iter()
                    .filter(|n| n.is_some())
                    .skip(self.stack.last().unwrap().0 as usize)
                    .enumerate()
                {
                    if let Some(node) = node {
                        self.stack.push((i as u8, node));
                        return Some(node);
                    }
                }

                // made it through children and are back up to root
                self.stack.pop();

                match self.stack.pop() {
                    Some(last) => self.stack.push((last.0 + 1, last.1)),
                    None => return None,
                }
            }
        }
    }
}

#[cfg(feature = "test")]
#[cfg(test)]
mod test {
    use super::{write, Node, NodeIntoIterator};
    use std::{path::Path, println, vec};

    include!("../tree.rs");

    #[test]
    fn write_bktree() {
        let path = Path::new(".").join("tree.rs");
        let word_list = &mut vec!["the", "them", "he", "car", "care", "card", "cake"];
        write::write_bktree(Some(path), word_list);
    }

    fn iter_tree() -> NodeIntoIterator {
        NodeIntoIterator::new(&TREE)
    }

    #[test]
    fn print() {
        for node in iter_tree() {
            println!("{}", node.word);
        }
    }
}
