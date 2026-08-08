use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ScreenOrientationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Record>,
}
#[derive(Clone)]
struct Record {
    angle: u16,
    orientation_type: String,
    onchange: Option<v8::Global<v8::Value>>,
    locked: bool,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ScreenOrientationStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ScreenOrientation", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ScreenOrientationStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "ScreenOrientation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::define_readonly_accessor(scope, p, "angle", get_angle)?;
    crate::webidl::define_readonly_accessor(scope, p, "type", get_type)?;
    crate::webidl::define_accessor(scope, p, "onchange", get_onchange, set_onchange)?;
    crate::webidl::define_method(scope, p, "lock", 1, lock)?;
    crate::webidl::define_method(scope, p, "unlock", 0, unlock)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<ScreenOrientationStore>()
        .ok_or_else(|| "ScreenOrientation state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create ScreenOrientation".to_owned());
    }
    super::event_target::attach(scope, o);
    let profile = crate::fingerprint::edge(scope).screen.clone();
    scope
        .get_slot_mut::<ScreenOrientationStore>()
        .ok_or_else(|| "ScreenOrientation state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            Record {
                angle: profile.orientation_angle,
                orientation_type: profile.orientation_type,
                onchange: None,
                locked: false,
            },
        );
    Ok(o)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ScreenOrientation': Illegal constructor",
    );
}
fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<ScreenOrientationStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_angle(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, v.angle as u32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, &v.orientation_type) {
        r.set(s.into())
    }
}
fn get_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(h) = v.onchange {
        r.set(v8::Local::new(scope, &h))
    } else {
        r.set(v8::null(scope).into())
    }
}
fn set_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0);
    let h = if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    };
    if let Some(v) = scope
        .get_slot_mut::<ScreenOrientationStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.onchange = h
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn lock(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    let allowed = matches!(
        value.as_str(),
        "any"
            | "natural"
            | "landscape"
            | "portrait"
            | "portrait-primary"
            | "portrait-secondary"
            | "landscape-primary"
            | "landscape-secondary"
    );
    if !allowed {
        crate::webidl::throw_type_error(scope, "Invalid orientation lock");
        return;
    }
    let Some(v) = scope
        .get_slot_mut::<ScreenOrientationStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    v.locked = true;
    if value.contains("portrait") {
        v.orientation_type = "portrait-primary".to_owned();
        v.angle = 90;
    } else if value.contains("landscape") {
        v.orientation_type = "landscape-primary".to_owned();
        v.angle = 0;
    }
    let value = v8::undefined(scope);
    if let Ok(p) = super::writable_stream::resolved_promise(scope, value.into()) {
        r.set(p.into())
    }
}
fn unlock(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = scope
        .get_slot_mut::<ScreenOrientationStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.locked = false
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
