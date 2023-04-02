use proc_macro::TokenStream;
use quote::{quote, ToTokens};

use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Data, DeriveInput, Fields, Token,
};

#[derive(Debug)]
struct SessionArgs {
    signer: syn::ExprAssign,
    authority: syn::ExprAssign,
}

impl Parse for SessionArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let signer = input.parse()?;

        input.parse::<Token![,]>()?;

        let authority = input.parse()?;
        Ok(SessionArgs { signer, authority })
    }
}

fn is_session(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("session")
}

// Macro to derive Session Trait
#[proc_macro_derive(Session, attributes(session))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input_parsed = parse_macro_input!(input as DeriveInput);

    let fields = match input_parsed.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields,
            _ => panic!("Session trait can only be derived for structs with named fields"),
        },
        _ => panic!("Session trait can only be derived for structs"),
    };

    // // Ensure that the struct has a session_token field
    let session_token_field = fields
        .named
        .iter()
        .find(|field| field.ident.as_ref().unwrap().to_string() == "session_token")
        .expect("Session trait can only be derived for structs with a session_token field");
    {
        let session_token_type = &session_token_field.ty;
        let session_token_type_string = quote! { #session_token_type }.to_string();
        assert!(
        session_token_type_string == "Option < Account < 'info, SessionToken > >",
        "Session trait can only be derived for structs with a session_token field of type Option<Account<'info, SessionToken>>"
        );
    };

    // Session Token field must have the #[session] attribute
    let session_attr = session_token_field
        .attrs
        .iter()
        .find(|attr| is_session(attr))
        .expect("Session trait can only be derived for structs with a session_token field with the #[session] attribute");

    let session_args = session_attr.parse_args::<SessionArgs>().unwrap();

    let session_signer = session_args.signer.right.into_token_stream();

    // Session Authority
    let session_authority = session_args.authority.right.into_token_stream();

    let struct_name = &input_parsed.ident;
    let (impl_generics, ty_generics, where_clause) = input_parsed.generics.split_for_impl();

    let output = quote! {

        #[automatically_derived]
        impl #impl_generics Session #ty_generics for #struct_name #ty_generics #where_clause {

            // Target Program
            fn target_program(&self) -> Pubkey {
                crate::id()
            }

            // Session Token
            fn session_token(&self) -> Account<'info, SessionToken> {
                self.session_token.clone().unwrap()
            }

            // Session Authority
            fn authority(&self) -> Pubkey {
                self.#session_authority
            }

            // Session Signer
            fn session_signer(&self) -> Pubkey {
                self.#session_signer
            }

        }
    };

    output.into()
}
