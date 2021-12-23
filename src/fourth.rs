use std::cell::{Ref, RefCell, RefMut};
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

    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(new_tail.clone());
                new_tail.borrow_mut().prev = Some(old_tail);
                self.tail = Some(new_tail);
            }
            None => {
                self.head = Some(new_tail.clone());
                self.tail = Some(new_tail);
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

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.borrow_mut().prev.take() {
                Some(new_tail) => {
                    new_tail.borrow_mut().next.take();
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head.take();
                }
            }
            Rc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }

    /// `RefCell`s return a type called `Ref` when calling `borrow`, which keeps track of when the current borrow
    /// should be `drop`ped, this function cannot return `Option<&T>`, as the resulting `Ref` coming from `borrow` would
    /// get `drop`ped inside this function, invalidating the underlying shared reference.
    /// Instead, an `Option<Ref<T>>` can be returned, which can be dereferenced the same ways a a `&T`, as
    /// it also implements the `Deref` trait. For this, the function `Ref::map` can be used, which creates a new
    /// `Ref` instance that holds the value defined by a mapping function it requires, which can extract data
    /// from the passed in `Ref<T>`, converting it to `Ref<U>`, which is connected to the same `RefCell` as
    /// the original `Ref<T>`, which is exactly what is needed in this case.
    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head
            .as_ref()
            // create a `borrow` for the underlying `Node`, and map it so only the `elem` is visible to the caller
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.elem))
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head
            .as_ref()
            .map(|node| RefMut::map(node.borrow_mut(), |node| &mut node.elem))
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // keep removing the `head` of the `List` until there is nothing left. after each removal,
        // the `Node` the appropriate reference counts decrement, which eventually lead to the whole
        // `List` get freed appropriately. This implementation is important, as otherwise,
        // the reference counts of `Rc`s would be stuck at 1 because they would be pointing at each other
        while self.pop_front().is_some() {}
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

        // check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);

        // ---- back -----

        // check empty list behaves right
        assert_eq!(list.pop_back(), None);

        // populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));

        // push some more just to make sure nothing is corrupted
        list.push_back(4);
        list.push_back(5);

        // check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }
}
