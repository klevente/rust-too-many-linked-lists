use std::ptr;

pub struct List<T> {
    head: Link<T>,
    // pointer to the end of the list (queue)
    tail: *mut Node<T>, // DANGER: raw pointer
}

// it is inadvisable to mix raw and 'safe' pointer types (like `Box`),
// so we'll use unsafe pointers everywhere, which can be `null`, so `Option` is not necessary
type Link<T> = *mut Node<T>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        unsafe {
            Iter {
                // `unsafe` function of converting an unsafe pointer to an `Option` of reference
                next: self.head.as_ref(),
            }
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        unsafe {
            IterMut {
                // `unsafe` function of converting an unsafe pointer to an `Option` of a mutable reference
                next: self.head.as_mut(),
            }
        }
    }

    pub fn push(&mut self, elem: T) {
        unsafe {
            // use a `Box` to create a pointer, then turn it into an unsafe one
            // with `into_raw` - the returned pointer has to be freed by us!
            let new_tail = Box::into_raw(Box::new(Node {
                elem,
                next: ptr::null_mut(), // when pushed onto the `tail`, the next is always `null`
            }));

            // `is_null` checks for null, equivalent to checking for `None`
            if !self.tail.is_null() {
                // dereferencing raw pointers must be put in an `unsafe` block,
                // other pointer operations (assignments, null-checks) are safe.
                // if the `tail` existed, update it to point to the `new_tail`
                (*self.tail).next = new_tail;
            } else {
                // otherwise, update the `head` to point to it
                self.head = new_tail;
            }

            self.tail = new_tail;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                None
            } else {
                // convert a raw pointer to a `Box`, so it is `drop`ped automatically
                let head = Box::from_raw(self.head);
                self.head = head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }

                Some(head.elem)
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        unsafe {
            // `unsafe` function of converting an unsafe pointer to an `Option` of reference
            self.head.as_ref().map(|node| &node.elem)
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe {
            // `unsafe` function of converting an unsafe pointer to an `Option` of a mutable reference
            self.head.as_mut().map(|node| &mut node.elem)
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        // go through the `List` and `pop` each element, which `drop`s all `Box`es
        // that have been created from `self.head`
        while let Some(_) = self.pop() {}
    }
}

pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.map(|node| {
                self.next = node.next.as_ref();
                &node.elem
            })
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            self.next.take().map(|node| {
                self.next = node.next.as_mut();
                &mut node.elem
            })
        }
    }
}

mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // check empty list behaves right
        assert_eq!(list.pop(), None);

        // populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }

    // `miri` is a tool for exploring Undefined Behaviour during runtime, so it can help catching
    // bugs in `unsafe` code
    // to install and run it, execute `cargo +nightly-<version> miri test`
    // for nightly version, check this page: https://rust-lang.github.io/rustup-components-history/
    // and choose the latest date available for `miri`.
    // in some cases, the appropriate toolchain also needs to be installed with:
    // `rustup toolchain add nightly-<version>`.
    // to enable additional checks relevant for this example, set the following environment variable:
    // `MIRIFLAGS="-Zmiri-tag-raw-pointers"`, or on Windows: `$env:MIRIFLAGS="-Zmiri-tag-raw-pointers"`
    #[test]
    fn miri_food() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(1));
        list.push(4);
        assert_eq!(list.pop(), Some(2));
        list.push(5);

        assert_eq!(list.peek(), Some(&3));
        list.push(6);
        list.peek_mut().map(|x| *x *= 10);
        assert_eq!(list.peek(), Some(&30));
        assert_eq!(list.pop(), Some(30));

        for elem in list.iter_mut() {
            *elem *= 100;
        }

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&400));
        assert_eq!(iter.next(), Some(&500));
        assert_eq!(iter.next(), Some(&600));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        assert_eq!(list.pop(), Some(400));
        list.peek_mut().map(|x| *x *= 10);
        assert_eq!(list.peek(), Some(&5000));
        list.push(7);

        // drop it on the ground and let `drop` exercise itself
    }
}
