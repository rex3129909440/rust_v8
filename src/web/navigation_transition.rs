use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigationTransitionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigationTransitionRecord>,
}
#[derive(Clone)]
struct NavigationTransitionRecord {
    navigation_type: String,
    from: v8::Global<v8::Object>,
    to: v8::Global<v8::Object>,
    committed: v8::Global<v8::Promise>,
    finished: v8::Global<v8::Promise>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigationTransitionStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigationTransition", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<NavigationTransitionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let c = crate::webidl::create_function(
        scope,
        "NavigationTransition",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "navigationType", get_navigation_type)?;
    crate::webidl::define_readonly_accessor(scope, p, "from", get_from)?;
    crate::webidl::define_readonly_accessor(scope, p, "to", get_to)?;
    crate::webidl::define_readonly_accessor(scope, p, "committed", get_committed)?;
    crate::webidl::define_readonly_accessor(scope, p, "finished", get_finished)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NavigationTransitionStore>()
        .ok_or_else(|| "NavigationTransition state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    navigation_type: String,
    from: v8::Local<'_, v8::Object>,
    to: v8::Local<'_, v8::Object>,
    commit_value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, p.into()) != Some(true) {
        return Err("cannot create NavigationTransition".to_owned());
    }
    let committed = super::writable_stream::resolved_promise(scope, commit_value)?;
    let finished = super::writable_stream::resolved_promise(scope, v8::undefined(scope).into())?;
    let record = NavigationTransitionRecord {
        navigation_type,
        from: v8::Global::new(scope, from),
        to: v8::Global::new(scope, to),
        committed: v8::Global::new(scope, committed),
        finished: v8::Global::new(scope, finished),
    };
    scope
        .get_slot_mut::<NavigationTransitionStore>()
        .ok_or_else(|| "NavigationTransition state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NavigationTransition': Illegal constructor",
    )
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigationTransitionRecord> {
    scope
        .get_slot::<NavigationTransitionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_navigation_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        if let Some(s) = v8::String::new(scope, &v.navigation_type) {
            r.set(s.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_from(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.from).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_to(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.to).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_committed(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.committed).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_finished(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &v.finished).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
