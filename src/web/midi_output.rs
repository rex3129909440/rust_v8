#[derive(Default)]
pub(crate) struct MidiOutputStore {
    constructor: crate::webidl::RealmConstructor,
    sent: std::collections::HashMap<i32, Vec<Vec<u8>>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MidiOutputStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MIDIOutput", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MidiOutputStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MIDIOutput",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "send", 1, send)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::midi_port::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MidiOutputStore>()
        .ok_or_else(|| "MIDIOutput state was not prepared".to_owned())?
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
        return Err("cannot create MIDIOutput".to_owned());
    }
    super::midi_port::attach(scope, object, "output", profile);
    scope
        .get_slot_mut::<MidiOutputStore>()
        .ok_or_else(|| "MIDIOutput state was not prepared".to_owned())?
        .sent
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}
fn send(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if scope
        .get_slot::<MidiOutputStore>()
        .and_then(|store| store.sent.get(&arguments.this().get_identity_hash().get()))
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "MIDI data must be a Uint8Array");
        return;
    };
    let mut bytes = vec![0; view.byte_length()];
    let copied = view.copy_contents(&mut bytes);
    bytes.truncate(copied);
    if let Some(sent) = scope.get_slot_mut::<MidiOutputStore>().and_then(|store| {
        store
            .sent
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        sent.push(bytes)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
