use genawaiter::rc::{Co, Gen};

async fn multiples_of<'a, T>(next: &'a Child<T>, co: Co<&'a T>) {
    let mut current = next;
    loop {
        if let Child::Next { next, val } = current {
            co.yield_(val).await;
            current = &*next;
        } else {
            break;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Child<T> {
    Next { next: Box<Child<T>>, val: T },
    None,
}

impl<T: PartialEq> Child<T> {
    fn new(val: T) -> Child<T> {
        Self::Next {
            next: Box::new(Child::None),
            val,
        }
    }

    fn set_next(&mut self, val: T) {
        *self = Child::new(val);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct List<T> {
    next: Child<T>,
}

impl<T: PartialEq + Clone> List<T> {
    fn new() -> List<T> {
        Self { next: Child::None }
    }

    fn insert(&mut self, val: T) {
        let mut current = &mut self.next;
        loop {
            if let Child::Next { next, .. } = current {
                current = &mut *next;
            } else {
                break;
            }
        }
        current.set_next(val);
    }
    fn iter(&self) -> impl Iterator<Item = &T> {
        let gen = Gen::new(|co| multiples_of(&self.next, co));
        gen.into_iter()
    }
}

fn main() {
    let mut list = List::new();
    list.insert(10);
    list.insert(11);
    list.insert(12);
    list.insert(13);
    println!("{:#?}", list);

    for x in list.iter() {
        println!("{:?}", x);
    }
}
