const ATTRIBUTE_NAME: &'static str = "peg";

/// The root element for which `peggle` is being derived.
pub enum DeriveInfo {
    /// A struct containing named fields (or no fields)
    NamedStruct(CollectionInfo),
    /// A struct containing unnamed fields
    UnnamedStruct(CollectionInfo),
    /// An enum
    Enum(EnumInfo),
}

impl DeriveInfo {
    /// Extracts all relevant information from the abstract syntax tree to populate a [`DeriveInfo`] instance.
    #[inline]
    pub fn new(ast: &syn::DeriveInput) -> Self {
        match &ast.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Named(fields),
                ..
            }) => {
                let syn::Meta::List(pegex_list) = &ast.attrs.iter().find(|&a| a.path().is_ident(ATTRIBUTE_NAME))
                    .expect("missing mandatory 'pegex' attribute for derived struct").meta else {
                    panic!("'pegex' attribute must be a Meta::List attribute");
                };

                let pegex = pegex_list
                    .parse_args::<syn::LitStr>()
                    .expect("Invalid format for pegex expression: must be a string literal")
                    .value();

                Self::NamedStruct(CollectionInfo {
                    name: ast.ident.clone(),
                    pegex,
                    fields: fields.named.iter().map(|field| {
                        let (inner_ty, cardinality, is_boxed) = get_inner_type(&field.ty);
                        let field_pegex = field.attrs.iter().find(|&a| a.path().is_ident(ATTRIBUTE_NAME)).and_then(|attr| {
                            let syn::Meta::List(pegex_list) = &attr.meta else {
                                panic!("'pegex' attribute must be a Meta::List attribute");
                            };
                            Some(pegex_list.parse_args::<syn::LitStr>().expect("Invalid format for pegex expression: must be a string literal").value())
                        });

                        FieldInfo {
                            ident: field.ident.as_ref().expect("TODO named struct missing ident").to_string(),
                            ty: field.ty.clone(),
                            inner_ty,
                            is_boxed,
                            cardinality,
                            pegex: field_pegex,
                        }
                    }).collect()
                })
            }
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(fields),
                ..
            }) => {
                let syn::Meta::List(pegex_list) = &ast.attrs.iter().find(|&a| a.path().is_ident(ATTRIBUTE_NAME))
                    .expect("missing mandatory 'pegex' attribute for derived struct").meta else {
                    panic!("'pegex' attribute must be a Meta::List attribute");
                };

                let pegex = pegex_list
                    .parse_args::<syn::LitStr>()
                    .expect("Invalid format for pegex expression: must be a string literal")
                    .value();

                Self::UnnamedStruct(CollectionInfo {
                    name: ast.ident.clone(),
                    pegex,
                    fields: fields.unnamed.iter().enumerate().map(|(idx, field)| {
                        let (inner_ty, cardinality, is_boxed) = get_inner_type(&field.ty);
                        let field_pegex = field.attrs.iter().find(|&a| a.path().is_ident(ATTRIBUTE_NAME)).and_then(|attr| {
                            let syn::Meta::List(pegex_list) = &attr.meta else {
                                panic!("'pegex' attribute must be a Meta::List attribute");
                            };
                            Some(pegex_list.parse_args::<syn::LitStr>().expect("Invalid format for pegex expression: must be a string literal").value())
                        });

                        FieldInfo {
                            ident: idx.to_string(),
                            ty: field.ty.clone(),
                            inner_ty,
                            is_boxed,
                            cardinality,
                            pegex: field_pegex,
                        }
                    }).collect()
                })
            }
            syn::Data::Enum(syn::DataEnum { variants, .. }) => {
                assert!(
                    ast.attrs
                        .iter()
                        .find(|&a| a.path().is_ident(ATTRIBUTE_NAME))
                        .is_none(),
                    "'pegex' attribute applied erroneously to enum type"
                );

                Self::Enum(EnumInfo {
                    name: ast.ident.clone(),
                    discriminants: Self::collect_enum_discriminants(variants),
                })
            }
            _ => panic!("derive applied to incompatible type (only structs and enums supported"),
        }
    }

    fn collect_enum_discriminants(
        variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    ) -> Vec<CollectionInfo> {
        variants.iter().map(|variant| {
            let syn::Meta::List(pegex_list) = &variant.attrs.iter().find(|&a| a.path().is_ident(ATTRIBUTE_NAME))
                .expect("missing mandatory 'pegex' attribute for enum discriminant").meta else {
                panic!("'pegex' attribute must be a Meta::List attribute");
            };

            let pegex = pegex_list.parse_args::<syn::LitStr>().expect("Invalid format for pegex expression: must be a string literal").value();

            CollectionInfo {
                name: variant.ident.clone(),
                pegex,
                fields: variant.fields.iter().enumerate().map(|(idx, field)| {
                    let (inner_ty, cardinality, is_boxed) = get_inner_type(&field.ty);
                    let field_pegex = field.attrs.iter().find(|&a| a.path().is_ident(ATTRIBUTE_NAME)).and_then(|attr| {
                        let syn::Meta::List(pegex_list) = &attr.meta else {
                            panic!("'pegex' attribute must be a Meta::List attribute");
                        };
                        Some(pegex_list.parse_args::<syn::LitStr>().expect("Invalid format for pegex expression: must be a string literal").value())
                    });

                    FieldInfo {
                        ident: idx.to_string(),
                        ty: field.ty.clone(),
                        is_boxed,
                        inner_ty,
                        cardinality,
                        pegex: field_pegex,
                   }
                }).collect()
            }
        }).collect()
    }
}

/// Information on an `enum` element.
pub struct EnumInfo {
    pub name: syn::Ident,
    pub discriminants: Vec<CollectionInfo>,
}

/// Information on an element that is either an `enum` discriminant or a `struct`.
#[derive(Clone)]
pub struct CollectionInfo {
    pub name: syn::Ident,
    pub pegex: String,
    pub fields: Vec<FieldInfo>,
}

/// Information on the field of an element, such as a `struct` member or `enum` discriminant tuple member.
#[derive(Clone)]
pub struct FieldInfo {
    pub ident: String,
    pub ty: syn::Type,
    pub inner_ty: syn::Type,
    pub is_boxed: bool,
    pub cardinality: FieldCardinality,
    pub pegex: Option<String>,
}

/// the "cardinality" of a field, or the minimum/maximum number of times that field is permitted to match in a PEG.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FieldCardinality {
    /// Exactly one instance of the field is required
    Single,
    /// Either zero or one instances of the field may exist
    Option,
    /// Any number of instances of the field may exist
    Vec,
}

/// Determines whether the supplied type is a Vec<T> or Option<T> type.
/// If it is, this method returns the inner type (`T`) as well as the appropriate cardinality of the input type; otherwise, it returns the input type and [`FieldCardinality::Single`].
fn get_inner_type(ty: &syn::Type) -> (syn::Type, FieldCardinality, bool) {
    match ty {
        syn::Type::Path(tp) => {
            let final_segment = tp.path.segments.last().unwrap();

            let cardinality = match final_segment.ident.to_string().as_str() {
                "Vec" => FieldCardinality::Vec,
                "Option" => FieldCardinality::Option,
                "Box" => FieldCardinality::Single,
                _ => return (ty.clone(), FieldCardinality::Single, false),
            };

            let syn::PathArguments::AngleBracketed(angle_args) = &final_segment.arguments else {
                panic!("Parantheses-Bracketed Type arguments not supported");
            };

            for arg in &angle_args.args {
                if let syn::GenericArgument::Type(ty) = arg {
                    let syn::Type::Path(tp) = ty else {
                        return (ty.clone(), cardinality, cardinality == FieldCardinality::Single) // Single => had Box wrapping outside, otherwise not
                    };

                    let inner_final_segment = tp.path.segments.last().unwrap();
                    if inner_final_segment.ident.to_string().as_str() != "Box" {
                        return (
                            ty.clone(),
                            cardinality,
                            cardinality == FieldCardinality::Single,
                        );
                    }

                    let syn::PathArguments::AngleBracketed(inner_angle_args) = &inner_final_segment.arguments else {
                        panic!("Parantheses-bracketed Type args not supported");
                    };

                    for inner_arg in &inner_angle_args.args {
                        if let syn::GenericArgument::Type(inner_ty) = inner_arg {
                            return (inner_ty.clone(), cardinality, true);
                        }
                    }
                }
            }

            panic!("Internal Error: no type found within angle brackets of Vec/Option (should be impossible)");
        }
        _ => (ty.clone(), FieldCardinality::Single, false),
    }
}
