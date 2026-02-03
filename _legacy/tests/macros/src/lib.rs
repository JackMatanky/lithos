extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Generates a test factory for a domain struct.
///
/// This macro generates a `Factory` struct that can produce valid instances
/// of the target type with sensible defaults and support for mandatory fields.
#[proc_macro_derive(TestFactory)]
pub fn derive_test_factory(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let factory_name = quote::format_ident!("{}Factory", name);

    let fields = if let Data::Struct(ref data) = input.data {
        if let Fields::Named(ref fields) = data.fields {
            &fields.named
        } else {
            return quote! { compile_error!("TestFactory only supports named fields"); }.into();
        }
    } else {
        return quote! { compile_error!("TestFactory only supports structs"); }
            .into();
    };

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let expanded = quote! {
        #[derive(Debug, Default)]
        pub struct #factory_name {
            #(pub #field_names: Option<#field_types>),*
        }

        impl #factory_name {
            pub fn new() -> Self {
                Self::default()
            }

            #(
                pub fn #field_names(mut self, value: #field_types) -> Self {
                    self.#field_names = Some(value);
                    self
                }
            )*

            pub fn build(self) -> #name {
                #name {
                    #(
                        #field_names: self.#field_names.unwrap_or_else(|| {
                            // In a real implementation, we would use Faker or standard defaults
                            Default::default()
                        })
                    ),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
