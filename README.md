# heapix

[![Crates.io](https://img.shields.io/crates/v/heapix.svg)](https://crates.io/crates/heapix)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A lightweight Rust library providing heap-based priority queue data structures with efficient operations, including decrease-key and heap construction.

---

## Features

- **Min-Heap** (`MinHeap<K>`): maintain a priority queue of `(item_id, key)` pairs, where the smallest key is always at the root.
- **Efficient operations**:
  - `insert` in _O(log n)_
  - `get_min` in _O(1)_
  - `delete_min` in _O(log n)_
  - `decrease_key` in _O(log n)_
  - `build_heap` from an unsorted list in _O(n)_

## Installation

Add `heapix` to your `Cargo.toml` dependencies:

```toml
[dependencies]
heapix = "0.1"
```

Or with `cargo`:

```bash
cargo add heapix
```

## Quick Start

```rust
use heapix::MinHeap;

fn main() {
    // Create a new min-heap
    let mut heap: MinHeap<i32> = MinHeap::new();

    // Insert items as (id, key) pairs
    heap.insert((0, 42));
    heap.insert((1, 17));
    heap.insert((2, 58));

    // Peek at the minimum element without removing it
    if let Some(&(id, key)) = heap.get_min() {
        println!("Min item: id={}, key={}", id, key);
    }

    // Decrease the key of an existing item
    heap.decrease_key(2, 13);

    // Pop elements in ascending order of key
    while let Some((id, key)) = heap.delete_min() {
        println!("Popped id={}, key={}", id, key);
    }
}
```

## API

### `MinHeap<K>`

- `MinHeap::new() -> MinHeap<K>`
- `MinHeap::build_heap(items: Vec<(usize, K)>) -> MinHeap<K>`
- `is_empty(&self) -> bool`
- `insert(&mut self, item: (usize, K))`
- `get_min(&self) -> Option<&(usize, K)>`
- `delete_min(&mut self) -> Option<(usize, K)>`
- `decrease_key(&mut self, id: usize, new_key: K)`

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
