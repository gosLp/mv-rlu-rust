use core::{fmt, panic};
use std::fmt::write;


const ORDER: usize = 4;

#[derive(Clone)]
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


impl<K: Ord + Clone + fmt::Debug + Ord , V: Clone + fmt::Debug + Ord > BPTree<K, V> {
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

    // pub fn insert(&mut self, key: K, value: V) {
    //     let root = self.root.as_mut();
    //     match root {
    //         BPTreeNode::LeafNode { keys, values, next: _ } => {
    //             // find position to insert the key
    //             let pos = keys.binary_search(&key).unwrap_or_else(|e| e);

    //             // inset key and value at the same position
    //             keys.insert(pos, key);
    //             values.insert(pos, value);

    //             // if overflow 
    //             if keys.len() > ORDER {
    //                 self.split_root();
    //             }

    //         },
    //         _ => {
    //             // handles internal node, we will implement this later
    //             panic!("Insertion into internal nodes not implemented yet");
    //         }
    //     }

    // }
    
    pub fn insert(&mut self, key: K, value: V) {
        if let Some((promoted_key, new_node)) = self.root.insert_internal(key, value) {
            // First store the old root
            let old_root = std::mem::replace(
                &mut self.root,
                Box::new(BPTreeNode::LeafNode { 
                    keys: Vec::new(),
                    values: Vec::new(),
                    next: None,
                })
            );
            // Then create new root with the old root and new node as children
            self.root = Box::new(BPTreeNode::InternalNode {
                keys: vec![promoted_key],
                children: vec![old_root, new_node],
            });
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
    pub fn copy_ref(&self) -> Self {
        Self {
            root: self.root.clone()
        }
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

impl <K:fmt::Debug + Ord + Clone, V: fmt::Debug + Ord + Clone> BPTreeNode<K, V> {
    // In BPTreeNode<K,V> enum impl block:
fn insert_internal(&mut self, key: K, value: V) -> Option<(K, Box<BPTreeNode<K,V>>)> {
    match self {
        BPTreeNode::InternalNode { keys, children } => {
            // Find the correct child to insert into
            let idx = keys.iter().position(|k| key < *k).unwrap_or(keys.len());
            
            // Recursively insert into child
            if let Some((promoted_key, new_node)) = children[idx].insert_internal(key, value) {
                // Handle the promoted key
                keys.insert(idx, promoted_key);
                children.insert(idx + 1, new_node);
                
                // Check if we need to split this node
                if keys.len() > ORDER {
                    // Split this internal node
                    let mid = keys.len() / 2;
                    let promoted = keys[mid].clone();
                    
                    let new_keys = keys.split_off(mid + 1);
                    let new_children = children.split_off(mid + 1);
                    
                    let new_node = Box::new(BPTreeNode::InternalNode {
                        keys: new_keys,
                        children: new_children,
                    });
                    
                    keys.pop(); // Remove the promoted key
                    return Some((promoted, new_node));
                }
            }
            None
        },
        BPTreeNode::LeafNode { keys, values, next } => {
            // Your existing leaf node insert logic
            let pos = keys.binary_search(&key).unwrap_or_else(|e| e);
            keys.insert(pos, key);
            values.insert(pos, value);
            
            if keys.len() > ORDER {
                // Split leaf node
                let mid = keys.len() / 2;
                let new_keys = keys.split_off(mid);
                let new_values = values.split_off(mid);
                let return_key = new_keys[0].clone();
                
                let new_node = Box::new(BPTreeNode::LeafNode {
                    keys: new_keys,
                    values: new_values,
                    next: next.take(),
                });
                
                *next = Some(new_node.clone());
                
                return Some((return_key, new_node));
            }
            None
        }
    }
}
}