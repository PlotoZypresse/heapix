# heapix

[![Crates.io](https://img.shields.io/crates/v/heapix.svg)](https://crates.io/crates/heapix)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A lightweight Rust library offering **two** heap‑based priority‑queue data structures with the same ergonomic API:

* **`MinHeap<K>`** – classic binary min‑heap (array‑based) with `O(log n)` insert / delete‑min.
* **`FibHeap<K>`** – a Fibonacci heap with `O(1)` amortised insert & decrease‑key and `O(log n)` delete‑min. Perfect for graph algorithms (Dijkstra, Prim) that call `decrease_key` frequently.

Both types store items as `(id, key)` tuples and keep a dense `positions` array so you can change priorities in constant time.

---

## Features

| Structure | insert     | delete‑min | get\_min | decrease\_key | build\_heap |
| --------- | ---------- | ---------- | -------- | ------------- | ----------- |
| `MinHeap` | `O(log n)` | `O(log n)` | `O(1)`   | `O(log n)`    | `O(n)`      |
| `FibHeap` | **`O(1)`** | `O(log n)` | `O(1)`   | **`O(1)`**    | `O(n)`      |

* Identical public API – swap one for the other via a simple `type` alias.
* Generic over any `K: PartialOrd + Copy` (integers, floats, etc.).
* Dense‑id `positions` table for constant‑time `decrease_key(id, new_key)`.
* `build_heap` to construct directly from an unsorted vector.

---

## Installation

```toml
[dependencies]
heapix = "0.4"
```

Or via the CLI:

```bash
cargo add heapix
```

---

## Quick Start

```rust
use heapix::{MinHeap, FibHeap};

fn main() {
    // Binary heap
    let mut bh: MinHeap<i32> = MinHeap::new();
    bh.insert((0, 42));
    bh.insert((1, 17));

    // Fibonacci heap (same calls!)
    let mut fh: FibHeap<i32> = FibHeap::new();
    fh.insert((0, 42));
    fh.insert((1, 17));

    // Decrease key in O(1)
    fh.decrease_key(1, 5);

    println!("min → {:?}", fh.get_min()); // (1, 5)
}
```

---

## API (common to both heaps)

```rust
new() -> Heap<K>
build_heap(items: Vec<(usize, K)>) -> Heap<K>
is_empty(&self) -> bool
len(&self) -> usize
clear(&mut self)
insert(&mut self, item: (usize, K))
get_min(&self) -> Option<&(usize, K)>
delete_min(&mut self) -> Option<(usize, K)>
decrease_key(&mut self, id: usize, new_key: K)
```

Replace `Heap<K>` with either `MinHeap<K>` or `FibHeap<K>`.

---

## Choosing a heap

* Use **`MinHeap`** when your workload rarely calls `decrease_key` (e.g. a simple priority‑queue for tasks).
* Use **`FibHeap`** for graph algorithms or any scenario heavy on `decrease_key` or heap melding.

Both share the same tests in `./tests` to guarantee identical behaviour.

---

## License

Licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
