#[derive(Default)]
pub(crate) struct IdbCursorWithValueStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IdbCursorWithValueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IDBCursorWithValue", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<IdbCursorWithValueStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IDBCursorWithValue",
        0,
        v8::ConstructorBehavior::Allow,
        super::webgl_object::illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "value", get_value)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::idb_cursor::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IdbCursorWithValueStore>()
        .ok_or_else(|| "IDBCursorWithValue state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match super::idb_cursor::current_value(scope, arguments.this()) {
        Some(value) => result.set(v8::Local::new(scope, &value)),
        None if super::idb_cursor::is_cursor(scope, arguments.this()) => {
            result.set(v8::undefined(scope).into())
        }
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<IdbCursorWithValueStore>() {
        store.constructor.remove(realm_id);
    }
}
