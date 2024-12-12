use std::time::Instant;
use std::thread;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rlu::BPlusTree;  
use rlu::BPTree as RegularBPlusTree;
use prettytable::{Table, row};
use std::collections::BTreeMap;


// Helper function to pre-populate the trees with a stable 2-level structure
fn populate_tree<T: BPlusTreeTrait>(mut tree: T) -> T {
    // For order 8, we can safely insert these without causing a 3rd level
    for i in (10..35).step_by(2) {
        tree.insert(i, i * 10);
    }
    tree
}

// Common trait to handle both tree types
trait BPlusTreeTrait {
    fn insert(&mut self, key: i32, value: i32);
    fn search(&self, key: &i32) -> Option<i32>;
}

impl BPlusTreeTrait for BPlusTree<i32, i32> {
    fn insert(&mut self, key: i32, value: i32) {
        self.insert(key, value);
    }
    fn search(&self, key: &i32) -> Option<i32> {
        self.search(key)
    }
}

impl BPlusTreeTrait for RegularBPlusTree<i32, i32> {
    fn insert(&mut self, key: i32, value: i32) {
        self.insert(key, value);
    }
    fn search(&self, key: &i32) -> Option<i32> {
        self.search(key)
    }
}

fn bench_rlu_bptree(num_threads: usize, num_searches: usize) -> u128 {
    let tree = BPlusTree::new();
    let tree = populate_tree(tree);
    
    let start = Instant::now();
    
    let worker = || {
        let tree = tree.clone_ref();
        thread::spawn(move || {
            let mut rng = SmallRng::from_seed([0; 16]);
            for _ in 0..num_searches {
                let i = rng.gen_range(10, 35);
                let _ = tree.search(&i);
            }
        })
    };

    let readers: Vec<_> = (0..num_threads).map(|_| worker()).collect();
    for t in readers {
        t.join().unwrap();
    }

    start.elapsed().as_millis()
}

fn bench_regular_bptree(num_threads: usize, num_searches: usize) -> u128 {
    let tree = RegularBPlusTree::new();
    let tree = populate_tree(tree);
    
    let start = Instant::now();
    
    let worker = || {
        let tree = tree.copy_ref();
        thread::spawn(move || {
            let mut rng = SmallRng::from_seed([0; 16]);
            for _ in 0..num_searches {
                let i = rng.gen_range(10, 35);
                let _ = tree.search(&i);
            }
        })
    };

    let readers: Vec<_> = (0..num_threads).map(|_| worker()).collect();
    for t in readers {
        t.join().unwrap();
    }

    start.elapsed().as_millis()
}

fn bench_btreemap(num_threads: usize, num_ops: usize) -> u128 {
    let mut tree = BTreeMap::new();
    
    // Pre-populate with same data
    for i in 10..30 {
        tree.insert(i, i * 10);
    }
    
    let start = Instant::now();
    
    let worker = || {
        let mut tree = tree.clone();
        thread::spawn(move || {
            let mut rng = SmallRng::from_seed([0; 16]);
            for _ in 0..num_ops {
                if rng.gen_bool(0.7) {
                    let i = rng.gen_range(10, 50);
                    let _ = tree.get(&i);
                } else {
                    let i = rng.gen_range(10, 50);
                    tree.insert(i, i * 10);
                }
            }
        })
    };

    let workers: Vec<_> = (0..num_threads).map(|_| worker()).collect();
    for t in workers {
        t.join().unwrap();
    }

    start.elapsed().as_millis()
}


// fn main() {
//     let num_searches = 1000000; // Large number of searches for meaningful timing
//     println!("number of searches: {}", num_searches);
//     println!("type,threads,time_ms");
    
//     // Run sequential baseline
//     let sequential_times: Vec<u128> = (0..5)
//         .map(|_| bench_regular_bptree(1, num_searches))
//         .collect();
//     let avg_sequential = (sequential_times.iter().sum::<u128>() as f64) / 5.0;
//     println!("sequential,1,{}", avg_sequential);
    
//     // Run RLU with different thread counts
//     for threads in 1..=4 {
//         let rlu_times: Vec<u128> = (0..5)
//             .map(|_| bench_rlu_bptree(threads, num_searches / threads))
//             .collect();
        
//         let avg_rlu = (rlu_times.iter().sum::<u128>() as f64) / 5.0;
//         println!("rlu,{},{}", threads, avg_rlu);
//     }
// }

fn main() {
    let operation_counts = [1_000_000, 100_000, 10_000];
    
    for &num_searches in &operation_counts {
        println!("\nnumber of searches: {}", num_searches);
        println!("type,threads,time_ms");
        
        // Benchmark BTreeMap
        let btree_times: Vec<u128> = (0..5)
            .map(|_| bench_btreemap(1,num_searches))
            .collect();
        let avg_btree = (btree_times.iter().sum::<u128>() as f64) / 5.0;
        println!("btreemap,1,{}", avg_btree);
        
        // Run sequential baseline
        let sequential_times: Vec<u128> = (0..5)
            .map(|_| bench_regular_bptree(1, num_searches))
            .collect();
        let avg_sequential = (sequential_times.iter().sum::<u128>() as f64) / 5.0;
        println!("sequential,1,{}", avg_sequential);
        
        // Run RLU with different thread counts
        for threads in 1..=4 {
            let rlu_times: Vec<u128> = (0..5)
                .map(|_| bench_rlu_bptree(threads, num_searches / threads))
                .collect();
            
            let avg_rlu = (rlu_times.iter().sum::<u128>() as f64) / 5.0;
            println!("rlu,{},{}", threads, avg_rlu);
        }
    }
}