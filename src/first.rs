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
    next: List,
}

impl List {
    pub fn new() -> Self {
        Self { head: Link::Empty }
    }
}
