use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct VideoColorSpaceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ColorSpaceRecord>,
}

#[derive(Clone)]
struct ColorSpaceRecord {
    primaries: Option<String>,
    transfer: Option<String>,
    matrix: Option<String>,
    full_range: Option<bool>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(VideoColorSpaceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "VideoColorSpace", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<VideoColorSpaceStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "VideoColorSpace",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "primaries", get_primaries)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "transfer", get_transfer)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "matrix", get_matrix)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "fullRange", get_full_range)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<VideoColorSpaceStore>()
        .ok_or_else(|| "VideoColorSpace state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "VideoColorSpace must be constructed with new");
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let primaries = init.and_then(|init| optional_string(scope, init, "primaries"));
    let transfer = init.and_then(|init| optional_string(scope, init, "transfer"));
    let matrix = init.and_then(|init| optional_string(scope, init, "matrix"));
    let full_range = init.and_then(|init| optional_boolean(scope, init, "fullRange"));
    attach(
        scope,
        arguments.this(),
        primaries,
        transfer,
        matrix,
        full_range,
    );
    result.set(arguments.this().into());
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    primaries: Option<String>,
    transfer: Option<String>,
    matrix: Option<String>,
    full_range: Option<bool>,
) {
    if let Some(store) = scope.get_slot_mut::<VideoColorSpaceStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            ColorSpaceRecord {
                primaries,
                transfer,
                matrix,
                full_range,
            },
        );
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    primaries: Option<String>,
    transfer: Option<String>,
    matrix: Option<String>,
    full_range: Option<bool>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create VideoColorSpace".to_owned());
    }
    attach(scope, object, primaries, transfer, matrix, full_range);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ColorSpaceRecord> {
    scope
        .get_slot::<VideoColorSpaceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_optional_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ColorSpaceRecord) -> Option<&str>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record).and_then(|value| v8::String::new(scope, value)) {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_primaries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_optional_string(s, a, r, |record| record.primaries.as_deref())
}
fn get_transfer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_optional_string(s, a, r, |record| record.transfer.as_deref())
}
fn get_matrix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_optional_string(s, a, r, |record| record.matrix.as_deref())
}

fn get_full_range(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.full_range {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    let primaries = record
        .primaries
        .as_deref()
        .and_then(|value| v8::String::new(scope, value))
        .map(Into::into)
        .unwrap_or_else(|| v8::null(scope).into());
    define_data(scope, output, "primaries", primaries);
    let transfer = record
        .transfer
        .as_deref()
        .and_then(|value| v8::String::new(scope, value))
        .map(Into::into)
        .unwrap_or_else(|| v8::null(scope).into());
    define_data(scope, output, "transfer", transfer);
    let matrix = record
        .matrix
        .as_deref()
        .and_then(|value| v8::String::new(scope, value))
        .map(Into::into)
        .unwrap_or_else(|| v8::null(scope).into());
    define_data(scope, output, "matrix", matrix);
    let full_range: v8::Local<v8::Value> = record
        .full_range
        .map(|value| v8::Boolean::new(scope, value).into())
        .unwrap_or_else(|| v8::null(scope).into());
    define_data(scope, output, "fullRange", full_range);
    result.set(output.into());
}

fn optional_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_null() && !value.is_undefined())
        .then(|| crate::webidl::value_to_string(scope, value))
}

fn optional_boolean(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<bool> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_null() && !value.is_undefined()).then(|| value.boolean_value(scope))
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<VideoColorSpaceStore>() {
        store.constructor.remove(realm_id);
    }
}
