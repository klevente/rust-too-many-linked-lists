use std::mem;

/// Declare a `List` type only containing the `head`, so that internal types are not leaked out to users
pub struct List {
    head: Link,
}

/// When an `enum` is defined like this, as in one element is empty, while the other has a non-null pointer in it,
/// the compiler performs null-pointer optimization, meaning that it essentially just stores the pointer,
/// which will be `0` if the `enum` is currently in the `Empty` state.
/// This results in the fact that the `Option` type can hold `&`, `&mut`, `Box`, `Rc`, `Arc`, `Vec`, ... types
/// without any overhead whatsoever!
enum Link {
    Empty,
    More(Box<Node>),
}

struct Node {
    elem: i32,
    next: Link,
}

impl List {
    pub fn new() -> Self {
        Self { head: Link::Empty }
    }

    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem,
            // temporarily replace `self.head` with `Empty`, while returning the old value to `next`,
            // so that the newly added `Node` points to the rest of the list
            next: mem::replace(&mut self.head, Link::Empty),
        });
        // link up `head` to point to the newly added `Node`
        self.head = Link::More(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        // temporarily replace `self.head` with `Empty`, so the head of the list can be moved out by value,
        // as it needs to be removed from the list
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}

impl Drop for List {
    fn drop(&mut self) {
        // replace the `head` with `Empty` and get the actual head of the list
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        // go through each element until the end
        while let Link::More(mut boxed_node) = cur_link {
            // move out the current `Node` and replace it with `Empty`
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
            // `boxed_node` goes out of scope here, which means it gets `drop`ped
            // as its internal contents have been replaced with `Empty`, no recursion occurs during `drop`ping

            // by resorting to the compiler's `Drop` implementation, unbounded recursion could occur,
            // which can overflow the stack
        }
    }
}

/// This indicates that the `test` module should only be compiled when running tests
#[cfg(test)]
mod test {
    // `List` needs to be explicitly pulled in from the parent module
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // check that empty list behaves right
        assert_eq!(list.pop(), None);

        // populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}
