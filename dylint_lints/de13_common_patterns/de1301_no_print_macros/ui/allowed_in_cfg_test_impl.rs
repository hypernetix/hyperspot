#[cfg(test)]
impl Helper {
    fn call() {
        // Should not trigger DE1301 - Print macros
        println!("allowed inside cfg(test) impl");
        // Should not trigger DE1301 - Print macros
        dbg!(42);
    }
}

struct Helper;

fn main() {
    // This call is not compiled, but keeps the impl reachable for the parser.
    #[cfg(test)]
    {
        Helper::call();
    }
}
