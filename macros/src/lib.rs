use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, GenericArgument, Lit, Path, PathArguments, Type,
};

#[proc_macro_derive(Rod, attributes(rod))]
pub fn derive_rod(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Inject generic bounds (T: RodSchema)
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Check for unsupported types
    match input.data {
        Data::Struct(data) => match impl_struct_schema(&data.fields) {
            Ok(expanded) => {
                let final_code = quote! {
                    impl #impl_generics ::rod_rs::RodSchema for #name #ty_generics #where_clause {
                        fn schema() -> Box<dyn ::rod_rs::RodValidator> {
                            use ::rod_rs::OptionalExtension as _;
                            use ::rod_rs::NullableExtension as _;
                            #expanded
                        }
                    }
                };
                TokenStream::from(final_code)
            }
            Err(e) => TokenStream::from(e.to_compile_error()),
        },
        Data::Enum(_) => {
            return quote_spanned! {
                name.span() => compile_error!("#[derive(Rod)] currently supports Structs only.");
            }
            .into();
        }
        Data::Union(_) => {
            return quote_spanned! {
                name.span() => compile_error!("#[derive(Rod)] does not support Unions.");
            }
            .into();
        }
    }
}
fn impl_struct_schema(fields: &Fields) -> Result<proc_macro2::TokenStream, syn::Error> {
    match fields {
        Fields::Named(named) => {
            let mut inserts = Vec::new();

            for f in &named.named {
                let ident = &f.ident;
                let name_str = ident.as_ref().unwrap().to_string();
                let name_clean = name_str.trim_start_matches("r#");
                let ty = &f.ty;

                let (inner_type, is_optional) = split_outer_option(ty);
                let is_string_like = is_type_string(inner_type);

                let mut validator = map_type_recursive(inner_type);

                for attr in &f.attrs {
                    if attr.path().is_ident("rod") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("min") {
                                let val = meta.value()?;
                                let lit: Lit = val.parse()?;
                                let n = parse_lit_to_f64(&lit)?;
                                if is_string_like {
                                    validator = quote! { #validator.min(#n as usize) };
                                } else {
                                    validator = quote! { #validator.min(#n) };
                                }
                                return Ok(());
                            }
                            if meta.path.is_ident("max") {
                                let val = meta.value()?;
                                let lit: Lit = val.parse()?;
                                let n = parse_lit_to_f64(&lit)?;
                                if is_string_like {
                                    validator = quote! { #validator.max(#n as usize) };
                                } else {
                                    validator = quote! { #validator.max(#n) };
                                }
                                return Ok(());
                            }
                            if meta.path.is_ident("email") {
                                validator = quote! { #validator.email() };
                                return Ok(());
                            }
                            if meta.path.is_ident("url") {
                                validator = quote! { #validator.url() };
                                return Ok(());
                            }
                            if meta.path.is_ident("uuid") {
                                validator = quote! { #validator.uuid() };
                                return Ok(());
                            }
                            if meta.path.is_ident("cuid") {
                                validator = quote! { #validator.cuid() };
                                return Ok(());
                            }
                            if meta.path.is_ident("regex") {
                                let val = meta.value()?;
                                let lit: Lit = val.parse()?;
                                if let Lit::Str(s) = lit {
                                    let pattern = s.value();
                                    validator = quote! { #validator.regex(#pattern) };
                                } else {
                                    return Err(meta.error("regex must be a string literal"));
                                }
                                return Ok(());
                            }
                            Ok(())
                        })?;
                    }
                }

                if is_optional {
                    validator = quote! { #validator.nullable().optional() };
                }

                inserts.push(quote! {
                    map.insert(
                        #name_clean.to_string(),
                        Box::new(#validator) as Box<dyn ::rod_rs::RodValidator>
                    );
                });
            }

            Ok(quote! {
                let mut map = std::collections::HashMap::new();
                #(#inserts)*
                Box::new(::rod_rs::object(map))
            })
        }
        _ => Ok(quote! {
            compile_error!("#[derive(Rod)] does not support Tuple Structs.")
        }),
    }
}

fn parse_lit_to_f64(lit: &Lit) -> Result<f64, syn::Error> {
    match lit {
        Lit::Int(i) => i.base10_parse::<f64>(),
        Lit::Float(f) => f.base10_parse::<f64>(),
        _ => Err(syn::Error::new_spanned(lit, "Expected a number")),
    }
}

fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(syn::parse_quote!(::rod_rs::RodSchema));
        }
    }
    generics
}

fn split_outer_option(ty: &Type) -> (&Type, bool) {
    if let Type::Path(tp) = ty {
        if path_is(&tp.path, "Option") {
            if let Some(segment) = tp.path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return (inner, true);
                    }
                }
            }
        }
    }
    (ty, false)
}

fn is_type_string(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        if path_is(&tp.path, "String") || path_is(&tp.path, "str") || path_is(&tp.path, "Cow") {
            return true;
        }
        if path_is(&tp.path, "Option") {
            if let Some(segment) = tp.path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return is_type_string(inner);
                    }
                }
            }
        }
    }
    if let Type::Reference(tr) = ty {
        return is_type_string(&tr.elem);
    }
    false
}

fn path_is(path: &Path, name: &str) -> bool {
    if let Some(seg) = path.segments.last() {
        return seg.ident == name;
    }
    false
}

fn map_type_recursive(ty: &Type) -> proc_macro2::TokenStream {
    if let Type::Reference(tr) = ty {
        return map_type_recursive(&tr.elem);
    }

    if let Type::Array(arr) = ty {
        let inner = map_type_recursive(&arr.elem);
        return quote! { ::rod_rs::array(#inner) };
    }

    if let Type::Tuple(tup) = ty {
        let items = tup.elems.iter().map(|elem| {
            let val = map_type_recursive(elem);
            quote! { Box::new(#val) }
        });
        return quote! { ::rod_rs::tuple(vec![ #(#items),* ]) };
    }

    if let Type::Path(tp) = ty {
        let path = &tp.path;

        // Option<T> inside a recursive type (e.g. Vec<Option<T>>)
        if path_is(path, "Option") {
            if let Some(segment) = path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        let v = map_type_recursive(inner);
                        return quote! { #v.nullable().optional() };
                    }
                }
            }
        }

        // Vec, VecDeque, LinkedList
        if path_is(path, "Vec") || path_is(path, "VecDeque") || path_is(path, "LinkedList") {
            if let Some(segment) = path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        let v = map_type_recursive(inner);
                        return quote! { ::rod_rs::array(#v) };
                    }
                }
            }
        }

        // Sets
        if path_is(path, "HashSet") || path_is(path, "BTreeSet") {
            if let Some(segment) = path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        let v = map_type_recursive(inner);
                        return quote! { ::rod_rs::set(#v) };
                    }
                }
            }
        }

        // Maps
        if path_is(path, "HashMap") || path_is(path, "BTreeMap") {
            if let Some(segment) = path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    let types: Vec<&Type> = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let GenericArgument::Type(t) = arg {
                                Some(t)
                            } else {
                                None
                            }
                        })
                        .collect();

                    if types.len() >= 2 {
                        let key_v = map_type_recursive(types[0]);
                        let val_v = map_type_recursive(types[1]);
                        return quote! { ::rod_rs::record(#key_v, #val_v) };
                    }
                }
            }
        }

        // Smart Pointers
        if path_is(path, "Box")
            || path_is(path, "Arc")
            || path_is(path, "Rc")
            || path_is(path, "Cell")
            || path_is(path, "RefCell")
            || path_is(path, "Mutex")
            || path_is(path, "RwLock")
        {
            if let Some(segment) = path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return map_type_recursive(inner);
                    }
                }
            }
        }

        // Cow
        if path_is(path, "Cow") {
            if let Some(segment) = path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let GenericArgument::Type(inner) = arg {
                            return map_type_recursive(inner);
                        }
                    }
                }
            }
        }

        // --- PRIMITIVES ---
        if path_is(path, "String") || path_is(path, "str") || path_is(path, "char") {
            return quote! { ::rod_rs::string() };
        }
        if path_is(path, "bool") {
            return quote! { ::rod_rs::boolean() };
        }
        if path_is(path, "f32") || path_is(path, "f64") {
            return quote! { ::rod_rs::number() };
        }
        if path_is(path, "i8")
            || path_is(path, "i16")
            || path_is(path, "i32")
            || path_is(path, "i64")
            || path_is(path, "i128")
            || path_is(path, "isize")
            || path_is(path, "u8")
            || path_is(path, "u16")
            || path_is(path, "u32")
            || path_is(path, "u64")
            || path_is(path, "u128")
            || path_is(path, "usize")
        {
            return quote! { ::rod_rs::number().int() };
        }

        // Dates (Chrono)
        if path_is(path, "DateTime")
            || path_is(path, "NaiveDate")
            || path_is(path, "NaiveDateTime")
            || path_is(path, "SystemTime")
        {
            return quote! { ::rod_rs::date() };
        }

        return quote! { <#ty as ::rod_rs::RodSchema>::schema() };
    }

    quote! { ::rod_rs::any() }
}
