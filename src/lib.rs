use darling::{ast, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, LitFloat, LitStr};

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

    let (fields, initializers): (Vec<_>, Vec<_>) = x
        .data
        .take_struct()
        .map(|f| {
            f.fields
                .into_iter()
                .map(|f| {
                    let field_name = f.ident.unwrap();
                    let ty = f.ty.clone();
                    let type_name_ident = match f.ty {
                        syn::Type::Path(tp) => tp.path.segments.first().unwrap().ident.clone(),
                        _ => unreachable!(),
                    };
                    let metric_name = f.name;
                    let metric_help = f.help;
                    let buckets = f.buckets;
                    let labels = f.labels;

                    let opts = if type_name_ident == "IntCounterVec" {
                        quote! {
                            Opts::new(#metric_name, #metric_help)
                        }
                    } else {
                        let my_buckets = buckets.unwrap();
                        quote! {
                            HistogramOpts::new(#metric_name, #metric_help).buckets(vec![#(#my_buckets),*])
                        }
                    };

                    (
                        field_name.clone(),
                        quote! {
                            let #field_name = #ty::new(
                                #opts,
                                &[#(#labels),*],
                            ).unwrap();
                        },
                    )
                })
                .unzip()
        })
        .unwrap();

    let name = x.ident;

    let impl_struct_new = quote! {
        impl #name {
            fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
                #(#initializers)*

                let metrics = Self {
                    #(#fields),*
                };
                Ok(metrics)
            }
        }
    };

    impl_struct_new.into()
}
