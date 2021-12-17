use std::mem;

/// declare a `List` type only containing the `head`, so that internal types are not leaked out to users
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
