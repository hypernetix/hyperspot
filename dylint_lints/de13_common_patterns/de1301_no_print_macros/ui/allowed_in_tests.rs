fn helper() {}

fn main() {
    helper();
}

#[test]
fn unit_test_allows_prints() {
    println!("hello from test");
    eprintln!("hello from test");
    dbg!(42);
}

#[tokio::test]
async fn tokio_test_allows_prints() {
    println!("hello from tokio test");
}

#[cfg(test)]
mod nested_tests {
    #[test]
    fn nested_test_allows_prints() {
        print!("nested test");
        eprint!("nested test");
    }
}
