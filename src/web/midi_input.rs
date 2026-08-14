use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct MidiInputStore {
    constructor: crate::webidl::RealmConstructor,
    handlers: HashMap<i32, Option<v8::Global<v8::Value>>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MidiInputStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MIDIInput", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MidiInputStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MIDIInput",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "onmidimessage", get_handler, set_handler)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::midi_port::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MidiInputStore>()
        .ok_or_else(|| "MIDIInput state was not prepared".to_owned())?
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
    profile: &crate::MidiPortFingerprint,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MIDIInput".to_owned());
    }
    super::midi_port::attach(scope, object, "input", profile);
    scope
        .get_slot_mut::<MidiInputStore>()
        .ok_or_else(|| "MIDIInput state was not prepared".to_owned())?
        .handlers
        .insert(object.get_identity_hash().get(), None);
    Ok(object)
}
fn get_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let valid = s.get_slot::<MidiInputStore>().is_some_and(|store| {
        store
            .handlers
            .contains_key(&a.this().get_identity_hash().get())
    });
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let handler = s
        .get_slot::<MidiInputStore>()
        .and_then(|store| store.handlers.get(&a.this().get_identity_hash().get()))
        .cloned()
        .flatten();
    super::window_event_handler_support::return_handler(s, handler, r)
}
fn set_handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(value) = s
        .get_slot_mut::<MidiInputStore>()
        .and_then(|store| store.handlers.get_mut(&a.this().get_identity_hash().get()))
    {
        *value = handler
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
