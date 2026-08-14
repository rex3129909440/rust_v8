use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AbortControllerStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    signals: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AbortControllerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AbortController", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AbortControllerStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AbortController",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "signal", get_signal)?;
    crate::webidl::define_method(scope, prototype, "abort", 0, abort)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AbortControllerStore>()
        .ok_or_else(|| "AbortController state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'AbortController': use new");
        return;
    }
    let signal = match super::abort_signal::create(scope, None) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    attach(scope, arguments.this(), signal);
    result.set(arguments.this().into());
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    controller: v8::Local<'_, v8::Object>,
    signal: v8::Local<'_, v8::Object>,
) {
    let signal = v8::Global::new(scope, signal);
    if let Some(store) = scope.get_slot_mut::<AbortControllerStore>() {
        store
            .signals
            .insert(controller.get_identity_hash().get(), signal);
    }
}

pub(crate) fn signal(
    scope: &v8::PinScope<'_, '_>,
    controller: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<AbortControllerStore>()?
        .signals
        .get(&controller.get_identity_hash().get())
        .cloned()
}

fn get_signal(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = signal(scope, a.this()) {
        r.set(v8::Local::new(scope, &v).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn abort(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = signal(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let signal = v8::Local::new(scope, &v);
    let reason = if a.length() == 0 {
        super::dom_exception::create(
            scope,
            "This operation was aborted".to_owned(),
            "AbortError".to_owned(),
        )
        .map(Into::into)
        .unwrap_or_else(|_| v8::undefined(scope).into())
    } else {
        a.get(0)
    };
    super::abort_signal::abort(scope, signal, reason);
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<AbortControllerStore>() {
        store.constructors.remove(&realm_id);
    }
}
