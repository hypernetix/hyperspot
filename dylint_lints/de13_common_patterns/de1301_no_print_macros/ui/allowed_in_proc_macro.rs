// compile-flags: --crate-type=proc-macro

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn my_macro(_input: TokenStream) -> TokenStream {
    eprintln!("warning from proc macro");
    TokenStream::new()
}

#[proc_macro]
pub fn my_another_macro(_input: TokenStream) -> TokenStream {
    nested_func();
    TokenStream::new()
}

fn nested_func() {
    println!("hello");
}
