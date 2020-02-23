#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

use genawaiter::rc::{Co, Gen};

async fn linked_list<'a, T>(next: &'a Child<T>, co: Co<&'a T>) {
    let mut current = next;
    while let Child::Next { next, val } = current {
        co.yield_(val).await;
        current = &*next;
    }
}

#[derive(Debug, PartialEq, Eq)]
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

impl<T: PartialEq> List<T> {
    fn new() -> List<T> {
        Self { next: Child::None }
    }

    fn insert(&mut self, val: T) {
        let mut current = &mut self.next;
        while let Child::Next { next, .. } = current {
            current = &mut *next;
        }
        current.set_next(val);
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        let gen = Gen::new(|co| linked_list(&self.next, co));
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
