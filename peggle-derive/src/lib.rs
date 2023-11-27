mod parse;
mod structure;

use structure::DeriveInfo;

#[proc_macro_derive(Parse, attributes(peg))]
pub fn derive_peg_parse(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(item as syn::DeriveInput);
    let derive_info = DeriveInfo::new(&ast);
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident = ast.ident;

    let parse_tokens = match derive_info {
        DeriveInfo::NamedStruct(st) => parse::derive_named_struct(st),
        DeriveInfo::UnnamedStruct(st) => parse::derive_unnamed_struct(st),
        DeriveInfo::Enum(e) => parse::derive_enum(e),
    };

    proc_macro::TokenStream::from(quote::quote! {
        #[automatically_derived]
        impl #impl_generics peggle::Parse for #ident #ty_generics #where_clause {
            fn parse_at<'a>(__peggle_index: peggle::Index<'a>) -> Result<(Self, peggle::Index<'a>), peggle::ParseError> {
                #parse_tokens
            }
        }
    })
}
