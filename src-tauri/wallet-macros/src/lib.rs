extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn;  // Must be imported

#[proc_macro_attribute]
pub fn tauri_command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::Item);
    #[cfg(feature = "gui")] {
        TokenStream::from(quote! {
            #[tauri::command]
            #input
        })
    }
    #[cfg(not(feature = "gui"))] {
        TokenStream::from(quote! {
            #[allow(unused)]
            #input
        })
    }
}