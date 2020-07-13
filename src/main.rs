use {
    std::{
        rc::Rc,
        collections::HashMap,
        cell::RefCell,
    },
};

// Shorthand for each link in the Doubly Linked List:
// - Reference Counter allows for multiple references to the same data
//      Stored in the HashMap
//      Or optionally in previous and next links
// - RefCell allows for interior mutability inside of the reference counter;
//   otherwise, only one reference can exist at a time in order for the shared
//   reference to be mutable.
//   In this implementation, the programmer verifies that there is no double
//   mutable reference.
type LinkedListUnit<T> = Rc<RefCell<DoublyLinkedList<T>>>;

// Doubly linked list used instead of Single (Forward) linked list
// so that removal of intermediate elements in the list can occur
enum DoublyLinkedList<T> {
    Nil,
    Cons(LinkedListUnit<T>, T, LinkedListUnit<T>),
}

// Separate Iterator struct provides
// forward iteration through current state of a
// LinkedHashMap
struct Iterator<K, V>
where
    K: std::hash::Hash + Clone + Eq,
    V: Clone,
{
    iter_link: LinkedListUnit<(K, V)>,
}

impl<K, V> std::iter::Iterator for Iterator<K, V>
where
    K: std::hash::Hash + Clone + Eq,
    V: Clone,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        let mut r_pair = None;
        let mut next_link = None;
        
        // Forward iteration, though backwards iteration
        // could and may be implemented later
        if let DoublyLinkedList::Cons(_, ref pair, ref link) = *self.iter_link.borrow() {
            r_pair = Some((pair.0.clone(), pair.1.clone()));
            next_link = Some(Rc::clone(link));
        };

        if let Some(next_link_unwrapped) = next_link {
            self.iter_link = next_link_unwrapped;
        }

        r_pair
    }
}

struct LinkedHashMap<K, V>
where
    // Hash + Eq traits needed for a key in the std HashMap
    // Key and Value should be cloneable when retrieving data
    K: std::hash::Hash + Clone + Eq,
    V: Clone,
{
    current_link: LinkedListUnit<(K, V)>,
    first_link: LinkedListUnit<(K, V)>,
    hashmap: HashMap<K, LinkedListUnit<(K, V)>>,
}

impl<K, V> LinkedHashMap<K, V>
where
    K: std::hash::Hash + Clone + Eq,
    V: Clone,
{
    fn new() -> LinkedHashMap<K, V> {
        let current_link = LinkedListUnit::new(RefCell::new(DoublyLinkedList::Nil));
        let first_link = Rc::clone(&current_link);

        LinkedHashMap {
            current_link,
            first_link,
            hashmap: HashMap::new(),
        }
    }

    fn insert(&mut self, key: K, val: V) {
        // State variables needed because modification of the links cannot occur when borrowing
        // them to perform pattern matching
        let mut initialized = false;
        let mut end_of_chain = None;
        let prev_link = Rc::clone(&self.current_link);

        if let DoublyLinkedList::Cons(_, _, ref chain_end) = *self.current_link.borrow() {
            initialized = true;
            end_of_chain = Some(Rc::clone(chain_end));
        } else {
            end_of_chain = Some(Rc::clone(&self.current_link));
        }

        let next_link = LinkedListUnit::new(RefCell::new(DoublyLinkedList::Cons(prev_link,
            (key.clone(), val),
            end_of_chain.unwrap())));

        self.hashmap.insert(key, Rc::clone(&next_link));

        if initialized {
            if let DoublyLinkedList::Cons(_, _, ref mut chain_end) = *self.current_link.borrow_mut() {
                *chain_end = Rc::clone(&next_link);
            }
            self.current_link = next_link;
        } else {
            self.current_link = next_link;
            self.first_link = Rc::clone(&self.current_link);
        }
    }

    fn get(&self, key: &K) -> Option<V> {
        let value_wrapped = self.hashmap.get(key);

        if let None = value_wrapped {
            return None;
        }

        if let DoublyLinkedList::Cons(_, (_, ref val), _) = *value_wrapped.unwrap().borrow() {
            return Some(val.clone());
        }

        None
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let link_wrapped = self.hashmap.get(key);

        if let None = link_wrapped {
            return None;
        }

        let mut returned_val = None;

        if let DoublyLinkedList::Cons(ref prev_link, (_, ref val), ref next_link) = *link_wrapped.unwrap().borrow() {
            returned_val = Some(val.clone());

            // If the previous link is not the beginning, then link forward, one past the removed
            // link
            if let DoublyLinkedList::Cons(_, (_, _), ref mut prev_link_next) = *prev_link.borrow_mut() {
                *prev_link_next = Rc::clone(next_link);
            // The previous link is the beginning, so set the reference to the first link one past the removed link
            } else {
                self.first_link = Rc::clone(next_link);
            }

            let mut is_current = false;

            // If the next link is not the end of the chain, then reference the next chain's
            // previous link one before the removed link
            if let DoublyLinkedList::Cons(ref mut next_link_prev, (_, _), _) = *next_link.borrow_mut() {
                *next_link_prev = Rc::clone(prev_link);
            // The next link is a Nil link,
            // so the removed link is the last one containing data.
            // Therefore, it is also the current link
            } else {
                is_current = true;
            }

            // If the removed link is the current link,
            // then set the new current link to the previous link
            if is_current {
                self.current_link = Rc::clone(prev_link);
            }
        }

        self.hashmap.remove(key);

        returned_val
    }

    // Reference to the current state of the LinkedHashMap
    fn iter(&self) -> Iterator<K, V> {
        Iterator {
            iter_link: Rc::clone(&self.first_link)
        }
    }
}

// Quick tests
fn main() {
    let mut linked_hash_map = LinkedHashMap::new();

    // Insertion
    linked_hash_map.insert("First", 5);
    linked_hash_map.insert("Second", 8);
    linked_hash_map.insert("Third", 9);
    linked_hash_map.insert("Fourth", 11);
    linked_hash_map.insert("Fifth", 15);
    linked_hash_map.insert("Sixth", 20);

    // Iteration in order of insertion
    for (key, val) in linked_hash_map.iter() {
        println!("{}: {}", key, val);
    }

    print!("\n");

    // Removal
    linked_hash_map.remove(&"Third");
    linked_hash_map.remove(&"Fourth");

    // Iteration in order of insertion
    // after removing intermediate elements
    for (key, val) in linked_hash_map.iter() {
        println!("{}: {}", key, val);
    }

    print!("\n");

    // Get a random element through the HashMap structure
    // which is best case O(1)
    println!("{}\n", linked_hash_map.get(&"Fifth").unwrap());

    // Remove all elements
    for (i, (key, _)) in linked_hash_map.iter().enumerate() {
        println!("Removing index: {}", i);
        linked_hash_map.remove(&key);
    }

    print!("\n");

    // Try to remove some non existent element
    linked_hash_map.remove(&"Garbage");

    // Print remaining elements
    // (None)
    for (_, val) in linked_hash_map.iter() {
        println!("{}", val);
    }
}
