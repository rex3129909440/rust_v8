use std::collections::HashMap;

#[derive(Clone)]
struct WebSocketErrorRecord {
    close_code: Option<u16>,
    reason: String,
}

#[derive(Default)]
pub(crate) struct WebSocketErrorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WebSocketErrorRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WebSocketErrorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WebSocketError", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<WebSocketErrorStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WebSocketError",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "closeCode", get_close_code)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reason", get_reason)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::dom_exception::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WebSocketErrorStore>()
        .ok_or_else(|| "WebSocketError state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WebSocketError': Please use the 'new' operator.",
        );
        return;
    }
    let message = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let close_code = number_property(scope, init, "closeCode");
    if let Some(code) = close_code
        && !valid_close_code(code)
    {
        throw_invalid_code(scope, code);
        return;
    }
    let reason = string_property(scope, init, "reason");
    super::dom_exception::attach(
        scope,
        arguments.this(),
        "WebSocketError".to_owned(),
        message,
        0,
    );
    scope
        .get_slot_mut::<WebSocketErrorStore>()
        .expect("WebSocketError state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            WebSocketErrorRecord { close_code, reason },
        );
    result.set(arguments.this().into());
}

pub(crate) fn valid_close_code(code: u16) -> bool {
    code == 1000 || (3000..=4999).contains(&code)
}

pub(crate) fn throw_invalid_code(scope: &mut v8::PinScope<'_, '_>, code: u16) {
    let message =
        format!("The close code must be either 1000, or between 3000 and 4999. {code} is neither.");
    if let Ok(exception) =
        super::dom_exception::create(scope, message, "InvalidAccessError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn number_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<u16> {
    let object = object?;
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() || value.is_null() {
        None
    } else {
        value
            .integer_value(scope)
            .map(|value| value.clamp(0, u16::MAX as i64) as u16)
    }
}
fn string_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> String {
    let Some(object) = object else {
        return String::new();
    };
    let Some(key) = v8::String::new(scope, name) else {
        return String::new();
    };
    let Some(value) = object.get(scope, key.into()) else {
        return String::new();
    };
    if value.is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, value)
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<WebSocketErrorRecord> {
    scope
        .get_slot::<WebSocketErrorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_close_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match record.close_code {
        Some(code) => result.set(v8::Integer::new_from_unsigned(scope, code as u32).into()),
        None => result.set(v8::null(scope).into()),
    }
}
fn get_reason(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.reason) {
        result.set(value.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<WebSocketErrorStore>() {
        store.constructor.remove(realm_id);
    }
}
