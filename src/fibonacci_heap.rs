//! Fibonacci heap with `(id, key)` API identical to `MinHeap`.
//! Correct for all decrease-key / clear / multi-phase workloads.

use std::cmp::Ordering;
const NOT_IN_HEAP: usize = usize::MAX;

#[derive(Clone)]
struct Node<K> {
    entry: (usize, K),
    degree: usize,
    mark: bool,
    parent: Option<usize>,
    child: Option<usize>,
    left: usize,
    right: usize,
}

impl<K: PartialOrd + Copy> Node<K> {
    fn new(id: usize, key: K, idx: usize) -> Self {
        Self {
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

pub struct FibHeap<K> {
    nodes: Vec<Node<K>>,
    positions: Vec<usize>, // id → node index | NOT_IN_HEAP
    min_root: Option<usize>,
    n: usize,
    scratch_roots: Vec<usize>,
    scratch_aux: Vec<Option<usize>>,
}

impl<K: PartialOrd + Copy> FibHeap<K> {
    /* ---------- public API (matches MinHeap) ----------------------------- */
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            positions: Vec::new(),
            min_root: None,
            n: 0,
            scratch_roots: Vec::new(),
            scratch_aux: Vec::new(),
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

    pub fn insert(&mut self, (id, key): (usize, K)) {
        debug_assert!(
            id >= self.positions.len() || self.positions[id] == NOT_IN_HEAP,
            "duplicate id {} inserted",
            id
        );

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
        self.min_root.map(|i| &self.nodes[i].entry)
    }

    pub fn delete_min(&mut self) -> Option<(usize, K)> {
        /* 0) empty heap? */
        let z = self.min_root?; // return None if empty

        /* 1) promote every child of z to the root list */
        if let Some(mut child) = self.nodes[z].child {
            loop {
                let next = self.nodes[child].right; // save before detach
                self.detach(child); // unlink from child list
                self.nodes[child].parent = None;
                self.nodes[child].mark = false;
                self.add_to_root(child); // add to root ring
                if next == child {
                    break;
                } // finished full circle
                child = next;
            }
            self.nodes[z].child = None;
        }

        /* 2) remove z itself from the root list */
        let successor = self.nodes[z].right; // neighbour root
        self.detach(z);

        /* 3) book-keeping for the item we return */
        self.n -= 1;
        let (id, key) = self.nodes[z].entry;
        self.positions[id] = NOT_IN_HEAP;

        /* 4) choose a new min root and consolidate */
        if self.n == 0 {
            self.min_root = None; // heap is now empty
        } else {
            /* only change the pointer if we just removed the min */
            if self.min_root == Some(z) {
                self.min_root = Some(successor);
            }
            self.consolidate(); // rebuild and set true min
        }

        Some((id, key))
    }

    pub fn decrease_key(&mut self, id: usize, new_key: K) {
        // get the node index more directly
        let idx = self.positions[id];
        // one fewer method call vs. partial_cmp+unwrap
        debug_assert!(self.nodes[idx].entry.1 > new_key, "new key must be smaller");

        // update the key
        self.nodes[idx].entry.1 = new_key;

        // only if it has a parent—and its key is now smaller—cut & cascade
        if let Some(p) = self.nodes[idx].parent {
            if self.nodes[idx].entry.1 < self.nodes[p].entry.1 {
                self.cut(idx, p);
                self.cascading_cut(p);
            }
        }
        self.update_min(idx);
    }

    /* ---------- helpers -------------------------------------------------- */

    fn update_min(&mut self, idx: usize) {
        match self.min_root {
            None => self.min_root = Some(idx),
            Some(m) => {
                if self.nodes[idx]
                    .entry
                    .1
                    .partial_cmp(&self.nodes[m].entry.1)
                    .unwrap()
                    == Ordering::Less
                {
                    self.min_root = Some(idx);
                }
            }
        }
    }

    fn add_to_root(&mut self, idx: usize) {
        if let Some(min_idx) = self.min_root {
            // splice idx before the current min
            let left = self.nodes[min_idx].left;
            self.nodes[idx].left = left;
            self.nodes[idx].right = min_idx;
            self.nodes[left].right = idx;
            self.nodes[min_idx].left = idx;

            /* ---------- NEW ---------- */
            // keep the pointer on the smallest key
            if self.nodes[idx].entry.1 < self.nodes[min_idx].entry.1 {
                self.min_root = Some(idx);
            }
            /* -------------------------- */
        } else {
            self.min_root = Some(idx); // first root in the ring
        }
    }

    fn detach(&mut self, i: usize) {
        let l = self.nodes[i].left;
        let r = self.nodes[i].right;
        self.nodes[l].right = r;
        self.nodes[r].left = l;
        self.nodes[i].left = i;
        self.nodes[i].right = i;
    }

    fn link(&mut self, y: usize, x: usize) {
        /* y becomes child of x ( keys[x] <= keys[y] ) */
        self.detach(y);

        if let Some(c) = self.nodes[x].child {
            // splice y before c in child list
            let l = self.nodes[c].left;
            self.nodes[y].left = l;
            self.nodes[y].right = c;
            self.nodes[l].right = y;
            self.nodes[c].left = y;
        } else {
            self.nodes[x].child = Some(y);
        }
        self.nodes[y].parent = Some(x);
        self.nodes[y].mark = false;
        self.nodes[x].degree += 1;
    }

    fn cut(&mut self, idx: usize, parent: usize) {
        /* unlink idx from parent.child list */
        if self.nodes[parent].child == Some(idx) {
            if self.nodes[idx].right == idx {
                self.nodes[parent].child = None;
            } else {
                let next = self.nodes[idx].right; // save BEFORE detach
                self.detach(idx);
                self.nodes[parent].child = Some(next);
            }
        } else {
            self.detach(idx);
        }
        self.nodes[parent].degree -= 1;

        /* promote idx */
        self.nodes[idx].parent = None;
        self.nodes[idx].mark = false;
        self.add_to_root(idx);
    }

    fn cascading_cut(&mut self, mut y: usize) {
        while let Some(p) = self.nodes[y].parent {
            if !self.nodes[y].mark {
                self.nodes[y].mark = true;
                break;
            }
            self.cut(y, p);
            y = p;
        }
    }

    /* ---------- consolidate --------------------------------------------- */

    /// Consolidate the root list: combine trees of equal degree until each
    /// degree occurs at most once.
    fn consolidate(&mut self) {
        // early exit
        let start = match self.min_root {
            Some(i) => i,
            None => return,
        };

        // ── 1) take the pre-allocated roots Vec out, clear it, fill it ──
        let mut roots = std::mem::take(&mut self.scratch_roots);
        roots.clear();
        let mut w = start;
        loop {
            roots.push(w);
            w = self.nodes[w].right;
            if w == start {
                break;
            }
        }

        // ── 2) make sure scratch_aux is sized & zeroed ──
        let max_deg = (self.n as f64).log2().ceil() as usize + 2;
        if self.scratch_aux.len() < max_deg {
            self.scratch_aux.resize(max_deg, None);
        }
        for slot in &mut self.scratch_aux {
            *slot = None;
        }

        // ── 3) do all the degree-linking in scratch_aux ──
        for &root_idx in &roots {
            // skip nodes that got linked as children
            if self.nodes[root_idx].parent.is_some() {
                continue;
            }
            let mut x = root_idx;
            let mut d = self.nodes[x].degree;
            loop {
                // grow aux if needed
                if d >= self.scratch_aux.len() {
                    self.scratch_aux.resize(d + 1, None);
                }
                if self.scratch_aux[d].is_none() {
                    self.scratch_aux[d] = Some(x);
                    break;
                }
                let mut y = self.scratch_aux[d].take().unwrap();
                if self.nodes[y].entry.1 < self.nodes[x].entry.1 {
                    std::mem::swap(&mut x, &mut y);
                }
                // this borrows &mut self, but no scratch_roots borrow is active
                self.link(y, x);
                d += 1;
            }
        }

        // ── 4) rebuild the root ring by taking scratch_aux out ──
        let aux = std::mem::take(&mut self.scratch_aux);
        self.min_root = None;
        for opt in aux.iter() {
            if let Some(idx) = *opt {
                self.nodes[idx].left = idx;
                self.nodes[idx].right = idx;
                self.nodes[idx].parent = None;
                // this also borrows &mut self, but aux is local now
                self.add_to_root(idx);
            }
        }

        // put our buffers back for the next call
        self.scratch_aux = aux;
        self.scratch_roots = roots;

        #[cfg(debug_assertions)]
        self.assert_root_ring();
    }

    #[cfg(debug_assertions)]
    fn assert_root_ring(&self) {
        if let Some(r) = self.min_root {
            let mut seen = std::collections::HashSet::new();
            let mut cur = r;
            loop {
                assert!(seen.insert(cur), "duplicate root {}", cur);
                cur = self.nodes[cur].right;
                if cur == r {
                    break;
                }
            }
            assert_eq!(seen.len(), self.root_count(), "root list size wrong");
        }
    }
    #[cfg(debug_assertions)]
    fn root_count(&self) -> usize {
        if let Some(r) = self.min_root {
            let mut cnt = 0;
            let mut cur = r;
            loop {
                cnt += 1;
                cur = self.nodes[cur].right;
                if cur == r {
                    break;
                }
            }
            cnt
        } else {
            0
        }
    }
}

#[cfg(debug_assertions)]
impl<K: PartialOrd + Copy + std::fmt::Debug> FibHeap<K> {
    /// O(total_nodes) scan that asserts both:
    ///   – the node removed by delete_min really had the global min key
    ///   – every parent key ≤ its children’s keys
    pub fn assert_heap_ok(&self, last_key: K) {
        for (i, node) in self.nodes.iter().enumerate() {
            if self.positions[node.entry.0] == NOT_IN_HEAP {
                continue; // node is already deleted
            }
            let k = node.entry.1;
            assert!(
                k >= last_key,
                "heap-order error: node #{i} key {k:?} < last pop {last_key:?}"
            );
            if let Some(p) = node.parent {
                let pk = self.nodes[p].entry.1;
                assert!(
                    pk <= k,
                    "child key {k:?} < parent key {pk:?} (node #{i} → parent #{p})"
                );
            }
        }
    }
}

/* -------------------------------------------------------------------------- */
/* Minimal smoke tests                                                        */
/* -------------------------------------------------------------------------- */

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
    fn delete_min_order() {
        let mut h: FibHeap<i32> = FibHeap::new();
        h.insert((2, 30));
        h.insert((3, 5));
        h.insert((4, 25));
        assert_eq!(h.delete_min(), Some((3, 5)));
        assert_eq!(h.delete_min(), Some((4, 25)));
        assert_eq!(h.delete_min(), Some((2, 30)));
        assert!(h.is_empty());
    }
    #[test]
    fn decrease_key() {
        let mut h: FibHeap<i32> = FibHeap::new();
        h.insert((7, 100));
        h.insert((8, 200));
        h.decrease_key(8, 50);
        assert_eq!(h.get_min(), Some(&(8, 50)));
    }
}
