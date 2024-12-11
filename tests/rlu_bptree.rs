#[cfg(test)]
mod tests {
    use std::sync::Barrier;
    use std::sync::Arc;
    use std::thread;
    use rand::thread_rng;
    use rand::Rng;
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
        bptree.insert(65, 'm');
        assert_eq!(bptree.search(&65), Some('m'));
        bptree.insert(70, 'n');
        assert_eq!(bptree.search(&70), Some('n'));
        bptree.insert(71, 'o');
        assert_eq!(bptree.search(&71), Some('o'));
        // bptree.insert(72, 'p');
        // assert_eq!(bptree.search(&72), Some('p'));

        bptree.debug_print_tree();
        // Vec::new(bptree.range_search(&5, &60));
        // assert_eq!(Vec::from(bptree.range_search(&5, &60)), vec![(5, 'h'), (10, 'a'), (15, 'c'), (17, 'j'), (18, 'k'), (20, 'b'), (25, 'd'), (32, 'l'), (35, 'e'), (40, 'f'), (45, 'g'), (60, 'i')]);
    }

    // #[test]
    // fn bptree_concurrent_inserts() {
    //     let tree = BPlusTree::new();

    //     let writer = || {
    //         let mut tree = tree.clone_ref();
    //         thread::spawn(move || {
    //             let mut rng = thread_rng();
    //             for _ in 0..10 {
    //                 let i = rng.gen_range(10, 50);
    //                 tree.insert(i, i *10);
    //             }
    //         })
    //     };

    //     let writers: Vec<_> = (0..4).map(|_| writer()).collect();

    //     for t in writers {
    //         t.join().unwrap();
    //     }


    //     // // Verify some randon keys were inserted
    //     // // assert_eq!(tree.search(&10), Some(100));
    //     // let mut found_count = 0;
    //     // for i in 10..50 {
    //     //     if tree.search(&i).is_some() {
    //     //         found_count += 1;
    //     //     }
    //     // }

    //     tree.print_tree();  

    //     // assert!(found_count > 0, "No keys were found");

    // }
    // #[test]
    // fn test_simple_thread() {
    //     let mut tree = BPlusTree::new();
        
    //     for i in 0..100 {
    //         tree.insert(i, i);
    //     }

    //     let reader = || {
    //         let tree = tree.clone_ref();
    //         thread::spawn(move || {
    //             let mut rng = thread_rng();
    
    //             for _ in 0..10000 {
    //                 let i = rng.gen_range(0, 500) * 2;
    //                 assert!(set.contains(i));
    //             }
    //         })
    //     };
    
    //     let writer = || {
    //         let set = set.clone_ref();
    //         thread::spawn(move || {
    //             let mut rng = thread_rng();
    
    //             for _ in 0..1000 {
    //                 let i = rng.gen_range(0, 499) * 2 + 1;
    //                 if random() {
    //                     set.insert(i);
    //                 } else {
    //                     set.delete(i);
    //                 }
    //             }
    //         })
    //     };
    
    //     let readers: Vec<_> = (0..16).map(|_| reader()).collect();
    //     let writers: Vec<_> = (0..4).map(|_| writer()).collect();
    
    //     for t in readers {
    //         t.join().unwrap();
    //     }
    
    //     for t in writers {
    //         t.join().unwrap();
    //     }


    // }

}
