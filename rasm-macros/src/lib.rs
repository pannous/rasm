use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Meta};

/// Derives accessor methods for WebAssembly GC struct fields
///
/// # Example
/// ```
/// #[derive(WasmAccess)]
/// #[wasm_fields(name: String, age: i32, email: String)]
/// pub struct Person {
///     inner: GcObject<RefCell<Store<()>>>,
/// }
///
/// // Generates methods:
/// // - person.name() -> Result<String>
/// // - person.age() -> Result<i32>
/// // - person.email() -> Result<String>
/// ```
#[proc_macro_derive(WasmAccess, attributes(wasm_fields))]
pub fn derive_wasm_access(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract field definitions from #[wasm_fields(...)] attribute
    let wasm_fields_attr = input.attrs.iter()
        .find(|attr| attr.path().is_ident("wasm_fields"))
        .expect("WasmAccess requires #[wasm_fields(...)] attribute");

    // Parse the attribute - this is simplified, real parsing would be more robust
    let fields_str = match &wasm_fields_attr.meta {
        Meta::List(list) => list.tokens.to_string(),
        _ => panic!("wasm_fields must be a list"),
    };

    // Parse field definitions (simplified - name: Type, ...)
    let field_defs: Vec<_> = fields_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Generate accessor methods
    let accessors = field_defs.iter().map(|field_def| {
        let parts: Vec<_> = field_def.split(':').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            panic!("Invalid field format: {}", field_def);
        }

        let field_name = syn::parse_str::<syn::Ident>(parts[0])
            .expect("Invalid field name");
        let field_type = syn::parse_str::<syn::Type>(parts[1])
            .expect("Invalid field type");
        let field_name_str = field_name.to_string();

        quote! {
            pub fn #field_name(&self) -> anyhow::Result<#field_type> {
                self.inner.get(#field_name_str)
            }
        }
    });

    let expanded = quote! {
        impl #name {
            #(#accessors)*
        }
    };

    TokenStream::from(expanded)
}
