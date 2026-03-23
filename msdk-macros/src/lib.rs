//! Procedural macros for memlink SDK.
//!
//! Provides the `#[memlink_export]` attribute macro for exporting Rust functions
//! as memlink module methods with automatic serialization and FFI bindings.

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    parse_macro_input, FnArg, Ident, ItemFn, ItemStruct, Pat, Signature, Token,
};

const fn fnv1a_hash_bytes(bytes: &[u8]) -> u32 {
    const FNV_OFFSET: u32 = 2166136261;
    const FNV_PRIME: u32 = 16777619;

    let mut hash = FNV_OFFSET;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

const fn fnv1a_hash_str(s: &str) -> u32 {
    fnv1a_hash_bytes(s.as_bytes())
}

struct ExportAttrs {
    name: Option<String>,
}

impl Parse for ExportAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: syn::LitStr = input.parse()?;

            if ident == "name" {
                name = Some(value.value());
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ExportAttrs { name })
    }
}

#[proc_macro_attribute]
pub fn memlink_export(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(args as ExportAttrs);
    let mut func = parse_macro_input!(input as ItemFn);

    let method_name = attrs.name.unwrap_or_else(|| func.sig.ident.to_string());
    let method_hash = fnv1a_hash_str(&method_name);

    let expanded = generate_export_code(&mut func, &method_name, method_hash);

    TokenStream::from(expanded)
}

fn generate_export_code(func: &mut ItemFn, _method_name: &str, method_hash: u32) -> proc_macro2::TokenStream {
    let func_name = &func.sig.ident;
    let _func_vis = &func.vis;
    let sig = &func.sig;

    let is_async = sig.asyncness.is_some();

    let (_context_param, other_params) = extract_params(sig);

    let args_struct = if !other_params.is_empty() {
        generate_args_struct(func_name, other_params.clone())
    } else {
        quote! {}
    };

    let wrapper_name = format_ident!("__{}_wrapper", func_name);
    let wrapper = generate_wrapper(func_name, &wrapper_name, other_params, is_async);

    let ffi_name = format_ident!("__{}_ffi", func_name);
    let ffi_func = generate_ffi_export(&wrapper_name, &ffi_name, method_hash, is_async);

    let register_func = generate_registration(func_name, method_hash, is_async);

    quote! {
        #func
        #args_struct
        #wrapper
        #ffi_func
        #register_func
    }
}

fn extract_params(sig: &Signature) -> (Option<&FnArg>, Vec<&FnArg>) {
    let params = sig.inputs.iter();
    let mut context_param = None;
    let mut other_params = Vec::new();

    for param in params {
        match param {
            FnArg::Typed(pat_type) => {
                let type_str = pat_type.ty.to_token_stream().to_string();
                if type_str.contains("CallContext") {
                    context_param = Some(param);
                } else {
                    other_params.push(param);
                }
            }
            FnArg::Receiver(_) => {
                other_params.push(param);
            }
        }
    }

    (context_param, other_params)
}

fn generate_args_struct(func_name: &Ident, params: Vec<&FnArg>) -> proc_macro2::TokenStream {
    let args_struct_name = format_ident!("__{}Args", func_name);

    let fields: Vec<_> = params.iter().map(|param| {
        if let FnArg::Typed(pat_type) = param {
            let pat = &pat_type.pat;
            let ty = &pat_type.ty;
            if let Pat::Ident(ident) = pat.as_ref() {
                let field_name = &ident.ident;
                quote! { pub #field_name: #ty }
            } else {
                quote! {}
            }
        } else {
            quote! {}
        }
    }).collect();

    quote! {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        struct #args_struct_name {
            #(#fields,)*
        }
    }
}

fn generate_wrapper(
    func_name: &Ident,
    wrapper_name: &Ident,
    params: Vec<&FnArg>,
    is_async: bool,
) -> proc_macro2::TokenStream {
    let args_struct_name = format_ident!("__{}Args", func_name);

    let field_names: Vec<_> = params.iter().filter_map(|param| {
        if let FnArg::Typed(pat_type) = param {
            let pat = &pat_type.pat;
            if let Pat::Ident(ident) = pat.as_ref() {
                Some(&ident.ident)
            } else {
                None
            }
        } else {
            None
        }
    }).collect();

    let call_args = if field_names.is_empty() {
        quote! { ctx }
    } else {
        let args_unpack = field_names.iter().map(|name| {
            quote! { args.#name }
        });
        quote! { ctx, #(#args_unpack),* }
    };

    if is_async {
        quote! {
            async fn #wrapper_name(
                ctx: &memlink_msdk::CallContext<'_>,
                args_bytes: &[u8],
            ) -> memlink_msdk::Result<Vec<u8>> {
                let args: #args_struct_name = memlink_msdk::serialize::default_serializer()
                    .deserialize(args_bytes)
                    .map_err(|e| memlink_msdk::ModuleError::Serialize(e.to_string()))?;

                let result = #func_name(#call_args).await?;

                memlink_msdk::serialize::default_serializer()
                    .serialize(&result)
                    .map_err(|e| memlink_msdk::ModuleError::Serialize(e.to_string()))
            }
        }
    } else {
        quote! {
            fn #wrapper_name(
                ctx: &memlink_msdk::CallContext<'_>,
                args_bytes: &[u8],
            ) -> memlink_msdk::Result<Vec<u8>> {
                let args: #args_struct_name = memlink_msdk::serialize::default_serializer()
                    .deserialize(args_bytes)
                    .map_err(|e| memlink_msdk::ModuleError::Serialize(e.to_string()))?;

                let result = #func_name(#call_args)?;

                memlink_msdk::serialize::default_serializer()
                    .serialize(&result)
                    .map_err(|e| memlink_msdk::ModuleError::Serialize(e.to_string()))
            }
        }
    }
}

fn generate_ffi_export(
    wrapper_name: &Ident,
    ffi_name: &Ident,
    _method_hash: u32,
    is_async: bool,
) -> proc_macro2::TokenStream {
    if is_async {
        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #ffi_name(
                ctx_ptr: *const memlink_msdk::CallContext<'static>,
                args_ptr: *const u8,
                args_len: usize,
                out_ptr: *mut u8,
                out_cap: usize,
            ) -> i32 {
                use memlink_msdk::panic::catch_module_panic;
                use memlink_msdk::request::Response;

                const CALL_SUCCESS: i32 = 0;
                const CALL_FAILURE: i32 = -1;
                const CALL_BUFFER_TOO_SMALL: i32 = -2;

                if args_len > 0 && args_ptr.is_null() {
                    return CALL_FAILURE;
                }
                if out_cap > 0 && out_ptr.is_null() {
                    return CALL_FAILURE;
                }

                let result = catch_module_panic(|| {
                    let ctx = unsafe { &*ctx_ptr };
                    let args = if args_len > 0 {
                        unsafe { std::slice::from_raw_parts(args_ptr, args_len) }.to_vec()
                    } else {
                        vec![]
                    };

                    let rt = tokio::runtime::Handle::current();
                    let result = rt.block_on(#wrapper_name(ctx, &args));

                    let response = match result {
                        Ok(data) => Response::success(data),
                        Err(_) => Response::error(CALL_FAILURE),
                    };

                    let response_bytes = match response.to_bytes() {
                        Ok(bytes) => bytes,
                        Err(_) => return CALL_FAILURE,
                    };

                    if response_bytes.len() > out_cap {
                        return CALL_BUFFER_TOO_SMALL;
                    }

                    std::ptr::copy_nonoverlapping(
                        response_bytes.as_ptr(),
                        out_ptr,
                        response_bytes.len(),
                    );

                    CALL_SUCCESS
                });

                match result {
                    Ok(code) => code,
                    Err(_) => CALL_FAILURE,
                }
            }
        }
    } else {
        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #ffi_name(
                ctx_ptr: *const memlink_msdk::CallContext<'static>,
                args_ptr: *const u8,
                args_len: usize,
                out_ptr: *mut u8,
                out_cap: usize,
            ) -> i32 {
                use memlink_msdk::panic::catch_module_panic;
                use memlink_msdk::request::Response;

                const CALL_SUCCESS: i32 = 0;
                const CALL_FAILURE: i32 = -1;
                const CALL_BUFFER_TOO_SMALL: i32 = -2;

                if args_len > 0 && args_ptr.is_null() {
                    return CALL_FAILURE;
                }
                if out_cap > 0 && out_ptr.is_null() {
                    return CALL_FAILURE;
                }

                let result = catch_module_panic(|| {
                    let ctx = unsafe { &*ctx_ptr };
                    let args = if args_len > 0 {
                        unsafe { std::slice::from_raw_parts(args_ptr, args_len) }.to_vec()
                    } else {
                        vec![]
                    };

                    let result = #wrapper_name(ctx, &args);

                    let response = match result {
                        Ok(data) => Response::success(data),
                        Err(_) => Response::error(CALL_FAILURE),
                    };

                    let response_bytes = match response.to_bytes() {
                        Ok(bytes) => bytes,
                        Err(_) => return CALL_FAILURE,
                    };

                    if response_bytes.len() > out_cap {
                        return CALL_BUFFER_TOO_SMALL;
                    }

                    std::ptr::copy_nonoverlapping(
                        response_bytes.as_ptr(),
                        out_ptr,
                        response_bytes.len(),
                    );

                    CALL_SUCCESS
                });

                match result {
                    Ok(code) => code,
                    Err(_) => CALL_FAILURE,
                }
            }
        }
    }
}

fn generate_registration(
    func_name: &Ident,
    method_hash: u32,
    _is_async: bool,
) -> proc_macro2::TokenStream {
    let register_func_name = format_ident!("__{}_register", func_name);

    quote! {
        #[used]
        static #register_func_name: unsafe extern "C" fn() = {
            unsafe extern "C" fn register() {
            }
            register
        };

        const _: () = {
            const _HASH: u32 = #method_hash;
        };
    }
}

#[proc_macro_attribute]
pub fn memlink_module(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let _struct_name = &item.ident;

    let expanded = quote! {
        #item

        #[no_mangle]
        pub unsafe extern "C" fn memlink_init(
            config_ptr: *const u8,
            config_len: usize,
            arena_ptr: *mut u8,
            arena_capacity: usize,
        ) -> i32 {
            use memlink_msdk::exports::{init_arena, INIT_SUCCESS, INIT_FAILURE};

            if !arena_ptr.is_null() && arena_capacity > 0 {
                init_arena(arena_ptr, arena_capacity);
            }

            __register_all_methods();

            INIT_SUCCESS
        }

        fn __register_all_methods() {
        }
    };

    TokenStream::from(expanded)
}
