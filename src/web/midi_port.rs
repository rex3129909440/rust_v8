use std::collections::HashMap;
#[derive(Clone)]
pub(crate) struct MidiPortRecord {
    pub(crate) connection: String,
    pub(crate) id: String,
    pub(crate) manufacturer: String,
    pub(crate) name: String,
    pub(crate) state: String,
    pub(crate) kind: String,
    pub(crate) version: String,
    pub(crate) onstatechange: Option<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct MidiPortStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MidiPortRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MidiPortStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MIDIPort", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MidiPortStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MIDIPort",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "connection", get_connection)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "manufacturer", get_manufacturer)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "state", get_state)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "version", get_version)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onstatechange",
        get_onstatechange,
        set_onstatechange,
    )?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "open", 0, open)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MidiPortStore>()
        .ok_or_else(|| "MIDIPort state was not prepared".to_owned())?
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
pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    kind: &str,
    profile: &crate::MidiPortFingerprint,
) {
    super::event_target::attach(scope, object);
    if let Some(store) = scope.get_slot_mut::<MidiPortStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            MidiPortRecord {
                connection: profile.connection.clone(),
                id: profile.id.clone(),
                manufacturer: profile.manufacturer.clone(),
                name: profile.name.clone(),
                state: profile.state.clone(),
                kind: kind.to_owned(),
                version: profile.version.clone(),
                onstatechange: None,
            },
        );
    }
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MidiPortRecord> {
    scope
        .get_slot::<MidiPortStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    pick: impl FnOnce(MidiPortRecord) -> String,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &pick(record))
    {
        result.set(value.into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_connection(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.connection)
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.id)
}
fn get_manufacturer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.manufacturer)
}
fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.name)
}
fn get_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.state)
}
fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.kind)
}
fn get_version(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.version)
}
fn get_onstatechange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(s, record.onstatechange, r)
}
fn set_onstatechange(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(record) = s
        .get_slot_mut::<MidiPortStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.onstatechange = handler
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn transition(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    state: &str,
    method_name: &str,
) {
    if let Some(record) = s
        .get_slot_mut::<MidiPortStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.connection = state.to_owned();
        let value = v8::undefined(s);
        if let Ok(promise) = super::writable_stream::resolved_promise(s, value.into()) {
            r.set(promise.into())
        }
    } else {
        crate::webidl::reject_illegal_invocation_promise(s, "MIDIPort", method_name, r)
    }
}
fn open(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    transition(s, a, r, "open", "open")
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    transition(s, a, r, "closed", "close")
}
