fn main() {
    // Should trigger DE1301 - Print macros
    println!("hello");

    // Should trigger DE1301 - Print macros
    eprintln!("hello");

    // Should trigger DE1301 - Print macros
    print!("hello");

    // Should trigger DE1301 - Print macros
    eprint!("hello");

    // Should trigger DE1301 - Print macros
    dbg!(42);
}
