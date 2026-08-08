use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct MidiMessageEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) data: HashMap<i32, v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MidiMessageEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MIDIMessageEvent", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MidiMessageEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MIDIMessageEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::midi_message_event_data_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MidiMessageEventStore>()
        .ok_or_else(|| "MIDIMessageEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "MIDIMessageEvent requires an event type");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let value = v8::Local::<v8::Object>::try_from(arguments.get(1))
        .ok()
        .and_then(|init| v8::String::new(scope, "data").and_then(|key| init.get(scope, key.into())))
        .unwrap_or_else(|| v8::null(scope).into());
    let value = v8::Global::new(scope, value);
    scope
        .get_slot_mut::<MidiMessageEventStore>()
        .expect("MIDIMessageEvent state")
        .data
        .insert(arguments.this().get_identity_hash().get(), value);
    result.set(arguments.this().into())
}
pub(crate) fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<MidiMessageEventStore>()
        .and_then(|store| store.data.get(&arguments.this().get_identity_hash().get()))
        .cloned()
    {
        result.set(v8::Local::new(scope, &value))
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
