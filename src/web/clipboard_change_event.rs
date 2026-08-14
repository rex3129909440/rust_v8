use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct ClipboardChangeEventRecord {
    pub(crate) types: v8::Global<v8::Array>,
    pub(crate) change_id: String,
}

#[derive(Default)]
pub(crate) struct ClipboardChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, ClipboardChangeEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ClipboardChangeEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ClipboardChangeEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ClipboardChangeEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ClipboardChangeEvent",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::clipboard_change_event_types_property::define(scope, prototype)?;
    super::clipboard_change_event_change_id_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ClipboardChangeEventStore>()
        .ok_or_else(|| "ClipboardChangeEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "ClipboardChangeEvent must be constructed");
        return;
    }
    let init = if arguments.length() == 0 {
        v8::Object::new(scope)
    } else {
        let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'ClipboardChangeEvent': The provided value is not of type 'ClipboardChangeEventInit'.",
            );
            return;
        };
        init
    };
    let event_type = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let (bubbles, cancelable, composed) = super::event::event_init(scope, init.into());
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let types = member(scope, init.into(), "types")
        .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
        .unwrap_or_else(|| v8::Array::new(scope, 0));
    let change_id = member(scope, init.into(), "changeId")
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default();
    let types = v8::Global::new(scope, types);
    scope
        .get_slot_mut::<ClipboardChangeEventStore>()
        .expect("ClipboardChangeEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ClipboardChangeEventRecord { types, change_id },
        );
    result.set(arguments.this().into());
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ClipboardChangeEventRecord> {
    scope
        .get_slot::<ClipboardChangeEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_types(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.types).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_change_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &record.change_id)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
