use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::__private::TokenStream2;

pub fn action(input: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(input).unwrap();

    let name = &input.ident;

    let syn::Data::Enum(enum_data) = &input.data else {
        panic!("`Register` can only be used on enums");
    };

    let variant_description = generate_variant_description(enum_data);
    let variant_constants = generate_variant_constants(name, enum_data);

    let expanded = compile_output(name, variant_description, variant_constants);

    TokenStream::from(expanded)
}

fn generate_variant_description(
    enum_data: &syn::DataEnum,
) -> impl Iterator<Item = TokenStream2> + '_ {
    let variant_matches_backend = enum_data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_lowercase =
            proc_macro2::Literal::string(&variant_name.to_string().to_lowercase());
        quote! {
            Self::#variant_name => #variant_lowercase,
        }
    });
    variant_matches_backend
}

fn generate_variant_constants<'a>(
    name: &'a syn::Ident,
    enum_data: &'a syn::DataEnum,
) -> impl Iterator<Item = TokenStream2> + 'a {
    let result = enum_data.variants.iter().map(move |variant| {
        let variant_name = &variant.ident;

        let variant_uppercase = proc_macro2::Ident::new(
            &variant_name.to_string().to_uppercase(),
            proc_macro2::Span::call_site(),
        );

        quote! {
            pub const #variant_uppercase: &str = #name::#variant_name.name();
        }
    });
    result
}

fn compile_output<T, U>(
    name: &syn::Ident,
    variant_description: T,
    variant_constants: U,
) -> TokenStream2
where
    T: Iterator<Item = TokenStream2>,
    U: Iterator<Item = TokenStream2>,
{
    let expanded = quote! {
        impl #name {
            pub const fn name(&self) -> &'static str {
                match self {
                    #(#variant_description)*
                }
            }
        }

        #(#variant_constants)*
    };
    expanded
}
