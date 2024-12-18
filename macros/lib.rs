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
        capacity_builder::StringBuilder::build(|builder| {
          self.string_build_with(builder);
        }).unwrap()
      }
    }

    impl std::fmt::Display for #name where #name: capacity_builder::StringBuildable {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        capacity_builder::StringBuilder::fmt(f, |builder| {
          self.string_build_with(builder);
        })
      }
    }

    impl<'a> capacity_builder::StringAppendable<'a> for &'a #name
    {
      #[inline(always)]
      fn append_to_builder(self, builder: &mut capacity_builder::StringBuilder<'a, '_, '_>) {
        self.string_build_with(builder);
      }
    }
  };

  // Return the modified implementation
  TokenStream::from(expanded)
}
