use std::mem;

pub struct List<T> {
    head: Link<T>,
}

/// As `Link` is basically an `Option`, use it instead of reinventing the wheel
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            // `take` is the same as `mem::replace`, but more idiomatic, i.e it moves out the value
            // contained by the `Option`, leaving a `None` in its place
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        // use `map` to apply a function to the inner value if it is available, i.e. `Some(v)`
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, None);

        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take()
        }
    }
}

#[cfg(test)]
mod test {

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
