use std::rc::Rc;

/// This is how memory should look when using this version of `List` (persistent `List`).
/// list1 -> A ---+
///               |
///               v
/// list2 ------> B -> C -> D
///               ^
///               |
/// list3 -> X ---+
///
/// If thread safety is needed, every `Rc` just needs to be replaced with `Arc`, and it will be safe.
/// This is because `Arc` = `Rc`, it is just a bit slower because it uses `Atomic`s instead of `Cell`s for reference counting

pub struct List<T> {
    head: Link<T>,
}

/// Use `Rc` for reference counting; the underlying `Node` is freed when the last reference gets dropped
type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    /// Return a new `List` that has the provided element added to the front, the original `List` is still usable
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            // create a new `head` that is wrapped in an `Rc`
            head: Some(Rc::new(Node {
                elem,
                // clone the `Option` holding the `Rc` pointing to the next element, which increments the reference count to it,
                // so now there are 2 `List`s pointing to the same sublist, this one being the original
                next: self.head.clone(),
            })),
        }
    }

    /// Return a `List` that contains everything but the first element of this one
    pub fn tail(&self) -> List<T> {
        List {
            // clone the second element's pointer and use it as this `List`'s `head`
            // `and_then` is basically `bind` from Haskell: unwraps the underlying value then calls `f` on it, which returns an `Option`
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }

    /// Returns a reference pointing to the first element
    pub fn head(&self) -> Option<&T> {
        // extract the element out of `Link`
        self.head.as_ref().map(|node| &node.elem)
    }
}

/// `Drop` is required here as well so there is no recursive destructor problem
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // remove the `head`
        let mut head = self.head.take();
        while let Some(node) = head {
            // `try_unwrap` only returns the value only if the `Rc`'s reference count is at 1,
            // so it is sure there are no others pointing to this node, which means it can be `drop`ped
            if let Ok(mut node) = Rc::try_unwrap(node) {
                // move on tho the next `Node`
                head = node.next.take();
            } else {
                // others are still using this `Node`, so stop destructing
                break;
            }
        }
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        // if an item is present, point to the next element and return a reference to the underlying value
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }
}
