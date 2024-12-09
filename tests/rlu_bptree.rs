#[cfg(test)]
mod tests {
    use rlu::BPlusTree;

    #[test]
    fn test_rlu_bplus_tree() {
        // Create a new BPlusTree
        let mut bptree = BPlusTree::<i32, char>::new();

        // Insert some key-value pairs
        bptree.insert(10, 'a');
        bptree.insert(20, 'b');
        bptree.insert(15, 'c');

        // Search for existing keys
        assert_eq!(bptree.search(&10), Some('a'));
        assert_eq!(bptree.search(&20), Some('b'));
        assert_eq!(bptree.search(&15), Some('c'));

        // // Search for a non-existing key
        assert_eq!(bptree.search(&30), None);

        // // Insert another key
        bptree.insert(25, 'd');

        // // Verify it was inserted
        assert_eq!(bptree.search(&25), Some('d'));

        // // Ensure previously inserted keys are still accessible
        assert_eq!(bptree.search(&10), Some('a'));
        assert_eq!(bptree.search(&20), Some('b'));

        bptree.insert(35, 'e');
        assert_eq!(bptree.search(&35), Some('e')); 

        bptree.insert(40, 'f');
        assert_eq!(bptree.search(&35), Some('e')); 
    }
}
