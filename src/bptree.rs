use core::fmt;
use std::fmt::write;

const ORDER: usize = 4;

pub enum BPTreeNode<K, V> {
    InternalNode {
        keys: Vec<K>,
        children: Vec<Box<BPTreeNode<K,V>>>,
    },
    LeafNode {
        keys: Vec<K>,
        values: Vec<V>,
        next: Option<Box<BPTreeNode<K,V>>> // Linked list to BPTreeLeafNode
    }
}

pub struct BPTree<K, V> {
    pub root: Box<BPTreeNode<K, V>>,
}


impl<K: Ord + Clone , V: Clone > BPTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: Box::new(BPTreeNode::LeafNode 
                { 
                    keys: Vec::new(),
                    values: Vec::new(),
                    next: None,
                }
            )
        }
    }
    pub fn is_leaf(&self) -> bool {
        matches!(*self.root, BPTreeNode::LeafNode { .. })
    }

    pub fn insert(&mut self, key: K, value: V) {
        let root = self.root.as_mut();
        match root {
            BPTreeNode::LeafNode { keys, values, next: _ } => {
                // find position to insert the key
                let pos = keys.binary_search(&key).unwrap_or_else(|e| e);

                // inset key and value at the same position
                keys.insert(pos, key);
                values.insert(pos, value);

                // if overflow 
                if keys.len() > ORDER {
                    self.split_root();
                }

            },
            _ => {
                // handles internal node, we will implement this later
                panic!("Insertion into internal nodes not implemented yet");
            }
        }

    }

    pub fn split_root(&mut self){
        if let BPTreeNode::LeafNode { keys, values, next} = self.root.as_mut() {
            let mid = keys.len() / 2;

            // create a new node and move half of the keys and values to it
            let right_node = Box::new(
                BPTreeNode::LeafNode { 
                    keys: keys.split_off(mid),
                    values: values.split_off(mid),
                    next: next.take(),
                }
            );

            // promote the middle key to the parent 

            let middle_key = keys[mid -1 ].clone();

            self.root = Box::new(
                BPTreeNode::InternalNode { 
                    keys: vec![middle_key],
                    children: vec![
                        Box::new(
                            BPTreeNode::LeafNode { 
                                keys: keys.clone(),
                                values: values.clone(),
                                next: Some(right_node),
                            }
                        )
                    ],
                }
            );
        }
    }

    pub fn search(&self, key: &K) -> Option<V> {
        let mut current_node = &*self.root;

        loop {
            match current_node {
                BPTreeNode::InternalNode { keys, children } => {
                    let idx = keys.iter().position(| k | key < k ).unwrap_or(keys.len());
                    current_node = &children[idx];
                }
                BPTreeNode::LeafNode { keys, values, next: _} => {
                    
                    if let Ok(idx) = keys.binary_search(key) {
                        return Some(values[idx].clone());
                    } else {
                        return None;
                    }
                }
            }
        }
    }

    pub fn range_query(&self, start_key: &K, end_key: &K) -> Vec<(K,V)> {
        let mut result = Vec::new();
        let mut current_node = &*self.root;


        // Step 1
        while let BPTreeNode::InternalNode { keys, children } = current_node {
            let idx = keys.iter().position(|k| start_key < k).unwrap_or(keys.len());
            current_node = &*children[idx];
        }

        // Collect the keys and values from the lead nodes
        while let BPTreeNode::LeafNode { keys, values, next } = current_node {
            for (i, key) in keys.iter().enumerate() {
                if key >= start_key && key <= end_key {
                    result.push((key.clone(), values[i].clone()));
                }
            }
            if let Some(next_node) = next {
                current_node = &*next_node;
            } else {
                break; // No more leaf nodes
            }
        }

        result
    }
}


impl <K: fmt::Debug, V: fmt::Debug> fmt::Debug for BPTreeNode<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BPTreeNode::InternalNode { keys, children } => write!(
                f, 
                "InternalNode {{ keys: {:?}, children: {:?} }}",
                keys, children
            ),
            BPTreeNode::LeafNode { keys, values, next } => write!(
                f, 
                "LeafNode {{ keys: {:?}, values: {:?}, next: {:?} }}",
                keys, values, next
            ),
        }
    }
}


impl <K: fmt::Debug, V: fmt::Debug> fmt::Debug for BPTree<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BPlusTree {{ root: {:?} }}", self.root)
    }
}