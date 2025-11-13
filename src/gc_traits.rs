#![allow(unused)]

use anyhow::Result;
use wasmtime::*;
use std::str;
use std::ops::Index;
use std::cell::RefCell;
use std::rc::Rc;
use paste::paste;

/// Trait for converting Val to Rust types
pub trait FromVal: Sized {
    fn from_val(val: Val, store: &mut Store<()>) -> Result<Self>;
}

/// Trait for converting Rust types to Val
pub trait ToVal {
    fn to_val(&self, store: &mut Store<()>, instance: Option<&Instance>) -> Result<Val>;
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

// ToVal implementations for converting Rust types to Val
impl ToVal for i32 {
    fn to_val(&self, _store: &mut Store<()>, _instance: Option<&Instance>) -> Result<Val> {
        Ok(Val::I32(*self))
    }
}

impl ToVal for i64 {
    fn to_val(&self, _store: &mut Store<()>, _instance: Option<&Instance>) -> Result<Val> {
        Ok(Val::I64(*self))
    }
}

impl ToVal for f32 {
    fn to_val(&self, _store: &mut Store<()>, _instance: Option<&Instance>) -> Result<Val> {
        Ok(Val::F32(self.to_bits()))
    }
}

impl ToVal for f64 {
    fn to_val(&self, _store: &mut Store<()>, _instance: Option<&Instance>) -> Result<Val> {
        Ok(Val::F64(self.to_bits()))
    }
}

impl ToVal for bool {
    fn to_val(&self, _store: &mut Store<()>, _instance: Option<&Instance>) -> Result<Val> {
        Ok(Val::I32(if *self { 1 } else { 0 }))
    }
}

impl ToVal for &str {
    fn to_val(&self, store: &mut Store<()>, instance: Option<&Instance>) -> Result<Val> {
        let instance = instance.ok_or_else(|| anyhow::anyhow!("Instance required for string creation"))?;
        GcString::create(store, instance, self)
    }
}

impl ToVal for String {
    fn to_val(&self, store: &mut Store<()>, instance: Option<&Instance>) -> Result<Val> {
        self.as_str().to_val(store, instance)
    }
}

impl ToVal for Rooted<StructRef> {
    fn to_val(&self, _store: &mut Store<()>, _instance: Option<&Instance>) -> Result<Val> {
        // Convert StructRef to AnyRef, then wrap in Val
        Ok(Val::AnyRef(Some(self.clone().into())))
    }
}

impl ToVal for Val {
    fn to_val(&self, _store: &mut Store<()>, _instance: Option<&Instance>) -> Result<Val> {
        Ok(self.clone())
    }
}

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
    store_ref: *const Rc<RefCell<Store<()>>>,
}

impl FieldValue {
    fn with_store<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Store<()>) -> R,
    {
        unsafe {
            let store_rc = &*self.store_ref;
            f(&mut *store_rc.borrow_mut())
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
/// let mut person = GcObject::new(person_val, store, Some(&instance));
/// let name: String = person.get(0)?;  // No &mut store needed!
/// let age: i32 = person.get(1)?;
/// person.set_field("age", 30)?;       // Mutation!
/// person.set_field("name", "Alice")?; // String mutation (requires instance)!
/// ```
pub struct GcObject<T> {
    inner: Rooted<StructRef>,
    store: T,
    instance: Option<Instance>,
    cached: RefCell<Option<FieldValue>>,
}

impl GcObject<Rc<RefCell<Store<()>>>> {
    /// Create a new GcObject that owns the store
    ///
    /// Pass `instance` if you need string mutation support
    pub fn new(val: Val, store: Store<()>, instance: Option<Instance>) -> Result<Self> {
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let inner = anyref.unwrap_struct(&store)?;
        Ok(Self {
            inner,
            store: Rc::new(RefCell::new(store)),
            instance,
            cached: RefCell::new(None)
        })
    }

    /// Create from an existing StructRef, sharing the store with another GcObject
    pub fn from_struct_shared(struct_ref: Rooted<StructRef>, store: Rc<RefCell<Store<()>>>, instance: Option<Instance>) -> Self {
        Self {
            inner: struct_ref,
            store,
            instance,
            cached: RefCell::new(None)
        }
    }

    /// Create from an existing StructRef (for backward compatibility)
    pub fn from_struct(struct_ref: Rooted<StructRef>, store: Store<()>, instance: Option<Instance>) -> Self {
        Self {
            inner: struct_ref,
            store: Rc::new(RefCell::new(store)),
            instance,
            cached: RefCell::new(None)
        }
    }

    /// Access the store mutably
    ///
    /// Useful for performing operations that need direct store access
    pub fn with_store<F, R>(&self, f: F) -> R
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
    pub fn get<T: FromVal, I: FieldIndex>(&self, field: I) -> Result<T> {
        self.with_store(|store| {
            let idx = field.to_field_index(&self.inner, &*store)?;
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

    /// Get nested struct as a GcObject that shares the same store
    ///
    /// This allows getting nested structs as proper wrapper objects
    pub fn get_struct_object<I: FieldIndex>(&self, index: I) -> Result<GcObject<Rc<RefCell<Store<()>>>>> {
        let struct_ref = self.get_struct(index)?;
        Ok(GcObject::from_struct_shared(struct_ref, self.store.clone(), self.instance.clone()))
    }

    /// Check if a field is null
    pub fn is_null<I: FieldIndex>(&self, index: I) -> Result<bool> {
        self.with_store(|store| {
            let idx = index.to_field_index(&self.inner, &*store)?;
            let val = self.inner.field(&mut *store, idx)?;
            Ok(val.unwrap_anyref().is_none())
        })
    }


    /// Check if a field is not null
    pub fn has<I: FieldIndex>(&self, index: I) -> Result<bool> {
        self.with_store(|store| {
            let idx = index.to_field_index(&self.inner, &*store)?;
            let val = self.inner.field(&mut *store, idx)?;
            Ok(!val.unwrap_anyref().is_none())
        })
    }

    /// Set a field value (for mutable fields)
    ///
    /// # Examples
    /// ```
    /// person.set_field("age", 29)?;        // Primitives
    /// person.set_field(1, 29)?;            // By index
    /// person.set_field("name", "Alice")?;  // Strings (requires instance)
    /// person.set_field("friend", &ellis)?; // Nested structs!
    /// ```
    pub fn set_field<T: ToVal, I: FieldIndex>(&self, field: I, value: T) -> Result<()> {
        self.with_store(|store| {
            let idx = field.to_field_index(&self.inner, &*store)?;
            let val = value.to_val(&mut *store, self.instance.as_ref())?;
            self.inner.set_field(&mut *store, idx, val)
        })
    }

    /// Get the inner StructRef for passing as a field value to other structs
    ///
    /// # Example
    /// ```
    /// let bob_ref = bob.as_struct_ref();
    /// employee.set_field("person", bob_ref)?;
    /// ```
    pub fn as_struct_ref(&self) -> &Rooted<StructRef> {
        &self.inner
    }

    /// Convert to Val for passing to WASM functions or setting in fields
    pub fn to_val(&self) -> Val {
        Val::AnyRef(Some(self.inner.clone().into()))
    }
}

// Index implementation for bracket syntax: person["name"]
impl Index<&str> for GcObject<Rc<RefCell<Store<()>>>> {
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

impl Index<usize> for GcObject<Rc<RefCell<Store<()>>>> {
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
    pub fn from_ref(val: Val, store: &'a mut Store<()>, instance: Option<Instance>) -> Result<Self> {
        let anyref = val.unwrap_anyref()
            .ok_or_else(|| anyhow::anyhow!("not an anyref"))?;
        let inner = anyref.unwrap_struct(&*store)?;
        Ok(Self { inner, store, instance, cached: RefCell::new(None) })
    }

    /// Create from an existing StructRef with borrowed store
    pub fn from_struct_ref(struct_ref: Rooted<StructRef>, store: &'a mut Store<()>, instance: Option<Instance>) -> Self {
        Self { inner: struct_ref, store, instance, cached: RefCell::new(None) }
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
            "friend" => 3,
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

/// Macro for defining type-safe struct wrappers with ergonomic field access
///
/// Generates a wrapper struct with typed accessor methods that hide the store.
///
/// Fields can be marked as `mut` to generate setter methods (only for types implementing ToVal).
///
/// # Example
/// ```
/// gc_struct! {
///     Person {
///         name: 0 => String,           // Immutable - only getter
///         age: 1 => mut i32,           // Mutable - getter + setter
///     }
/// }
///
/// // Usage:
/// let person = Person::new(GcObject::new(val, store)?);
/// let name: String = person.name()?;    // Getter
/// let age: i32 = person.age()?;         // Getter
/// person.set_age(30)?;                  // Setter (only for mut fields)
/// ```
#[macro_export]
macro_rules! gc_struct {
    // Entry point
    (
        $name:ident {
            $($field_spec:tt)*
        }
    ) => {
        pub struct $name {
            pub inner: $crate::gc_traits::GcObject<std::rc::Rc<std::cell::RefCell<wasmtime::Store<()>>>>,
        }

        impl $name {
            /// Create from a GcObject that owns the store
            pub fn new(obj: $crate::gc_traits::GcObject<std::rc::Rc<std::cell::RefCell<wasmtime::Store<()>>>>) -> Self {
                Self { inner: obj }
            }

            /// Create directly from Val, Store, and optional Instance (needed for string mutation)
            pub fn from_val(val: wasmtime::Val, store: wasmtime::Store<()>, instance: Option<wasmtime::Instance>) -> anyhow::Result<Self> {
                let obj = $crate::gc_traits::GcObject::new(val, store, instance)?;
                Ok(Self::new(obj))
            }

            /// Generic field setter - delegates to inner GcObject
            ///
            /// Useful for fields not defined in the struct (like nested object references)
            pub fn set_field<T: $crate::gc_traits::ToVal, I: $crate::gc_traits::FieldIndex>(&self, field: I, value: T) -> anyhow::Result<()> {
                self.inner.set_field(field, value)
            }

            /// Get nested struct field - delegates to inner GcObject
            pub fn get_struct<I: $crate::gc_traits::FieldIndex>(&self, field: I) -> anyhow::Result<wasmtime::Rooted<wasmtime::StructRef>> {
                self.inner.get_struct(field)
            }

            /// Get nested struct as a GcObject wrapper (shares the same store)
            ///
            /// Returns a GcObject that can be wrapped in any gc_struct! type
            pub fn get_struct_object<I: $crate::gc_traits::FieldIndex>(&self, field: I) -> anyhow::Result<$crate::gc_traits::GcObject<std::rc::Rc<std::cell::RefCell<wasmtime::Store<()>>>>> {
                self.inner.get_struct_object(field)
            }

            /// Get a field from a nested struct in one call
            ///
            /// # Example
            /// ```
            /// let friend_name: String = bob.get_nested("friend", "name")?;
            /// let friend_age: i32 = bob.get_nested("friend", 1)?;
            /// ```
            pub fn get_nested<T: $crate::gc_traits::FromVal, I1: $crate::gc_traits::FieldIndex, I2: $crate::gc_traits::FieldIndex>(
                &self,
                struct_field: I1,
                nested_field: I2
            ) -> anyhow::Result<T> {
                self.with_store(|store| {
                    let struct_field_idx = struct_field.to_field_index(self.inner.as_struct_ref(), &*store)?;
                    let nested_struct = self.inner.as_struct_ref().field(&mut *store, struct_field_idx)?;
                    let nested_anyref = nested_struct.unwrap_anyref()
                        .ok_or_else(|| anyhow::anyhow!("nested field is null or not a struct"))?;
                    let nested_struct_ref = nested_anyref.unwrap_struct(&*store)?;
                    let field_idx = nested_field.to_field_index(&nested_struct_ref, &*store)?;
                    let val = nested_struct_ref.field(&mut *store, field_idx)?;
                    T::from_val(val, &mut *store)
                })
            }

            /// Access the store with a closure
            pub fn with_store<F, R>(&self, f: F) -> R
            where
                F: FnOnce(&mut wasmtime::Store<()>) -> R,
            {
                self.inner.with_store(f)
            }
        }

        // Generate getters and conditional setters
        $crate::gc_struct!(@parse_fields $name; $($field_spec)*);
    };

    // Parse mutable String field (special case for ergonomic &str setters)
    (@parse_fields $name:ident; $field_name:ident : $field_idx:literal => mut String, $($rest:tt)*) => {
        $crate::gc_struct!(@impl_mut_string_field $name, $field_name);
        $crate::gc_struct!(@parse_fields $name; $($rest)*);
    };

    // Parse mutable String field (last one, no comma)
    (@parse_fields $name:ident; $field_name:ident : $field_idx:literal => mut String) => {
        $crate::gc_struct!(@impl_mut_string_field $name, $field_name);
    };

    // Parse mutable field (general case)
    (@parse_fields $name:ident; $field_name:ident : $field_idx:literal => mut $field_type:ty, $($rest:tt)*) => {
        $crate::gc_struct!(@impl_mut_field $name, $field_name, $field_type);
        $crate::gc_struct!(@parse_fields $name; $($rest)*);
    };

    // Parse mutable field (last one, no comma)
    (@parse_fields $name:ident; $field_name:ident : $field_idx:literal => mut $field_type:ty) => {
        $crate::gc_struct!(@impl_mut_field $name, $field_name, $field_type);
    };

    // Parse immutable field
    (@parse_fields $name:ident; $field_name:ident : $field_idx:literal => $field_type:ty, $($rest:tt)*) => {
        $crate::gc_struct!(@impl_field $name, $field_name, $field_type);
        $crate::gc_struct!(@parse_fields $name; $($rest)*);
    };

    // Parse immutable field (last one, no comma)
    (@parse_fields $name:ident; $field_name:ident : $field_idx:literal => $field_type:ty) => {
        $crate::gc_struct!(@impl_field $name, $field_name, $field_type);
    };

    // Base case: no more fields
    (@parse_fields $name:ident;) => {};

    // Implement getter + setter for mutable String field (takes &str)
    (@impl_mut_string_field $name:ident, $field_name:ident) => {
        paste::paste! {
            impl $name {
                /// Get field value
                pub fn $field_name(&self) -> anyhow::Result<String> {
                    self.inner.get(stringify!($field_name))
                }

                /// Set field value (for mutable string fields) - automatically creates GC string!
                pub fn [<set_ $field_name>](&self, value: &str) -> anyhow::Result<()> {
                    self.inner.set_field(stringify!($field_name), value)
                }
            }
        }
    };

    // Implement getter + setter for mutable field (general case)
    (@impl_mut_field $name:ident, $field_name:ident, $field_type:ty) => {
        paste::paste! {
            impl $name {
                /// Get field value
                pub fn $field_name(&self) -> anyhow::Result<$field_type> {
                    self.inner.get(stringify!($field_name))
                }

                /// Set field value (for mutable fields)
                pub fn [<set_ $field_name>](&self, value: $field_type) -> anyhow::Result<()> {
                    self.inner.set_field(stringify!($field_name), value)
                }
            }
        }
    };

    // Implement getter only for immutable field
    (@impl_field $name:ident, $field_name:ident, $field_type:ty) => {
        impl $name {
            /// Get field value
            pub fn $field_name(&self) -> anyhow::Result<$field_type> {
                self.inner.get(stringify!($field_name))
            }
        }
    };
}

// Keep WasmAccess export for potential future use
use rasm_macros::WasmAccess;



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
    /// Create a new GC string from a Rust &str
    ///
    /// This creates a WebAssembly GC array of i8 bytes from the given string.
    ///
    /// # Example
    /// ```
    /// let val = GcString::create(&mut store, &instance, "Hello")?;
    /// // Now you can pass `val` to WASM functions or set it in struct fields
    /// ```
    pub fn create(store: &mut Store<()>, instance: &Instance, s: &str) -> Result<Val> {
        // Get the string array type from the instance
        let new_string = instance
            .get_func(&mut *store, "new_string")
            .ok_or_else(|| anyhow::anyhow!("new_string function not found - ensure gc_types.wat exports it"))?;

        // Write string to linear memory
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| anyhow::anyhow!("memory not found"))?;

        let offset = 0;  // Use beginning of memory (safe in our demo)
        memory.write(&mut *store, offset, s.as_bytes())?;

        // Call new_string to create GC array
        let mut results = vec![Val::I32(0)];
        new_string.call(
            &mut *store,
            &[Val::I32(offset as i32), Val::I32(s.len() as i32)],
            &mut results,
        )?;

        Ok(results[0].clone())
    }

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
