use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FastDisplay)]
pub fn fast_display_derive(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let name = &input.ident;

  let expanded = quote! {
    impl #name {
      pub fn to_string(&self) -> String {
        capacity_builder::StringBuilder::<String>::build(|builder| {
          builder.append(self)
        }).unwrap()
      }
    }

    impl std::fmt::Display for #name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        capacity_builder::StringBuilder::<String>::fmt(f, |builder| {
          builder.append(self)
        })
      }
    }
  };

  // Return the modified implementation
  TokenStream::from(expanded)
}
