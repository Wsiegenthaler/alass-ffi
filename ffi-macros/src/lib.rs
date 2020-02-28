#![warn(clippy::all)]

extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, Expr, ItemFn};

///
/// Wraps a function body with `std::panic::catch_unwind` to ensure that panics won't unwind
/// accross the ffi boundary. The macro argument should be the return value of the function
/// in case of panic, or no argument if it returns void.
/// 
/// Also logs `error!` message upon panic so `log` crate must
/// be in scope.
/// 
/// ### Example function with no return value:
/// ```
///     # use alass_ffi_macros::catch_panic;
/// 
///     #[catch_panic]
///     pub extern "C" fn foo() {
///         // may panic here...
///     }
/// ```
///
/// ### Example function that returns `0` on success and `-1` on panic:
/// ```
///     # use alass_ffi_macros::catch_panic;
///     # use std::os::raw::c_int;
/// 
///     #[catch_panic(-1)]
///     pub extern "C" fn bar() -> c_int {
///         // may panic here...
///         0
///     }
/// ```
/// 
#[proc_macro_attribute]
pub fn catch_panic(args: TokenStream, item: TokenStream) -> TokenStream {

    let ItemFn { attrs, vis, sig, block } = parse_macro_input!(item as ItemFn);

    // If args aren't specified, assume function returns void
    let args = if args.is_empty() { quote!(()).into() } else { args };

    let ret_expr = parse_macro_input!(args as Expr);

    let wrapped = quote! {
        #(#attrs)*
        #vis #sig {
            match std::panic::catch_unwind(|| { #block }) {
                Ok(v) => v,
                Err(e) => {
                    match e.downcast_ref::<&'static str>() {
                        Some(s) => log::error!("{}", s),
                        None => log::error!("Unknown panic!"),
                    };
                    #ret_expr
                }
            }
        }
    };
 
    TokenStream::from(wrapped)
}
