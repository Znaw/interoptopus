use crate::types::Attributes;
use crate::util::extract_doc_lines;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Expr, ExprLit, ExprUnary, ItemEnum, Lit, Meta, NestedMeta, UnOp};

fn derive_variant_info(item: ItemEnum, idents: &[Ident], names: &[String], values: &[i64], docs: &[String]) -> TokenStream {
    let name = item.ident.to_string();
    let name_ident = syn::Ident::new(&name, item.ident.span());

    quote! {
        unsafe impl ::interoptopus::lang::rust::VariantInfo for #name_ident {
            fn variant_info(&self) -> ::interoptopus::lang::c::Variant {
                match self {
                    #(
                       Self::#idents => {
                            let documentation = ::interoptopus::lang::c::Documentation::from_line(#docs);
                            ::interoptopus::lang::c::Variant::new(#names.to_string(), #values, documentation)
                       },
                    )*
                }
            }
        }
    }
}

fn find_valid_repr(item: &ItemEnum) -> Option<String> {
    let valid_reprs = ["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "C"];
    let mut found_repr = false;

    for attr in &item.attrs {
        if attr.path.is_ident("repr") {
            found_repr = true;
            if let Ok(Meta::List(list)) = attr.parse_meta() {
                for repr in list.nested {
                    if let NestedMeta::Meta(Meta::Path(path)) = repr {
                        if let Some(ident) = path.get_ident() {
                            let repr_str = ident.to_string();
                            // Check if it's an integer type
                            if valid_reprs.contains(&repr_str.as_str()) {
                                return Some(repr_str);
                            } else if repr_str == "C" {
                                return None;
                            }
                        }
                    }
                }
            }
        }
    }

    if !found_repr {
        panic!("Enum `{}` must have `#[repr()]` annotation.", item.ident);
    }

    None
}

fn find_lit_int(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Int(int), .. }) => Some(int.base10_parse().unwrap()),
        Expr::Unary(ExprUnary { expr, op: UnOp::Neg(_), .. }) => {
            match find_lit_int(expr) {
                Some(int) => Some(-int), // Change the sign
                None => None,
            }
        }
        _ => None,
    }
}

pub fn ffi_type_enum(attributes: &Attributes, input: TokenStream, item: ItemEnum) -> TokenStream {
    let doc_line = extract_doc_lines(&item.attrs).join("\n");

    // Find the valid repr attribute, if present
    let primitive_type = find_valid_repr(&item);
    let ctype_from_string = match primitive_type {
        Some(type_string) => match type_string.as_str() {
            "u8" => quote! { Some(::interoptopus::lang::c::PrimitiveType::U8) },
            "u16" => quote! { Some(::interoptopus::lang::c::PrimitiveType::U16) },
            "u32" => quote! { Some(::interoptopus::lang::c::PrimitiveType::U32) },
            "u64" => quote! { Some(::interoptopus::lang::c::PrimitiveType::U64) },
            "i8" => quote! { Some(::interoptopus::lang::c::PrimitiveType::I8) },
            "i16" => quote! { Some(::interoptopus::lang::c::PrimitiveType::I16) },
            "i32" => quote! { Some(::interoptopus::lang::c::PrimitiveType::I32) },
            "i64" => quote! { Some(::interoptopus::lang::c::PrimitiveType::I64) },
            _ => quote! { None },
        },
        None => quote! { None },
    };

    let span = item.ident.span();
    let name = item.ident.to_string();
    let ffi_name = attributes.name.clone().unwrap_or_else(|| name.clone());
    let name_ident = syn::Ident::new(&name, span);
    let namespace = attributes.namespace.clone().unwrap_or_default();

    let mut variant_names = Vec::new();
    let mut variant_idents = Vec::new();
    let mut variant_values = Vec::new();
    let mut variant_docs = Vec::new();
    let mut next_id = 0;

    for variant in &item.variants {
        let ident = variant.ident.to_string();
        let variant_doc_line = extract_doc_lines(&variant.attrs).join("\n");

        let this_id = if let Some((_, expr)) = &variant.discriminant {
            if let Some(id) = find_lit_int(expr) {
                next_id = id + 1;
                id
            } else {
                panic!("Invalid expression.");
            }
        } else {
            let id = next_id;
            next_id += 1;
            id
        };

        if !attributes.skip.contains_key(&ident) {
            variant_idents.push(syn::Ident::new(&ident, span));
            variant_names.push(ident);
            variant_values.push(this_id);
            variant_docs.push(variant_doc_line);
        }
    }

    let variant_infos = derive_variant_info(item, &variant_idents, &variant_names, &variant_values, &variant_docs);

    let ctype_info_return = if attributes.patterns.contains_key("ffi_error") {
        quote! {
            use ::interoptopus::patterns::result::FFIError as _;
            let success_variant = Self::SUCCESS.variant_info();
            let the_success_enum = ::interoptopus::patterns::result::FFIErrorEnum::new(rval, success_variant);
            let the_pattern = ::interoptopus::patterns::TypePattern::FFIErrorEnum(the_success_enum);
            ::interoptopus::lang::c::CType::Pattern(the_pattern)
        }
    } else {
        quote! { ::interoptopus::lang::c::CType::Enum(rval) }
    };

    quote! {
        #input

        #variant_infos

        unsafe impl ::interoptopus::lang::rust::CTypeInfo for #name_ident {
            fn type_info() -> ::interoptopus::lang::c::CType {
                use ::interoptopus::lang::rust::VariantInfo;

                let mut variants = ::std::vec::Vec::new();
                let documentation = ::interoptopus::lang::c::Documentation::from_line(#doc_line);
                let mut meta = ::interoptopus::lang::c::Meta::with_namespace_documentation(#namespace.to_string(), documentation);

                #({
                    variants.push(Self::#variant_idents.variant_info());
                })*

                let rval = ::interoptopus::lang::c::EnumType::new(#ffi_name.to_string(), variants, #ctype_from_string, meta);

                #ctype_info_return
            }
        }
    }
}
