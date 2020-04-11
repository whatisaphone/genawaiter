use genawaiter::stack::{let_gen_using, Co};

async fn producer(mut co: Co<'static, i32>) {
    co.yield_(10).await;
}

fn main() {
    let_gen_using!(gen, producer);
    let _ = gen;
}
