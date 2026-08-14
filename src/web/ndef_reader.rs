use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct NdefReaderStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
    reading: HashMap<i32, v8::Global<v8::Value>>,
    reading_error: HashMap<i32, v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NdefReaderStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(scope)?;
    crate::webidl::define_global(scope, "NDEFReader", c.into())
}
fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(scope);
    if let Some(v) = scope
        .get_slot::<NdefReaderStore>()
        .and_then(|s| s.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &v));
    }
    let c = crate::webidl::create_function(
        scope,
        "NDEFReader",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_accessor(scope, p, "onreading", get_reading, set_reading)?;
    crate::webidl::define_accessor(
        scope,
        p,
        "onreadingerror",
        get_reading_error,
        set_reading_error,
    )?;
    crate::webidl::define_method(scope, p, "makeReadOnly", 0, make_read_only)?;
    crate::webidl::define_method(scope, p, "scan", 0, scan)?;
    crate::webidl::define_method(scope, p, "write", 1, write)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    super::android_api_support::set_tag(scope, p, "NDEFReader")?;
    let stored_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<NdefReaderStore>()
        .unwrap()
        .constructor
        .insert(realm, stored_constructor);
    Ok(c)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'NDEFReader': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    super::event_target::attach(scope, a.this());
    scope
        .get_slot_mut::<NdefReaderStore>()
        .unwrap()
        .instances
        .insert(a.this().get_identity_hash().get());
    r.set(a.this().into());
}
fn valid(
    scope: &mut v8::PinScope<'_, '_>,
    a: &v8::FunctionCallbackArguments<'_>,
    op: &str,
) -> bool {
    let valid = scope
        .get_slot::<NdefReaderStore>()
        .unwrap()
        .instances
        .contains(&a.this().get_identity_hash().get());
    super::android_api_support::require_brand(scope, valid, "NDEFReader", op)
}
fn denied(scope: &mut v8::PinScope<'_, '_>, operation: &str, mut r: v8::ReturnValue<'_>) {
    let message =
        format!("Failed to execute '{operation}' on 'NDEFReader': NFC permission request denied.");
    if let Some(p) =
        super::android_api_support::rejected_dom_exception(scope, "NotAllowedError", &message)
    {
        r.set(p.into());
    }
}
fn scan(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if valid(s, &a, "scan") {
        denied(s, "scan", r)
    }
}
fn make_read_only(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if valid(s, &a, "makeReadOnly") {
        denied(s, "makeReadOnly", r)
    }
}
fn write(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if !valid(s, &a, "write") {
        return;
    }
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to execute 'write' on 'NDEFReader': 1 argument required, but only 0 present.",
        );
        return;
    }
    denied(s, "write", r)
}
fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    error: bool,
) {
    if !valid(
        scope,
        &a,
        if error { "onreadingerror" } else { "onreading" },
    ) {
        return;
    }
    let store = scope.get_slot::<NdefReaderStore>().unwrap();
    let value = if error {
        store.reading_error.get(&a.this().get_identity_hash().get())
    } else {
        store.reading.get(&a.this().get_identity_hash().get())
    };
    if let Some(v) = value {
        r.set(v8::Local::new(scope, v));
    } else {
        r.set(v8::null(scope).into());
    }
}
fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    error: bool,
) {
    if !valid(
        scope,
        &a,
        if error { "onreadingerror" } else { "onreading" },
    ) {
        return;
    }
    let id = a.this().get_identity_hash().get();
    let value = super::window_event_handler_support::handler_value(scope, a.get(0));
    let present = {
        let store = scope.get_slot_mut::<NdefReaderStore>().unwrap();
        let values = if error {
            &mut store.reading_error
        } else {
            &mut store.reading
        };
        if let Some(value) = value {
            values.insert(id, value);
        } else {
            values.remove(&id);
        }
        values.contains_key(&id)
    };
    super::event_target::set_attribute_handler(
        scope,
        a.this(),
        if error { "readingerror" } else { "reading" },
        present,
    );
}
fn get_reading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, false)
}
fn set_reading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, false)
}
fn get_reading_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, true)
}
fn set_reading_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, true)
}
