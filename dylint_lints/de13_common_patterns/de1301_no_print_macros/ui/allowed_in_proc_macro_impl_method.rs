// compile-flags: --crate-type=proc-macro

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn my_macro(_input: TokenStream) -> TokenStream {
    Helper::private_method();
    TokenStream::new()
}

struct Helper;

impl Helper {
    fn private_method() {
        // Should not trigger DE1301 - Print macros
        println!("allowed in private impl method");
    }
}
