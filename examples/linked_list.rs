#![warn(future_incompatible, rust_2018_compatibility, rust_2018_idioms, unused)]
#![warn(clippy::pedantic)]
#![cfg_attr(feature = "strict", deny(warnings))]

// from the famous or infamous
// https://rust-unofficial.github.io/too-many-lists/second-final.html

use genawaiter::{rc::gen, yield_};

#[derive(Debug)]
pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    val: T,
    next: Link<T>,
}

impl<T> List<T> {
    fn new() -> Self {
        Self { head: None }
    }

    fn push(&mut self, val: T) {
        let new_head = Box::new(Node {
            val,
            next: self.head.take(),
        });
        self.head = Some(new_head);
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        let mut current = &self.head;
        gen!({
            while let Some(next) = current {
                yield_!(&next.val);
                current = &next.next;
            }
        })
        .into_iter()
    }
}

fn main() {
    let mut list = List::new();

    list.push(10);
    list.push(11);
    list.push(12);
    list.push(13);

    for x in list.iter() {
        println!("{:?}", x);
    }
}
