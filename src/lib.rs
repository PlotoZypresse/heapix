mod minheap;
pub use minheap::MinHeap;

#[cfg(test)]
mod tests {
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
}
