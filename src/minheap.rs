use std::usize;

pub struct MinHeap<K> {
    // an entry is an item_id and key tuple
    heap: Vec<(usize, K)>,
    //holds the position/index of an item in the heap
    positions: Vec<usize>,
}

impl<K: Ord> MinHeap<K> {
    // New minheap
    pub fn new() -> Self {
        MinHeap {
            heap: Vec::new(),
            positions: Vec::new(),
        }
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
        unimplemented!()
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

            if self.heap[index].1 < self.heap[parent].1 {
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
        while index < self.heap.len() {
            let left_child = (2 * index) + 1;
            let right_child = (2 * index) + 2;

            if self.heap[index].1 > self.heap[left_child].1 {
                self.heap.swap(index, left_child);
            }
            if self.heap[index].1 > self.heap[right_child].1 {
                self.heap.swap(index, right_child);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_once() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((0, 42));

        // After one insert:
        // - heap.len() == 1
        // - heap[0] == (0, 42)
        // - positions.len() == 1
        // - positions[0] == 0

        assert_eq!(mh.heap.len(), 1);
        assert_eq!(mh.heap[0], (0, 42));
        assert_eq!(mh.positions.len(), 1);
        assert_eq!(mh.positions[0], 0);
    }

    #[test]
    fn test_insert_twice() {
        let mut mh: MinHeap<i32> = MinHeap::new();
        mh.insert((5, 69));
        mh.insert((3, 8));

        // After one insert:
        // - heap.len() == 1
        // - heap[0] == (0, 42)
        // - positions.len() == 1
        // - positions[0] == 0

        assert_eq!(mh.heap.len(), 2);
        assert_eq!(mh.heap[0], (3, 8));
        assert_eq!(mh.heap[1], (5, 69));
        assert_eq!(mh.positions.len(), 6);
        assert_eq!(mh.positions[3], 0);
        assert_eq!(mh.positions[5], 1);
    }
}
