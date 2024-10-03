use darling::{ast, FromDeriveInput, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DataStruct, DeriveInput, Fields, FieldsNamed, LitFloat, LitStr};

#[derive(FromDeriveInput, Debug)]
#[darling(forward_attrs(ident, data), supports(struct_any))]
struct MetronomStruct {
    /// Get the ident of the field. For fields in tuple or newtype structs or
    /// enum bodies, this can be `None`.
    ident: syn::Ident,

    /// This magic field name pulls the type from the input.
    data: ast::Data<(), MetricFieldOptions>,
}

#[derive(FromField, Debug)]
#[darling(attributes(metronom), forward_attrs(ident, ty))]
struct MetricFieldOptions {
    /// Get the ident of the field. For fields in tuple or newtype structs or
    /// enum bodies, this can be `None`.
    ident: Option<syn::Ident>,
    ty: syn::Type,

    name: String,
    help: String,
    #[darling(default)]
    labels: Vec<LitStr>,
    #[darling(default)]
    buckets: Option<Vec<LitFloat>>,
}

#[proc_macro_derive(Metronom, attributes(metronom))]
pub fn derive_metronom(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let x = MetronomStruct::from_derive_input(&ast).unwrap();

    let name = x.ident;

    let impl_struct_new = quote! {
        impl #name {
        }
    };

    impl_struct_new.into()
}
