use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct ErrorEventRecord {
    pub(crate) message: String,
    pub(crate) filename: String,
    pub(crate) lineno: u32,
    pub(crate) colno: u32,
    pub(crate) error: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct ErrorEventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, ErrorEventRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ErrorEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ErrorEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<ErrorEventStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ErrorEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::error_event_message_property::define(scope, prototype)?;
    super::error_event_filename_property::define(scope, prototype)?;
    super::error_event_lineno_property::define(scope, prototype)?;
    super::error_event_colno_property::define(scope, prototype)?;
    super::error_event_error_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ErrorEventStore>()
        .ok_or_else(|| "ErrorEvent state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ErrorEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let message = string_property(scope, init, "message");
    let filename = string_property(scope, init, "filename");
    let lineno = number_property(scope, init, "lineno") as u32;
    let colno = number_property(scope, init, "colno") as u32;
    let error = init
        .and_then(|object| property(scope, object, "error"))
        .unwrap_or_else(|| v8::null(scope).into());
    let error = v8::Global::new(scope, error);
    let bubbles =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "bubbles"));
    let cancelable =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "cancelable"));
    let composed =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "composed"));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<ErrorEventStore>()
        .expect("ErrorEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ErrorEventRecord {
                message,
                filename,
                lineno,
                colno,
                error,
            },
        );
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    message: String,
    error: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_detailed(scope, event_type, message, String::new(), 0, 0, error)
}

pub(crate) fn create_detailed<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: &str,
    message: String,
    filename: String,
    lineno: u32,
    colno: u32,
    error: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create ErrorEvent".to_owned());
    }
    super::event::attach(scope, event, event_type.to_owned(), false, true, false);
    let error = v8::Global::new(scope, error);
    scope
        .get_slot_mut::<ErrorEventStore>()
        .ok_or_else(|| "ErrorEvent state was not prepared".to_owned())?
        .records
        .insert(
            event.get_identity_hash().get(),
            ErrorEventRecord {
                message,
                filename,
                lineno,
                colno,
                error,
            },
        );
    Ok(event)
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> String {
    object
        .and_then(|object| property(scope, object, name))
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}

pub(crate) fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> f64 {
    object
        .and_then(|object| property(scope, object, name))
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(0.0)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ErrorEventRecord> {
    scope
        .get_slot::<ErrorEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn string_get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&ErrorEventRecord) -> &str,
) {
    if let Some(record) = record(s, a.this()) {
        if let Some(value) = v8::String::new(s, select(&record)) {
            r.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_get(s, a, r, |record| &record.message)
}
pub(crate) fn get_filename(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_get(s, a, r, |record| &record.filename)
}
pub(crate) fn get_lineno(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, record.lineno).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_colno(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, record.colno).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Local::new(s, &record.error))
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ErrorEventStore>() {
        store.constructors.remove(&realm_id);
    }
}
