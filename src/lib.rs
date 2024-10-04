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

#[derive(FromField, Debug, Clone)]
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
    let struct_data = MetronomStruct::from_derive_input(&ast).unwrap();

    let fields: Vec<_> = struct_data.data.take_struct().map(|f| f.fields).unwrap();

    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.clone()).collect();
    let initializers = create_initialzers(&fields);

    let name = struct_data.ident;

    let impl_struct_new = quote! {
        impl #name {
            fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
                #(#initializers)*

                let metrics = Self {
                    #(#field_idents),*
                };

                #(registry.register(Box::new(metrics.#field_idents.clone()))?;)*

                Ok(metrics)
            }
        }
    };

    impl_struct_new.into()
}

fn create_initialzers(fields: &[MetricFieldOptions]) -> Vec<proc_macro2::TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field = field.clone();
            let ty = field.ty;
            let field_name = field.ident;
            let metric_name = field.name;
            let metric_help = field.help;
            let buckets = field.buckets;
            let labels = field.labels;

            let type_name_ident = match &ty {
                syn::Type::Path(ref tp) => tp.path.segments.first().unwrap().ident.clone(),
                _ => unreachable!(),
            };

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

            quote! {
                let #field_name = #ty::new(
                    #opts,
                    &[#(#labels),*],
                ).unwrap();
            }
        })
        .collect()
}
