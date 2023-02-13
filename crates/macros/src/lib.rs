mod action;
mod register;

use proc_macro::TokenStream;

#[proc_macro_derive(Register)]
pub fn register(input: TokenStream) -> TokenStream {
    register::register(input)
}

#[proc_macro_derive(Action)]
pub fn action(input: TokenStream) -> TokenStream {
    action::action(input)
}
