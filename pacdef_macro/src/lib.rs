use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Register)]
pub fn register(input: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(input).unwrap();
    let name = &input.ident;
    let enum_data = if let syn::Data::Enum(enum_data) = &input.data {
        enum_data
    } else {
        panic!("`Register` can only be used on enums");
    };
    let first_variant = &enum_data.variants[0].ident;

    let variant_matches_backend = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            Self::#variant_name => Box::new(#variant_name::new()),
        }
    });

    let variant_matches_next = enum_data.variants.iter().enumerate().map(|(i, variant)| {
        let variant_name = &variant.ident;
        if i == enum_data.variants.len() - 1 {
            quote! {
                Self::#variant_name => None,
            }
        } else {
            let next_variant = &enum_data.variants[i + 1].ident;
            quote! {
                Self::#variant_name => Some(Self::#next_variant),
            }
        }
    });

    let variant_imports = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        // let variant_module = variant_name.clone();

        let variant_module = proc_macro2::Ident::new(
            &variant_name.to_string().to_lowercase(),
            proc_macro2::Span::call_site(),
        );

        quote! {
            pub(crate) use #variant_module::#variant_name;

        }
    });

    let expanded = quote! {
        #(#variant_imports)*

        impl #name {
            pub fn iter() -> BackendIter {
                BackendIter {
                    next: Some(Self::#first_variant),
                }
            }
            fn get_backend(&self) -> Box<dyn Backend> {
                match self {
                    #(#variant_matches_backend)*
                }
            }
            fn next(&self) -> Option<Self> {
                match self {
                    #(#variant_matches_next)*
                }
            }
        }
    };
    TokenStream::from(expanded)
}
