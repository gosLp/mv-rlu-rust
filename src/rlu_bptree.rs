use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use crate::rlu::{RluObj, RluObjHdr,  WsHdr, PTR_ID_OBJ_COPY};
use crate::{rlu_abort, rlu_dereference, rlu_reader_lock, rlu_reader_unlock, rlu_thread_init, GlobalRlu, rlu_try_lock};
use std::alloc::{alloc, Layout};


// lets define the node order for simplicity
const B: usize = 4; // small order for demonstration


pub struct Node<K:Clone , V: Clone> {
    // The RLU header for managing concurreny:
    pub hdr: RluObjHdr<Node<K, V>>,

    pub is_leaf: bool,
    pub num_keys: usize,
    pub keys: [Option<K>; B],

    // In a B+tree leaf node, values are typically stored directly
    // In internal nodes, children pointers are stored
    pub values: [Option<V>; B],

    // For internal nodes, values would be child pointers, for now, lets keep it simple
    // we'll refine this in a leter step.
}


impl<K: Clone + Copy, V: Clone + Copy> Node<K, V> {
    pub fn new(is_leaf: bool) -> Self {
        Node {
            hdr: RluObjHdr {
                p_obj_copy: AtomicPtr::new(ptr::null_mut()),
                ws_hdr: None,
            },
            is_leaf,
            num_keys: 0,
            keys: [None; B],
            values: [None; B],
        }
    }
}

impl<K: Clone, V: Clone> RluObj for Node<K, V> {
    fn get_p_obj_copy(&self) -> *mut Self {
        self.hdr.p_obj_copy.load(Ordering::SeqCst)
    }

    fn is_locked(&self) -> bool {
        // if p_obj_copy is not null and not equal to PTR_ID_OBJ_COPY, then it is locked
        let p = self.hdr.p_obj_copy.load(Ordering::SeqCst);
        !p.is_null() && p != PTR_ID_OBJ_COPY as *mut Self
    }

    fn is_copy(&self) -> bool {
        self.hdr.ws_hdr.is_some()
    }

    fn has_ws_hdr(&self) -> bool {
        self.hdr.ws_hdr.is_some()
    }

    fn get_p_original(&self) -> *mut Self {
        // If this is a copy, we should have a ws_hdr. If not, return self pointer.
        if let Some(ws) = &self.hdr.ws_hdr {
            ws.p_obj_actual
        } else {
            self as *const Self as *mut Self
        }
    }

    fn get_locking_thread_from_ws_obj(&self) -> usize {
        if let Some(ws) = &self.hdr.ws_hdr {
            ws.thread_id
        } else {
            // if no ws_hdr, it's not locked, return invalid
            usize::MAX
        }
    }

    fn get_ws_run_counter(&self) -> u64 {
        if let Some(ws) = &self.hdr.ws_hdr {
            ws.run_counter
        } else {
            0
        }
    }

    fn get_copy_with_ws_hdr(&self, run_counter: u64, thread_id: usize) -> Self {
        // Creat e a copy of the node:
        let mut copy = Node {
            hdr: RluObjHdr {
                p_obj_copy: AtomicPtr::new(ptr::null_mut()),
                ws_hdr: Some(WsHdr {
                    p_obj_actual: self.get_p_original(),
                    run_counter,
                    thread_id,
                }),
            },
            is_leaf: self.is_leaf,
            num_keys: self.num_keys,
            keys: self.keys.clone(),
            values: self.values.clone(),
        };

        // Mar the copy's p_obj_copy to point as PTR_ID_OBJ_COPy to indicate it is a copy
        copy.hdr
            .p_obj_copy
            .store(PTR_ID_OBJ_COPY as *mut Self, Ordering::SeqCst);
        copy
    }

    fn cas(&self, new_obj: *mut Self) -> bool {
        // Compre-and-swap on p_pbj_copy. We expect it to be nul if unlocked, and set it to newoj.
        let expected = ptr::null_mut();
        self.hdr
            .p_obj_copy
            .compare_exchange(
                expected, 
                new_obj, 
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok() 
    }

    fn copy_back_to_original(&self) {
        // In a full implementation, we would copy all fields from this ( a ws copy) tk the original
        // self is the copy, so its original is get_p_original()
        // Safety: we know self is copy, so ws_hdr is SOme and p_obj_actual is valid.
        let orig = self.get_p_original();
        unsafe {
            (*orig).is_leaf = self.is_leaf;
            (*orig).num_keys = self.num_keys;
            for i in 0..B {
                (*orig).keys[i] = self.keys[i].clone();
                (*orig).values[i] = self.values[i].clone();
            }
        }
    }

    fn unlock_original(&self) {
        let orig = self.get_p_original();
        unsafe {
            (*orig)
                .hdr
                .p_obj_copy
                .store(PTR_ID_OBJ_COPY as *mut Self, Ordering::SeqCst);
        }
    }

    fn unlock(&self) {
        // Just unlock this node, if its locked. if its a copy we set original's p_obj_copy to PTR_ID_OBJ_COPY
        // Actually, unlcok is typically only called on the orignal or somethin  or something that was locked
        // Acoording to this RLU logic, unlocking happens via unlock_original().
        self.unlock_original();
    }
}


pub struct BPlusTree<K: Clone, V: Clone> {
    rlu: *mut GlobalRlu<Node<K, V>>,
    id: usize,
    root: *mut Node<K, V>,
}

impl<K: Ord + Clone + Copy, V: Ord + Clone + Copy> BPlusTree<K, V> {
    pub fn new() -> Self {
        // Initialise globa RLU
        let rlu = GlobalRlu::<Node<K,V>>::init_rlu();
        let id = rlu_thread_init(rlu);

        // for a brand new tree, creeate a single leaf node as root
        // We'll allocate it on the heap:
        let root_box = Box::new(Node::new(true));
        let root_ptr = Box::into_raw(root_box);

        BPlusTree {
            rlu,
            id,
            root: root_ptr,
        }
    }

    // Basi search method
    // FOr now, we assume the three has only a root node ( no splits) and keys are only in root.
    // later, we'lll traverse the treee properly,
    pub fn search(&self, key: &K) -> Option<V> {
        // Aquire reader lock 
        unsafe {
            rlu_reader_lock(self.rlu, self.id);
            // In a full B+tree, we'd descend through internal nodes to find the leaf node
            // For now, just read the root node directly
            let root = rlu_dereference(self.rlu, self.id, self.root);
            if root.is_null() {
                rlu_reader_unlock(self.rlu, self.id);
                return None;
            }

            let root_ref = &*root;

            for i in 0..root_ref.num_keys {
                if let Some(ref k) = root_ref.keys[i] {
                    match key.cmp(k) {
                        std::cmp::Ordering::Equal => {
                            // Found the key
                            let val = root_ref.values[i].clone();
                            rlu_reader_unlock(self.rlu, self.id);
                            return val;
                        }
                        std::cmp::Ordering::Less =>{
                            // Since keys are sorted, if we passsed the position where the key would be, 
                            // it not in the tree
                            break;
                        }
                        std::cmp::Ordering::Greater => continue,
                    } 
                }
            }
            rlu_reader_unlock(self.rlu, self.id);
            None
        }
    }
    /// Insert operation (just a placeholder for now)
    /// In the future, we will:
    /// - Acquire reader lock
    /// - Find leaf node
    /// - Upgrade to writer lock using rlu_try_lock for that node
    /// - Insert the key, possibly splitting if full, locking additional nodes as needed
    /// - Commit or abort the transaction
    
    pub fn insert(&self, key:K, value:V) {
        unsafe  {
            rlu_reader_lock(self.rlu, self.id);

            let root = rlu_dereference(self.rlu, self.id, self.root);
            if root.is_null() {
                // Tree is empty, we need to create root node
                // We'll do something simplistic: The root node is already created in new().
                // If it’s empty, we just add directly (need to lock the root)

                rlu_reader_unlock(self.rlu, self.id);
                return;
            }

            // Try to uograde lock on root
            let mut p_root = root;
            if !rlu_try_lock(self.rlu, self.id, &mut p_root) {
                // Could not lock, abort and retry or handle concureny
                rlu_abort(self.rlu, self.id);
                return;
            }

            let root_mut = &mut *p_root;
            // Insert key if there's space
            if root_mut.num_keys < root_mut.keys.len() {
                // simple insertion in sorted order
                let pos= root_mut.keys.iter().position(|k_opt| {
                    k_opt.as_ref().map_or(true, |k| k > &key)
                }).unwrap_or(root_mut.num_keys);

                // Shift keys/values to make room
                for i in (pos..root_mut.num_keys).rev() {
                    root_mut.keys[i+1] = root_mut.keys[i].take();
                    root_mut.values[i+1] = root_mut.values[i].take();
                }
                root_mut.keys[pos] = Some(key);
                root_mut.values[pos] = Some(value);
                root_mut.num_keys += 1;
            } else {
                // If full, we would need to handle splitting.
                // This is where the complexity of a  B+tree comes in.
                // For now, we won't implement splitting
            }
            
            //unlock and commit
            rlu_reader_unlock(self.rlu, self.id);


        }
    }
}

