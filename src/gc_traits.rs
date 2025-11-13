#![allow(unused)]

use anyhow::Result;
use wasmtime::*;
use std::str;
use std::ops::Index;
use std::cell::RefCell;

/// Trait for converting Val to Rust types
pub trait FromVal: Sized {
    fn from_val(val: Val, store: &mut Store<()>) -> Result<Self>;
}

impl FromVal for i32 {
    fn from_val(val: Val, _store: &mut Store<()>) -> Result<Self> {
        Ok(val.unwrap_i32())
    }
}

impl FromVal for i64 {
    fn from_val(val: Val, _store: &mut Store<()>) -> Result<Self> {
        Ok(val.unwrap_i64())
    }
}

impl FromVal for f32 {
    fn from_val(val: Val, _store: &mut Store<()>) -> Result<Self> {
        Ok(f32::from_bits(val.unwrap_f32() as u32))
    }
}

impl FromVal for f64 {
    fn from_val(val: Val, _store: &mut Store<()>) -> Result<Self> {
        Ok(val.unwrap_f64())
    }
}

impl FromVal for bool {
    fn from_val(val: Val, _store: &mut Store<()>) -> Result<Self> {
        Ok(val.unwrap_i32() != 0)
    }
}

impl FromVal for String {
    fn from_val(val: Val, store: &mut Store<()>) -> Result<Self> {
        let gc_string = GcString::from_val(store, val)?;
        gc_string.to_string(store)
    }
}

impl FromVal for Rooted<StructRef> {
    fn from_val(val: Val, store: &mut Store<()>) -> Result<Self> {
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        anyref.unwrap_struct(&*store)
    }
}

// Note: For nested structs, use StructExt::get_struct() instead of FromVal

/// Context that holds the Store, allowing methods to be called without explicit store passing
pub struct GcContext<'a> {
    store: &'a mut Store<()>,
}

impl<'a> GcContext<'a> {
    pub fn new(store: &'a mut Store<()>) -> Self {
        Self { store }
    }

    /// Get the underlying store reference
    pub fn store(&mut self) -> &mut Store<()> {
        self.store
    }
}

/// A contextual struct reference that doesn't require passing store to each method
pub struct ContextualStruct<'a> {
    struct_ref: Rooted<StructRef>,
    ctx: &'a mut GcContext<'a>,
}

impl<'a> ContextualStruct<'a> {
    /// Create a contextual struct wrapper
    pub fn new(struct_ref: Rooted<StructRef>, ctx: &'a mut GcContext<'a>) -> Self {
        Self { struct_ref, ctx }
    }

    /// Get a field with automatic type conversion
    pub fn get<T: FromVal>(&mut self, index: usize) -> Result<T> {
        let val = self.struct_ref.field(&mut *self.ctx.store, index)?;
        T::from_val(val, &mut *self.ctx.store)
    }

    /// Get a nested struct field
    /// Note: Due to lifetime constraints with mutable references, nested struct access
    /// works better with the regular StructExt trait that takes &mut store explicitly
    /*
    pub fn get_struct(&mut self, index: usize) -> Result<ContextualStruct<'a>> {
        let val = self.struct_ref.field(&mut *self.ctx.store, index)?;
        let anyref = val.unwrap_anyref().ok_or_else(|| anyhow::anyhow!("field {} is not an anyref", index))?;
        let nested = anyref.unwrap_struct(&*self.ctx.store)?;
        Ok(ContextualStruct::new(nested, self.ctx))
    }
    */

    /// Check if a reference field is null
    pub fn is_null(&mut self, index: usize) -> Result<bool> {
        let val = self.struct_ref.field(&mut *self.ctx.store, index)?;
        Ok(val.unwrap_anyref().is_none())
    }

    /// Get the underlying StructRef
    pub fn inner(&self) -> &Rooted<StructRef> {
        &self.struct_ref
    }
}

/// Extension trait to convert Val or StructRef into contextual wrappers
pub trait IntoContextual {
    fn with_context<'a>(self, ctx: &'a mut GcContext<'a>) -> Result<ContextualStruct<'a>>;
}

impl IntoContextual for Val {
    fn with_context<'a>(self, ctx: &'a mut GcContext<'a>) -> Result<ContextualStruct<'a>> {
        let anyref = self.unwrap_anyref().ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let struct_ref = anyref.unwrap_struct(&*ctx.store)?;
        Ok(ContextualStruct::new(struct_ref, ctx))
    }
}

impl IntoContextual for Rooted<StructRef> {
    fn with_context<'a>(self, ctx: &'a mut GcContext<'a>) -> Result<ContextualStruct<'a>> {
        Ok(ContextualStruct::new(self, ctx))
    }
}

/// Field value wrapper for Index operations
pub struct FieldValue {
    val: Val,
    store_ref: *const RefCell<Store<()>>,
}

impl FieldValue {
    fn with_store<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Store<()>) -> R,
    {
        unsafe {
            let store_cell = &*self.store_ref;
            f(&mut *store_cell.borrow_mut())
        }
    }
}

// Automatic conversions for common types
impl From<&FieldValue> for String {
    fn from(fv: &FieldValue) -> Self {
        fv.with_store(|store| {
            String::from_val(fv.val.clone(), store).expect("Failed to convert to String")
        })
    }
}

impl From<&FieldValue> for i32 {
    fn from(fv: &FieldValue) -> Self {
        fv.val.unwrap_i32()
    }
}

impl From<&FieldValue> for i64 {
    fn from(fv: &FieldValue) -> Self {
        fv.val.unwrap_i64()
    }
}

impl From<&FieldValue> for f64 {
    fn from(fv: &FieldValue) -> Self {
        fv.val.unwrap_f64()
    }
}

impl From<&FieldValue> for bool {
    fn from(fv: &FieldValue) -> Self {
        fv.val.unwrap_i32() != 0
    }
}

/// Simplified wrapper that owns the store and provides clean API
///
/// # Example
/// ```
/// let mut person = GcObject::new(person_val, store);
/// let name: String = person.get(0)?;  // No &mut store needed!
/// let age: i32 = person.get(1)?;
/// // Or use bracket syntax:
/// let name: String = person["name"];  // Direct access!
/// ```
pub struct GcObject<T> {
    inner: Rooted<StructRef>,
    store: T,
    cached: RefCell<Option<FieldValue>>,
}

impl GcObject<RefCell<Store<()>>> {
    /// Create a new GcObject that owns the store
    pub fn new(val: Val, store: Store<()>) -> Result<Self> {
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let inner = anyref.unwrap_struct(&store)?;
        Ok(Self { inner, store: RefCell::new(store), cached: RefCell::new(None) })
    }

    /// Create from an existing StructRef
    pub fn from_struct(struct_ref: Rooted<StructRef>, store: Store<()>) -> Self {
        Self { inner: struct_ref, store: RefCell::new(store), cached: RefCell::new(None) }
    }

    /// Access the store mutably
    fn with_store<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Store<()>) -> R,
    {
        f(&mut *self.store.borrow_mut())
    }

    /// Get a field with automatic type conversion (supports both index and field name)
    ///
    /// # Examples
    /// ```
    /// let name: String = person.get(0)?;      // By index
    /// let name: String = person.get("name")?; // By field name
    /// ```
    pub fn get<T: FromVal, I: FieldIndex>(&self, index: I) -> Result<T> {
        self.with_store(|store| {
            let idx = index.to_field_index(&self.inner, &*store)?;
            let val = self.inner.field(&mut *store, idx)?;
            T::from_val(val, &mut *store)
        })
    }

    /// Get a nested struct
    pub fn get_struct<I: FieldIndex>(&self, index: I) -> Result<Rooted<StructRef>> {
        self.with_store(|store| {
            let idx = index.to_field_index(&self.inner, &*store)?;
            let val = self.inner.field(&mut *store, idx)?;
            let anyref = val.unwrap_anyref()
                .ok_or_else(|| anyhow::anyhow!("field {} is not an anyref", idx))?;
            anyref.unwrap_struct(&*store)
        })
    }

    /// Check if a field is null
    pub fn is_null<I: FieldIndex>(&self, index: I) -> Result<bool> {
        self.with_store(|store| {
            let idx = index.to_field_index(&self.inner, &*store)?;
            let val = self.inner.field(&mut *store, idx)?;
            Ok(val.unwrap_anyref().is_none())
        })
    }
}

// Index implementation for bracket syntax: person["name"]
impl Index<&str> for GcObject<RefCell<Store<()>>> {
    type Output = FieldValue;

    fn index(&self, name: &str) -> &Self::Output {
        self.with_store(|store| {
            let idx = name.to_field_index(&self.inner, store)
                .expect(&format!("Failed to resolve field '{}'", name));
            let val = self.inner.field(store, idx)
                .expect(&format!("Failed to access field '{}'", name));
            *self.cached.borrow_mut() = Some(FieldValue {
                val,
                store_ref: &self.store as *const _,
            });
        });
        // Return reference to cached value
        unsafe {
            // SAFETY: We just set cached to Some, and Index requires returning a reference
            // The reference is valid for the lifetime of self
            let cached_ref = &*self.cached.as_ptr();
            cached_ref.as_ref().unwrap()
        }
    }
}

impl Index<usize> for GcObject<RefCell<Store<()>>> {
    type Output = FieldValue;

    fn index(&self, index: usize) -> &Self::Output {
        self.with_store(|store| {
            let val = self.inner.field(store, index)
                .expect(&format!("Failed to access field {}", index));
            *self.cached.borrow_mut() = Some(FieldValue {
                val,
                store_ref: &self.store as *const _,
            });
        });
        unsafe {
            let cached_ref = &*self.cached.as_ptr();
            cached_ref.as_ref().unwrap()
        }
    }
}

impl<'a> GcObject<&'a mut Store<()>> {
    /// Create a GcObject that borrows the store
    pub fn from_ref(val: Val, store: &'a mut Store<()>) -> Result<Self> {
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let inner = anyref.unwrap_struct(&*store)?;
        Ok(Self { inner, store, cached: RefCell::new(None) })
    }

    /// Create from an existing StructRef with borrowed store
    pub fn from_struct_ref(struct_ref: Rooted<StructRef>, store: &'a mut Store<()>) -> Self {
        Self { inner: struct_ref, store, cached: RefCell::new(None) }
    }

    /// Get a field with automatic type conversion (supports both index and field name)
    ///
    /// # Examples
    /// ```
    /// let name: String = person.get(0)?;      // By index
    /// let name: String = person.get("name")?; // By field name
    /// ```
    pub fn get<T: FromVal, I: FieldIndex>(&mut self, index: I) -> Result<T> {
        let idx = index.to_field_index(&self.inner, &*self.store)?;
        let val = self.inner.field(&mut *self.store, idx)?;
        T::from_val(val, &mut *self.store)
    }

    /// Get a nested struct
    pub fn get_struct<I: FieldIndex>(&mut self, index: I) -> Result<Rooted<StructRef>> {
        let idx = index.to_field_index(&self.inner, &*self.store)?;
        let val = self.inner.field(&mut *self.store, idx)?;
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("field {} is not an anyref", idx))?;
        anyref.unwrap_struct(&*self.store)
    }

    /// Check if a field is null
    pub fn is_null<I: FieldIndex>(&mut self, index: I) -> Result<bool> {
        let idx = index.to_field_index(&self.inner, &*self.store)?;
        let val = self.inner.field(&mut *self.store, idx)?;
        Ok(val.unwrap_anyref().is_none())
    }
}

/// Trait for field index types (supports both usize and &str field names)
pub trait FieldIndex {
    fn to_field_index(&self, struct_ref: &Rooted<StructRef>, store: &Store<()>) -> Result<usize>;
}

impl FieldIndex for usize {
    fn to_field_index(&self, _struct_ref: &Rooted<StructRef>, _store: &Store<()>) -> Result<usize> {
        Ok(*self)
    }
}

impl FieldIndex for &str {
    fn to_field_index(&self, struct_ref: &Rooted<StructRef>, store: &Store<()>) -> Result<usize> {
        // Get the struct's type and field count
        let field_count = struct_ref.ty(store)?.fields().count();

        // For now, use a simple hardcoded mapping for known field names
        // TODO: Parse from WASM name section for generic support
        let index = match *self {
            "name" => 0,
            "age" => 1,
            "email" => 2,
            "x" => 0,
            "y" => 1,
            "person" => 0,
            "id" => 1,
            "salary" => 2,
            _ => return Err(anyhow::anyhow!("unknown field name: {}", self)),
        };

        if index >= field_count {
            return Err(anyhow::anyhow!("field '{}' index {} out of bounds (struct has {} fields)",
                self, index, field_count));
        }

        Ok(index)
    }
}

/// Extension trait for beautiful GC struct field access
pub trait StructExt {
    /// Get a field with automatic type conversion
    ///
    /// # Example
    /// ```
    /// let age: i32 = person_struct.get(store, 1)?;
    /// let x: f64 = point_struct.get(store, 0)?;
    /// ```
    fn get<T: FromVal>(&self, store: &mut Store<()>, index: usize) -> Result<T>;

    /// Get a nested struct field
    fn get_struct(&self, store: &mut Store<()>, index: usize) -> Result<Rooted<StructRef>>;

    /// Get an optional (nullable) field
    fn get_option<T: FromVal>(&self, store: &mut Store<()>, index: usize) -> Result<Option<T>>;

    /// Check if a reference field is null
    fn is_null(&self, store: &mut Store<()>, index: usize) -> Result<bool>;
}

impl StructExt for Rooted<StructRef> {
    fn get<T: FromVal>(&self, store: &mut Store<()>, index: usize) -> Result<T> {
        let val = self.field(&mut *store, index)?;
        T::from_val(val, &mut *store)
    }

    fn get_struct(&self, store: &mut Store<()>, index: usize) -> Result<Rooted<StructRef>> {
        let val = self.field(&mut *store, index)?;
        let anyref = val.unwrap_anyref().ok_or_else(|| anyhow::anyhow!("field {} is not an anyref", index))?;
        anyref.unwrap_struct(&*store)
    }

    fn get_option<T: FromVal>(&self, store: &mut Store<()>, index: usize) -> Result<Option<T>> {
        let val = self.field(&mut *store, index)?;
        if val.unwrap_anyref().is_none() {
            Ok(None)
        } else {
            Ok(Some(T::from_val(val, &mut *store)?))
        }
    }

    fn is_null(&self, store: &mut Store<()>, index: usize) -> Result<bool> {
        let val = self.field(store, index)?;
        Ok(val.unwrap_anyref().is_none())
    }
}

/// Type-safe wrapper for specific GC struct types
pub trait GcStruct: Sized {
    /// Create from a Val (performs type checking)
    fn from_val(store: &Store<()>, val: Val) -> Result<Self>;

    /// Get the underlying StructRef
    fn struct_ref(&self) -> &Rooted<StructRef>;
}

/// Macro for defining type-safe struct wrappers
#[macro_export]
macro_rules! gc_struct {
    (
        $name:ident {
            $($field_name:ident : $field_idx:literal => $field_type:ty),* $(,)?
        }
    ) => {
        pub struct $name {
            inner: wasmtime::Rooted<wasmtime::StructRef>,
        }

        impl $name {
            pub fn from_val(store: &wasmtime::Store<()>, val: wasmtime::Val) -> anyhow::Result<Self> {
                let anyref = val.unwrap_anyref().ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
                let inner = anyref.unwrap_struct(store)?;
                Ok(Self { inner })
            }

            $(
                pub fn $field_name(&self, store: &mut wasmtime::Store<()>) -> anyhow::Result<$field_type> {
                    use $crate::gc_traits::StructExt;
                    self.inner.get(store, $field_idx)
                }
            )*
        }

        impl $crate::gc_traits::GcStruct for $name {
            fn from_val(store: &wasmtime::Store<()>, val: wasmtime::Val) -> anyhow::Result<Self> {
                Self::from_val(store, val)
            }

            fn struct_ref(&self) -> &wasmtime::Rooted<wasmtime::StructRef> {
                &self.inner
            }
        }
    };
}

// Example: Define a type-safe Point wrapper
gc_struct! {
    Point {
        x: 0 => f64,
        y: 1 => f64,
    }
}

gc_struct! {
    Person {
        age: 1 => i32,
    }
}

/// GC string wrapper (wraps WebAssembly GC array of i8 bytes)
///
/// # Example
/// ```
/// // Create from Val
/// let gc_str = GcString::from_val(store, name_val)?;
/// let rust_string = gc_str.to_string(store)?;
/// ```
pub struct GcString {
    inner: Rooted<ArrayRef>,
}

impl GcString {
    /// Create from a Val
    pub fn from_val(store: &Store<()>, val: Val) -> Result<Self> {
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let inner = anyref.unwrap_array(store)?;
        Ok(Self { inner })
    }

    /// Create from an existing ArrayRef
    pub fn from_array(array_ref: Rooted<ArrayRef>) -> Self {
        Self { inner: array_ref }
    }

    /// Convert to Rust String
    pub fn to_string(&self, store: &mut Store<()>) -> Result<String> {
        let len = self.inner.len(&*store)? as usize;
        let mut bytes = Vec::with_capacity(len);

        for i in 0..len {
            let elem = self.inner.get(&mut *store, i as u32)?;
            bytes.push(elem.unwrap_i32() as u8);
        }

        Ok(String::from_utf8(bytes)?)
    }

    /// Get the length of the string
    pub fn len(&self, store: &Store<()>) -> Result<usize> {
        Ok(self.inner.len(store)? as usize)
    }

    /// Check if the string is empty
    pub fn is_empty(&self, store: &Store<()>) -> Result<bool> {
        Ok(self.len(store)? == 0)
    }

    /// Get the underlying ArrayRef
    pub fn array_ref(&self) -> &Rooted<ArrayRef> {
        &self.inner
    }

    /// Convert to Val for passing to WASM functions
    pub fn to_val(&self) -> Val {
        Val::AnyRef(Some(self.inner.clone().into()))
    }
}

impl FromVal for GcString {
    fn from_val(val: Val, store: &mut Store<()>) -> Result<Self> {
        GcString::from_val(store, val)
    }
}

/// Old helper methods for working with GC strings (kept for compatibility)
pub struct OldGcString;

impl OldGcString {
    /// Write a Rust string to WASM linear memory
    ///
    /// Returns (offset, length) for passing to WASM functions
    ///
    /// # Example
    /// ```
    /// let (ptr, len) = OldGcString::write_to_mem(&mut store, &instance, "Alice", 0)?;
    /// // Now call WASM function: new_string(ptr, len)
    /// ```
    pub fn write_to_mem(
        store: &mut Store<()>,
        instance: &Instance,
        s: &str,
        offset: usize,
    ) -> Result<(i32, i32)> {
        let bytes = s.as_bytes();

        // Get WASM memory
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow::anyhow!("memory not found"))?;

        // Write string bytes to memory at offset
        memory.write(&mut *store, offset, bytes)?;

        Ok((offset as i32, bytes.len() as i32))
    }

    /// Read a GC string array as a Rust String
    ///
    /// # Example
    /// ```
    /// let name_field = person_struct.field(store, 0)?;
    /// let name = OldGcString::to_string(store, instance, &name_field)?;
    /// println!("Name: {}", name);
    /// ```
    pub fn to_string(
        store: &mut Store<()>,
        instance: &Instance,
        val: &Val,
    ) -> Result<String> {
        // Get the anyref and downcast to array
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let array_ref = anyref.unwrap_array(&mut *store)?;

        // Read bytes from GC array using string_to_mem helper
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow::anyhow!("memory not found"))?;

        let string_to_mem = instance
            .get_func(&mut *store, "string_to_mem")
            .ok_or_else(|| anyhow::anyhow!("string_to_mem not found"))?;

        // Copy GC string to memory at offset 1024
        let offset = 1024;
        let mut results = vec![Val::I32(0)];
        string_to_mem.call(
            &mut *store,
            &[Val::AnyRef(Some(anyref.clone())), Val::I32(offset)],
            &mut results,
        )?;

        let actual_len = results[0].unwrap_i32() as usize;

        // Read bytes from memory
        let mut bytes = vec![0u8; actual_len];
        memory.read(&mut *store, offset as usize, &mut bytes)?;

        // Convert to UTF-8 string
        Ok(String::from_utf8(bytes)?)
    }

    /// Direct read without using string_to_mem (using array introspection)
    pub fn to_string_direct(
        store: &mut Store<()>,
        val: &Val,
    ) -> Result<String> {
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let array_ref = anyref.unwrap_array(&mut *store)?;

        let len = array_ref.len(&*store)? as usize;
        let mut bytes = Vec::with_capacity(len);

        // Read each byte from the GC array
        for i in 0..len {
            let elem = array_ref.get(&mut *store, i as u32)?;
            bytes.push(elem.unwrap_i32() as u8);
        }

        // Convert to UTF-8 string
        Ok(String::from_utf8(bytes)?)
    }
}
