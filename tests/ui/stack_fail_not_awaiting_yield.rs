use genawaiter::{stack::Co};

#[allow(unused_variables)]
async fn wrong(mut co: Co<i32>) {
    let foo = co.yield_(10);
    let bar = co.yield_(20);
}

fn main() {

}
