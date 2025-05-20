//! Fibonacci Heap
//!
//! Same public API (`insert`, `delete_min`, `decrease_key`, …) but with
//! Fibonacci‑heap amortised costs.
//! Uses a dense‑id `positions` array so the caller continues to pass an
//! `(id, key)` tuple just like the original binary heap.
//!
//! ---------------------------------------------------------------------------

use std::cmp::Ordering;

const NOT_IN_HEAP: usize = usize::MAX;

#[derive(Clone)]
struct Node<K> {
    entry: (usize, K), // (id, key)
    degree: usize,
    mark: bool,
    parent: Option<usize>,
    child: Option<usize>,
    left: usize,
    right: usize,
}

impl<K: PartialOrd + Copy> Node<K> {
    fn new(id: usize, key: K, idx: usize) -> Self {
        Node {
            entry: (id, key),
            degree: 0,
            mark: false,
            parent: None,
            child: None,
            left: idx,
            right: idx,
        }
    }
}

/// Same interface as `MinHeap`, backed by a Fibonacci heap.
pub struct FibHeap<K> {
    nodes: Vec<Node<K>>,   // backing storage
    positions: Vec<usize>, // id → node‑index | NOT_IN_HEAP
    min_root: Option<usize>,
    n: usize,
}

impl<K: PartialOrd + Copy> FibHeap<K> {
    // ------------------------------------------------------------------
    // Public API (identical to MinHeap)
    // ------------------------------------------------------------------

    pub fn new() -> Self {
        FibHeap {
            nodes: Vec::new(),
            positions: Vec::new(),
            min_root: None,
            n: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }
    pub fn len(&self) -> usize {
        self.n
    }

    pub fn clear(&mut self) {
        for node in &self.nodes {
            self.positions[node.entry.0] = NOT_IN_HEAP;
        }
        self.nodes.clear();
        self.min_root = None;
        self.n = 0;
    }

    pub fn build_heap(items: Vec<(usize, K)>) -> Self {
        let mut h = Self::new();
        for (id, key) in items {
            h.insert((id, key));
        }
        h
    }

    pub fn insert(&mut self, item: (usize, K)) {
        let (id, key) = item;
        let idx = self.nodes.len();
        self.nodes.push(Node::new(id, key, idx));

        if id >= self.positions.len() {
            self.positions.resize(id + 1, NOT_IN_HEAP);
        }
        self.positions[id] = idx;

        self.add_to_root(idx);
        self.n += 1;
        self.update_min(idx);
    }

    pub fn get_min(&self) -> Option<&(usize, K)> {
        self.min_root.map(|idx| &self.nodes[idx].entry)
    }

    pub fn delete_min(&mut self) -> Option<(usize, K)> {
        let z = self.min_root?;

        // (1) promote all children to root list
        if let Some(first_child) = self.nodes[z].child {
            let mut cur = first_child;
            loop {
                let next = self.nodes[cur].right; // save before detaching
                self.nodes[cur].parent = None;
                self.nodes[cur].mark = false;
                self.detach(cur);
                self.add_to_root(cur);
                if next == first_child {
                    break;
                }
                cur = next;
            }
            self.nodes[z].child = None;
        }

        // (2) remove z from root list
        let successor = self.nodes[z].right;
        self.detach(z);

        // (3) choose new min root / consolidate
        if successor == z {
            self.min_root = None; // heap is now empty
        } else {
            self.min_root = Some(successor);
            self.consolidate();
        }

        self.n -= 1;
        let (id, key) = self.nodes[z].entry;
        self.positions[id] = NOT_IN_HEAP;
        Some((id, key))
    }

    pub fn decrease_key(&mut self, id: usize, new_key: K) {
        let idx = *self.positions.get(id).unwrap_or(&NOT_IN_HEAP);
        assert!(idx != NOT_IN_HEAP, "id not present in heap");
        assert!(
            self.nodes[idx].entry.1.partial_cmp(&new_key).unwrap() == Ordering::Greater,
            "new key must be smaller"
        );

        self.nodes[idx].entry.1 = new_key;

        if let Some(parent) = self.nodes[idx].parent {
            if self.nodes[idx]
                .entry
                .1
                .partial_cmp(&self.nodes[parent].entry.1)
                .unwrap()
                == Ordering::Less
            {
                self.cut(idx, parent);
                self.cascading_cut(parent);
            }
        }
        self.update_min(idx);
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    fn update_min(&mut self, idx: usize) {
        if let Some(m) = self.min_root {
            if self.nodes[idx]
                .entry
                .1
                .partial_cmp(&self.nodes[m].entry.1)
                .unwrap()
                == Ordering::Less
            {
                self.min_root = Some(idx);
            }
        } else {
            self.min_root = Some(idx);
        }
    }

    fn add_to_root(&mut self, idx: usize) {
        if let Some(min_idx) = self.min_root {
            let left = self.nodes[min_idx].left;
            self.nodes[idx].left = left;
            self.nodes[idx].right = min_idx;
            self.nodes[left].right = idx;
            self.nodes[min_idx].left = idx;
        } else {
            self.min_root = Some(idx);
        }
    }

    fn detach(&mut self, idx: usize) {
        let l = self.nodes[idx].left;
        let r = self.nodes[idx].right;
        self.nodes[l].right = r;
        self.nodes[r].left = l;
        self.nodes[idx].left = idx;
        self.nodes[idx].right = idx;
    }

    fn link(&mut self, y: usize, x: usize) {
        // make y child of x
        self.detach(y);
        if let Some(c) = self.nodes[x].child {
            // insert y into child list
            let left = self.nodes[c].left;
            self.nodes[y].left = left;
            self.nodes[y].right = c;
            self.nodes[left].right = y;
            self.nodes[c].left = y;
        } else {
            self.nodes[x].child = Some(y);
        }
        self.nodes[y].parent = Some(x);
        self.nodes[x].degree += 1;
        self.nodes[y].mark = false;
    }

    /// Consolidate the root list: combine trees of equal degree until each
    /// degree occurs at most once in the root list.
    fn consolidate(&mut self) {
        if self.min_root.is_none() {
            return;
        }
        let max_deg = (self.n as f64).log2().ceil() as usize + 2;
        let mut aux: Vec<Option<usize>> = vec![None; max_deg];

        // Collect current roots into a vector first (because we’ll mutate links)
        let start = self.min_root.unwrap();
        let mut roots = Vec::new();
        let mut w = start;
        loop {
            roots.push(w);
            w = self.nodes[w].right;
            if w == start {
                break;
            }
        }

        // Process each root
        for mut x in roots {
            let mut d = self.nodes[x].degree;
            loop {
                match aux[d] {
                    None => {
                        aux[d] = Some(x);
                        break;
                    }
                    Some(mut y) => {
                        // Ensure x has smaller key
                        if self.nodes[y]
                            .entry
                            .1
                            .partial_cmp(&self.nodes[x].entry.1)
                            .unwrap()
                            == Ordering::Less
                        {
                            std::mem::swap(&mut x, &mut y);
                        }
                        // Link y under x and continue with new degree
                        aux[d] = None;
                        self.link(y, x);
                        d += 1;
                        if d >= aux.len() {
                            aux.resize(d + 1, None);
                        }
                        continue;
                    }
                }
            }
        }

        // Rebuild root list from aux array and re‑compute min
        self.min_root = None;
        for idx in aux.into_iter().flatten() {
            self.nodes[idx].left = idx;
            self.nodes[idx].right = idx;
            self.nodes[idx].parent = None;
            self.add_to_root(idx);
            self.update_min(idx);
        }
    }

    fn cut(&mut self, idx: usize, parent: usize) {
        if self.nodes[idx].right == idx {
            self.nodes[parent].child = None;
        } else {
            if self.nodes[parent].child == Some(idx) {
                self.nodes[parent].child = Some(self.nodes[idx].right);
            }
            self.detach(idx);
        }
        self.nodes[parent].degree -= 1;
        self.nodes[idx].parent = None;
        self.nodes[idx].mark = false;
        self.add_to_root(idx);
    }

    fn cascading_cut(&mut self, mut idx: usize) {
        while let Some(p) = self.nodes[idx].parent {
            if !self.nodes[idx].mark {
                self.nodes[idx].mark = true;
                break;
            }
            self.cut(idx, p);
            idx = p;
        }
    }
}

// ---------------------------------------------------------------------------
// Basic coverage tests mirroring original MinHeap suite (subset)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::FibHeap;

    #[test]
    fn insert_and_get_min() {
        let mut h: FibHeap<i32> = FibHeap::new();
        h.insert((0, 20));
        h.insert((1, 10));
        assert_eq!(h.get_min(), Some(&(1, 10)));
    }

    #[test]
    fn delete_min_sequence() {
        let mut h: FibHeap<i32> = FibHeap::new();
        h.insert((3, 15));
        h.insert((2, 25));
        h.insert((5, 5));
        assert_eq!(h.delete_min(), Some((5, 5)));
        assert_eq!(h.delete_min(), Some((3, 15)));
        assert_eq!(h.delete_min(), Some((2, 25)));
        assert_eq!(h.delete_min(), None);
    }

    #[test]
    fn decrease_key_basic() {
        let mut h: FibHeap<i32> = FibHeap::new();
        h.insert((0, 100));
        h.insert((1, 200));
        h.insert((2, 300));
        h.decrease_key(2, 50);
        assert_eq!(h.get_min(), Some(&(2, 50)));
    }
}
