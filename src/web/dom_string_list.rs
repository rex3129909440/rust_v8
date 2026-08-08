use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DomStringListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<String>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomStringListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMStringList", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<DomStringListStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMStringList",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "contains", 1, contains)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomStringListStore>()
        .ok_or_else(|| "DOMStringList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    mut values: Vec<String>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    values.sort();
    values.dedup();
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DOMStringList".to_owned());
    }
    for (index, value) in values.iter().enumerate() {
        let value = crate::webidl::string(scope, value)?;
        let key = crate::webidl::string(scope, &index.to_string())?;
        let _ = object.define_own_property(
            scope,
            key.into(),
            value.into(),
            v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
        );
    }
    scope
        .get_slot_mut::<DomStringListStore>()
        .ok_or_else(|| "DOMStringList state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), values);
    Ok(object)
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<Vec<String>> {
    scope
        .get_slot::<DomStringListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn contains(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    result.set(v8::Boolean::new(scope, record.contains(&value)).into());
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(value) = record.get(index) {
        if let Some(value) = v8::String::new(scope, value) {
            result.set(value.into());
        }
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DomStringListStore>() {
        store.constructor.remove(realm_id);
    }
}
