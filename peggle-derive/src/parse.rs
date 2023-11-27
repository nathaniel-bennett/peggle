use std::collections::HashMap;
use std::iter::Iterator;

use crate::structure::{CollectionInfo, EnumInfo, FieldCardinality, FieldInfo};

use peggle::Parse;

// TODO: support Box<T> types, Option<Box<T>> types and Vec<Box<T>> types

// field_name, (min_instances, max_instances)
struct FieldCounter(pub HashMap<String, (usize, usize)>);

impl FieldCounter {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

struct FieldRequirements {
    fields: HashMap<String, FieldInfo>,
    field_min_max_stack: Vec<(Option<FieldCounter>, FieldCounter)>,
}

impl FieldRequirements {
    pub fn new(collection: &CollectionInfo) -> Self {
        let mut fields = HashMap::new();

        for field in &collection.fields {
            fields.insert(field.ident.clone(), field.clone());
        }

        Self {
            fields,
            field_min_max_stack: vec![(None, FieldCounter::new())],
        }
    }

    fn check_field(&self, field_name: &String, min: usize, max: usize) -> Result<(), String> {
        let Some(field_info) = self.fields.get(field_name) else {
            return Err(format!("Field '{}' not found during check_field", field_name))
        };

        match field_info.cardinality {
            FieldCardinality::Single if max > 1 || min < 1 =>
                Err(format!("field '{}' required exactly once, yet regular expression allows for variable number of instantiations", field_name)),
            FieldCardinality::Option if max > 1 =>
                Err(format!("field '{}' is optional (0 or 1 instantiations), yet regular expression could allow for more than one instantiation", field_name)),
            _ => Ok(()),
        }
    }

    fn check_choice_difference(&self) -> Result<(), String> {
        if let (Some(expected), actual) = self.field_min_max_stack.last().expect("1") {
            for field_name in expected.0.keys() {
                let field_info = self.fields.get(field_name).expect("Missing field name");

                if field_info.cardinality == FieldCardinality::Single
                    && !actual.0.contains_key(field_name)
                {
                    return Err(format!(
                        "field {} required, yet missing in a choice option",
                        field_name
                    ));
                }
            }

            for field_name in actual.0.keys() {
                let field_info = self.fields.get(field_name).expect("missing field name");

                if field_info.cardinality == FieldCardinality::Single
                    && !expected.0.contains_key(field_name)
                {
                    return Err(format!(
                        "field {} required, yet missing in a choice option",
                        field_name
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn add_field(&mut self, field_name: String, min: usize, max: usize) -> Result<(), String> {
        let Some(field_info) = self.fields.get(&field_name) else {
            return Err(format!("Field '{}' not found", field_name))
        };

        if field_info.cardinality == FieldCardinality::Vec {
            return Ok(()); // Variable fields can have any count of fields
        }

        let counter = &mut self.field_min_max_stack.last_mut().expect("2").1;
        let (new_min, new_max) = match counter.0.get_mut(&field_name) {
            Some((curr_min, curr_max)) => {
                *curr_min = std::cmp::min(*curr_min, min);
                *curr_max = curr_max.saturating_add(max);
                (*curr_min, *curr_max)
            }
            None => {
                counter.0.insert(field_name.clone(), (min, max));
                (min, max)
            }
        };

        self.check_field(&field_name, new_min, new_max)
    }

    pub fn push_nested_expr(&mut self) {
        self.field_min_max_stack.push((None, FieldCounter::new()));
    }

    pub fn pop_nested_expr(&mut self, min: usize, max: usize) -> Result<(), String> {
        // First, check the existing choice against expected choice (if applicable)
        self.check_choice_difference()?;

        let popped_expr_counter = self.field_min_max_stack.pop().expect("3").1;
        for (field_name, (field_min, field_max)) in popped_expr_counter.0 {
            self.add_field(
                field_name,
                field_min.saturating_mul(min),
                field_max.saturating_mul(max),
            )?;
        }

        Ok(())
    }

    pub fn add_choice_split(&mut self) -> Result<(), String> {
        self.check_choice_difference()?;

        match self.field_min_max_stack.last_mut().expect("4") {
            (Some(_), current) => *current = FieldCounter::new(),
            (old @ None, current) => {
                let mut swap = FieldCounter::new();
                std::mem::swap(&mut swap, current);
                *old = Some(swap);
            }
        }

        Ok(())
    }

    pub fn check_final_fields(&self) -> Result<(), String> {
        assert!(self.field_min_max_stack.len() == 1);

        self.check_choice_difference()?;

        let field_counter = &self.field_min_max_stack.last().expect("5").1;
        // Now check to ensure all required fields exist
        for field in self.fields.iter().filter_map(|(field_name, field)| {
            if field.cardinality == FieldCardinality::Single {
                Some(field_name)
            } else {
                None
            }
        }) {
            if !field_counter.0.contains_key(field) {
                return Err(format!(
                    "required field {} missing in regular expression",
                    field
                ));
            }
        }

        Ok(())
    }
}

pub fn derive_unnamed_struct(struct_info: CollectionInfo) -> proc_macro2::TokenStream {
    // First generate field declarations for struct members
    let field_declarations = struct_info.fields.iter().map(|info| {
        let identity = quote::format_ident!("__peggle_field_{}", &info.ident);
        let ty = info.ty.clone();

        match info.cardinality {
            FieldCardinality::Single => quote::quote! { let mut #identity: Option<#ty> = None; },
            FieldCardinality::Option => quote::quote! { let mut #identity: #ty = None; },
            FieldCardinality::Vec => quote::quote! { let mut #identity: #ty = Vec::new(); },
        }
    });

    // Then generate actual parsing code that fills in fields
    let parse_steps = derive_fields_steps(&struct_info);

    // Lastly, generate fields for instantiation of the struct
    let field_comma_list = struct_info.fields.iter().map(|info| {
        let identity = quote::format_ident!("__peggle_field_{}", &info.ident);
        match info.cardinality {
            FieldCardinality::Single => quote::quote! { #identity.unwrap(), },
            _ => quote::quote! { #identity, },
        }
    });

    quote::quote! {
        let mut __peggle_curr = __peggle_index;
        let mut __peggle_failure = false;
        #(#field_declarations)*

        #parse_steps

        if __peggle_failure {
            return Err(peggle::ParseError::from_index(__peggle_curr))
        }

        Ok((
            Self (
                #(#field_comma_list)*
            ),
            __peggle_curr,
        ))
    }
}

pub fn derive_named_struct(struct_info: CollectionInfo) -> proc_macro2::TokenStream {
    // First generate field declarations for struct members
    let field_declarations = struct_info.fields.iter().map(|info| {
        let identity = quote::format_ident!("__peggle_field_{}", &info.ident);
        let ty = info.ty.clone();

        match info.cardinality {
            FieldCardinality::Single => quote::quote! { let mut #identity: Option<#ty> = None; },
            FieldCardinality::Option => quote::quote! { let mut #identity: #ty = None; },
            FieldCardinality::Vec => quote::quote! { let mut #identity: #ty = Vec::new(); },
        }
    });

    // Then generate actual parsing code that fills in fields
    let parse_steps = derive_fields_steps(&struct_info);

    // Lastly, generate fields for instantiation of the struct
    let field_comma_list = struct_info.fields.iter().map(|info| {
        let identity = quote::format_ident!("__peggle_field_{}", &info.ident);
        let original_identity = quote::format_ident!("{}", &info.ident);
        match info.cardinality {
            FieldCardinality::Single => quote::quote! { #original_identity: #identity.unwrap(), },
            _ => quote::quote! { #original_identity: #identity, },
        }
    });

    quote::quote! {
        let mut __peggle_curr = __peggle_index;
        let mut __peggle_failure = false;
        #(#field_declarations)*

        #parse_steps

        if __peggle_failure {
            return Err(peggle::ParseError::from_index(__peggle_curr))
        }

        Ok((
            Self {
                #(#field_comma_list)*
            },
            __peggle_curr,
        ))
    }
}

pub fn derive_enum(e: EnumInfo) -> proc_macro2::TokenStream {
    derive_enum_steps(e.name, e.discriminants)
}

fn derive_enum_steps(
    enum_name: syn::Ident,
    discriminants: Vec<CollectionInfo>,
) -> proc_macro2::TokenStream {
    let mut expr_tokens = Vec::new();

    for discriminant in discriminants {
        let discriminant_name = &discriminant.name;

        // First generate field declarations for struct members
        let field_declarations = discriminant.fields.iter().map(|info| {
            let identity = quote::format_ident!("__peggle_field_{}", &info.ident);
            let ty = info.ty.clone();

            match info.cardinality {
                FieldCardinality::Single => {
                    quote::quote! { let mut #identity: Option<#ty> = None; }
                }
                FieldCardinality::Option => quote::quote! { let mut #identity: #ty = None; },
                FieldCardinality::Vec => quote::quote! { let mut #identity: #ty = Vec::new(); },
            }
        });

        // Then generate actual parsing code that fills in fields
        let parse_steps = derive_fields_steps(&discriminant);

        // Lastly, generate fields for instantiation of the struct
        let field_comma_list = discriminant.fields.iter().map(|info| {
            let identity = quote::format_ident!("__peggle_field_{}", &info.ident);
            match info.cardinality {
                FieldCardinality::Single => quote::quote! { #identity.unwrap(), },
                _ => quote::quote! { #identity, },
            }
        });

        let enum_fields = if field_comma_list.len() > 0 {
            quote::quote! { (#(#field_comma_list)*) }
        } else {
            quote::quote! {}
        };

        expr_tokens.push(quote::quote! {
            '__choice_lifetime_0: {
                __peggle_curr = __peggle_index;
                __peggle_failure = false;
                #(#field_declarations)*

                #parse_steps

                if !__peggle_failure {
                    return Ok((#enum_name::#discriminant_name #enum_fields, __peggle_curr))
                }
            }
        });
    }

    quote::quote! {
        let mut __peggle_curr;
        let mut __peggle_failure;

        '__expression_lifetime_0: {
            #(#expr_tokens)*
        }

        Err(peggle::ParseError::from_index(__peggle_index))
    }
}

fn derive_single_field_fns(field: &FieldInfo) -> proc_macro2::TokenStream {
    if let Some(pegex) = &field.pegex {
        let restrict_fn = quote::format_ident!("__peggle_restrict_{}", field.ident);
        let field_fn = quote::format_ident!("__peggle_parse_{}", field.ident);
        let field_ty = &field.inner_ty;

        let restrict_fn_tokens = derive_single_field_steps(pegex);

        quote::quote! {
            #[inline]
            fn #restrict_fn<'a>(__peggle_index: peggle::Index<'a>) -> Result<(&'a str, peggle::Index<'a>), peggle::ParseError> {
                let mut __peggle_curr: peggle::Index<'_>;
                let mut __peggle_failure: bool;
                #restrict_fn_tokens
            }

            #[inline]
            fn #field_fn<'a>(__peggle_index: peggle::Index<'a>) -> Result<(#field_ty, peggle::Index<'a>), peggle::ParseError> {
                let (__peggle_restricted_str, __peggle_new_index) = #restrict_fn(__peggle_index)?;

                let __peggle_restricted_index = peggle::Index {
                    remaining: __peggle_restricted_str,
                    lineno: __peggle_index.lineno,
                    colno: __peggle_index.colno,
                };
                let (__peggle_out, __peggle_end_idx) = #field_ty::parse_at(__peggle_restricted_index)?;

                if __peggle_end_idx.remaining.is_empty() {
                    Ok((__peggle_out, __peggle_new_index))
                } else {
                    // This may happen if the restriction regex is not a proper subset of the type's input parsing
                    Err(peggle::ParseError::from_index(__peggle_new_index))
                }
            }
        }
    } else {
        let field_fn = quote::format_ident!("__peggle_parse_{}", field.ident);
        let field_ty = &field.inner_ty;

        quote::quote! {
            #[inline]
            fn #field_fn<'a>(__peggle_index: peggle::Index<'a>) -> Result<(#field_ty, peggle::Index<'a>), peggle::ParseError> {
                #field_ty::parse_at(__peggle_index)
            }
        }
    }
}

fn derive_single_field_steps(pegex: &str) -> proc_macro2::TokenStream {
    let mut index = peggle::Index::new(pegex);
    let mut nested_expressions = vec![vec![Vec::new()]];

    while let Some(c) = index.next() {
        match c {
            '(' => nested_expressions.push(vec![Vec::new()]),
            ')' => coalesce_topmost_expression(&mut index, &mut nested_expressions, None),
            '|' => nested_expressions.last_mut().expect("6").push(Vec::new()), // Add another option to the current expression
            '[' => match_one_of(&mut index, &mut nested_expressions),
            '<' | '>' => panic!("carrot brackets reserved for member fields, which are not allowed within a field derive. Use \"[<]\" and \"[>]\" to specify literal carrot brackets"),
            ']' | '*' | '+' | '?' | '{' | '}' => panic!("unexpected token {} at peggle index {}", c, index.colno),
            '^' | '$' => panic!("{} regex symbol not implemented (prepend a backslash to use the literal `{}` character)", c, c),
            '.' => match_any_character(&mut index, &mut nested_expressions),
            _ => match_character(c, &mut index, &mut nested_expressions),
        }
    }

    if nested_expressions.len() > 1 {
        panic!("missing close parantheses in peggle expression");
    }

    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_1"));
    let expr_tokens = nested_expressions
        .last_mut()
        .expect("missing last expr_token")
        .iter_mut()
        .map(|choice_tokens| {
            quote::quote! {
                __peggle_curr = __peggle_index;
                __peggle_failure = false;
                #choice_lifetime: {
                    #(#choice_tokens)*
                }

                if !__peggle_failure {
                    break '__expression_lifetime_1
                }
            }
        });

    quote::quote! {
        '__expression_lifetime_1: {
            #(#expr_tokens)*
        }

        if __peggle_failure {
            Err(peggle::ParseError::from_index(__peggle_curr))
        } else {
            let __peggle_restricted_str = &__peggle_index.remaining.get(..__peggle_index.remaining.len() - __peggle_curr.remaining.len()).ok_or(peggle::ParseError::from_index(__peggle_curr))?;
            Ok((__peggle_restricted_str, __peggle_curr))
        }
    }
}

fn derive_fields_steps(collection: &CollectionInfo) -> proc_macro2::TokenStream {
    let mut index = peggle::Index::new(collection.pegex.as_str());
    let mut nested_expressions = vec![vec![Vec::new()]]; // TODO: a triple-Vec is GNARLY--fix...
    let mut requirements = FieldRequirements::new(collection);

    let mut field_steps = Vec::new();
    for field in &collection.fields {
        field_steps.push(derive_single_field_fns(field));
    }

    while let Some(c) = index.next() {
        match c {
            '(' => {
                nested_expressions.push(vec![Vec::new()]); // Add another nested expression on the stack
                requirements.push_nested_expr();
            }
            ')' => coalesce_topmost_expression(&mut index, &mut nested_expressions, Some(&mut requirements)),
            '|' => {
                // Add another option to the current expression
                nested_expressions.last_mut().expect("6").push(Vec::new());
                requirements.add_choice_split().expect("failed to add choice split");
            }
            '<' => match_field(&mut index, &mut requirements, &mut nested_expressions),
            '[' => match_one_of(&mut index, &mut nested_expressions),
            '>' | ']' | '*' | '+' | '?' | '{' | '}' => panic!("unexpected token {} at peggle index {}", c, index.colno),
            '^' | '$' => panic!("{} regex symbol not implemented (prepend a backslash to use the literal `{}` character)", c, c),
            '.' => match_any_character(&mut index, &mut nested_expressions),
            '\\' => match_backslash_class(&mut index, &mut nested_expressions),
            _ => match_character(c, &mut index, &mut nested_expressions),
        }
    }

    if nested_expressions.len() > 1 {
        panic!("missing close parantheses in peggle expression");
    }

    requirements.check_final_fields().expect("7");

    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_1"));
    let expr_tokens = nested_expressions
        .last_mut()
        .expect("missing last expr_token")
        .iter_mut()
        .map(|choice_tokens| {
            quote::quote! {
                __peggle_curr = __peggle_index;
                __peggle_failure = false;
                #choice_lifetime: {
                    #(#choice_tokens)*
                }

                if !__peggle_failure {
                    break '__expression_lifetime_1
                }
            }
        });

    quote::quote! {
        #(#field_steps)*

        '__expression_lifetime_1: {
            #(#expr_tokens)*
        }
    }
}

fn match_field(
    index: &mut peggle::Index,
    requirements: &mut FieldRequirements,
    nested_expressions: &mut Vec<Vec<Vec<proc_macro2::TokenStream>>>,
) {
    let mut field_name = String::new();
    loop {
        match index.next() {
            Some('>') => break,
            Some(c) => field_name.push(c),
            None => panic!(
                "parameter field missing closing '>' bracket at column {}",
                index.colno
            ),
        }
    }

    // Handle any possible repetition/variable number suffixes
    let (min, max, advanced_index) = get_repetition_bounds(*index);
    *index = advanced_index;

    requirements
        .add_field(field_name.clone(), min, max)
        .expect("8");

    let field_fn = quote::format_ident!("__peggle_parse_{}", field_name);

    let expr_depth = nested_expressions.len().to_string();
    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_{}", expr_depth));
    let loop_revert_index = quote::format_ident!("__loop_revert_{}", expr_depth);
    let loop_iter_ident = quote::format_ident!("__loop_iter_{}", expr_depth);

    let Some(field_info) = requirements.fields.get(&field_name) else {
        panic!("Unrecognized field {}", field_name)
    };

    let field_name = quote::format_ident!("__peggle_field_{}", field_name);

    let assign_tokens = match (field_info.cardinality, field_info.is_boxed) {
        (FieldCardinality::Single, false) => quote::quote! { #field_name = Some(__peggle_val); },
        (FieldCardinality::Single, true) => {
            quote::quote! { #field_name = Some(Box::new(__peggle_val)); }
        }
        (FieldCardinality::Option, false) => quote::quote! { #field_name = Some(__peggle_val); },
        (FieldCardinality::Option, true) => {
            quote::quote! { #field_name = Some(Box::new(__peggle_val)); }
        }
        (FieldCardinality::Vec, false) => quote::quote! { #field_name.push(__peggle_val); },
        (FieldCardinality::Vec, true) => {
            quote::quote! { #field_name.push(Box::new(__peggle_val)); }
        }
    };

    nested_expressions
        .last_mut()
        .expect("missing last expression")
        .last_mut()
        .expect("missing last choice")
        .push(quote::quote! {
            let mut #loop_revert_index = __peggle_curr;
            for #loop_iter_ident in 0..#max {
                #loop_revert_index = __peggle_curr;

                match #field_fn(__peggle_curr) {
                    Ok((__peggle_val, new_idx)) => {
                        __peggle_curr = new_idx;
                        #assign_tokens // assign val, Some(val) or .push(val) depending on type
                    }
                    Err(_) => {
                        __peggle_failure = true;
                        if #loop_iter_ident >= #min {
                            __peggle_curr = #loop_revert_index; // Rewind to where last successful loop iteration finished
                            __peggle_failure = false;
                        }
                        break
                    }
                }
            }

            if __peggle_failure {
                break #choice_lifetime
            }
        });
}

fn id_to_lifetime(ident: syn::Ident) -> syn::Lifetime {
    syn::Lifetime {
        apostrophe: ident.span(),
        ident,
    }
}

fn match_posix_class(
    index: &mut peggle::Index,
    bracket_possibilities: &mut Vec<proc_macro2::TokenStream>,
    bracket_lifetime: &syn::Lifetime,
) {
    index.next(); // Consume ':'
    let label: String = index.take(5).collect::<String>();
    match label.as_str() {
        "upper" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= 'A' && __bracket_char <= 'Z') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "lower" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= 'a' && __bracket_char <= 'z') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "alpha" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= 'A' && __bracket_char <= 'Z') || (__bracket_char >= 'a' && __bracket_char <= 'z') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "digit" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= '0' && __bracket_char <= '9') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "xdigi" => {
            let Some('t') = index.next() else {
                panic!("unrecognized POSIX character class");
            };
            bracket_possibilities.push(quote::quote!{
                if (__bracket_char >= '0' && __bracket_char <= '9') || (__bracket_char >= 'A' && __bracket_char <= 'F') || (__bracket_char >= 'a' && __bracket_char <= 'f') {
                    __peggle_failure = false;
                    break #bracket_lifetime
                }
            });
        },
        "alnum" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= 'A' && __bracket_char <= 'Z') || (__bracket_char >= 'a' && __bracket_char <= 'z') || (__bracket_char >= '0' || __bracket_char <= '9') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "punct" => bracket_possibilities.push(quote::quote!{
            if "][!\"#$%&'()*+,./:;<=>?@\\^_`{|}~-".contains(__bracket_char) {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "blank" => bracket_possibilities.push(quote::quote!{
            if __bracket_char == ' ' || __bracket_char == '\t' {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "space" => bracket_possibilities.push(quote::quote!{
            if __bracket_char == ' ' || __bracket_char == '\t' || __bracket_char == '\n' || __bracket_char == '\r' || __bracket_char == '\x0c' || __bracket_char == '\x0b' {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "cntrl" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= '\x00' && __bracket_char <= '\x1f') || (__bracket_char == '\x7f') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "graph" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= '\x21' || __bracket_char <= '\x7e') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        "print" => bracket_possibilities.push(quote::quote!{
            if (__bracket_char >= '\x20' && __bracket_char <= '\x7e') {
                __peggle_failure = false;
                break #bracket_lifetime
            }
        }),
        _ => panic!("unrecognized POSIX character class '{}'", label),
    }
    let Some(':') = index.next() else {
        panic!("POSIX character class {} missing closing ':]' bracket", label);
    };
    let Some(']') = index.next() else {
        panic!("POSIX character class {} missing closing ']' bracket", label);
    };
}

fn match_backslash_class(
    index: &mut peggle::Index,
    nested_expressions: &mut Vec<Vec<Vec<proc_macro2::TokenStream>>>,
) {
    let Some(character) = index.next() else {
        panic!("expected character after backslash")
    };

    let (min, max, advanced_index) = get_repetition_bounds(*index);
    *index = advanced_index;

    let expr_depth = nested_expressions.len().to_string();
    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_{}", expr_depth));
    let loop_iter_ident = quote::format_ident!("__loop_iter_{}", expr_depth);

    let character_match_tokens = match character {
        'w' => quote::quote! {
            if (__backslash_char < 'A' || __backslash_char > 'Z') && (__backslash_char < 'a' || __backslash_char > 'z') && __backslash_char != '_' {
                __peggle_failure = true;
            }
        },
        'W' => quote::quote! {
            if (__backslash_char >= 'A' && __backslash_char <= 'Z') || (__backslash_char >= 'a' && __backslash_char <= 'z') || __backslash_char == '_' {
                __peggle_failure = true;
            }
        },
        'd' => quote::quote! {
            if __backslash_char < '0' || __backslash_char > '9' {
                __peggle_failure = true;
            }
        },
        'D' => quote::quote! {
            if __backslash_char >= '0' && __backslash_char <= '9' {
                __peggle_failure = true;
            }
        },
        's' => quote::quote! {
            if __backslash_char != ' ' && __backslash_char != '\t' && __backslash_char != '\n' && __backslash_char != '\r' && __backslash_char != '\x0b' && __backslash_char != '\x0c' {
                __peggle_failure = true;
            }
        },
        'S' => quote::quote! {
            if __backslash_char == ' ' || __backslash_char == '\t' || __backslash_char == '\n' || __backslash_char == '\r' || __backslash_char == '\x0b' || __backslash_char == '\x0c' {
                __peggle_failure = true;
            }
        },
        '\\' | '{' | '}' | '[' | ']' | '(' | ')' | '^' | '$' | '.' | '|' | '*' | '+' | '?'
        | '<' | '>' | '&' => quote::quote! {
            if __backslash_char != #character {
                __peggle_failure = true;
            }
        },
        _ => panic!("unrecognized backslash-escaped character '{}'", character),
    };

    nested_expressions
        .last_mut()
        .unwrap()
        .last_mut()
        .unwrap()
        .push(quote::quote! {
            __peggle_failure = false;
            for #loop_iter_ident in 0..#max {
                if let Some(__backslash_char) = __peggle_curr.peek() {
                    #character_match_tokens
                } else {
                    __peggle_failure = true;
                }

                if __peggle_failure {
                    if #loop_iter_ident >= #min {
                        __peggle_failure = false;
                    }
                    break
                }
                __peggle_curr.next();
            }

            if __peggle_failure {
                break #choice_lifetime
            }
        });
}

fn match_one_of(
    index: &mut peggle::Index,
    nested_expressions: &mut Vec<Vec<Vec<proc_macro2::TokenStream>>>,
) {
    let mut bracket_possibilities = Vec::new();

    let expr_depth = nested_expressions.len().to_string();
    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_{}", expr_depth));
    let bracket_lifetime =
        id_to_lifetime(quote::format_ident!("__bracket_lifetime_{}", expr_depth));

    let inverted = Some('^') == index.peek();
    if inverted {
        index.next();
    }

    'refactor_this: {
        if let Some(lit @ (']' | '-')) = index.peek() {
            index.next();
            if let Some('-') = index.peek() {
                // TODO: this code is duplicated
                index.next();
                match index
                    .next()
                    .expect("missing closing bracket (`]`) character in regex")
                {
                    ']' => {
                        bracket_possibilities.push(quote::quote! {
                            if __bracket_char == #lit || __bracket_char == '-' {
                                __peggle_failure = false;
                                break #bracket_lifetime
                            }
                        });
                        break 'refactor_this;
                    }
                    end_char if end_char < lit => panic!(
                        "range {}-{} invalid: {} comes before {} in ordering",
                        lit, end_char, end_char, lit
                    ),
                    end_char => bracket_possibilities.push(quote::quote! {
                        if __bracket_char >= #lit && __bracket_char <= #end_char {
                            __peggle_failure = false;
                            break #bracket_lifetime
                        }
                    }),
                }
            } else {
                bracket_possibilities.push(quote::quote! {
                    if __bracket_char == #lit {
                        __peggle_failure = false;
                        break #bracket_lifetime
                    }
                })
            }
        }

        loop {
            let c = index
                .next()
                .expect("missing closing bracket (`]`) character in regex");
            match (c, index.peek()) {
                (']', _) => break,
                ('-', _) => panic!("dash must be preceded by starting value that is not also an ending value for another dash"),
                (_, Some('-')) => {
                    // Handle "a-z" case (using a dash to indicate a range of values)
                    index.next();
                    match index.next().expect("missing closing bracket (`]`) character in regex") {
                        ']' => {
                            bracket_possibilities.push(quote::quote! {
                                if __bracket_char == #c || __bracket_char == '-' {
                                    __peggle_failure = false;
                                    break #bracket_lifetime
                                }
                            });
                            break
                        }
                        end_char if end_char < c => panic!("range {}-{} invalid: {} comes before {} in ordering", c, end_char, end_char, c),
                        end_char => bracket_possibilities.push(quote::quote! {
                            if __bracket_char >= #c && __bracket_char <= #end_char {
                                __peggle_failure = false;
                                break #bracket_lifetime
                            }
                        }),
                    }
                }
                ('[', Some(':')) => match_posix_class(index, &mut bracket_possibilities, &bracket_lifetime),
                _ => bracket_possibilities.push(quote::quote!{
                    if __bracket_char == #c {
                        __peggle_failure = false;
                        break #bracket_lifetime
                    }
                }),
            }
        }
    }

    // Handle any possible repetition/variable number suffixes
    let (min, max, advanced_index) = get_repetition_bounds(*index);
    *index = advanced_index;

    let loop_revert_index = quote::format_ident!("__loop_revert_{}", expr_depth);
    let loop_iter_ident = quote::format_ident!("__loop_iter_{}", expr_depth);

    nested_expressions
        .last_mut()
        .expect("9")
        .last_mut()
        .expect("10")
        .push(quote::quote! {
            let mut #loop_revert_index = __peggle_curr;
            for #loop_iter_ident in 0..#max {
                #loop_revert_index = __peggle_curr;

                match __peggle_curr.next() {
                    Some(__bracket_char) => {
                        __peggle_failure = true;
                        #bracket_lifetime: {
                            #(#bracket_possibilities)*
                        }

                        if #inverted {
                            __peggle_failure = !__peggle_failure
                        }
                    }
                    None => __peggle_failure = true,
                }

                if __peggle_failure {
                    if #loop_iter_ident >= #min {
                        __peggle_curr = #loop_revert_index; // Rewind to where last successful loop iteration finished
                        __peggle_failure = false;
                    }
                    break
                }
            }

            if __peggle_failure {
                break #choice_lifetime
            }
        });
}

fn match_character(
    character: char,
    index: &mut peggle::Index,
    nested_expressions: &mut Vec<Vec<Vec<proc_macro2::TokenStream>>>,
) {
    let (min, max, advanced_index) = get_repetition_bounds(*index);
    *index = advanced_index;

    let expr_depth = nested_expressions.len().to_string();
    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_{}", expr_depth));
    let loop_iter_ident = quote::format_ident!("__loop_iter_{}", expr_depth);

    nested_expressions
        .last_mut()
        .expect("11")
        .last_mut()
        .expect("12")
        .push(quote::quote! {
            __peggle_failure = false;
            for #loop_iter_ident in 0..#max {
                let Some(#character) = __peggle_curr.peek() else {
                    if #loop_iter_ident < #min {
                        __peggle_failure = true;
                        break #choice_lifetime
                    } else {
                        break
                    }
                };
                __peggle_curr.next();
            }
        });
}

fn match_any_character(
    index: &mut peggle::Index,
    nested_expressions: &mut Vec<Vec<Vec<proc_macro2::TokenStream>>>,
) {
    let (min, max, advanced_index) = get_repetition_bounds(*index);
    *index = advanced_index;

    let expr_depth = nested_expressions.len().to_string();
    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_{}", expr_depth));
    let loop_iter_ident = quote::format_ident!("__loop_iter_{}", expr_depth);

    nested_expressions
        .last_mut()
        .expect("13")
        .last_mut()
        .expect("14")
        .push(quote::quote! {
            __peggle_failure = false;
            for #loop_iter_ident in 0..#max {
                let Some(_) = __peggle_curr.next() else {
                    if #loop_iter_ident < #min {
                        __peggle_failure = true;
                        break #choice_lifetime
                    } else {
                        break
                    }
                };
            }
        });
}

fn coalesce_topmost_expression(
    index: &mut peggle::Index,
    nested_expressions: &mut Vec<Vec<Vec<proc_macro2::TokenStream>>>,
    requirements: Option<&mut FieldRequirements>,
) {
    let expr_depth = nested_expressions.len().to_string();
    let choice_revert_index = quote::format_ident!("__choice_revert_index_{}", expr_depth);
    let expr_lifetime =
        id_to_lifetime(quote::format_ident!("__expression_lifetime_{}", expr_depth));
    let choice_lifetime = id_to_lifetime(quote::format_ident!("__choice_lifetime_{}", expr_depth));

    let mut expression_tokens = quote::quote! {
        let #choice_revert_index = __peggle_curr;
    };

    for choice_tokens in nested_expressions.pop().expect("15") {
        // Handle each possible choice in order, breaking upon the first success
        expression_tokens.extend(quote::quote! {
            __peggle_failure = false;
            __peggle_curr = #choice_revert_index;
            #choice_lifetime: {
                #(#choice_tokens)*
            }
            if !__peggle_failure {
                break #expr_lifetime // Choice matched--break at the given index
            }
            // No match--move to next choice (resetting both `__peggle_failure` and `__peggle_curr`) or else return failure
        });
    }

    let underlayer_choice_lifetime = id_to_lifetime(quote::format_ident!(
        "__choice_lifetime_{}",
        nested_expressions.len().to_string()
    ));

    let (min, max, advanced_index) = get_repetition_bounds(*index);
    *index = advanced_index;

    if let Some(requirements) = requirements {
        requirements.pop_nested_expr(min, max).expect("Failed");
    }

    let loop_revert_index = quote::format_ident!("__loop_revert_{}", expr_depth);
    let loop_iter_ident = quote::format_ident!("__loop_iter_{}", expr_depth);

    nested_expressions
        .last_mut()
        .expect("peggle expression had one too many closing parentheses")
        .last_mut()
        .expect("17")
        .push(quote::quote! {
            let mut #loop_revert_index = __peggle_curr;
            for #loop_iter_ident in 0..#max {
                #loop_revert_index = __peggle_curr;

                #expr_lifetime: {
                    #expression_tokens
                }

                if __peggle_failure {
                    if #loop_iter_ident >= #min {
                        __peggle_curr = #loop_revert_index; // Rewind to where last successful loop iteration finished
                        __peggle_failure = false;
                    }
                    break
                }
            }

            if __peggle_failure {
                break #underlayer_choice_lifetime
            }
        });
}

fn get_repetition_bounds(mut index: peggle::Index<'_>) -> (usize, usize, peggle::Index<'_>) {
    match index.peek() {
        Some('?') => {
            index.next();
            (0usize, 1usize, index)
        }
        Some('*') => {
            index.next();
            (0, usize::MAX, index)
        }
        Some('+') => {
            index.next();
            (1, usize::MAX, index)
        }
        Some('{') => {
            index.next();

            let min: usize;
            let max: usize;

            if let Some(',') = index.peek() {
                min = 0;
                index.next();
                if let Some('}') = index.peek() {
                    panic!("Range missing high value after comma within curly braces")
                }
            } else {
                (min, index) = usize::parse_at(index)
                    .expect("invalid `min` value contained within curly braces: {}");
                let Some(',') = index.peek() else {
                    let Some('}') = index.next() else {
                        panic!("invalid value contained within curly braces: missing closing curly brace")
                    };
                    return (min, min, index)
                };
                index.next();
            }

            if let Some('}') = index.peek() {
                max = usize::MAX;
                index.next();
            } else {
                (max, index) = usize::parse_at(index)
                    .expect("invalid `max` value contained within curly braces: {}");
                let Some('}') = index.next() else {
                    panic!("invalid value contained within curly braces: missing closing curly brace")
                };
            }
            assert!(
                min <= max,
                "min value must be less than or equal to max value in {{min,max}} range"
            );
            (min, max, index)
        }
        _ => (1usize, 1usize, index),
    }
}
