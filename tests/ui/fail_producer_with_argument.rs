use genawaiter::stack::producer_fn;

#[producer_fn(u8)]
async fn odds(co: Co<'_, u8>) {
    co.yield_(10).await;
}

fn main() {}
