use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DragEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) data_transfers: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DragEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DragEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(c) = s
        .get_slot::<DragEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "DragEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::mouse_event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::drag_event_data_transfer_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<DragEventStore>()
        .ok_or_else(|| "DragEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let event_type = crate::webidl::string(scope, "")?;
    constructor
        .new_instance(scope, &[event_type.into()])
        .ok_or_else(|| "cannot create DragEvent".to_owned())
}

pub(crate) fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "Failed to construct 'DragEvent': 1 argument required");
        return;
    }
    let event_type = crate::webidl::value_to_string(s, a.get(0));
    let data = super::mouse_event::read_init(s, a.get(1));
    let transfer = v8::Local::<v8::Object>::try_from(a.get(1))
        .ok()
        .and_then(|init| {
            let key = v8::String::new(s, "dataTransfer")?;
            init.get(s, key.into())
        })
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .map(|value| v8::Global::new(s, value));
    super::mouse_event::attach(s, a.this(), event_type, data);
    s.get_slot_mut::<DragEventStore>()
        .expect("DragEvent state")
        .data_transfers
        .insert(a.this().get_identity_hash().get(), transfer);
    r.set(a.this().into())
}
pub(crate) fn get_data_transfer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match s
        .get_slot::<DragEventStore>()
        .and_then(|x| x.data_transfers.get(&a.this().get_identity_hash().get()))
    {
        Some(Some(value)) => r.set(v8::Local::new(s, value).into()),
        Some(None) => r.set(v8::null(s).into()),
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
