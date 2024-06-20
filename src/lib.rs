#![no_std]
#[cfg(feature = "std")]
extern crate std;
// BK Tree for no_std enviroments using Levenshtein for the diff

#[cfg(feature = "read")]
pub use read::Node;
#[cfg(feature = "write")]
pub use write::{write_bktree, Node};

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
pub mod write {
    use super::{CHILDREN_LENGTH, ROOT_WORD};
    use levenshtein::levenshtein;
    use std::{
        boxed::Box, env::var, fmt, format, fs::File, io::Write, path::Path, string::String,
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
            if let Some(node) = self.children[diff].as_mut() {
                node.add(word);
            } else {
                self.children[diff] = Some(Box::new(Node::new(word)));
            }
        }
        pub fn as_string(&self) -> String {
            assert_eq!(ROOT_WORD, self.word);
            let string = format!("static TREE: Node = {:?};", self);
            // ensuring children are refs
            string.replace("Some(", "Some(&")
        }
    }

    pub fn write_bktree<'a>(file_name: &str, word_list: &mut Vec<&'a str>) {
        let mut tree = Node::new(ROOT_WORD); // root node
        let index = word_list
            .iter()
            .position(|x| *x == ROOT_WORD)
            .expect(format!("{} was not found in word_list", ROOT_WORD).as_str());
        word_list.remove(index); // remove root node word
        word_list.dedup();
        word_list.iter().for_each(|w| tree.add(w));

        let mut contents = tree.as_string();

        // write the tree to cargo out's directory
        let file_path = Path::new(&var("OUT_DIR").unwrap()).join(file_name);
        let mut buffer = File::create(file_path).unwrap();
        buffer.write_all(contents.as_bytes()).unwrap();
    }
}

/// read is explicitly for reading the contents of the tree
/// during runtime.
///
/// use embedded_bktree::read::*;
/// include!(concat!(env!("OUT_DIR"), "tree.rs"));
/// let corrections = TREE.corrections("foo");
///
#[allow(unused_attributes)]
#[cfg(feature = "read")]
pub mod read {
    use super::CHILDREN_LENGTH;

    #[derive(Debug, Clone)]
    pub struct Node {
        pub word: &'static str,
        pub children: [Option<&'static Node>; CHILDREN_LENGTH],
    }

    impl Node {}
}
