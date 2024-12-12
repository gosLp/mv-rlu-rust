# RLU B+ Tree in Rust

## Overview
This project implements a concurrent B+ tree data structure using the Read-Log-Update (RLU) concurrency mechanism in Rust. It builds upon the RLU implementation by [hudson-ayers/rlu-rust](https://github.com/hudson-ayers/rlu-rust), extending it to support B+ tree operations.

The B+ tree implementation offers:
- Concurrent search operations
- Thread-safe insertions
- Range query support
- Multiple reader threads with RLU synchronization

## Getting Started

### Prerequisites
- Rust and Cargo (latest stable version)
- Git

### Installation
```bash
git clone https://github.com/gosLp/mv-rlu-rust.git
cd [repo-name]
cargo build
```

## Running Tests

To verify the implementation is working correctly:
```bash
cargo test
```

### Benchmarks
The project includes several benchmarks to compare performance between different B+ tree implementations:

RLU B+ Tree (our implementation)
Sequential B+ Tree
Rust's built-in BTreeMap

### Running Benchmarks

1. Normal Search Benchmark:

```bash
cargo run --bin bench_bp
```
This will run search operations with varying thread counts (1-4) and operation counts (10K, 100K, 1M).

2. Range Query Benchmark:

```bash
cargo run --bin range_bench
```
This benchmark compares range query performance across implementations.
Uses RLU (Read-Log-Update) for concurrency control
Order-8 B+ tree implementation
Supports concurrent reads with write operations
Maintains tree balance during insertions


### Performance
Our implementation shows significant performance improvements with multiple threads:

1. Up to 4x speedup with 4 threads for search operations
2. Competitive performance against Rust's built-in BTreeMap
3. Efficient range query support

### Credits

Original RLU implementation by hudson-ayers/rlu-rust
Based on the Read-Log-Update concurrency mechanism (SOSP '15)

## Note
This is a research implementation and may contain memory safety issues not caught by the test suite. It should not be used in production environments without thorough validation.

