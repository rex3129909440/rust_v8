use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DomExceptionStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, DomExceptionRecord>,
}

#[derive(Clone)]
struct DomExceptionRecord {
    name: String,
    message: String,
    code: i32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomExceptionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMException", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<DomExceptionStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }

    let constructor = crate::webidl::create_function(
        scope,
        "DOMException",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "code", get_code)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "message", get_message)?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;

    let error_key = crate::webidl::string(scope, "Error")?;
    let error_value = scope
        .get_current_context()
        .global(scope)
        .get(scope, error_key.into())
        .ok_or_else(|| "global Error is unavailable".to_owned())?;
    let error = v8::Local::<v8::Function>::try_from(error_value)
        .map_err(|_| "global Error is not a constructor".to_owned())?;
    let error_prototype = crate::webidl::prototype(scope, error)?;
    if crate::webidl::set_platform_prototype(scope, prototype, error_prototype.into()) != Some(true)
    {
        return Err("cannot set DOMException prototype inheritance".to_owned());
    }

    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomExceptionStore>()
        .ok_or_else(|| "DOMException state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "INDEX_SIZE_ERR", 1)?;
    crate::webidl::define_constant(scope, object, "DOMSTRING_SIZE_ERR", 2)?;
    crate::webidl::define_constant(scope, object, "HIERARCHY_REQUEST_ERR", 3)?;
    crate::webidl::define_constant(scope, object, "WRONG_DOCUMENT_ERR", 4)?;
    crate::webidl::define_constant(scope, object, "INVALID_CHARACTER_ERR", 5)?;
    crate::webidl::define_constant(scope, object, "NO_DATA_ALLOWED_ERR", 6)?;
    crate::webidl::define_constant(scope, object, "NO_MODIFICATION_ALLOWED_ERR", 7)?;
    crate::webidl::define_constant(scope, object, "NOT_FOUND_ERR", 8)?;
    crate::webidl::define_constant(scope, object, "NOT_SUPPORTED_ERR", 9)?;
    crate::webidl::define_constant(scope, object, "INUSE_ATTRIBUTE_ERR", 10)?;
    crate::webidl::define_constant(scope, object, "INVALID_STATE_ERR", 11)?;
    crate::webidl::define_constant(scope, object, "SYNTAX_ERR", 12)?;
    crate::webidl::define_constant(scope, object, "INVALID_MODIFICATION_ERR", 13)?;
    crate::webidl::define_constant(scope, object, "NAMESPACE_ERR", 14)?;
    crate::webidl::define_constant(scope, object, "INVALID_ACCESS_ERR", 15)?;
    crate::webidl::define_constant(scope, object, "VALIDATION_ERR", 16)?;
    crate::webidl::define_constant(scope, object, "TYPE_MISMATCH_ERR", 17)?;
    crate::webidl::define_constant(scope, object, "SECURITY_ERR", 18)?;
    crate::webidl::define_constant(scope, object, "NETWORK_ERR", 19)?;
    crate::webidl::define_constant(scope, object, "ABORT_ERR", 20)?;
    crate::webidl::define_constant(scope, object, "URL_MISMATCH_ERR", 21)?;
    crate::webidl::define_constant(scope, object, "QUOTA_EXCEEDED_ERR", 22)?;
    crate::webidl::define_constant(scope, object, "TIMEOUT_ERR", 23)?;
    crate::webidl::define_constant(scope, object, "INVALID_NODE_TYPE_ERR", 24)?;
    crate::webidl::define_constant(scope, object, "DATA_CLONE_ERR", 25)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'DOMException': Please use the 'new' operator",
        );
        return;
    }
    let message = if arguments.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let name = if arguments.get(1).is_undefined() {
        "Error".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    let code = legacy_code(&name);
    attach(scope, arguments.this(), name, message, code);
    result.set(arguments.this().into());
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: String,
    message: String,
    code: i32,
) {
    if let Some(store) = scope.get_slot_mut::<DomExceptionStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            DomExceptionRecord {
                name,
                message,
                code,
            },
        );
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    message: String,
    name: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let exception = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, exception, prototype.into()) != Some(true) {
        return Err("cannot create DOMException".to_owned());
    }
    let code = legacy_code(&name);
    attach(scope, exception, name, message, code);
    Ok(exception)
}

fn legacy_code(name: &str) -> i32 {
    match name {
        "IndexSizeError" => 1,
        "DOMStringSizeError" => 2,
        "HierarchyRequestError" => 3,
        "WrongDocumentError" => 4,
        "InvalidCharacterError" => 5,
        "NoDataAllowedError" => 6,
        "NoModificationAllowedError" => 7,
        "NotFoundError" => 8,
        "NotSupportedError" => 9,
        "InUseAttributeError" => 10,
        "InvalidStateError" => 11,
        "SyntaxError" => 12,
        "InvalidModificationError" => 13,
        "NamespaceError" => 14,
        "InvalidAccessError" => 15,
        "ValidationError" => 16,
        "TypeMismatchError" => 17,
        "SecurityError" => 18,
        "NetworkError" => 19,
        "AbortError" => 20,
        "URLMismatchError" => 21,
        "QuotaExceededError" => 22,
        "TimeoutError" => 23,
        "InvalidNodeTypeError" => 24,
        "DataCloneError" => 25,
        _ => 0,
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DomExceptionRecord> {
    scope
        .get_slot::<DomExceptionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.code).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.name) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.message) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DomExceptionStore>() {
        store.constructors.remove(&realm_id);
    }
}
