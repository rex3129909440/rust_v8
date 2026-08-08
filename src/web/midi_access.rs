use std::collections::HashMap;
#[derive(Clone)]
struct AccessRecord {
    inputs: v8::Global<v8::Object>,
    outputs: v8::Global<v8::Object>,
    onstatechange: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct MidiAccessStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AccessRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MidiAccessStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MIDIAccess", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MidiAccessStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MIDIAccess",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "inputs", get_inputs)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "outputs", get_outputs)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sysexEnabled", get_sysex_enabled)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onstatechange",
        get_onstatechange,
        set_onstatechange,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MidiAccessStore>()
        .ok_or_else(|| "MIDIAccess state was not prepared".to_owned())?
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
        return Err("cannot create MIDIAccess".to_owned());
    }
    super::event_target::attach(scope, object);
    let inputs = super::midi_input_map::create(scope)?;
    let outputs = super::midi_output_map::create(scope)?;
    let record = AccessRecord {
        inputs: v8::Global::new(scope, inputs),
        outputs: v8::Global::new(scope, outputs),
        onstatechange: None,
    };
    scope
        .get_slot_mut::<MidiAccessStore>()
        .ok_or_else(|| "MIDIAccess state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}
fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<AccessRecord> {
    scope
        .get_slot::<MidiAccessStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_inputs(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.inputs).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_outputs(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.outputs).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_sysex_enabled(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        let enabled = crate::fingerprint::edge(s)
            .hardware_devices
            .midi_sysex_enabled;
        r.set(v8::Boolean::new(s, enabled).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_onstatechange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.onstatechange),
        r,
    )
}
fn set_onstatechange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<MidiAccessStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.onstatechange = handler
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
