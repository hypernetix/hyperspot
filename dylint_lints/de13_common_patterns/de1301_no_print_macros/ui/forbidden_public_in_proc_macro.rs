// compile-flags: --crate-type=proc-macro

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn my_macro(_input: TokenStream) -> TokenStream {
    private_helper();
    public_helper();
    TokenStream::new()
}

pub(crate) fn public_helper() {
    // Should trigger DE1301 - Print macros
    eprintln!("not allowed in public helper");
}

fn private_helper() {
    println!("allowed in private helper");
}
