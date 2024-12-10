#[cfg(test)]
mod tests {
    use std::sync::Barrier;
    use std::sync::Arc;
    use std::thread;
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
        assert_eq!(bptree.search(&40), Some('f'));
        assert_eq!(bptree.search(&25), Some('d'));
        bptree.insert(45, 'g');
        assert_eq!(bptree.search(&45), Some('g'));
        bptree.insert(5, 'h');
        assert_eq!(bptree.search(&5), Some('h'));
        bptree.insert(60, 'i');
        assert_eq!(bptree.search(&60), Some('i'));
        // bptree.insert(25, 'j');
        assert_eq!(bptree.search(&25), Some('d'));
        assert_eq!(bptree.search(&5), Some('h'));
        bptree.insert(17, 'j');
        assert_eq!(bptree.search(&17), Some('j'));
        bptree.insert(18, 'k');
        assert_eq!(bptree.search(&18), Some('k'));
        bptree.insert(32, 'l');
        assert_eq!(bptree.search(&32), Some('l'));

        bptree.print_tree();
        // Vec::new(bptree.range_search(&5, &60));
        assert_eq!(Vec::from(bptree.range_search(&5, &60)), vec![(5, 'h'), (10, 'a'), (15, 'c'), (17, 'j'), (18, 'k'), (20, 'b'), (25, 'd'), (32, 'l'), (35, 'e'), (40, 'f'), (45, 'g'), (60, 'i')]);
    }



}
