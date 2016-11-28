use std::mem;

#[derive(Clone)]
pub struct Node<K, V> {
    pub key_value: (K, V),
    pub left: Option<Box<Node<K, V>>>,
    pub right: Option<Box<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    pub fn new(k: K, v: V,
               l: Option<Box<Node<K, V>>>,
               r: Option<Box<Node<K, V>>>) -> Box<Node<K, V>> {
        Box::new(Node {
            key_value: (k, v),
            left: l,
            right: r,
        })
    }

    pub fn pop_left(&mut self) -> Option<Box<Node<K, V>>> {
        self.left.take()
    }

    pub fn take_left(&mut self) -> Box<Node<K, V>> {
        self.left.take().unwrap()
    }

    pub fn pop_right(&mut self) -> Option<Box<Node<K, V>>> {
        self.right.take()
    }

    pub fn take_right(&mut self) -> Box<Node<K, V>> {
        self.right.take().unwrap()
    }
}
