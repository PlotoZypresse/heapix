//! Fibonacci heap with `(id, key)` API identical to `MinHeap`.
//! Correct for all decrease-key / clear / multi-phase workloads.

use std::cmp::Ordering;
const NOT_IN_HEAP: usize = usize::MAX;

/* -------------------------------------------------------------------------- */
/* Node                                                                      */
/* -------------------------------------------------------------------------- */

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

/* -------------------------------------------------------------------------- */
/* Heap                                                                       */
/* -------------------------------------------------------------------------- */

pub struct FibHeap<K> {
    nodes: Vec<Node<K>>,
    positions: Vec<usize>, // id → node index | NOT_IN_HEAP
    min_root: Option<usize>,
    n: usize,
}

impl<K: PartialOrd + Copy> FibHeap<K> {
    /* ---------- public API (matches MinHeap) ----------------------------- */
    pub fn new() -> Self {
        Self {
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

    pub fn insert(&mut self, (id, key): (usize, K)) {
        assert!(
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
        let z = self.min_root?; // empty ⇢ None

        /* 1) promote children */
        if let Some(child0) = self.nodes[z].child {
            let mut c = child0;
            loop {
                let next = self.nodes[c].right;
                self.detach(c);
                self.nodes[c].parent = None;
                self.nodes[c].mark = false;
                self.add_to_root(c);
                if next == child0 {
                    break;
                }
                c = next;
            }
            self.nodes[z].child = None;
        }

        /* 2) remove z itself from root list */
        let succ = self.nodes[z].right; // save before detach
        self.detach(z);

        self.n -= 1;
        let (id, key) = self.nodes[z].entry;
        self.positions[id] = NOT_IN_HEAP;

        if self.n == 0 {
            self.min_root = None;
        } else {
            self.min_root = Some(succ);
            self.consolidate();
        }
        Some((id, key))
    }

    pub fn decrease_key(&mut self, id: usize, new_key: K) {
        let idx = *self.positions.get(id).unwrap_or(&NOT_IN_HEAP);
        assert!(idx != NOT_IN_HEAP, "id {} not in heap", id);
        assert!(
            self.nodes[idx].entry.1.partial_cmp(&new_key).unwrap() == Ordering::Greater,
            "new key must be smaller"
        );

        self.nodes[idx].entry.1 = new_key;

        if let Some(p) = self.nodes[idx].parent {
            if self.nodes[idx]
                .entry
                .1
                .partial_cmp(&self.nodes[p].entry.1)
                .unwrap()
                == Ordering::Less
            {
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
        if let Some(r) = self.min_root {
            let l = self.nodes[r].left;
            self.nodes[idx].left = l;
            self.nodes[idx].right = r;
            self.nodes[l].right = idx;
            self.nodes[r].left = idx;
        } else {
            self.min_root = Some(idx);
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
        let Some(start) = self.min_root else { return };

        /* ---------- 1. gather all current roots ---------- */
        let mut roots = Vec::new();
        let mut w = start;
        loop {
            roots.push(w);
            w = self.nodes[w].right;
            if w == start {
                break;
            }
        }

        /* aux[d] will hold at most one root of degree d */
        let max_deg = (self.n as f64).log2().ceil() as usize + 2;
        let mut aux: Vec<Option<usize>> = vec![None; max_deg];

        /* ---------- 2. meld equal-degree trees ---------- */
        /* meld equal-degree trees */
        /* meld equal-degree trees */
        for mut x in roots {
            if self.nodes[x].parent.is_some() {
                continue; // became child earlier
            }
            let mut d = self.nodes[x].degree;
            loop {
                /* ---------- NEW guard ---------- */
                if d >= aux.len() {
                    aux.resize(d + 1, None);
                }
                /* -------------------------------- */

                if aux[d].is_none() {
                    aux[d] = Some(x);
                    break;
                }

                let mut y = aux[d].take().unwrap();
                if self.nodes[y].entry.1 < self.nodes[x].entry.1 {
                    std::mem::swap(&mut x, &mut y); // keep smaller key in x
                }
                self.link(y, x); // y becomes child of x
                d += 1; // x’s degree just grew
            }
        }

        /* ---------- 3. rebuild ONE fresh root ring ---------- */
        self.min_root = None; // we’ll discover the new min
        for idx in aux.into_iter().flatten() {
            // isolate the node first
            self.nodes[idx].left = idx;
            self.nodes[idx].right = idx;
            self.nodes[idx].parent = None;

            // splice into root ring & update minimum pointer
            self.add_to_root(idx);
            self.update_min(idx);
        }

        #[cfg(debug_assertions)]
        self.assert_root_ring(); // O(roots) sanity check
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
