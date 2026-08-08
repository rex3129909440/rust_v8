use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SecurityPolicyViolationEventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone, Default)]
pub(crate) struct Record {
    pub(crate) document_uri: String,
    pub(crate) referrer: String,
    pub(crate) blocked_uri: String,
    pub(crate) violated_directive: String,
    pub(crate) effective_directive: String,
    pub(crate) original_policy: String,
    pub(crate) disposition: String,
    pub(crate) source_file: String,
    pub(crate) status_code: u16,
    pub(crate) line_number: u32,
    pub(crate) column_number: u32,
    pub(crate) sample: String,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SecurityPolicyViolationEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SecurityPolicyViolationEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SecurityPolicyViolationEventStore>()
        .and_then(|s| s.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "SecurityPolicyViolationEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::security_policy_violation_event_document_uri_property::define(scope, p)?;
    super::security_policy_violation_event_referrer_property::define(scope, p)?;
    super::security_policy_violation_event_blocked_uri_property::define(scope, p)?;
    super::security_policy_violation_event_violated_directive_property::define(scope, p)?;
    super::security_policy_violation_event_effective_directive_property::define(scope, p)?;
    super::security_policy_violation_event_original_policy_property::define(scope, p)?;
    super::security_policy_violation_event_disposition_property::define(scope, p)?;
    super::security_policy_violation_event_source_file_property::define(scope, p)?;
    super::security_policy_violation_event_status_code_property::define(scope, p)?;
    super::security_policy_violation_event_line_number_property::define(scope, p)?;
    super::security_policy_violation_event_column_number_property::define(scope, p)?;
    super::security_policy_violation_event_sample_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let event = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, event)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<SecurityPolicyViolationEventStore>()
        .ok_or_else(|| "SecurityPolicyViolationEvent state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn string(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> String {
    v8::String::new(scope, n)
        .and_then(|k| o.get(scope, k.into()))
        .filter(|v| !v.is_undefined())
        .map(|v| crate::webidl::value_to_string(scope, v))
        .unwrap_or_default()
}
pub(crate) fn uint(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str) -> u32 {
    v8::String::new(scope, n)
        .and_then(|k| o.get(scope, k.into()))
        .and_then(|v| v.uint32_value(scope))
        .unwrap_or(0)
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "SecurityPolicyViolationEvent requires an event type",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, a.get(0));
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let record = init
        .map(|o| Record {
            document_uri: string(scope, o, "documentURI"),
            referrer: string(scope, o, "referrer"),
            blocked_uri: string(scope, o, "blockedURI"),
            violated_directive: string(scope, o, "violatedDirective"),
            effective_directive: string(scope, o, "effectiveDirective"),
            original_policy: string(scope, o, "originalPolicy"),
            disposition: string(scope, o, "disposition"),
            source_file: string(scope, o, "sourceFile"),
            status_code: uint(scope, o, "statusCode") as u16,
            line_number: uint(scope, o, "lineNumber"),
            column_number: uint(scope, o, "columnNumber"),
            sample: string(scope, o, "sample"),
        })
        .unwrap_or_default();
    let bubbles = init.is_some_and(|o| super::event::boolean_property(scope, o, "bubbles"));
    let cancelable = init.is_some_and(|o| super::event::boolean_property(scope, o, "cancelable"));
    let composed = init.is_some_and(|o| super::event::boolean_property(scope, o, "composed"));
    super::event::attach(scope, a.this(), event_type, bubbles, cancelable, composed);
    scope
        .get_slot_mut::<SecurityPolicyViolationEventStore>()
        .expect("SecurityPolicyViolationEvent state")
        .records
        .insert(a.this().get_identity_hash().get(), record);
    r.set(a.this().into())
}
pub(crate) fn record(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    scope
        .get_slot::<SecurityPolicyViolationEventStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> &str,
) {
    let Some(v) = record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(s) = v8::String::new(scope, select(&v)) {
        r.set(s.into())
    }
}
pub(crate) fn return_uint(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&Record) -> u32,
) {
    if let Some(v) = record(scope, a.this()) {
        r.set(v8::Integer::new_from_unsigned(scope, select(&v)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_document_uri(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.document_uri)
}
pub(crate) fn get_referrer(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.referrer)
}
pub(crate) fn get_blocked_uri(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.blocked_uri)
}
pub(crate) fn get_violated_directive(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.violated_directive)
}
pub(crate) fn get_effective_directive(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.effective_directive)
}
pub(crate) fn get_original_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.original_policy)
}
pub(crate) fn get_disposition(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.disposition)
}
pub(crate) fn get_source_file(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.source_file)
}
pub(crate) fn get_sample(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.sample)
}
pub(crate) fn get_status_code(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_uint(s, a, r, |v| v.status_code as u32)
}
pub(crate) fn get_line_number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_uint(s, a, r, |v| v.line_number)
}
pub(crate) fn get_column_number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_uint(s, a, r, |v| v.column_number)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SecurityPolicyViolationEventStore>() {
        store.constructors.remove(&realm_id);
    }
}
