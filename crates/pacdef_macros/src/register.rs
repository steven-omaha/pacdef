use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::__private::TokenStream2;

pub fn register(input: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(input).unwrap();

    let name = &input.ident;

    let syn::Data::Enum(enum_data) = &input.data else {
        panic!("`Register` can only be used on enums");
    };

    let first_variant = &enum_data.variants[0].ident;

    let variant_matches_backend = generate_variant_backend(enum_data);
    let variant_matches_next = generate_variant_matches_next(enum_data);
    let variant_imports = generate_variant_imports(enum_data);

    let expanded = compile_output(
        name,
        first_variant,
        variant_matches_backend,
        variant_matches_next,
        variant_imports,
    );

    TokenStream::from(expanded)
}

fn compile_output<T, U, V>(
    name: &syn::Ident,
    first_variant: &syn::Ident,
    variant_backend: T,
    variant_next: U,
    variant_imports: V,
) -> TokenStream2
where
    T: Iterator<Item = TokenStream2>,
    U: Iterator<Item = TokenStream2>,
    V: Iterator<Item = TokenStream2>,
{
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
                    #(#variant_backend)*
                }
            }
            fn next(&self) -> Option<Self> {
                match self {
                    #(#variant_next)*
                }
            }
        }
    };
    expanded
}

fn generate_variant_imports(enum_data: &syn::DataEnum) -> impl Iterator<Item = TokenStream2> + '_ {
    let variant_imports = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let variant_module = proc_macro2::Ident::new(
            &variant_name.to_string().to_lowercase(),
            proc_macro2::Span::call_site(),
        );

        quote! {
            pub use actual::#variant_module::#variant_name;

        }
    });
    variant_imports
}

fn generate_variant_matches_next(
    enum_data: &syn::DataEnum,
) -> impl Iterator<Item = TokenStream2> + '_ {
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
    variant_matches_next
}

fn generate_variant_backend(enum_data: &syn::DataEnum) -> impl Iterator<Item = TokenStream2> + '_ {
    let variant_matches_backend = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            Self::#variant_name => Box::new(#variant_name::new()),
        }
    });
    variant_matches_backend
}
