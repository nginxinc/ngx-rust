extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod loc_conf;

use proc_macro::TokenStream;
use syn::DeriveInput;
use loc_conf::expand_loc_conf;


#[proc_macro_derive(NgxLocConf)]
pub fn ngx_loc_conf(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input: DeriveInput = syn::parse(input).unwrap();

    match expand_loc_conf(&input) {
        Ok(expanded) => expanded.into(),
        Err(msg) => panic!(msg),
    }

}
