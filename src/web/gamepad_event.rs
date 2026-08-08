use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct GamepadEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Option<v8::Global<v8::Object>>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(GamepadEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "GamepadEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<GamepadEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let parent = super::event::ensure_constructor(s)?;
    let c = crate::webidl::create_function(
        s,
        "GamepadEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::gamepad_event_gamepad_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<GamepadEventStore>()
        .ok_or_else(|| "GamepadEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(s, "Please use the 'new' operator");
        return;
    }
    if a.length() < 1 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'GamepadEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(s, a.get(0));
    let gamepad = v8::Local::<v8::Object>::try_from(a.get(1))
        .ok()
        .and_then(|init| {
            v8::String::new(s, "gamepad")
                .and_then(|key| init.get(s, key.into()))
                .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok())
        })
        .map(|o| v8::Global::new(s, o));
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(s, a.this(), event_type, bubbles, cancelable, composed);
    s.get_slot_mut::<GamepadEventStore>()
        .expect("GamepadEvent state")
        .records
        .insert(a.this().get_identity_hash().get(), gamepad);
    r.set(a.this().into())
}
pub(crate) fn get_gamepad(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match s
        .get_slot::<GamepadEventStore>()
        .and_then(|x| x.records.get(&a.this().get_identity_hash().get()))
    {
        Some(Some(v)) => r.set(v8::Local::new(s, v).into()),
        Some(None) => r.set(v8::null(s).into()),
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
