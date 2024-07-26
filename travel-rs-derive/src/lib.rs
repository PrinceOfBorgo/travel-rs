use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Table)]
pub fn derive_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(fields),
        ..
    }) = input.data
    {
        fields.named
    } else {
        unimplemented!("Table derive macro only supports structs with named fields");
    };

    let name_str = name.to_string();
    let name_snake_case = name_str.to_snake_case();

    let mod_name = syn::Ident::new(&name_snake_case, name.span());

    let table = quote! {
        pub const TABLE: &'static str = #name_snake_case;
    };

    let field_consts = fields.iter().map(|field| {
        let field_name = format_ident!("{}", &field.ident.as_ref().unwrap()); // Can unwrap because fields are named
        let field_name_str = field_name.to_string();
        let const_name = syn::Ident::new(&field_name_str.to_uppercase(), field_name.span());
        quote! {
            pub const #const_name: &'static str = #field_name_str;
        }
    });

    let expanded = quote! {
        mod #mod_name {
            #table
            #(#field_consts)*
        }
        pub use #mod_name::*;
    };

    TokenStream::from(expanded)
}

trait ToSnakeCase {
    fn to_snake_case(&self) -> String;
}

impl<T: AsRef<str>> ToSnakeCase for T {
    fn to_snake_case(&self) -> String {
        let mut words = vec![];
        // Preserve leading underscores
        let str = self.as_ref().trim_start_matches(|c: char| {
            if c == '_' {
                words.push(String::new());
                true
            } else {
                false
            }
        });
        for s in str.split('_') {
            let mut last_upper = false;
            let mut buf = String::new();
            if s.is_empty() {
                continue;
            }
            for ch in s.chars() {
                if !buf.is_empty() && buf != "'" && ch.is_uppercase() && !last_upper {
                    words.push(buf);
                    buf = String::new();
                }
                last_upper = ch.is_uppercase();
                buf.extend(ch.to_lowercase());
            }
            words.push(buf);
        }
        words.join("_")
    }
}
