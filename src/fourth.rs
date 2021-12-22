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

    pub fn pop_front(&mut self) -> Option<T> {
        // need to take the `old_head` off the beginning, ensuring it is -2
        // -1 `old_head`
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                // `List` that has not emptied by taking out `head`
                // -1 `new_head`
                Some(new_head) => {
                    new_head.borrow_mut().prev.take(); // -1 `old_head`
                    self.head = Some(new_head); // +1 `new_head`
                                                // total: -2 `old_head`, +0 `new_head`
                }
                None => {
                    // `List` that has become empty by taking out `head`
                    self.tail.take(); // -1 `old_head`
                                      // total: -2 `old_head`, +0 `new_head`
                }
            }
            // `try_unwrap` is required so the underlying `Refcell<Node<T>>` is moved out of the pointer,
            // this should always succeed, as the program is written correctly, i.e. the variable named `old_head`
            // is the last one referencing the data held by this `Rc`, so it can safely unwrap it.
            // as `try_unwrap` returns a `Result`, it must be `unwrap`ped, but `unwrap` requires that the
            // underlying value implements the `Debug` trait; while `RefCell` does, our `Node<T`> does not,
            // as it would required `T` to be `Debug`, which is an unnecessary constraint, hence the `Result`
            // is converted into an `Option` using `ok`.
            // after this, the resulting `RefCell` is consumed using `into_inner`, which returns the
            // value that is contained by it, so finally, the element can be safely moved out to the caller
            Rc::try_unwrap(old_head).ok().unwrap().into_inner().elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // push some more just to make sure nothing is corrupted
        list.push_front(4);
        list.push_front(5);

        // check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }
}
