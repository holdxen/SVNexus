use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Data, DeriveInput, Expr, Fields, Ident, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

// 定义参数结构
#[derive(Debug)]
struct KeyValue {
    key: Ident,
    value: Expr,
}

// 定义参数列表
#[derive(Debug)]
struct MacroArgs {
    args: Punctuated<KeyValue, Token![,]>,
}

// 实现 Parse trait 来解析 k=v 格式的参数
impl Parse for KeyValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<Expr>()?;
        Ok(KeyValue { key, value })
    }
}

impl Parse for MacroArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let args = Punctuated::parse_terminated(input)?;
        Ok(MacroArgs { args })
    }
}

#[proc_macro_attribute]
pub fn return_as_is(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn enum_converter(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);

    // 确保这是枚举
    let Data::Enum(data_enum) = &mut input.data else {
        panic!("enum_converter 属性宏仅支持枚举类型");
    };

    // 从宏属性中提取repr_type（例如 #[enum_converter(repr_type = ffi::svn_depth_t)]）
    // 这里假设attr是 `repr_type = Type`，使用syn解析

    // let repr_name;
    let repr_type = if attr.is_empty() {
        // repr_name = quote! { u32 };
        quote!(u32) // 默认i32
    } else {
        // panic!("attr expr: {:#?}", attr);
        let attr_parser = parse_macro_input!(attr as MacroArgs);
        // let syn::Expr::Assign(expr) = attr_parser else {
        //     panic!("Unsuppored expr: {:?}", attr_parser)
        // };
        //

        let item = attr_parser
            .args
            .iter()
            .find(|v| v.key == "repr_type")
            .map(|v| v.value.to_token_stream())
            .unwrap_or(quote! {u32});
        // if let Some(item) = item {
        //     let syn::Expr::Path(path) = item.clone() else {
        //         panic!("Unsuppored expr: {:?}", item)
        //     };

        //     repr_name = path.path.segments.last().map(|segment| &segment.ident).to_token_stream();

        //     item.to_token_stream()
        // } else {
        //     quote! { u32 }
        // }

        item
    };

    // 提取枚举名
    let enum_name = &input.ident;

    // 收集变体信息：变体名和discriminant表达式
    let mut clean_variants = Vec::new();
    let mut from_matches = Vec::new();
    let mut try_from_matches = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;

        // 假设无字段变体（unit variants），如你的示例。如果有字段，需调整
        if !matches!(variant.fields, Fields::Unit) {
            panic!("仅支持无字段的unit枚举变体");
        }

        // 获取discriminant表达式（如果没有，panic，因为需求是转换有disc的）
        let disc = variant
            .discriminant
            .as_ref()
            .expect("变体必须有discriminant表达式");

        // disc.1 是Expr，我们转换为TokenStream以复制原表达式（如 ffi::const）
        let disc_expr = &disc.1;

        // 为clean枚举添加变体（无disc）
        clean_variants.push(quote! {
            #variant_name,
        });

        // 为 From<Enum> for ReprType 生成 match arm: Enum::Var => 原disc_expr,
        from_matches.push(quote! {
            #enum_name::#variant_name => #disc_expr,
        });

        // 为 TryFrom<ReprType> for Enum 生成 match arm: 原disc_expr => Ok(Enum::Var),
        try_from_matches.push(quote! {
            #disc_expr => Ok(#enum_name::#variant_name),
        });
    }

    // 清空原discriminants以“替换”：修改data_enum的variants，移除disc
    for variant in data_enum.variants.iter_mut() {
        variant.discriminant = None;
    }

    // 生成额外代码：转换impl
    let extra_code = quote! {
        // 实现 From<Enum> for ReprType（使用修改后的enum_name，即无disc的）
        impl From<#enum_name> for #repr_type {
            fn from(value: #enum_name) -> Self {
                match value {
                    #(#from_matches)*
                }
            }
        }


        // 实现 TryFrom<ReprType> for Enum
        impl TryFrom<#repr_type> for #enum_name {
            type Error = ();  // 可以自定义错误类型

            fn try_from(value: #repr_type) -> Result<Self, Self::Error> {
                match value {
                    #(#try_from_matches)*
                    _ => Err(()),
                }
            }
        }

    };

    // 输出：原修改后的枚举定义 + 额外impl
    let modified_enum = quote!(#input);
    let output = quote! {
        #modified_enum
        #extra_code
    };

    TokenStream::from(output)
}

#[proc_macro_derive(EnumConverter, attributes(repr_type))]
pub fn enum_converter_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // 确保这是枚举
    let Data::Enum(data_enum) = input.data else {
        panic!("EnumConverter 仅支持枚举类型");
    };

    // 从属性中提取repr_type（用户需指定，如 #[repr_type(i32)] 或 #[repr_type(ffi::svn_depth_t)]）
    let repr_type = input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path().is_ident("repr_type") {
                attr.parse_args::<Ident>().ok().map(|id| quote!(#id))
            } else {
                None
            }
        })
        .unwrap_or_else(|| quote!(i32)); // 默认i32

    // 提取枚举名
    let enum_name = &input.ident;

    // 生成新枚举名：原名 + Clean（无discriminants）
    let clean_enum_name = Ident::new(&format!("{}Clean", enum_name), enum_name.span());

    // 收集变体信息：变体名和discriminant表达式（作为TokenStream复制）
    let mut clean_variants = Vec::new();
    let mut from_matches = Vec::new();
    let mut try_from_matches = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;

        // 假设无字段变体（unit variants），如你的示例。如果有字段，需调整
        if !matches!(variant.fields, Fields::Unit) {
            panic!("仅支持无字段的unit枚举变体");
        }

        // 获取discriminant表达式（如果没有，panic，因为需求是转换有disc的）
        let disc = variant
            .discriminant
            .as_ref()
            .expect("变体必须有discriminant表达式");

        // disc.1 是Expr，我们转换为TokenStream以复制原表达式（如 ffi::const）
        let disc_expr = &disc.1;

        // 为clean枚举添加变体（无disc）
        clean_variants.push(quote! {
            #variant_name,
        });

        // 为 From<Clean> for ReprType 生成 match arm: Clean::Var => 原disc_expr,
        from_matches.push(quote! {
            #clean_enum_name::#variant_name => #disc_expr,
        });

        // 为 TryFrom<ReprType> for Clean 生成 match arm: 原disc_expr => Ok(Clean::Var),
        try_from_matches.push(quote! {
            #disc_expr => Ok(#clean_enum_name::#variant_name),
        });
    }

    // 生成代码
    let expanded = quote! {
        // 生成干净的枚举（无discriminants）
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #clean_enum_name {
            #(#clean_variants)*
        }

        // 实现 From<Clean> for ReprType
        impl From<#clean_enum_name> for #repr_type {
            fn from(value: #clean_enum_name) -> Self {
                match value {
                    #(#from_matches)*
                }
            }
        }

        // 实现 TryFrom<ReprType> for Clean（使用Result以处理不匹配）
        impl TryFrom<#repr_type> for #clean_enum_name {
            type Error = ();  // 可以自定义错误类型

            fn try_from(value: #repr_type) -> Result<Self, Self::Error> {
                match value {
                    #(#try_from_matches)*
                    _ => Err(()),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IsMethods)]
pub fn derive_is_methods(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // 只支持 struct
    let fields = match &input.data {
        Data::Struct(s) => &s.fields,
        _ => {
            return quote! {
                compile_error!("IsMethods can only be derived for structs.");
            }
            .into();
        }
    };

    // 只支持具名字段 struct
    let named = match fields {
        Fields::Named(named) => &named.named,
        _ => {
            return quote! {
                compile_error!("IsMethods requires a struct with named fields (e.g. struct S { a: i32 }).");
            }
            .into();
        }
    };

    // 生成方法，并且校验每个字段类型必须是 i32
    let mut methods = Vec::new();

    for f in named {
        let Some(field_ident) = &f.ident else {
            continue;
        };

        // if !is_i32_type(&f.ty) {
        //     let msg = format!(
        //         "IsMethods requires all fields to be i32. Field `{}` is not i32.",
        //         field_ident
        //     );
        //     return quote! { compile_error!(#msg); }.into();
        // }

        let method_ident = quote::format_ident!("is_{}", field_ident);
        let ty = &f.ty;

        methods.push(quote! {
            #[inline]
            pub fn #method_ident(&self, value: #ty) -> bool {
                self.#field_ident == value
            }
        });
    }
    let hash = quote!(#);

    quote! {
        #hash[uniffi::export]
        impl #impl_generics #name #ty_generics #where_clause {
            #(#methods)*
        }
    }
    .into()
}

// fn is_i32_type(ty: &Type) -> bool {
//     match ty {
//         Type::Path(p) => p
//             .path
//             .segments
//             .last()
//             .map(|seg| seg.ident == "i32")
//             .unwrap_or(false),
//         _ => false,
//     }
// }
