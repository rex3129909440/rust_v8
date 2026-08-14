use std::collections::HashMap;
#[derive(Clone)]
struct ActuatorRecord {
    effects: v8::Global<v8::Array>,
    actuator_type: String,
    playing: bool,
}
#[derive(Default)]
pub(crate) struct GamepadHapticActuatorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ActuatorRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(GamepadHapticActuatorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "GamepadHapticActuator", c.into())
}
fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<GamepadHapticActuatorStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "GamepadHapticActuator",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "effects", get_effects)?;
    crate::webidl::define_readonly_accessor(s, p, "type", get_type)?;
    crate::webidl::define_method(s, p, "playEffect", 2, play_effect)?;
    crate::webidl::define_method(s, p, "reset", 0, reset)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<GamepadHapticActuatorStore>()
        .ok_or_else(|| "GamepadHapticActuator state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create GamepadHapticActuator".to_owned());
    }
    let effects = v8::Array::new(s, 2);
    if let Some(v) = v8::String::new(s, "dual-rumble") {
        let _ = effects.set_index(s, 0, v.into());
    }
    if let Some(v) = v8::String::new(s, "trigger-rumble") {
        let _ = effects.set_index(s, 1, v.into());
    }
    let effects = v8::Global::new(s, effects);
    s.get_slot_mut::<GamepadHapticActuatorStore>()
        .ok_or_else(|| "GamepadHapticActuator state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            ActuatorRecord {
                effects,
                actuator_type: "vibration".to_owned(),
                playing: false,
            },
        );
    Ok(o)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ActuatorRecord> {
    s.get_slot::<GamepadHapticActuatorStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_effects(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.effects).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        if let Some(v) = v8::String::new(s, &x.actuator_type) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn resolved<'s>(s: &mut v8::PinScope<'s, '_>, value: &str) -> Option<v8::Local<'s, v8::Promise>> {
    let resolver = v8::PromiseResolver::new(s)?;
    let promise = resolver.get_promise(s);
    let value = v8::String::new(s, value)?;
    let _ = resolver.resolve(s, value.into());
    Some(promise)
}
fn play_effect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = s
        .get_slot_mut::<GamepadHapticActuatorStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.playing = true;
        if let Some(p) = resolved(s, "complete") {
            r.set(p.into())
        }
    } else {
        crate::webidl::reject_illegal_invocation_promise(
            s,
            "GamepadHapticActuator",
            "playEffect",
            r,
        )
    }
}
fn reset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = s
        .get_slot_mut::<GamepadHapticActuatorStore>()
        .and_then(|q| q.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.playing = false;
        if let Some(p) = resolved(s, "complete") {
            r.set(p.into())
        }
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "GamepadHapticActuator", "reset", r)
    }
}
