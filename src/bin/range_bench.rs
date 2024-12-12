use std::time::Instant;
use std::thread;
use std::collections::BTreeMap;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use rlu::BPlusTree;  
use rlu::BPTree as RegularBPlusTree;

fn populate_trees() -> (BPlusTree<i32, i32>, RegularBPlusTree<i32, i32>, BTreeMap<i32, i32>) {
    let mut rlu_tree = BPlusTree::new();
    let mut seq_tree = RegularBPlusTree::new();
    let mut btree_map = BTreeMap::new();
    
    // Insert same data into all trees
    for i in (10..35).step_by(2) {
        rlu_tree.insert(i, i * 10);
        seq_tree.insert(i, i * 10);
        btree_map.insert(i, i * 10);
    }
    
    (rlu_tree, seq_tree, btree_map)
}

fn bench_rlu_range(num_threads: usize, num_queries: usize) -> u128 {
    let (tree, _, _) = populate_trees();
    let start = Instant::now();
    
    let worker = || {
        let tree = tree.clone_ref();
        thread::spawn(move || {
            let mut rng = SmallRng::from_seed([0; 16]);
            for _ in 0..num_queries {
                let start_key = rng.gen_range(10, 30);
                let end_key = start_key + rng.gen_range(1, 6);
                let _ = tree.range_search(&start_key, &end_key);
            }
        })
    };

    let readers: Vec<_> = (0..num_threads).map(|_| worker()).collect();
    for t in readers {
        t.join().unwrap();
    }

    start.elapsed().as_millis()
}

fn bench_sequential_range(num_queries: usize) -> u128 {
    let (_, tree, _) = populate_trees();
    let start = Instant::now();
    
    let mut rng = SmallRng::from_seed([0; 16]);
    for _ in 0..num_queries {
        let start_key = rng.gen_range(10, 30);
        let end_key = start_key + rng.gen_range(1, 6);
        let _ = tree.range_query(&start_key, &end_key);
    }
    
    start.elapsed().as_millis()
}

fn bench_btreemap_range(num_queries: usize) -> u128 {
    let (_, _, tree) = populate_trees();
    let start = Instant::now();
    
    let mut rng = SmallRng::from_seed([0; 16]);
    for _ in 0..num_queries {
        let start_key = rng.gen_range(10, 30);
        let end_key = start_key + rng.gen_range(1, 6);
        let _ = tree.range(start_key..=end_key).collect::<Vec<_>>();
    }
    
    start.elapsed().as_millis()
}

fn main() {
    let operation_counts = [1_000_000, 100_000, 10_000];

    for &num_queries in &operation_counts {
    println!("type,threads,time_ms");
    
    // Benchmark BTreeMap
    let btree_times: Vec<u128> = (0..5)
        .map(|_| bench_btreemap_range(num_queries))
        .collect();
    let avg_btree = (btree_times.iter().sum::<u128>() as f64) / 5.0;
    println!("btreemap,1,{}", avg_btree);
    
    // Benchmark sequential B+ tree
    let sequential_times: Vec<u128> = (0..5)
        .map(|_| bench_sequential_range(num_queries))
        .collect();
    let avg_sequential = (sequential_times.iter().sum::<u128>() as f64) / 5.0;
    println!("sequential,1,{}", avg_sequential);
    
    // Benchmark RLU B+ tree with different thread counts
    for threads in 1..=4 {
        let rlu_times: Vec<u128> = (0..5)
            .map(|_| bench_rlu_range(threads, num_queries / threads))
            .collect();
        
        let avg_rlu = (rlu_times.iter().sum::<u128>() as f64) / 5.0;
        println!("rlu,{},{}", threads, avg_rlu);
    }
    }
}