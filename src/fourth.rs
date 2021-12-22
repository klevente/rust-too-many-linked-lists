use std::cell::RefCell;
use std::rc::Rc;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

/// `RefCell` is a type that enforces borrowing at runtime. If any rules are broken, it `panic`s.
/// This means that the value inside can be `borrow`ed any number of times, but can only be borrowed
/// once using `borrow_mut`.
/// `Rc` is used here as well to enable persistent functionality, meaning that multiple `List`s can
/// re-use the same part of the list for better memory usage and reducing the number of copies made.
type Link<T> = Option<Rc<RefCell<Node<T>>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem,
            prev: None,
            next: None,
        }))
    }
}

/// Invariant to keep in mind when writing all `List` operations: each `Node` should have exactly 2 pointers to it.
/// `Node`s in the middle are pointed by their predecessor and successor, while the `Node`s on the end are
/// pointed by their sole neighbour and the `List` itself.
/// `List`s having only one element point both their pointers to the sole element.
impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        // new `Node` needs +2 links, while everything else should be +0
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                // non-empty `List`, need to connect `old_head` to `new_head` and vice-versa
                // use `borrow_mut` to access the underlying `Node` in a mutable way
                old_head.borrow_mut().prev = Some(new_head.clone()); // +1 `new_head`
                new_head.borrow_mut().next = Some(old_head); // +1 `old_head`
                self.head = Some(new_head); // +1 `new_head`, -1 `old_head`
                                            // total: +2 `new_head`, +0 `old_head`
            }
            None => {
                // empty `List`, need to set the `head` and `tail`
                self.tail = Some(new_head.clone()); // +1 `new_head`
                self.head = Some(new_head); // +1 `new_head`
                                            // total: +2 `new_head`
            }
        }
    }
}
