use embedded_bktree::write;

fn main() {
    write::write_bktree(
        "tree.rs",
        &mut vec!["what", "why", "whys", "how", "cow", "brown", "the"],
    );
}
