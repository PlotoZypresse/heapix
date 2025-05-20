mod fibonacci_heap;
mod minheap;
pub use fibonacci_heap::FibHeap;
pub use minheap::MinHeap;

#[cfg(test)]
mod tests {

    use super::FibHeap;
    use super::MinHeap;

    #[test]
    fn basic_minheap_ops() {
        let mut h = MinHeap::new();
        h.insert((0, 10));
        h.insert((1, 5));
        assert_eq!(*h.get_min().unwrap(), (1, 5));
        let popped = h.delete_min().unwrap();
        assert_eq!(popped, (1, 5));
        assert_eq!(*h.get_min().unwrap(), (0, 10));
    }
    #[test]
    fn basic_fibheap_ops() {
        let mut f = FibHeap::new();
        f.insert((0, 10));
        f.insert((1, 5));
        assert_eq!(*f.get_min().unwrap(), (1, 5));
        let popped = f.delete_min().unwrap();
        assert_eq!(popped, (1, 5));
        assert_eq!(*f.get_min().unwrap(), (0, 10));
    }
}
