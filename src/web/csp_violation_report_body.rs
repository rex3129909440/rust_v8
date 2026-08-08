use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct CspViolationRecord {
    pub document_url: String,
    pub referrer: String,
    pub blocked_url: String,
    pub effective_directive: String,
    pub original_policy: String,
    pub source_file: String,
    pub sample: String,
    pub disposition: String,
    pub status_code: u32,
    pub line_number: u32,
    pub column_number: u32,
}

#[derive(Default)]
pub(crate) struct CspViolationReportBodyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CspViolationRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CspViolationReportBodyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSPViolationReportBody", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CspViolationReportBodyStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSPViolationReportBody",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "documentURL", get_document_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "referrer", get_referrer)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "blockedURL", get_blocked_url)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "effectiveDirective",
        get_effective_directive,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "originalPolicy",
        get_original_policy,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sourceFile", get_source_file)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sample", get_sample)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "disposition", get_disposition)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "statusCode", get_status_code)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "lineNumber", get_line_number)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "columnNumber", get_column_number)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::report_body::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CspViolationReportBodyStore>()
        .ok_or_else(|| "CSPViolationReportBody state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: CspViolationRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSPViolationReportBody".to_owned());
    }
    scope
        .get_slot_mut::<CspViolationReportBodyStore>()
        .ok_or_else(|| "CSPViolationReportBody state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CspViolationRecord> {
    scope
        .get_slot::<CspViolationReportBodyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<String>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value
        && let Some(value) = v8::String::new(scope, &value)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_document_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.document_url);
    return_string(s, value, r);
}
fn get_referrer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.referrer);
    return_string(s, value, r);
}
fn get_blocked_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.blocked_url);
    return_string(s, value, r);
}
fn get_effective_directive(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.effective_directive);
    return_string(s, value, r);
}
fn get_original_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.original_policy);
    return_string(s, value, r);
}
fn get_source_file(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.source_file);
    return_string(s, value, r);
}
fn get_sample(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.sample);
    return_string(s, value, r);
}
fn get_disposition(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.disposition);
    return_string(s, value, r);
}

fn return_u32(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<u32>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Integer::new_from_unsigned(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_status_code(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.status_code);
    return_u32(s, value, r);
}
fn get_line_number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.line_number);
    return_u32(s, value, r);
}
fn get_column_number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = record(s, a.this()).map(|v| v.column_number);
    return_u32(s, value, r);
}

fn define_string(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let (Some(key), Some(value)) = (v8::String::new(scope, name), v8::String::new(scope, value))
    {
        let _ = object.set(scope, key.into(), value.into());
    }
}

fn define_u32(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: u32,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let number = v8::Integer::new_from_unsigned(scope, value);
        let _ = object.set(scope, key.into(), number.into());
    }
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    define_string(scope, object, "documentURL", &record.document_url);
    define_string(scope, object, "referrer", &record.referrer);
    define_string(scope, object, "blockedURL", &record.blocked_url);
    define_string(
        scope,
        object,
        "effectiveDirective",
        &record.effective_directive,
    );
    define_string(scope, object, "originalPolicy", &record.original_policy);
    define_string(scope, object, "sourceFile", &record.source_file);
    define_string(scope, object, "sample", &record.sample);
    define_string(scope, object, "disposition", &record.disposition);
    define_u32(scope, object, "statusCode", record.status_code);
    define_u32(scope, object, "lineNumber", record.line_number);
    define_u32(scope, object, "columnNumber", record.column_number);
    result.set(object.into());
}
