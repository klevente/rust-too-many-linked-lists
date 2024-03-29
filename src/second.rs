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

    // `into_iter` consumes the original collection, hence type parameter `<T>` and taking `self` by value
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    // `iter` returns a type for iterating over the collection, the head is passed by reference to `Iter`,
    // along with taking self by a const reference (`&self`)
    // because of lifetime elision rules, the compiler assumes that `self` must live as long as `Iter`, which is correct
    pub fn iter(&self) -> Iter<T> {
        Iter {
            // `as_deref` takes the underlying value as a reference, instead of having to use
            // `as_ref`, `map` and an assortment of `*`s and `&`s to get the desired type (namely `|node| &**node`)
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
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

    pub fn peek(&self) -> Option<&T> {
        // use `as_ref` in order to not consume the `Option`, just get access to a reference to its internals
        // essentially, this results in the following conversion: `Option<T>` -> `Option<&T>`
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        // use `as_mut` to get a mutable reference to the `Option`'s internal value
        self.head.as_mut().map(|node| &mut node.elem)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();

        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take()
        }
    }
}

// tuple struct for holding the `List` converted into an `Iterator`
// these structs are useful for wrapping values simply (newtype)
pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    // as the collection is consumed, this `Iterator` yields `T`s by value
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // simply access the underlying `List` and `pop` the front element, which already returns an `Option<T>`
        self.0.pop()
    }
}

// struct for handling `iter()`, which holds a reference to the `Node` it needs to yield next, or `None`, if exhausted
pub struct Iter<'a, T> {
    // as this structs holds a reference, it must name the lifetime that reference needs to be valid for
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    // as the collection is not consumed and cannot be modified, this `Iterator` yields `T`s by const reference
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        // unwrap the value contained by the current node, alongside with moving to the next one
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        // `take` the current element in order to allow returning a mutable reference of the wrapped element
        // this also ensures that the reference to the actual element is singleton, as the `Option` is `None` after `take`
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
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

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        // test whether the value can actually be mutated
        list.peek_mut().map(|value| *value = 42);

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}
