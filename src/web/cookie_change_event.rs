use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CookieChangeEventRecord {
    pub(crate) changed: v8::Global<v8::Array>,
    pub(crate) deleted: v8::Global<v8::Array>,
}

#[derive(Default)]
pub(crate) struct CookieChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, CookieChangeEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CookieChangeEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CookieChangeEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CookieChangeEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CookieChangeEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::cookie_change_event_changed_property::define(scope, prototype)?;
    super::cookie_change_event_deleted_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CookieChangeEventStore>()
        .ok_or_else(|| "CookieChangeEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn array_member<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> v8::Local<'s, v8::Array> {
    v8::Local::<v8::Object>::try_from(value)
        .ok()
        .and_then(|object| {
            let key = v8::String::new(scope, name)?;
            object.get(scope, key.into())
        })
        .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
        .unwrap_or_else(|| v8::Array::new(scope, 0))
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CookieChangeEvent requires an event type");
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
    let changed = array_member(scope, arguments.get(1), "changed");
    let deleted = array_member(scope, arguments.get(1), "deleted");
    let changed = v8::Global::new(scope, changed);
    let deleted = v8::Global::new(scope, deleted);
    scope
        .get_slot_mut::<CookieChangeEventStore>()
        .expect("CookieChangeEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CookieChangeEventRecord { changed, deleted },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CookieChangeEventRecord> {
    scope
        .get_slot::<CookieChangeEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_changed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.changed).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_deleted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.deleted).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    changed: v8::Local<'_, v8::Array>,
    deleted: v8::Local<'_, v8::Array>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create CookieChangeEvent".to_owned());
    }
    super::event::attach(scope, event, "change".to_owned(), false, false, false);
    let changed = v8::Global::new(scope, changed);
    let deleted = v8::Global::new(scope, deleted);
    scope
        .get_slot_mut::<CookieChangeEventStore>()
        .ok_or_else(|| "CookieChangeEvent state was not prepared".to_owned())?
        .records
        .insert(
            event.get_identity_hash().get(),
            CookieChangeEventRecord { changed, deleted },
        );
    Ok(event)
}
