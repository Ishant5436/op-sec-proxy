use std::collections::HashMap;
use std::hash::Hash;

/// Zero-dependency, memory-safe O(1) Least-Recently-Used (LRU) cache.
/// Backed by a HashMap for O(1) lookups and an intrusive doubly-linked list
/// allocated within a contiguous slot arena for zero heap fragmentation.
#[derive(Debug, Clone)]
pub struct LruCache<K: Eq + Hash + Clone, V: Clone> {
    capacity: usize,
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,
    head: Option<usize>, // Most recently used
    tail: Option<usize>, // Least recently used
    free_head: Option<usize>,
}

#[derive(Debug, Clone)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

impl<K: Eq + Hash + Clone, V: Clone> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be non-zero");
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            free_head: None,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(&idx) = self.map.get(key) {
            self.detach(idx);
            self.attach_head(idx);
            return Some(self.nodes[idx].value.clone());
        }
        None
    }

    #[allow(dead_code)]
    pub fn peek(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|&idx| &self.nodes[idx].value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.detach(idx);
            self.attach_head(idx);
            return;
        }

        let idx = if self.map.len() >= self.capacity {
            // Evict tail (least recently used)
            let evict_idx = self.tail.expect("Tail must exist when capacity reached");
            let evict_key = self.nodes[evict_idx].key.clone();
            self.map.remove(&evict_key);
            self.detach(evict_idx);
            evict_idx
        } else if let Some(free_idx) = self.free_head {
            self.free_head = self.nodes[free_idx].next;
            free_idx
        } else {
            let new_idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value: value.clone(),
                prev: None,
                next: None,
            });
            new_idx
        };

        self.nodes[idx].key = key.clone();
        self.nodes[idx].value = value;
        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;

        self.map.insert(key, idx);
        self.attach_head(idx);
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn detach(&mut self, idx: usize) {
        let prev = self.nodes[idx].prev;
        let next = self.nodes[idx].next;

        if let Some(p) = prev {
            self.nodes[p].next = next;
        } else {
            self.head = next;
        }

        if let Some(n) = next {
            self.nodes[n].prev = prev;
        } else {
            self.tail = prev;
        }

        self.nodes[idx].prev = None;
        self.nodes[idx].next = None;
    }

    fn attach_head(&mut self, idx: usize) {
        self.nodes[idx].next = self.head;
        self.nodes[idx].prev = None;

        if let Some(h) = self.head {
            self.nodes[h].prev = Some(idx);
        } else {
            self.tail = Some(idx);
        }

        self.head = Some(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_insertion_and_eviction() {
        let mut lru = LruCache::<u32, &'static str>::new(3);
        lru.insert(1, "one");
        lru.insert(2, "two");
        lru.insert(3, "three");
        assert_eq!(lru.len(), 3);

        // Access key 1 -> promotes 1 to MRU, making 2 the LRU
        assert_eq!(lru.get(&1), Some("one"));

        // Insert key 4 -> should evict key 2
        lru.insert(4, "four");
        assert_eq!(lru.get(&2), None);
        assert_eq!(lru.get(&1), Some("one"));
        assert_eq!(lru.get(&3), Some("three"));
        assert_eq!(lru.get(&4), Some("four"));
    }
}
