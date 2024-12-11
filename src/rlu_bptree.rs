use std::fmt::Debug;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use crate::rlu::{RluObj, RluObjHdr,  WsHdr, PTR_ID_OBJ_COPY, RLU_MAX_THREADS};
use crate::{rlu_abort, rlu_dereference, rlu_reader_lock, rlu_reader_unlock, rlu_thread_init, GlobalRlu, rlu_try_lock};
use std::alloc::{alloc, Layout};


// lets define the node order for simplicity
const B: usize = 4; // small order for demonstration

#[derive(Debug)]
pub struct Node<K:Clone , V: Clone> {
    // The RLU header for managing concurreny:
    pub hdr: RluObjHdr<Node<K, V>>,

    pub is_leaf: bool,
    pub num_keys: usize,
    // keys arre always sorted in B+Tree
    pub keys: [Option<K>; B],

    // For leaf nodes: `vals` holds values associated with keys.
    // For internal nodes: `vals` holds pointers to child nodes.
    // We'll store them as a union or a single array of *mut Node<K,V>, but we need a safe abstraction.
    // For simplicity, we’ll store them as `children` for internal nodes and `values` for leaves.
    // We'll interpret these arrays based on `is_leaf`.

    //Child pointer for internal nodes
    pub children: [*mut Node<K, V>; B + 1],

    // In a B+tree leaf node, values are typically stored directly
    // In internal nodes, children pointers are stored
    pub values: [Option<V>; B],

    // For internal nodes, values would be child pointers, for now, lets keep it simple
    // we'll refine this in a leter step.

    // If leaf, a pointer to the next leaf node (for range queries):
    pub next_leaf: *mut Node<K, V>,

    // parent pointr
    pub parent: *mut Node<K, V>,
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
            children:[ptr::null_mut(); B+1],
            values: [None; B],
            next_leaf: ptr::null_mut(),
            parent: ptr::null_mut(),

        }
    }
    pub fn set_parent(&mut self, parent: *mut Node<K, V>) {
        self.parent = parent;
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
            if ws.thread_id >= RLU_MAX_THREADS {
                // Invalid thread ID detected
                dbg!("Invalid thread ID in ws_hdr", ws.thread_id);
                return RLU_MAX_THREADS;  // Return invalid ID
            }
            ws.thread_id
        } else {
            // if no ws_hdr, it's not locked, return invalid
            RLU_MAX_THREADS  // Return invalid ID for unlocked objects
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
        // Create a copy of the node:
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
            children: self.children,
            values: self.values.clone(),
            next_leaf: self.next_leaf,
            parent: self.parent,
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
        dbg!("copy_back_to_original called");
        // In a full implementation, we would copy all fields from this ( a ws copy) tk the original
        // self is the copy, so its original is get_p_original()
        // Safety: we know self is copy, so ws_hdr is SOme and p_obj_actual is valid.
        let orig = self.get_p_original();
        unsafe {
            (*orig).is_leaf = self.is_leaf;
            (*orig).num_keys = self.num_keys;
            for i in 0..B {
                (*orig).keys[i] = self.keys[i].clone();
                assert!((*orig).num_keys <= B, "num_keys out of range after copy_back");
                (*orig).values[i] = self.values[i].clone();
            }
            for i in 0..=B {
                (*orig).children[i] = self.children[i];
            }
            (*orig).parent = self.parent;
            (*orig).next_leaf = self.next_leaf;
            // Clear the ws_hdr so this node is no longer considered a copy
            (*orig).hdr.ws_hdr = None;
            // IMPORTANT: unlock the node after copy-back
            
            (*orig).hdr.p_obj_copy.store(ptr::null_mut(), Ordering::SeqCst);
        }
        dbg!("copy_back_to_original completed");
    }

    fn unlock_original(&self) {
        // dbg!("unlock_original called");
        // let orig = self.get_p_original();
        // unsafe {
        //     (*orig)
        //         .hdr
        //         .p_obj_copy
        //         .store(ptr::null_mut(), Ordering::SeqCst);
        // }
        // dbg!("unlock_original completed");
    }

    fn unlock(&self) {
        // Just unlock this node, if its locked. if its a copy we set original's p_obj_copy to PTR_ID_OBJ_COPY
        // Actually, unlcok is typically only called on the orignal or somethin  or something that was locked
        // Acoording to this RLU logic, unlocking happens via unlock_original().
        // self.unlock_original();
    }
}

#[derive(Debug)]
pub struct BPlusTree<K: Clone, V: Clone> {
    rlu: *mut GlobalRlu<Node<K, V>>,
    id: usize,
    root: *mut Node<K, V>,
}

unsafe impl<K: Clone, V: Clone> Send for BPlusTree<K, V> {}
unsafe impl<K: Clone, V: Clone > Sync for BPlusTree<K, V> {}

impl<K: Ord + Clone + Copy + Debug + Unpin, V: Ord + Clone + Copy + Debug + Unpin> BPlusTree<K, V> {
    pub fn clone_ref(&self) -> Self {
        let thread_id = rlu_thread_init(self.rlu);
        BPlusTree {
            rlu: self.rlu,
            root: self.root,
            id: thread_id,
        }
    }
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
            // acrue reader lock for duration of search
            rlu_reader_lock(self.rlu, self.id);

            // If tree is empty
            let mut node_ptr = rlu_dereference(self.rlu, self.id, self.root);
            dbg!("search called for key {:?}", &key);
            dbg!("search called and root is {:?}", &*node_ptr);
            if node_ptr.is_null() {
                // othing in tree
                rlu_reader_unlock(self.rlu, self.id);
                return None;
            }

            loop {
                let node = &*node_ptr;

                if node.num_keys == 0 {
                    // Empty node
                    rlu_reader_unlock(self.rlu, self.id);
                    return None;
                }

                if node.is_leaf {
                    // we've reached a lead, search linearly for noww do binary search later
                    for i in 0..node.num_keys {
                        if let Some(ref k) =node.keys[i] {
                            match key.cmp(k) {
                                std::cmp::Ordering::Equal => {
                                    // FOund the key
                                    let val = node.values[i].clone();
                                    rlu_reader_unlock(self.rlu, self.id);
                                    return val;
                                }
                                std::cmp::Ordering::Less => {
                                    // Since keys are sorted, once we find a key> search key,
                                    // We won't find the target key futrther.
                                    break;
                                }
                                std::cmp::Ordering::Greater => continue,
                            }
                        } else {
                            // No key at i, should not happen if num_keys is correct, but just in case
                            break;
                        }
                    }

                    // key not found in this leaf
                    rlu_reader_unlock(self.rlu, self.id);
                    return None;
                } else {
                    // Internal Node, find the correct child to descend into

                    // the logic : 
                    // if key < keys[0], child = children[0]
                    // If keys[i-1] <= key < keys[i], child = children[i]
                    // If key >= keys[num_keys - 1], child = children[num_keys]

                    let mut child_idx = node.num_keys; // default to last child

                    for i in 0..node.num_keys {
                        if let Some(ref k) = node.keys[i] {
                            if key < k {
                                child_idx = i;
                                break;
                            }
                        } else {
                            // unexpected missing key, just descent into i
                            child_idx = i;
                            break;
                        }
                    }
                    dbg!("Descending into child index", child_idx);
                    let child_ptr = node.children[child_idx];
                    if child_ptr.is_null() {
                        // No child? tree might be in an incomplete state or something went wrong
                        rlu_reader_unlock(self.rlu, self.id);
                        return None;
                    }

                    // Move down to child
                    node_ptr = rlu_dereference(self.rlu, self.id, child_ptr);
                    dbg!("dereferenced child node", &*node_ptr);
                    if node_ptr.is_null() {
                        // Child pointer incalid 
                        rlu_reader_unlock(self.rlu, self.id);
                        return None;
                    }
                    // Loop continues untill we reach a leaf or fail
                }
            }
            // // In a full B+tree, we'd descend through internal nodes to find the leaf node
            // // For now, just read the root node directly
        }
    }
    
    
    
    /// Insert operation (just a placeholder for now)
    /// In the future, we will:
    /// - Acquire reader lock
    /// - Find leaf node
    /// - Upgrade to writer lock using rlu_try_lock for that node
    /// - Insert the key, possibly splitting if full, locking additional nodes as needed
    /// - Commit or abort the transaction
    
    pub fn insert(&mut self, key:K, value:V) {
        unsafe  {
            rlu_reader_lock(self.rlu, self.id);
            

            // First descent down to the appropriate leaf node
            let leaf_ptr = self.find_leaf_for_key(&key);
            if leaf_ptr.is_null() {
                // Tree might be empty, create irst leaf node as root
                // Let's lock the root (whicj mightbe null) - in a real scenario
                // we’d have logic to handle empty tree carefully.
                rlu_reader_unlock(self.rlu, self.id);
                self.insert_empty_tree_case(key, value);
                return;
            }

            // Now we have a leaf node pointer:
            // Let's try to lock it as a writer
            let mut p_leaf = leaf_ptr;
            dbg!("Attempting to lock leaf node for insertion");
            if !rlu_try_lock(self.rlu, self.id, &mut p_leaf)  {
                // Could not lock, abort this attempt and rety
                dbg!("Locking leaf failedd, abort and retry");
                rlu_abort(self.rlu, self.id);
                // self.insert(key, value); // naive retry, ideally handle more gracefully
                return;
            }

            let leaf_ref = &mut *p_leaf;

            //Insert ey into leaf. If there's more room, just insert
            assert!(leaf_ref.num_keys <= B, "num_keys out of range before insertion");
            if leaf_ref.num_keys < B {
                dbg!(" Leaf has space, inserting key direectly");
                self.insert_into_leaf(leaf_ref, key, value);
                // Reader unlock will commit changes
                dbg!("Inserted key into leaf, unlocking reader (commit)");
                rlu_reader_unlock(self.rlu, self.id);
                return;
            } else {
                // Need to split leaf
                dbg!("Leaf is full, splitting");
                let (new_leaf_ptr, split_key) = self.split_leaf(leaf_ref, key, value);
                dbg!(" new leaf and split key are", &*new_leaf_ptr, &split_key);
                // In insert_into_parent after creating new root:

                // WE just created a new leaf and got a split_key to push up

                // Unlock leaf and commit to ensure leaf changes are visible before dealing with parent
                // Note: In traditional RLU usage, you might choose to do all changes in one atomic RLU write transaction,
                // but let's keep it simple and commit step by step.
                rlu_reader_unlock(self.rlu, self.id);

                // Now we must insert `split_key` into the parent. This means we must:
                // 1) Re-lock as a reader
                // 2) Find the parent of this leaf (for now, assume we store parent pointers or handle a root special case)
                // 3) Lock parent and insert the split_key as a separator, possibly splitting again if needed.
                rlu_reader_lock(self.rlu, self.id);

                // // Instead of using leaf_ptr directly, find the leaf again:
                let leaf_ptr_deref = self.find_leaf_for_key(&split_key);
                if leaf_ptr_deref.is_null() {
                    dbg!("could not find leaf after commit");
                    rlu_reader_unlock(self.rlu, self.id);
                    return;
                }

                // // Now re-derive new_leaf_ptr:
                let new_leaf_ptr_deref = rlu_dereference(self.rlu, self.id, new_leaf_ptr);
                self.insert_into_parent(leaf_ptr_deref, new_leaf_ptr_deref, split_key);
                rlu_reader_unlock(self.rlu, self.id);
                return;

            }
        }
    }


    // Find the lead node where the key should be inserted.
    // SImilar to searhc, but we stop when we find a leaf
    unsafe fn find_leaf_for_key(&self, key: &K) -> *mut Node<K, V> {
        
        let mut node_ptr = rlu_dereference(self.rlu, self.id, self.root);

        while !node_ptr.is_null() {
            let node = &*node_ptr;
            if node.is_leaf {
                return node_ptr;
            } else {
                // Internalnode : find child
                let mut child_idx = node.num_keys;
                for i in 0..node.num_keys {
                    if let Some(ref k) = node.keys[i] {
                        if key < k {
                            child_idx = i;
                            break;
                        }

                    } else {
                        child_idx = i;
                        break;
                    }

                }

                let child_ptr = node.children[child_idx];
                if child_ptr.is_null() {
                    return  ptr::null_mut();
                }
                node_ptr = rlu_dereference(self.rlu, self.id, child_ptr);
            }
        }
        ptr::null_mut()
    }

    unsafe fn insert_into_leaf(&self, leaf: &mut Node<K,V>, key: K, value: V) {
        dbg!("insert_into_leaf called", &key, &value);
        // insert key in sorted orer
        let pos = leaf.keys.iter()
                                    .take(leaf.num_keys)
                                    .position(|k_opt| k_opt.as_ref().map_or(true, |k|  k > &key))
                                    .unwrap_or(leaf.num_keys);
        
        // SHift to make room
        for i in (pos..leaf.num_keys).rev(){
            leaf.keys[i+1] = leaf.keys[i].take();
            leaf.values[i+1] = leaf.values[i].take();
        }

        leaf.keys[pos] = Some(key);
        leaf.values[pos] = Some(value);
        leaf.num_keys += 1;
        assert!(leaf.num_keys <= B, "num_keys exceeded B after inserting into leaf");
        dbg!("Inserted keys and values are:");
        dbg!("number of keys are {:?}", leaf.num_keys);
        // dbg!("the root of this leaf is {:?}", );
        dbg!("the root looks like {:?}", &self.root);
        dbg!(&leaf.keys[..leaf.num_keys], &leaf.values[..leaf.num_keys]);

    }

    unsafe fn split_leaf(&self, leaf: &mut Node<K, V>, key: K, value: V) ->  (*mut Node<K, V>, K) {
        // temporary array to hold all keys+values including the new one

        let mut temp_keys = Vec::with_capacity(B+1);
        let mut temp_values = Vec::with_capacity(B+1);

        for i in 0..leaf.num_keys {
            temp_keys.push(leaf.keys[i].take().unwrap());
            temp_values.push(leaf.values[i].take().unwrap());
        }

        // Insert new key/ value into temp arrays
        let pos = temp_keys.iter()
                            .position(| k| k > &key )
                            .unwrap_or(temp_keys.len());
        temp_keys.insert(pos, key);
        temp_values.insert(pos, value);

        // SPlit into two halves
        let split = (B+1)/2;

        // // Clear the original leaf first
        // for i in 0..B {
        //     leaf.keys[i] = None;
        //     leaf.values[i] = None;
        // }

        // left half goes back into leaf
        for i in 0..split {
            leaf.keys[i] = Some(temp_keys[i].clone());
            leaf.values[i] = Some(temp_values[i].clone());
        }

        leaf.num_keys = split;
        for i in split..B {
            leaf.keys[i] = None;
            leaf.values[i] = None;
        }

        
        // keep track of old next_leaf
        let old_next_leaf = leaf.next_leaf;

        // Right half goes into new leaf
        let new_leaf_box = Box::new(Node::new(true));
        let new_leaf_ptr = Box::into_raw(new_leaf_box);

        // setup new leaf properties
        // lock the new leaf
        let mut p_new_leaf = new_leaf_ptr;
        if !rlu_try_lock(self.rlu, self.id, &mut p_new_leaf) {
            rlu_abort(self.rlu, self.id);
            return (ptr::null_mut(), key);
        }

        let new_leaf = &mut *p_new_leaf;
        let mut j =0;

        for i in split..(B+1) {
            new_leaf.keys[j] = Some(temp_keys[i].clone());
            new_leaf.values[j] = Some(temp_values[i].clone());
            j += 1;
        }
        // new_leaf.num_keys = j;

        // assert!(leaf.num_keys <= B, "Left leaf too many keys after split");
        // assert!(new_leaf.num_keys <= B, "Right leaf too many keys after split");


        new_leaf.num_keys = (B+1) - split;
        // fix leaf links
        new_leaf.next_leaf = old_next_leaf;
        // new_leaf.parent = leaf.parent;
        // (*new_leaf_ptr).parent = leaf.parent;
        // print!("Leaf split done. Old leaf keys: {:?}, new leaf keys: {:?}", 
        // &leaf.keys[..leaf.num_keys], 
        // &(*new_leaf_ptr).keys[..(*new_leaf_ptr).num_keys]);
        
        // leaf.next_leaf = p_new_leaf;
        leaf.next_leaf = new_leaf_ptr;
        // Important: Keep parent pointer of new leaf temporarily null
        // It will be set properly in insert_into_parent
        new_leaf.parent = ptr::null_mut();

        // The split key for the parent is the first key of the new leaf
        let split_key = new_leaf.keys[0].as_ref().unwrap().clone();
        dbg!("look at the new right leaf", &*p_new_leaf);

        (new_leaf_ptr, split_key)
        // (p_new_leaf, split_key)
    }

    // unsafe fn insert_into_parent(&self, old_node: *mut Node<K, V>, new_node: *mut Node<K, V>, key: K) {
    //     // todo!("insert_into_parent");
    //     // This is complex for now let's ust put a placeholder
    //     // A real implementation would:
    //     // 1) If old_node is root, create a new root and assign old_node and new_node as children.
    //     // 2) Otherwise, find parent node (we need a way to store parent pointers or do root splitting),
    //     //    lock the parent, insert key and pointer to new_node, and if parent is full, split again.

    //     dbg!("insert_into_parent called - not yet implemented");
    //     // For now, let's assume we are dealing with a single-level tree (no internal nodes).
    //     // Proper implementation will come later.
    // }

    unsafe fn insert_empty_tree_case(&mut self, key: K, value: V) {
        // CReate a new leaf node as root

        rlu_reader_lock(self.rlu, self.id);
        let root_box = Box::new(Node::new(true));
        let root_ptr = Box::into_raw(root_box);
        // lock root (which doesn't exist yet, so we jsut assign)
        self.root = root_ptr;

        let mut p_root = root_ptr;
        if !rlu_try_lock(self.rlu, self.id, &mut p_root) {
            dbg!("COuld not lock new root");
            rlu_abort(self.rlu, self.id);
            return;
        }
        let root_ref = &mut *p_root;
        root_ref.keys[0] = Some(key);
        root_ref.values[0] = Some(value);
        root_ref.num_keys = 1;
        // dbg!(root_created = &root_ref.keys[..root_ref.num_keys]);

        rlu_reader_unlock(self.rlu, self.id);
    }

    // helper function to safely set up parent-child relationships
    // unsafe fn link_nodes(&self, parent: *mut Node<K, V>, child: *mut Node<K, V>, child_index: usize) -> bool {
    //     let mut p_parent = parent;
    //     let mut p_child = child;
        
    //     // Lock both nodes in consistent order
    //     if !rlu_try_lock(self.rlu, self.id, &mut p_parent) ||
    //     !rlu_try_lock(self.rlu, self.id, &mut p_child) {
    //         rlu_abort(self.rlu, self.id);
    //         return false;
    //     }
        
    //     (*p_parent).children[child_index] = child;
    //     (*p_child).parent = parent;
    //     true
    // }

    unsafe fn insert_into_parent(&mut self, left: *mut Node<K, V>, right: *mut Node<K, V>, key: K) -> bool {
        dbg!("insert_into_parent called", &key);
        dbg!("Left node: {:?}", &*left);
        dbg!("Right node: {:?}", &*right);

        let left_node = &*left;
        let parent_ptr = left_node.parent;

        if parent_ptr.is_null() {
            // create a new root
            dbg!("No parent, creating a new root");
            
            // create a new root
            let root_box = Box::new(Node::new(false));
            // dbg!("the ws_hdr of new root is {:?}", &root_box.hdr.ws_hdr);
            let root_ptr = Box::into_raw(root_box);


            let mut p_root = root_ptr;

            if !rlu_try_lock(self.rlu, self.id, &mut p_root) {
                dbg!("Failed to  lock new root");
                rlu_abort(self.rlu, self.id);
                return false; // try again or handle error
            }


            let root_node = &mut *p_root;
            root_node.num_keys = 1;
            root_node.keys[0] = Some(key);
            root_node.children[0] = left;
            root_node.children[1] = right;

            // Update parent pointers
            // let mut p_left = left;
            // let mut p_right = right;
            (*left).parent = root_ptr;
            (*right).parent = root_ptr;

            

            dbg!("New root created with key {:?}", key);
            dbg!("Root ptr: 0x{:x}", root_ptr as usize);
            dbg!("Left ptr: 0x{:x}", left as usize);
            dbg!("Right ptr: 0x{:x}", right as usize);

            // Use link_nodes to establish parent-child relationships
            // if !self.link_nodes(root_ptr, left, 0) ||
            //     !self.link_nodes(root_ptr, right, 1) {
            //     rlu_abort(self.rlu, self.id);
            //     return;
            // }

            // // Lock both children to update their parent pointers
            // if !rlu_try_lock(self.rlu, self.id, &mut p_left) ||
            //     !rlu_try_lock(self.rlu, self.id, &mut p_right) {
            //     dbg!("Failed to lock children");
            //     rlu_abort(self.rlu, self.id);
            //     return;
            // }
            // (*p_left).parent = root_ptr;
            // (*p_right).parent = root_ptr;

            // Update tree root
            self.root = root_ptr;

            // self.debug_print_tree();
            // Now commit the transaction which will copy back all locked nodes
            // rlu_reader_unlock(self.rlu, self.id);

            // println!("Created new root with key: {:?}", key);
            // println!("Root keys: {:?}", &(*p_root).keys[..(*p_root).num_keys]);
            // println!("Left child keys: {:?}", &(*left).keys[..(*left).num_keys]);
            // println!("Right child keys: {:?}", &(*right).keys[..(*right).num_keys]);


            // Start a new read transaction to ensure we see the committed state
            // rlu_reader_lock(self.rlu, self.id);
            // let root_check = rlu_dereference(self.rlu, self.id, self.root);
            // dbg!("New root state after commit:", &*root_check);
            // rlu_reader_unlock(self.rlu, self.id);
            // rlu_reader_unlock(self.rlu, self.id);
            return true;
        }

        // LOCK EXISTING PARENT

        dbg!("Parent exists, inserting key into parent");
        // dbg!("Parent keys before insert: {:?}", &(*parent_ptr).keys[..(*parent_ptr).num_keys]);

        // If we have a parent
        let mut p_parent = parent_ptr;
        // rlu_reader_lock(self.rlu, self.id);
        if !rlu_try_lock(self.rlu, self.id, &mut p_parent) {
            dbg!("Could not lock parent, aborting insert_into_parent");
            rlu_abort(self.rlu, self.id);
            return false;
        }

        let parent_node = &mut *p_parent;

        // FInd insert position in parent
        let pos = parent_node.keys[..parent_node.num_keys]
            .iter()
            .position(|k_opt| k_opt.as_ref().map_or(true, |k| k > &key))
            .unwrap_or(parent_node.num_keys);

        dbg!("position in parent for key {:?} is {:?}", key, pos);

        if parent_node.num_keys < B {
            // Simple insert case
            // Shift existing keys/children
            for i in (pos..parent_node.num_keys).rev() {
                parent_node.keys[i+1] = parent_node.keys[i].take();
                parent_node.children[i+2] = parent_node.children[i+1];
            }

            parent_node.keys[pos] = Some(key);
            parent_node.children[pos+1] = right;
            parent_node.num_keys += 1;

            // Update the right node's parent pointer
            (*right).parent = parent_ptr;

            // Lock right child to update parent
            // let mut p_right = right;
            // if !rlu_try_lock(self.rlu, self.id, &mut p_right) {
            //     rlu_abort(self.rlu, self.id);
            //     return;
            // }
            // (*p_right).parent = p_parent;

            // Use link_nodes to set up the new child's parent pointer
            // if !self.link_nodes(p_parent, right, pos + 1) {
            //     rlu_abort(self.rlu, self.id);
            //     return;
            // }
            
            return true;
        }
        println!("Inserting key {:?} into parent. Parent keys before: {:?}", key, &parent_node.keys[..parent_node.num_keys]);
        dbg!("Inserting key into parent. Parent keys before:", &parent_node.keys[..parent_node.num_keys]);

        // // Insert the key into the parent, keeping keys sorted:
        // let pos = parent_node.keys.iter()
        //     .take(parent_node.num_keys)
        //     .position(|k_opt| k_opt.as_ref().map_or(true, |k| k > &key))
        //     .unwrap_or(parent_node.num_keys);

        // assert!(parent_node.num_keys < B, "Parent full before insertion! Must split first.");


        // // Make room for new key/child
        // for i in (pos..parent_node.num_keys).rev() {
        //     parent_node.keys[i+1] = parent_node.keys[i].take();
        //     parent_node.children[i+2] = parent_node.children[i+1];
        // }

        // parent_node.keys[pos] = Some(key);
        // parent_node.children[pos+1] = right;
        // parent_node.num_keys += 1;
        // (*right).parent = p_parent;



        
            // Fits, no split needed
            // rlu_reader_unlock(self.rlu, self.id);
        
        // Parent is full, need to split
        // Parent needs splitting
        let (new_parent_ptr, parent_split_key) = self.split_internal_node(parent_node);
        dbg!("new_parent , left node and parent_split_key are", &*new_parent_ptr, &*parent_node, &parent_split_key);
        if new_parent_ptr.is_null() {
            // self.insert_into_parent(p_parent, new_parent_ptr, parent_split_key);
            return false;
        }


        self.insert_into_parent(p_parent, new_parent_ptr, parent_split_key)

    }

    unsafe fn split_internal_node(&self, node: &mut Node<K,V>) -> (*mut Node<K,V>, K) {
        dbg!("Splitting internal node, parent node looks like:", &node.keys[..node.num_keys]);
        let mut temp_keys = Vec::with_capacity(B+1);
        let mut temp_children = Vec::with_capacity(B+2);

        // Copy existing keys and children
        for i in 0..node.num_keys {
            temp_keys.push(node.keys[i].take().unwrap());
            temp_children.push(node.children[i]);
        }
        temp_children.push(node.children[node.num_keys]);
        dbg!("Temp keys and children", &temp_keys, &temp_children);

        // for i in 0..=node.num_keys {
        //     temp_children.push(node.children[i]);
        // }

        let split = (B+1)/2;
        let split_key = temp_keys[split -  1].clone();
        dbg!("Split key is", &split_key);

        // Reset left node
        // Left node keeps keys[0..split], right node gets keys[split+1..]
        for i in 0..split - 1{
            node.keys[i] = Some(temp_keys[i].clone());
            node.children[i] = temp_children[i];
        }

        node.children[split - 1] = temp_children[split - 1];
        node.num_keys = split - 1;

        // Update parent pointers for left node's children
        for i in 0..=node.num_keys {
            if !node.children[i].is_null() {
                (*node.children[i]).parent = node as *mut Node<K, V>;
            }
        }

        for i in split - 1..B {
            node.keys[i] = None;
            if i < B{
                node.children[i+1] = ptr::null_mut();
            }
            
        }
        dbg!("Left node after split and children is :", &node.keys[..node.num_keys], &node.children[..node.num_keys+1]);
        dbg!("left node parent is ", &node.parent);

        // create nnew right node
        // New node (right side)
        let new_node_box = Box::new(Node::new(false));
        let new_node_ptr = Box::into_raw(new_node_box);

        let mut p_new_node = new_node_ptr;
        if !rlu_try_lock(self.rlu, self.id, &mut p_new_node) {
            rlu_abort(self.rlu, self.id);
            return (ptr::null_mut(), split_key);
        }
        let new_node = &mut *p_new_node;

        // Fill the right node
        let mut j = 0;
        for i in split..temp_keys.len() {
            new_node.keys[j] = Some(temp_keys[i].clone());
            new_node.children[j] = temp_children[i];
            // Update child's parent pointer
            // if !temp_children[i].is_null() {
            //     (*temp_children[i]).parent = new_node_ptr;
            // }
            j += 1;
        }
        new_node.children[j] = temp_children[temp_children.len() - 1];
        new_node.num_keys = j;

        // Update parent pointers for all children in right node
        for i in 0..=new_node.num_keys {
            if !new_node.children[i].is_null() {
                (*new_node.children[i]).parent = p_new_node;
            }
        }

        dbg!(" right node after split", &new_node.keys[..new_node.num_keys], &new_node.children[..new_node.num_keys+1]);


        // let temp_children_store = temp_children.clone();
        // dbg!("Temp children store", &temp_children_store);
        // new_node.children[j] = temp_children_store[temp_children_store.len() - 1];
        // if !temp_children[temp_children_store.len() - 1].is_null() {
        //     (*temp_children[temp_children_store.len() - 1]).parent = new_node_ptr;
        // }
        // new_node.num_keys = j;

        // Set parent pointer for new node (will be properly set in insert_into_parent)
        dbg!("Right node after setup:", &new_node.keys[..new_node.num_keys]);
        dbg!("Right node children:", &new_node.children[..=new_node.num_keys]);
        dbg!("Split key being returned:", split_key);
        dbg!("new_node is", &*p_new_node);

        // let new_parent_box = Box::new(Node::new(false));
        // let new_parent_ptr = Box::into_raw(new_parent_box);
        // let mut p_new_parent = new_parent_ptr;
        // if !rlu_try_lock(self.rlu, self.id, &mut p_new_parent) {
        //     dbg!("Failed to lock new internal node after creation");
        //     rlu_abort(self.rlu, self.id);
        //     // rlu_reader_unlock(self.rlu, self.id);
        //     return (ptr::null_mut(), split_key); // error scenario
        // }

        // let new_node = &mut *p_new_parent;
        // let mut j = 0;
        // for i in (split+1)..(B+1) {
        //     new_node.keys[j] = Some(temp_keys[i].clone());
        //     new_node.children[j] = temp_children[i];
        //     // Set children's parent
        //     if !new_node.children[j].is_null() {
        //         (*new_node.children[j]).parent = p_new_parent;
        //     }
        //     j += 1;
        // }
        // new_node.children[j] = temp_children[B+1]; 
        // if !new_node.children[j].is_null() {
        //     (*new_node.children[j]).parent = p_new_parent;
        // }
        // new_node.num_keys = j;
        // dbg!("Split internal node", left_keys = &node.keys[..node.num_keys], right_keys = &new_node.keys[..new_node.num_keys]);
        // dbg!("Split internal node");
        // dbg!(&node.keys[..node.num_keys], &new_node.keys[..new_node.num_keys]);

        
        // // Set parents of moved children
        // for i in 0..=new_node.num_keys {
        //     if !new_node.children[i].is_null() {
        //         // let mut child_ptr = new_node.children[i];
        //         // if rlu_try_lock(self.rlu, self.id, &mut child_ptr) {
        //         //     (*child_ptr).parent = p_new_parent;
        //         // }
        //         if !self.link_nodes(new_parent_ptr, new_node.children[i], i) {
        //             rlu_abort(self.rlu, self.id);
        //             return (ptr::null_mut(), split_key);
        //         }
        //     }
        // }

        // println!("Splitting internal node with {:?} keys", &temp_keys);
        // println!("Split key: {:?}", split_key);
        // println!("Left node keys after split: {:?}", &node.keys[..node.num_keys]);
        // println!("New node keys after split: {:?}", &(*p_new_parent).keys[..(*p_new_parent).num_keys]);


        // rlu_reader_unlock(self.rlu, self.id);
        // dbg!("new node in split_internal_node is", &*new_node_ptr);
        (p_new_node, split_key)
    }

    pub fn range_search(&self, start_key: &K, end_key: &K) -> Vec<(K, V)> {
        let mut result = Vec::new();
        
        unsafe {
            rlu_reader_lock(self.rlu, self.id);
            
            // Find the leaf containing start_key
            let mut current = self.find_leaf_for_key(start_key);
            if current.is_null() {
                rlu_reader_unlock(self.rlu, self.id);
                return result;
            }
            
            // Traverse the leaves using next_leaf pointers
            loop {
                let node = &*current;
                
                // Add all keys/values in current leaf that are in range
                for i in 0..node.num_keys {
                    if let (Some(k), Some(v)) = (&node.keys[i], &node.values[i]) {
                        if k >= start_key && k <= end_key {
                            result.push((*k, *v));
                        }
                        if k > end_key {
                            rlu_reader_unlock(self.rlu, self.id);
                            return result;
                        }
                    }
                }
                
                // Move to next leaf if it exists
                if node.next_leaf.is_null() {
                    break;
                }
                current = rlu_dereference(self.rlu, self.id, node.next_leaf);
                if current.is_null() {
                    break;
                }
            }
            
            rlu_reader_unlock(self.rlu, self.id);
        }
        result
    }

    
}
impl<K: Ord + Clone + Copy + Debug, V: Ord + Clone + Copy + Debug> BPlusTree<K, V> {
    pub fn debug_print_tree(&self) {
        println!("\n=== B+ Tree Structure with Detailed Pointer Analysis ===");
        println!("Order (B) = {}", B);
        
        unsafe {
            rlu_reader_lock(self.rlu, self.id);
            let root = rlu_dereference(self.rlu, self.id, self.root);
            if root.is_null() {
                println!("Empty tree");
                rlu_reader_unlock(self.rlu, self.id);
                return;
            }

            println!("Root address: 0x{:x}", root as usize);
            
            let mut current_level = vec![root];
            let mut level_number = 0;
            let mut all_nodes = Vec::new(); // Keep track of all valid node addresses

            while !current_level.is_empty() {
                println!("\nLevel {}: ", level_number);
                println!("{}", "=".repeat(80));
                
                let mut next_level = Vec::new();
                
                for (node_idx, &node_ptr) in current_level.iter().enumerate() {
                    let node = &*node_ptr;
                    all_nodes.push(node_ptr as usize);
                    
                    print!("Node {}: [", node_idx);
                    for i in 0..node.num_keys {
                        if let Some(key) = &node.keys[i] {
                            print!("{:?}", key);
                            if i < node.num_keys - 1 {
                                print!(" | ");
                            }
                        }
                    }
                    print!("] (addr: 0x{:x})", node_ptr as usize);
                    
                    if node.is_leaf {
                        print!("🍃");
                        if !node.next_leaf.is_null() {
                            print!(" next_leaf: 0x{:x}", node.next_leaf as usize);
                        }
                    } else {
                        print!("🔵");
                        for i in 0..=node.num_keys {
                            if !node.children[i].is_null() {
                                next_level.push(node.children[i]);
                                print!("\n    child[{}] addr: 0x{:x}", i, node.children[i] as usize);
                            }
                        }
                    }
                    
                    if !node.parent.is_null() {
                        print!("\n    parent addr: 0x{:x}", node.parent as usize);
                        if !all_nodes.contains(&(node.parent as usize)) {
                            print!(" WARNING: Parent not found in previous levels!");
                        }
                    }
                    println!();
                }
                
                println!("{}", "-".repeat(80));
                current_level = next_level;
                level_number += 1;
            }
            
            rlu_reader_unlock(self.rlu, self.id);
        }
        println!("\n=== End of Tree ===\n");
    }

}

//  impl<K: Clone + Copy + Debug + Unpin + Ord, V: Clone + Copy + Debug + Unpin + Ord> BPlusTree<K, V> {
//     pub fn remove(&mut self, key: &K) -> bool {
//         unsafe {
//             rlu_reader_lock(self.rlu, self.id);
            

//             // find the leaf contianing the key
//             let mut leaf_ptr = self.find_leaf_for_key(key);
//             if leaf_ptr.is_null() {
//                 rlu_reader_lock(self.rlu, self.id);
//                 return false;
//             }

//             if !rlu_try_lock(self.rlu, self.id, &mut leaf_ptr) {
//                 rlu_abort(self.rlu, self.id);
//                 return false;
//             }

//             let leaf = &mut *leaf_ptr;


//             // find the key in the leaf to delete

//             let key_pos = match leaf.keys[..leaf.num_keys]
//                 .iter()
//                 .position(|k| k.as_ref() == Some(key))    
//             {
//                 Some(pos) => pos,
//                 None => {
//                     rlu_reader_unlock(self.rlu, self.id);
//                     return false;
//                 }   
//             };

//             self.remove_from_leaf(leaf, key_pos);

//             // Check if we need to handle underflow
//             let min_keys = (B-1)/2;
//             if leaf.num_keys >= min_keys && !leaf_ptr.eq(&self.root){
            
//                 self.handle_underflow(leaf_ptr)?;
//             }

//             rlu_reader_unlock(self.rlu, self.id);
//             return true;
//         }
//     }

//     unsafe fn remove_from_leaf(&self, leaf: &mut Node<K, V>, pos: usize) {
//         // Shift remaining keys and values left
//         for i in pos..leaf.num_keys-1 {
//             leaf.keys[i] = leaf.keys[i+1].take();
//             leaf.values[i] = leaf.values[i+1].take();
//         }

//         // Clear the last position
//         leaf.keys[leaf.num_keys -1] = None;
//         leaf.values[leaf.num_keys -1] = None;
//         leaf.num_keys -= 1;

//     }

//     unsafe fn handle_underflow(&mut self, node_ptr: *mut Node<K, V>) -> bool {
//         let node = &mut *node_ptr;
//         let min_keys = (B-1)/2;

//         // Get parent 
//         let parent_ptr = node.parent;
//         if parent_ptr.is_null() {
//             // Root node, no underflow possible
//             return true;
//         }

//         // Lock parent
//         let mut p_parent = parent_ptr;
//         if !rlu_try_lock(self.rlu, self.id, &mut p_parent) {
//             rlu_abort(self.rlu, self.id);
//             return false;
//         }

//         let parent = &mut *p_parent;

//         // Find the current nodes position in the parent
//         let node_pos = parent.children[..=parent.num_keys]
//             .iter()
//             .position(|child| child.eq(&node_ptr))
//             .unwrap();

//         // try borrowing from the left sibling
//         if node_pos > 0 {
//             let left_sibling_ptr = parent.children[node_pos -1];
//             let mut p_left_sibling = left_sibling_ptr;

//             if !rlu_try_lock(self.rlu, self.id, &mut p_left_sibling) {
//                 rlu_abort(self.rlu, self.id);
//                 return false;
//             }

//             let left_sibling = &mut *p_left_sibling;

//             if left_sibling.num_keys > min_keys {
//                 return self.redistribute_from_left(node_ptr, left_sibling, parent_ptr,  node_pos);

//             }

//         }
//         // Try to borrow from right sibling
//         if node_pos < parent.num_keys {
//             let right_sibling_ptr = parent.children[node_pos + 1];
//             let mut p_right_sibling = right_sibling_ptr;
            
//             if !rlu_try_lock(self.rlu, self.id, &mut p_right_sibling) {
//                 rlu_abort(self.rlu, self.id);
//                 return false;
//             }

//             let right_sibling = &mut *p_right_sibling;
            
//             if right_sibling.num_keys > min_keys {
//                 return self.redistribute_from_right(node_ptr, right_sibling_ptr, parent_ptr, node_pos);
//             }
//         }

//         // If we get here, we need to merge
//         if node_pos > 0 {
//             // Merge with left sibling
//             let left_sibling_ptr = parent.children[node_pos -1];
//             self.merge_nodes(left_sibling_ptr, node_ptr,  parent_ptr, node_pos -1)
//         } else {
//             let right_sibling_ptr = parent.children[node_pos +1];
//             self.merge_nodes(node_ptr, right_sibling_ptr, parent_ptr, node_pos)
//         }
//     }   

//     unsafe fn redistribute_from_left(
//         &self, 
//         node_ptr: *mut Node<K, V>,
//         left_ptr: *mut Node<K, V>,
//         parent_ptr: *mut Node<K, V>,
//         node_pos: usize,
//     ) -> bool {
//         let node = &mut *node_ptr;
//         let left = &mut *left_ptr;
//         let parent = &mut *parent_ptr;

//         // Make space in the target node
//         for i in (0..node.num_keys).rev() {
//             node.keys[i + 1] = node.keys[i].take();
//             node.values[i + 1] = node.values[i].take();
//         }

//         // Move the last key/value from left sibling
//         node.keys[0] = left.keys[left.num_keys - 1].take();
//         node.values[0] = left.values[left.num_keys - 1].take();
//         node.num_keys += 1;
//         left.num_keys -= 1;

//         // Update parent's separator key
//         parent.keys[node_pos - 1] = Some(node.keys[0].unwrap().clone());
        
//         true
//     }

//     unsafe fn redistribute_from_right (
//         &self,
//         node_ptr: *mut Node<K, V>,
//         right_ptr: *mut Node<K, V>,
//         parent_ptr: *mut Node<K, V>,
//         node_pos: usize,
//     ) -> bool {
//         let node = &mut *node_ptr;
//         let right = &mut *right_ptr;
//         let parent = &mut *parent_ptr;

//         // Move first key/value from right sibling to end of current node
//         node.keys[node.num_keys] = right.keys[0].take();
//         node.values[node.num_keys] = right.values[0].take();
//         node.num_keys += 1;

//         // Shift remaining keys in right sibling left
//         for i in 0..right.num_keys - 1 {
//             right.keys[i] = right.keys[i + 1].take();
//             right.values[i] = right.values[i + 1].take();
//         }
//         right.num_keys -= 1;

//         // Update parent's separator key
//         parent.keys[node_pos] = Some(right.keys[0].unwrap().clone());
        
//         true
//     }

//     unsafe fn merge_nodes(
//         &mut self, 
//         left_ptr: *mut Node<K, V>,
//         right_ptr: *mut Node<K, V>,
//         parent_ptr: *mut Node<K, V>,
//         left_pos: usize,
//     ) -> bool {
//         let left = &mut *left_ptr;
//         let right = &mut *right_ptr;
//         let parent = &mut *parent_ptr;

//         // copy all keys and values from right to left
//         for i in 0..right.num_keys {
//             left.keys[left.num_keys + i] = right.keys[i].take();
//             left.values[left.num_keys + i] = right.values[i].take();
//         }

//         left.num_keys += right.num_keys;

//         // Update leaf pointers if these are leaf nodes
//         if left.is_leaf {
//             left.next_leaf = right.next_leaf;
//         }

//         // remove the separator key from parent and adjust child pointers
//         for i in left_pos + 1..parent.num_keys {
//             parent.keys[i - 1] = parent.keys[i].take();
//             parent.children[i] = parent.children[i + 1];
//         }
//         parent.num_keys -= 1;
//         parent.children[parent.num_keys + 1] = ptr::null_mut();

//         // Handle parent underflow if needed
//         if parent.num_keys < (B - 1) / 2 {
//             if parent_ptr == self.root {
//                 if parent.num_keys == 0 {
//                     // Root is empty, make left child the new root
//                     self.root = left_ptr;
//                     (*left_ptr).parent = ptr::null_mut();
//                 }
//             } else {
//                 self.handle_underflow(parent_ptr)?;
//             }
//         }

//         true

//     }
// }