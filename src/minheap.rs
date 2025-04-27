use std::cmp::Ordering;
use std::usize;

pub struct MinHeap<K> {
    // an entry is an item_id and key tuple
    heap: Vec<(usize, K)>,
    //holds the position/index of an item in the heap
    positions: Vec<usize>,
}

impl<K: PartialOrd + Copy> MinHeap<K> {
    // New minheap
    pub fn new() -> Self {
        MinHeap {
            heap: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    // build min heap from an unsorted vec of (item_id, key)
    pub fn build_heap(items: Vec<(usize, K)>) -> Self {
        let heap = items;

        // find size of positions array
        let pos_max = heap.iter().map(|(id, _)| *id).max().unwrap_or(0);

        let mut positions = vec![usize::MAX; pos_max + 1];

        // create positions so that position[id] is the items index in the heap
        for (idx, (id, _)) in heap.iter().enumerate() {
            positions[*id] = idx;
        }

        // create a MinHeap instance
        let mut min_heap = MinHeap { heap, positions };

        let n = min_heap.heap.len();
        if n > 1 {
            for i in (0..=(n / 2 - 1)).rev() {
                min_heap.bubble_down(i);
            }
        }

        min_heap
    }

    // inserts a value and moves it to the right place
    pub fn insert(&mut self, item: (usize, K)) {
        // add item to the heap
        self.heap.push(item);

        // get the items index to store its position in the position array
        let idx = self.heap.len() - 1;

        // get the id of an item and only get the id, disregarding the key
        let id = self.heap[idx].0;

        if id >= self.positions.len() {
            self.positions.resize(id + 1, usize::MAX);
        }

        // the item (id) is at heap index (idx)
        self.positions[id] = idx;

        // recreate heap order
        self.bubble_up(idx)
    }

    pub fn delete_min(&mut self) -> Option<(usize, K)> {
        // delete min hep item
        //
        //return min item using get min
        //swap min item with last item in the heap
        //remove last item from heap
        //bubble down the swapped item to its right place.
        if self.heap.is_empty() {
            return None;
        }

        // index of last item in the heap
        let last_item = self.heap.len() - 1;
        // swap min item with last item
        self.heap.swap(0, last_item);

        let (min_id, min_key) = self.heap.pop().unwrap();

        self.positions[min_id] = usize::MAX;

        if !self.heap.is_empty() {
            let root_id = self.heap[0].0;
            self.positions[root_id] = 0;

            self.bubble_down(0);
        }

        return Some((min_id, min_key));
    }

    pub fn get_min(&self) -> Option<&(usize, K)> {
        // return min item
        self.heap.get(0)
    }

    // bubble up an item
    pub fn bubble_up(&mut self, mut index: usize) {
        // swap child with parent until root is reached or min heap property holds
        while index > 0 {
            let parent = (index - 1) / 2;

            if self.heap[index]
                .1
                .partial_cmp(&self.heap[parent].1)
                .unwrap()
                == Ordering::Less
            {
                // swap child and parent
                self.heap.swap(index, parent);

                // updatre positions for child and parent
                let child_id = self.heap[index].0;
                let parent_id = self.heap[parent].0;
                self.positions[child_id] = index;
                self.positions[parent_id] = parent;

                //update parent
                index = parent;
            } else {
                break;
            }
        }
    }

    // bubble an item down
    pub fn bubble_down(&mut self, mut index: usize) {
        let heap_len = self.heap.len();

        loop {
            let left_child = (2 * index) + 1;
            let right_child = (2 * index) + 2;
            //check if there is a left child
            if left_child >= heap_len {
                // no children
                break;
            }
            // check which child is smaller
            let smaller_child: usize;
            if right_child < heap_len
                && self.heap[right_child]
                    .1
                    .partial_cmp(&self.heap[left_child].1)
                    .unwrap()
                    == Ordering::Less
            {
                smaller_child = right_child;
            } else {
                smaller_child = left_child;
            }

            // if the smallest child is smaller than the current swap
            if self.heap[smaller_child]
                .1
                .partial_cmp(&self.heap[index].1)
                .unwrap()
                == Ordering::Less
            {
                let child_id = self.heap[smaller_child].0;
                let parent_id = self.heap[index].0;

                self.heap.swap(smaller_child, index);

                self.positions[parent_id] = smaller_child;
                self.positions[child_id] = index;

                index = smaller_child;
            } else {
                break;
            }
        }
    }

    pub fn decrease_key(&mut self, id: usize, new_key: K) {
        let pos_id = self.positions[id];
        self.heap[pos_id].1 = new_key;
        self.bubble_up(pos_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_once() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((0, 42));
        assert_eq!(mh.heap.len(), 1);
        assert_eq!(mh.heap[0], (0, 42));
        assert_eq!(mh.positions.len(), 1);
        assert_eq!(mh.positions[0], 0);
    }

    #[test]
    fn test_insert_bubble_up() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((5, 69));
        mh.insert((3, 8));
        assert_eq!(mh.heap, vec![(3, 8), (5, 69)]);
        assert_eq!(mh.positions[3], 0);
        assert_eq!(mh.positions[5], 1);
    }

    #[test]
    fn test_get_min_peek_only() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((0, 20));
        mh.insert((1, 10));
        let first = *mh.get_min().unwrap();
        let second = *mh.get_min().unwrap();
        assert_eq!(first, (1, 10));
        assert_eq!(second, (1, 10));
        assert_eq!(mh.heap.len(), 2);
    }

    #[test]
    fn test_delete_min_basic() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((0, 20));
        mh.insert((1, 10));
        mh.insert((2, 30));
        let min = mh.delete_min().unwrap();
        assert_eq!(min, (1, 10));
        assert_eq!(*mh.get_min().unwrap(), (0, 20));
        assert_eq!(mh.positions[0], 0);
        assert_eq!(mh.positions[2], 1);
    }

    #[test]
    fn test_delete_min_empty() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        assert!(mh.delete_min().is_none());
    }

    #[test]
    fn test_bubble_up_manual() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.heap = vec![(0, 10), (1, 5)];
        mh.positions = vec![0, 1];
        mh.bubble_up(1);
        assert_eq!(mh.heap, vec![(1, 5), (0, 10)]);
        assert_eq!(mh.positions, vec![1, 0]);
    }

    #[test]
    fn test_bubble_down_manual() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.heap = vec![(0, 50), (1, 20), (2, 30)];
        mh.positions = vec![0, 1, 2];
        mh.bubble_down(0);
        assert_eq!(mh.heap[0], (1, 20));
        assert_eq!(mh.positions[1], 0);
        assert_eq!(mh.positions[0], 1);
    }

    #[test]
    fn test_mixed_operations() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((3, 15));
        mh.insert((2, 25));
        mh.insert((5, 5));
        assert_eq!(*mh.get_min().unwrap(), (5, 5));
        let order: Vec<_> = (0..3).map(|_| mh.delete_min().unwrap()).collect();
        assert_eq!(order, vec![(5, 5), (3, 15), (2, 25)]);
        assert!(mh.delete_min().is_none());
    }

    #[test]
    fn test_build_heap() {
        let items = vec![(2, 50), (0, 10), (3, 20), (1, 5)];
        let mut mh = MinHeap::build_heap(items);
        assert_eq!(*mh.get_min().unwrap(), (1, 5));
        assert_eq!(mh.positions[1], 0);
        assert_eq!(mh.heap.len(), 4);
        let order: Vec<_> = (0..4).map(|_| mh.delete_min().unwrap()).collect();
        assert_eq!(order, vec![(1, 5), (0, 10), (3, 20), (2, 50)]);
        assert!(mh.delete_min().is_none());
    }

    #[test]
    fn test_is_empty() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        assert!(mh.is_empty());
        mh.insert((0, 100));
        assert!(!mh.is_empty());
        mh.delete_min();
        assert!(mh.is_empty());
    }

    #[test]
    fn test_decrease_key() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((0, 100));
        mh.insert((1, 200));
        mh.insert((2, 300));
        assert_eq!(*mh.get_min().unwrap(), (0, 100));
        mh.decrease_key(2, 50);
        assert_eq!(*mh.get_min().unwrap(), (2, 50));
        assert_eq!(mh.positions[2], 0);
    }

    #[test]
    fn test_float_keys_basic() {
        let mut mh: MinHeap<f64> = MinHeap::new();
        mh.insert((0, 3.14));
        mh.insert((1, 2.71));
        mh.insert((2, -1.0));

        // The minimum should be the -1.0 entry
        assert_eq!(mh.get_min(), Some(&(2, -1.0)));

        // Deleting all in order should yield increasing keys
        let mut seq = Vec::new();
        while let Some((id, key)) = mh.delete_min() {
            seq.push((id, key));
        }
        assert_eq!(seq, vec![(2, -1.0), (1, 2.71), (0, 3.14),]);
    }

    #[test]
    fn test_float_decrease_key() {
        let mut mh: MinHeap<f64> = MinHeap::new();
        mh.insert((0, 10.0));
        mh.insert((1, 20.0));

        // Lower the key of id=1 below id=0’s key
        mh.decrease_key(1, 5.0);
        assert_eq!(mh.get_min(), Some(&(1, 5.0)));

        // And deleting-min should respect that
        let first = mh.delete_min().unwrap();
        let second = mh.delete_min().unwrap();
        assert_eq!(first, (1, 5.0));
        assert_eq!(second, (0, 10.0));
    }
}
