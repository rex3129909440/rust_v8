use std::collections::{HashMap, HashSet};
#[derive(Default)]
pub(crate) struct KeyboardStore {
    constructor: crate::webidl::RealmConstructor,
    locks: HashMap<i32, HashSet<String>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(KeyboardStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Keyboard", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<KeyboardStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Keyboard",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getLayoutMap", 0, get_layout_map)?;
    crate::webidl::define_method(scope, prototype, "lock", 0, lock)?;
    crate::webidl::define_method(scope, prototype, "unlock", 0, unlock)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<KeyboardStore>()
        .ok_or_else(|| "Keyboard state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Keyboard".to_owned());
    }
    scope
        .get_slot_mut::<KeyboardStore>()
        .ok_or_else(|| "Keyboard state was not prepared".to_owned())?
        .locks
        .insert(object.get_identity_hash().get(), HashSet::new());
    Ok(object)
}
fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<KeyboardStore>()
        .is_some_and(|store| store.locks.contains_key(&object.get_identity_hash().get()))
}
fn resolve(
    s: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(s, value) {
        r.set(promise.into())
    }
}
fn get_layout_map(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "Keyboard", "getLayoutMap", r);
        return;
    }
    if let Ok(map) = super::keyboard_layout_map::create(s) {
        resolve(s, map.into(), r)
    }
}
fn lock(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, a.this()) {
        crate::webidl::reject_illegal_invocation_promise(s, "Keyboard", "lock", r);
        return;
    }
    let mut names = HashSet::new();
    if let Ok(array) = v8::Local::<v8::Array>::try_from(a.get(0)) {
        for index in 0..array.length() {
            if let Some(value) = array.get_index(s, index) {
                names.insert(crate::webidl::value_to_string(s, value));
            }
        }
    }
    if let Some(locks) = s
        .get_slot_mut::<KeyboardStore>()
        .and_then(|store| store.locks.get_mut(&a.this().get_identity_hash().get()))
    {
        *locks = names;
        resolve(s, v8::undefined(s).into(), r)
    }
}
fn unlock(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(locks) = s
        .get_slot_mut::<KeyboardStore>()
        .and_then(|store| store.locks.get_mut(&a.this().get_identity_hash().get()))
    {
        locks.clear()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
