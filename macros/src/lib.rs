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
            Ok((validator_expanded, spec_expanded)) => {
                let expanded = quote! {
                    impl #impl_generics ::rod_rs::RodSchema for #name #ty_generics #where_clause {
                        fn schema() -> Box<dyn ::rod_rs::RodValidator> {
                            use ::rod_rs::OptionalExtension as _;
                            use ::rod_rs::NullableExtension as _;
                            #validator_expanded
                        }

                        fn spec() -> ::rod_rs::RodSpec {
                            #spec_expanded
                        }
                    }

                    impl #impl_generics #name #ty_generics #where_clause {
                        pub fn json_schema() -> ::serde_json::Value {
                            let mut s = <Self as ::rod_rs::RodSchema>::spec().to_json_schema();
                            if let Some(obj) = s.as_object_mut() {
                                obj.insert("$schema".into(), "https://json-schema.org/draft/2020-12/schema".into());
                            }
                            s
                        }
                    }
                };
                TokenStream::from(expanded)
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

#[derive(Default)]
struct FieldAttributes {
    min: Option<f64>,
    max: Option<f64>,
    email: bool,
    url: bool,
    uuid: bool,
    cuid: bool,
    regex: Option<String>,
}

fn impl_struct_schema(
    fields: &Fields,
) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), syn::Error> {
    match fields {
        Fields::Named(named) => {
            let mut validator_inserts = Vec::new();
            let mut spec_inserts = Vec::new();

            for f in &named.named {
                let ident = f.ident.as_ref().unwrap();
                let name_str = ident.to_string().trim_start_matches("r#").to_string();
                let ty = &f.ty;
                let (inner_type, is_optional) = split_outer_option(ty);
                let is_string_like = is_type_string(inner_type);

                let mut attrs = FieldAttributes::default();
                for attr in &f.attrs {
                    if attr.path().is_ident("rod") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("min") {
                                attrs.min = Some(parse_lit_to_f64(&meta.value()?.parse()?)?);
                            } else if meta.path.is_ident("max") {
                                attrs.max = Some(parse_lit_to_f64(&meta.value()?.parse()?)?);
                            } else if meta.path.is_ident("email") {
                                attrs.email = true;
                            } else if meta.path.is_ident("url") {
                                attrs.url = true;
                            } else if meta.path.is_ident("uuid") {
                                attrs.uuid = true;
                            } else if meta.path.is_ident("cuid") {
                                attrs.cuid = true;
                            } else if meta.path.is_ident("regex") {
                                let val: Lit = meta.value()?.parse()?;
                                if let Lit::Str(s) = val {
                                    attrs.regex = Some(s.value());
                                }
                            }
                            Ok(())
                        })?;
                    }
                }

                let mut v_code = map_type_recursive(inner_type);
                if let Some(n) = attrs.min {
                    v_code = if is_string_like {
                        quote! { #v_code.min(#n as usize) }
                    } else {
                        quote! { #v_code.min(#n) }
                    };
                }
                if let Some(n) = attrs.max {
                    v_code = if is_string_like {
                        quote! { #v_code.max(#n as usize) }
                    } else {
                        quote! { #v_code.max(#n) }
                    };
                }
                if attrs.email {
                    v_code = quote! { #v_code.email() };
                }
                if attrs.url {
                    v_code = quote! { #v_code.url() };
                }
                if attrs.uuid {
                    v_code = quote! { #v_code.uuid() };
                }
                if attrs.cuid {
                    v_code = quote! { #v_code.cuid() };
                }
                if let Some(ref re) = attrs.regex {
                    v_code = quote! { #v_code.regex(#re) };
                }
                if is_optional {
                    v_code = quote! { #v_code.nullable().optional() };
                }

                validator_inserts.push(quote! { map.insert(#name_str.to_string(), Box::new(#v_code) as Box<dyn ::rod_rs::RodValidator>); });

                let mut s_code = map_type_to_spec_recursive(inner_type, &attrs);
                if is_optional {
                    s_code = quote! { ::rod_rs::RodSpec::Optional(Box::new(::rod_rs::RodSpec::Nullable(Box::new(#s_code)))) };
                }
                spec_inserts.push(quote! { properties.insert(#name_str.to_string(), #s_code); });
            }

            let v_final = quote! { let mut map = std::collections::HashMap::new(); #(#validator_inserts)* Box::new(::rod_rs::object(map)) };
            let s_final = quote! { let mut properties = std::collections::HashMap::new(); #(#spec_inserts)* ::rod_rs::RodSpec::Object { properties, strict: Some(false) } };
            Ok((v_final, s_final))
        }

        Fields::Unnamed(unnamed) => {
            let mut validator_items = Vec::new();
            let mut spec_items = Vec::new();

            for f in &unnamed.unnamed {
                let ty = &f.ty;
                let v_code = map_type_recursive(ty);
                let s_code = map_type_to_spec_recursive(ty, &FieldAttributes::default());

                validator_items
                    .push(quote! { Box::new(#v_code) as Box<dyn ::rod_rs::RodValidator> });
                spec_items.push(quote! { #s_code });
            }

            let v_final = quote! { Box::new(::rod_rs::tuple(vec![ #(#validator_items),* ])) };
            let s_final = quote! { ::rod_rs::RodSpec::Tuple { items: vec![ #(#spec_items),* ] } };

            Ok((v_final, s_final))
        }

        Fields::Unit => Ok((
            quote! { Box::new(::rod_rs::object(std::collections::HashMap::new())) },
            quote! { ::rod_rs::RodSpec::Object { properties: std::collections::HashMap::new(), strict: Some(false) } },
        )),
    }
}

fn map_type_to_spec_recursive(ty: &Type, attrs: &FieldAttributes) -> proc_macro2::TokenStream {
    if let Type::Reference(tr) = ty {
        return map_type_to_spec_recursive(&tr.elem, attrs);
    }

    if let Type::Path(tp) = ty {
        let path = &tp.path;

        if is_type_string(ty) {
            let min = opt_to_tokens(attrs.min.map(|n| n as usize));
            let max = opt_to_tokens(attrs.max.map(|n| n as usize));
            let email = attrs.email;
            let url = attrs.url;
            let uuid = attrs.uuid;
            let cuid = attrs.cuid;
            let regex = opt_to_tokens(attrs.regex.clone());

            return quote! {
                ::rod_rs::RodSpec::String {
                    min: #min, max: #max, length: None, email: Some(#email), url: Some(#url),
                    uuid: Some(#uuid), cuid: Some(#cuid), datetime: None, ip: None,
                    regex: #regex, starts_with: None, ends_with: None, includes: None, trim: false
                }
            };
        }

        if is_int_type(path) || path_is(path, "f32") || path_is(path, "f64") {
            let min = opt_to_tokens(attrs.min);
            let max = opt_to_tokens(attrs.max);
            let is_int = is_int_type(path);
            return quote! { ::rod_rs::RodSpec::Number { min: #min, max: #max, int: Some(#is_int) } };
        }

        if path_is(path, "bool") {
            return quote! { ::rod_rs::RodSpec::Boolean };
        }

        if path_is(path, "Vec") {
            if let Some(inner) = get_first_generic(path) {
                let inner_spec = map_type_to_spec_recursive(inner, &FieldAttributes::default());
                return quote! {
                    ::rod_rs::RodSpec::Array { items: Box::new(#inner_spec), min: None, max: None }
                };
            }
        }

        return quote! { <#ty as ::rod_rs::RodSchema>::spec() };
    }

    quote! { ::rod_rs::RodSpec::Any }
}

fn opt_to_tokens<T: quote::ToTokens>(opt: Option<T>) -> proc_macro2::TokenStream {
    match opt {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    }
}

fn get_first_generic(path: &Path) -> Option<&Type> {
    let segment = path.segments.last()?;
    if let PathArguments::AngleBracketed(args) = &segment.arguments {
        if let Some(GenericArgument::Type(ty)) = args.args.first() {
            return Some(ty);
        }
    }
    None
}

fn is_int_type(path: &Path) -> bool {
    [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
    ]
    .iter()
    .any(|&t| path_is(path, t))
}

fn path_is(path: &Path, name: &str) -> bool {
    path.segments.last().map_or(false, |s| s.ident == name)
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
            if let Some(inner) = get_first_generic(&tp.path) {
                return (inner, true);
            }
        }
    }
    (ty, false)
}

fn is_type_string(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        let path = &tp.path;
        if path_is(path, "String") || path_is(path, "str") || path_is(path, "Cow") {
            return true;
        }
        if path_is(path, "Option") {
            if let Some(inner) = get_first_generic(path) {
                return is_type_string(inner);
            }
        }
    }
    if let Type::Reference(tr) = ty {
        return is_type_string(&tr.elem);
    }
    false
}

fn parse_lit_to_f64(lit: &Lit) -> Result<f64, syn::Error> {
    match lit {
        Lit::Int(i) => i.base10_parse::<f64>(),
        Lit::Float(f) => f.base10_parse::<f64>(),
        _ => Err(syn::Error::new_spanned(lit, "Expected a number")),
    }
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
        if path_is(path, "Option") {
            if let Some(inner) = get_first_generic(path) {
                let v = map_type_recursive(inner);
                return quote! { #v.nullable().optional() };
            }
        }
        if path_is(path, "Vec") || path_is(path, "VecDeque") || path_is(path, "LinkedList") {
            if let Some(inner) = get_first_generic(path) {
                let v = map_type_recursive(inner);
                return quote! { ::rod_rs::array(#v) };
            }
        }
        if path_is(path, "HashSet") || path_is(path, "BTreeSet") {
            if let Some(inner) = get_first_generic(path) {
                let v = map_type_recursive(inner);
                return quote! { ::rod_rs::set(#v) };
            }
        }
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
        if ["Box", "Arc", "Rc", "Cell", "RefCell", "Mutex", "RwLock"]
            .iter()
            .any(|&s| path_is(path, s))
        {
            if let Some(inner) = get_first_generic(path) {
                return map_type_recursive(inner);
            }
        }
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
        if is_type_string(ty) {
            return quote! { ::rod_rs::string() };
        }
        if path_is(path, "bool") {
            return quote! { ::rod_rs::boolean() };
        }
        if path_is(path, "f32") || path_is(path, "f64") {
            return quote! { ::rod_rs::number() };
        }
        if is_int_type(path) {
            return quote! { ::rod_rs::number().int() };
        }
        if ["DateTime", "NaiveDate", "NaiveDateTime", "SystemTime"]
            .iter()
            .any(|&s| path_is(path, s))
        {
            return quote! { ::rod_rs::date() };
        }

        return quote! { <#ty as ::rod_rs::RodSchema>::schema() };
    }

    quote! { ::rod_rs::any() }
}
