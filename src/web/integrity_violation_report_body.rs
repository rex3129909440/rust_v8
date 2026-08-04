use std::collections::HashMap;

#[derive(Clone)]
struct IntegrityViolationReportBodyRecord {
    document_url: String,
    blocked_url: String,
    destination: String,
    report_only: bool,
}

#[derive(Default)]
pub(crate) struct IntegrityViolationReportBodyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IntegrityViolationReportBodyRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(IntegrityViolationReportBodyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "IntegrityViolationReportBody", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<IntegrityViolationReportBodyStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "IntegrityViolationReportBody",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "documentURL", get_document_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "blockedURL", get_blocked_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "destination", get_destination)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reportOnly", get_report_only)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::report_body::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<IntegrityViolationReportBodyStore>()
        .ok_or_else(|| "IntegrityViolationReportBody state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    document_url: String,
    blocked_url: String,
    destination: String,
    report_only: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let body = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, body, prototype.into()) != Some(true) {
        return Err("cannot create IntegrityViolationReportBody".to_owned());
    }
    scope
        .get_slot_mut::<IntegrityViolationReportBodyStore>()
        .ok_or_else(|| "IntegrityViolationReportBody state was not prepared".to_owned())?
        .records
        .insert(
            body.get_identity_hash().get(),
            IntegrityViolationReportBodyRecord {
                document_url,
                blocked_url,
                destination,
                report_only,
            },
        );
    Ok(body)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<IntegrityViolationReportBodyRecord> {
    scope
        .get_slot::<IntegrityViolationReportBodyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'IntegrityViolationReportBody': Illegal constructor",
    )
}

fn get_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&IntegrityViolationReportBodyRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into())
    }
}
fn get_document_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_text(s, a, r, |x| &x.document_url)
}
fn get_blocked_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_text(s, a, r, |x| &x.blocked_url)
}
fn get_destination(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_text(s, a, r, |x| &x.destination)
}
fn get_report_only(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Boolean::new(scope, record.report_only).into())
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
    let value = v8::Object::new(scope);
    let _ = define_text(scope, value, "documentURL", &record.document_url);
    let _ = define_text(scope, value, "blockedURL", &record.blocked_url);
    let _ = define_text(scope, value, "destination", &record.destination);
    let Ok(key) = crate::webidl::string(scope, "reportOnly") else {
        return;
    };
    let report = v8::Boolean::new(scope, record.report_only);
    let _ = value.define_own_property(
        scope,
        key.into(),
        report.into(),
        v8::PropertyAttribute::NONE,
    );
    result.set(value.into())
}
fn define_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    let value = crate::webidl::string(scope, value)?;
    if object.define_own_property(scope, key.into(), value.into(), v8::PropertyAttribute::NONE)
        == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define integrity report field".to_owned())
    }
}
