


#[cfg(test)]
mod tests {
    use core::panic;
    use rlu::{BPTree, BPTreeNode};

    #[test]
    fn test_bptree_new() {
        let bptree: BPTree<i32, i32> = BPTree::new();
        assert!(bptree.is_leaf());
    }

    #[test]
    fn test_bptree_insert() {
        let mut bptree: BPTree<i32, char> = BPTree::new();
        bptree.insert(10, 'a');
        bptree.insert(20, 'b');
        bptree.insert(30, 'c');

        // check if the root is still a leaf node
        if let BPTreeNode::LeafNode { keys, values, next: _} = *bptree.root {
            assert_eq!(keys, vec![10, 20, 30]);
            assert_eq!(values, vec!['a', 'b', 'c']);
        } else {
            panic!("Root is not a leaf node");
        }

    }


}