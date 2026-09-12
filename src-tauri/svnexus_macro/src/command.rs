//! # messagepack_command 过程宏
//!
//! ## 背景
//!
//! Tauri 框架默认使用 JSON 进行前端与后端的 IPC 通信。但 JSON 是文本格式，
//! 存在以下问题：
//! 1. 性能开销：JSON 解析/序列化比二进制格式慢
//! 2. 类型受限：不支持 Uint8Array、DateTime 等二进制友好类型
//! 3. 体积冗余：文本格式通常比二进制格式更大
//!
//! 本宏提供了一种替代方案：用 MessagePack（Binary JSON）替代 JSON 进行 IPC 通信。
//!
//! ## 工作原理
//!
//! ```text
//! 前端 (TypeScript)                      后端 (Rust)
//! ┌──────────────────┐                   ┌──────────────────────────────────┐
//! │ invokeMessagePack(cmd,  │                   │  生成的包装函数 (#[tauri::command])│
//! │   { arg1, arg2 })│  ── MessagePack bytes ──>│                                  │
//! │                  │                   │  1. 反序列化 MessagePackMessage<Request> │
//! │ serialize({      │                   │  2. 解构出参数                    │
//! │   value: args    │                   │  3. 调用原函数 __cmd(arg1, arg2)  │
//! │ })               │  <── MessagePack bytes ──│  4. 序列化返回值                  │
//! │                  │                   │                                  │
//! │ deserialize()    │                   │  原函数 (不动):                   │
//! │ -> T             │                   │  fn __cmd(arg1, arg2) -> T { ... }│
//! └──────────────────┘                   └──────────────────────────────────┘
//! ```
//!
//! ## 通信协议
//!
//! 前后端使用统一的 `MessagePackMessage<T>` 结构：
//!
//! ```rust
//! #[derive(Serialize, Deserialize)]
//! struct MessagePackMessage<T> {
//!     value: T,
//! }
//! ```
//!
//! - 前端发送：`MessagePackMessage { value: Request }`，其中 Request 的字段是原函数的参数
//! - 后端返回：`MessagePackMessage { value: Response }`，其中 Response 是原函数的返回值
//!
//! ## 生成的代码结构
//!
//! 对于 `#[messagepack_command] fn get_user(name: String, age: i32) -> Result<User, Error> { ... }`，
//! 宏生成：
//!
//! ```rust
//! #[tauri::command]
//! fn get_user(request: tauri::ipc::Request) -> Result<Response, crate::error::Error> {
//!     // 自动生成的 Request 结构体，用于 MessagePack 反序列化
//!     #[derive(serde::Serialize, serde::Deserialize)]
//!     struct Request {
//!         name: String,
//!         age: i32,
//!     }
//!
//!     // 原函数（改名为 __get_user，逻辑不变）
//!     fn __get_user(name: String, age: i32) -> Result<User, Error> {
//!         // ... 原始实现 ...
//!     }
//!
//!     // 从 request 中解码参数，调用原函数，编码返回值
//!     let body = request.body();
//!     let message: MessagePackMessage<Request> = rmp_serde::deserialize_from_slice(body)?;
//!     let Request { name, age } = message.value;
//!     let result = __get_user(name, age);
//!     result_to_messagepack_response(result)
//! }
//! ```
//!
//! ## 属性选项
//!
//! | 属性 | 说明 |
//! |------|------|
//! | `#[messagepack_command]` | 基本用法，自动检测返回值是否为 Result |
//! | `#[messagepack_command(result = true)]` | 显式指定返回值是 Result（用于 type alias） |
//!
//! ## 限制
//!
//! 1. Result 检测基于类型名字符串匹配，type alias 需要手动指定 `result = true`
//! 2. 每个参数必须是具名参数（不支持 `self`、元组解构等）
//! 3. 普通参数类型必须实现 `serde::Serialize + serde::Deserialize`
//! 4. `#[channel]` 参数必须是 `MessagePackChannel`
//!
use proc_macro::TokenStream;
use syn::{Ident, ItemFn, PatType, Type, parse_macro_input, punctuated::Punctuated, token::Comma};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;

/// messagepack_command 宏的配置属性
struct MessagePackCommandAttrs {
    /// 是否显式指定返回值是 Result 类型
    result: bool,
}

/// 解析 #[messagepack_command(result = true)] 属性
fn parse_attrs(attr: TokenStream) -> MessagePackCommandAttrs {
    let mut config = MessagePackCommandAttrs { result: false };

    let attr_str = attr.to_string();
    if attr_str.is_empty() {
        return config;
    }

    // 简单字符串解析 result = true
    if attr_str.contains("result") && attr_str.contains("true") {
        config.result = true;
    }

    config
}

/// # messagepack_command 过程宏
///
/// ## 这个宏是干什么的？
///
/// `messagepack_command` 是一个 Tauri 命令包装宏，用于替代 Tauri 默认的 JSON 序列化，
/// 改用 **MessagePack（Binary JSON）** 作为 IPC 通信的序列化格式。
///
/// ## 为什么要用 MessagePack？
///
/// 1. **性能更好**：MessagePack 是二进制格式，比 JSON 解析更快
/// 2. **类型更丰富**：MessagePack 支持更多类型（如 Uint8Array、DateTime 等）
/// 3. **体积更小**：二进制格式通常比 JSON 文本更紧凑
///
/// ## 宏做了什么？
///
/// 对于这样一个函数：
/// ```rust
/// #[messagepack_command]
/// fn get_user(name: String, age: i32) -> Result<User, Error> {
///     // ...
/// }
/// ```
///
/// 宏会生成一个包装函数，大致等价于：
/// ```rust
/// #[tauri::command]
/// fn get_user(request: tauri::ipc::Request) -> Result<Response, Error> {
///     // 1. 从 request.body() 获取 MessagePack 字节
///     // 2. 反序列化为 MessagePackMessage<Request>，提取参数
///     // 3. 调用原函数
///     // 4. 将返回值序列化为 MessagePack 并返回
/// }
/// ```
///
/// ## 前端怎么调用？
///
/// ```typescript
/// import { invokeMessagePack } from "@/utils/MessagePack";
///
/// // invokeMessagePack 会自动将参数包装成 MessagePackMessage 并序列化为 MessagePack
/// const user = await invokeMessagePack<User>("get_user", { name: "Alice", age: 30 });
/// ```
///
/// ## 支持的用法
///
/// ```rust
/// // 1. 基本用法 - 自动检测返回值类型
/// #[messagepack_command]
/// fn my_command(arg: String) -> String { ... }
///
/// // 2. 返回 Result 类型（自动检测）
/// #[messagepack_command]
/// fn my_command(arg: String) -> Result<String, Error> { ... }
///
/// // 3. 显式指定返回值是 Result（当使用 type alias 时）
/// type MyResult = Result<String, Error>;
///
/// #[messagepack_command(result = true)]
/// fn my_command(arg: String) -> MyResult { ... }
///
/// // 4. async 函数也支持
/// #[messagepack_command]
/// async fn my_async_command(arg: String) -> Result<String, Error> { ... }
///
/// // 5. 标记 Channel 参数
/// #[messagepack_command]
/// fn subscribe(#[channel] channel: MessagePackChannel) -> Result<()> { ... }
/// ```
pub fn messagepack_command_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_attrs(attr);
    let input = parse_macro_input!(item as ItemFn);

    // 根据是否是 async 函数，分别处理
    if input.sig.asyncness.is_some() {
        handle_async_fn(input, config)
    } else {
        handle_sync_fn(input, config)
    }
}

/// 处理同步函数
///
/// 生成的包装函数：
/// - 签名：`fn xxx(request: tauri::ipc::Request) -> Result<Response, Error>`
/// - 带有 `#[tauri::command]` 属性，可以直接注册到 Tauri
fn handle_sync_fn(input: ItemFn, config: MessagePackCommandAttrs) -> TokenStream {
    let fn_name = &input.sig.ident;
    // 原函数重命名为 __{fn_name}，放在包装函数内部
    let prefixed_name = Ident::new(&format!("__{}", fn_name), fn_name.span());
    let visibility = &input.vis;
    let params = extract_params(&input.sig.inputs);
    let has_channel = params.iter().any(|param| param.channel);
    let webview_argument = webview_argument(has_channel);
    let needs_lifetime = params.iter().any(|p| p.is_borrowable || p.has_lifetime);
    let has_mut_ref = params.iter().any(|p| p.is_mut_ref);

    // 步骤 1：生成 Request 结构体
    // 将原函数的参数转为结构体字段，用于从 MessagePack 反序列化
    let fields = extract_fields(&params);
    let request_struct = if needs_lifetime {
        quote::quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Request<'__a> {
                #fields
            }
        }
    } else {
        quote::quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Request {
                #fields
            }
        }
    };

    // 步骤 2：将原函数改名（加 __ 前缀）
    // 原函数的逻辑不变，只是换了个名字
    let mut original_fn = input.clone();
    original_fn.sig.ident = prefixed_name.clone();
    remove_channel_attributes(&mut original_fn);

    // 步骤 3：收集参数名，用于解构 Request
    let arg_names: Vec<_> = params.iter().map(|param| &param.name).collect();
    let call_args = build_call_args(&params);
    let channel_conversions = channel_conversions(&params);

    // 步骤 4：判断返回值是否是 Result 类型
    let is_result = match &input.sig.output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            if config.result {
                true
            } else {
                is_result_type(ty)
            }
        }
    };

    // 步骤 5：生成解构逻辑
    let destructure = if has_mut_ref {
        quote::quote! {
            let mut Request { #(#arg_names),* } = __request;
        }
    } else {
        quote::quote! {
            let Request { #(#arg_names),* } = __request;
        }
    };

    // 步骤 6：生成反序列化逻辑
    let deserialize_args = if needs_lifetime {
        quote::quote! {
            let __body = request.body();
            let tauri::ipc::InvokeBody::Raw(__body) = __body else {
                return Err(crate::error::builder::General {
                    detail: "Expected InvokeBody::Raw".to_string(),
                }.build());
            };
            let __request: Request<'_> = rmp_serde::from_slice(__body).map_err(|e| {
                    crate::error::builder::General {
                        detail: format!("MessagePack deserialization error: {}", e),
                    }
                    .build()
                })?;
            #destructure
            #channel_conversions
        }
    } else {
        quote::quote! {
            let __body = request.body();
            let tauri::ipc::InvokeBody::Raw(__body) = __body else {
                return Err(crate::error::builder::General {
                    detail: "Expected InvokeBody::Raw".to_string(),
                }.build());
            };
            let __request: Request = rmp_serde::from_slice(__body).map_err(|e| {
                    crate::error::builder::General {
                        detail: format!("MessagePack deserialization error: {}", e),
                    }
                    .build()
                })?;
            #destructure
            #channel_conversions
        }
    };

    // 步骤 7：根据返回值类型生成调用和返回逻辑
    let call_and_return = match &input.sig.output {
        syn::ReturnType::Default => {
            quote::quote! {
                #deserialize_args
                #prefixed_name(#call_args);
                crate::messagepack_command::to_messagepack_response(())
            }
        }
        syn::ReturnType::Type(_, _) => {
            if is_result {
                quote::quote! {
                    #deserialize_args
                    let __result = #prefixed_name(#call_args);
                    crate::messagepack_command::result_to_messagepack_response(__result)
                }
            } else {
                quote::quote! {
                    #deserialize_args
                    let __result = #prefixed_name(#call_args);
                    crate::messagepack_command::to_messagepack_response(__result)
                }
            }
        }
    };

    // 步骤 8：生成最终的包装函数
    let output = quote::quote! {
        #[tauri::command]
        #visibility fn #fn_name(
            request: tauri::ipc::Request,
            #webview_argument
        ) -> Result<tauri::ipc::Response, crate::error::Error> {
            #request_struct
            #original_fn

            #call_and_return
        }
    };

    TokenStream::from(output)
}

/// 处理异步函数
///
/// 逻辑与 handle_sync_fn 几乎相同，区别：
/// 1. 包装函数是 async 的
/// 2. 调用原函数时需要 .await
fn handle_async_fn(input: ItemFn, config: MessagePackCommandAttrs) -> TokenStream {
    let fn_name = &input.sig.ident;
    let prefixed_name = Ident::new(&format!("__{}", fn_name), fn_name.span());
    let visibility = &input.vis;
    let params = extract_params(&input.sig.inputs);
    let has_channel = params.iter().any(|param| param.channel);
    let webview_argument = webview_argument(has_channel);
    let needs_lifetime = params.iter().any(|p| p.is_borrowable || p.has_lifetime);
    let has_mut_ref = params.iter().any(|p| p.is_mut_ref);

    // 步骤 1：生成 Request 结构体
    let fields = extract_fields(&params);
    let request_struct = if needs_lifetime {
        quote::quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Request<'__a> {
                #fields
            }
        }
    } else {
        quote::quote! {
            #[derive(serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Request {
                #fields
            }
        }
    };

    // 步骤 2：将原函数改名（保持 async）
    let mut original_fn = input.clone();
    original_fn.sig.ident = prefixed_name.clone();
    remove_channel_attributes(&mut original_fn);

    // 步骤 3：收集参数名
    let arg_names: Vec<_> = params.iter().map(|param| &param.name).collect();
    let call_args = build_call_args(&params);
    let channel_conversions = channel_conversions(&params);

    // 步骤 4：判断返回值是否是 Result 类型
    let is_result = match &input.sig.output {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, ty) => {
            if config.result {
                true
            } else {
                is_result_type(ty)
            }
        }
    };

    // 步骤 5：生成解构逻辑
    let destructure = if has_mut_ref {
        quote::quote! {
            let mut Request { #(#arg_names),* } = __request;
        }
    } else {
        quote::quote! {
            let Request { #(#arg_names),* } = __request;
        }
    };

    // 步骤 6：生成反序列化逻辑
    let deserialize_args = if needs_lifetime {
        quote::quote! {
            let __body = request.body();
            let tauri::ipc::InvokeBody::Raw(__body) = __body else {
                return Err(crate::error::builder::General {
                    detail: "Expected InvokeBody::Raw".to_string(),
                }.build());
            };
            let __request: Request<'_> = rmp_serde::from_slice(__body).map_err(|e| {
                    crate::error::builder::General {
                        detail: format!("MessagePack deserialization error: {}", e),
                    }
                    .build()
                })?;
            #destructure
            #channel_conversions
        }
    } else {
        quote::quote! {
            let __body = request.body();
            let tauri::ipc::InvokeBody::Raw(__body) = __body else {
                return Err(crate::error::builder::General {
                    detail: "Expected InvokeBody::Raw".to_string(),
                }.build());
            };
            let __request: Request = rmp_serde::from_slice(__body).map_err(|e| {
                    crate::error::builder::General {
                        detail: format!("MessagePack deserialization error: {}", e),
                    }
                    .build()
                })?;
            #destructure
            #channel_conversions
        }
    };

    // 步骤 7：生成调用和返回逻辑（注意 .await）
    let call_and_return = match &input.sig.output {
        syn::ReturnType::Default => {
            quote::quote! {
                #deserialize_args
                #prefixed_name(#call_args).await;
                crate::messagepack_command::to_messagepack_response(())
            }
        }
        syn::ReturnType::Type(_, _) => {
            if is_result {
                quote::quote! {
                    #deserialize_args
                    let __result = #prefixed_name(#call_args).await;
                    crate::messagepack_command::result_to_messagepack_response(__result)
                }
            } else {
                quote::quote! {
                    #deserialize_args
                    let __result = #prefixed_name(#call_args).await;
                    crate::messagepack_command::to_messagepack_response(__result)
                }
            }
        }
    };

    // 步骤 8：生成 async 包装函数
    let output = quote::quote! {
        #[tauri::command]
        #visibility async fn #fn_name(
            request: tauri::ipc::Request<'_>,
            #webview_argument
        ) -> Result<tauri::ipc::Response, crate::error::Error> {
            #request_struct
            #original_fn

            #call_and_return
        }
    };

    TokenStream::from(output)
}

/// 判断类型是否是 Result（自动检测）
///
/// 通过检查类型路径中是否包含 "Result" 标识符来判断。
///
/// 局限性：
/// - 只能识别直接使用 `Result` 的情况
/// - 无法识别 type alias，如 `type MyResult = Result<...>`
/// - 无法识别其他名为 "Result" 的类型
///
/// 解决方案：
/// - 对于 type alias，使用 #[messagepack_command(result = true)] 显式指定
fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Result")
    } else {
        false
    }
}

struct CommandParam {
    name: Ident,
    ty: Type,
    channel: bool,
    is_ref: bool,
    is_mut_ref: bool,
    is_borrowable: bool,
    has_lifetime: bool,
}

struct LifetimeChecker {
    has_lifetime: bool,
}

impl<'ast> Visit<'ast> for LifetimeChecker {
    fn visit_lifetime(&mut self, lt: &'ast syn::Lifetime) {
        if lt.ident != "static" {
            self.has_lifetime = true;
        }
        syn::visit::visit_lifetime(self, lt);
    }
}

struct LifetimeReplacer;

impl VisitMut for LifetimeReplacer {
    fn visit_lifetime_mut(&mut self, lt: &mut syn::Lifetime) {
        if lt.ident != "static" {
            *lt = syn::Lifetime::new("'__a", lt.span());
        }
        syn::visit_mut::visit_lifetime_mut(self, lt);
    }
}

fn is_borrowable_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path.path.is_ident("str"),
        Type::Slice(slice) => {
            if let Type::Path(elem_path) = &*slice.elem {
                elem_path.path.is_ident("u8")
            } else {
                false
            }
        }
        _ => false,
    }
}

fn get_ref_inner(ty: &Type) -> Type {
    if let Type::Reference(ref_ty) = ty {
        (*ref_ty.elem).clone()
    } else {
        ty.clone()
    }
}

fn extract_params(inputs: &Punctuated<syn::FnArg, Comma>) -> Vec<CommandParam> {
    inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg {
                if let syn::Pat::Ident(pat_ident) = pat.as_ref() {
                    let (is_ref, is_mut_ref, inner_ty) = match &**ty {
                        Type::Reference(ref_ty) => {
                            let is_mut = ref_ty.mutability.is_some();
                            (true, is_mut, (*ref_ty.elem).clone())
                        }
                        _ => (false, false, (**ty).clone()),
                    };

                    let is_borrowable = is_ref && !is_mut_ref && is_borrowable_type(&inner_ty);

                    let mut checker = LifetimeChecker { has_lifetime: false };
                    checker.visit_type(&inner_ty);
                    let has_lifetime = checker.has_lifetime;

                    return Some(CommandParam {
                        name: pat_ident.ident.clone(),
                        ty: (**ty).clone(),
                        channel: attrs.iter().any(|attr| attr.path().is_ident("channel")),
                        is_ref,
                        is_mut_ref,
                        is_borrowable,
                        has_lifetime,
                    });
                }
            }
            None
        })
        .collect()
}

fn remove_channel_attributes(function: &mut ItemFn) {
    for arg in &mut function.sig.inputs {
        if let syn::FnArg::Typed(arg) = arg {
            arg.attrs.retain(|attr| !attr.path().is_ident("channel"));
        }
    }
}

fn channel_conversions(params: &[CommandParam]) -> proc_macro2::TokenStream {
    let conversions = params.iter().filter(|param| param.channel).map(|param| {
        let name = &param.name;
        quote::quote! {
            let #name = crate::messagepack_command::MessagePackChannel::from_id(&webview, #name)?;
        }
    });

    quote::quote! { #(#conversions)* }
}

fn webview_argument(has_channel: bool) -> proc_macro2::TokenStream {
    if has_channel {
        quote::quote! { webview: tauri::Webview }
    } else {
        quote::quote! {}
    }
}

/// 从函数参数中提取结构体字段
///
/// 将函数签名中的参数转为结构体字段定义。
///
/// 例如：
/// - 输入：`fn foo(name: String, age: i32)`
/// - 输出：`name: String, age: i32,`
fn extract_fields(params: &[CommandParam]) -> proc_macro2::TokenStream {
    let mut fields = Vec::new();

    for param in params {
        let name = &param.name;
        let ty = if param.channel {
            quote::quote! { String }
        } else if param.is_borrowable {
            let inner = get_ref_inner(&param.ty);
            quote::quote! { &'__a #inner }
        } else if param.is_ref {
            let inner = get_ref_inner(&param.ty);
            let mut replaced = inner;
            LifetimeReplacer.visit_type_mut(&mut replaced);
            quote::quote! { #replaced }
        } else if param.has_lifetime {
            let mut replaced = param.ty.clone();
            LifetimeReplacer.visit_type_mut(&mut replaced);
            quote::quote! { #replaced }
        } else {
            let ty = &param.ty;
            quote::quote! { #ty }
        };
        fields.push(quote::quote! {
            #name: #ty,
        });
    }

    quote::quote! { #(#fields)* }
}

fn build_call_args(params: &[CommandParam]) -> proc_macro2::TokenStream {
    let args: Vec<_> = params
        .iter()
        .map(|param| {
            let name = &param.name;
            if param.channel || param.is_borrowable {
                quote::quote! { #name }
            } else if param.is_mut_ref {
                quote::quote! { &mut #name }
            } else if param.is_ref {
                quote::quote! { &#name }
            } else {
                quote::quote! { #name }
            }
        })
        .collect();
    quote::quote! { #(#args),* }
}
