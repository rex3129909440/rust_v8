use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcErrorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RtcErrorRecord>,
}

#[derive(Clone)]
struct RtcErrorRecord {
    error_detail: String,
    sdp_line_number: Option<f64>,
    http_request_status_code: Option<f64>,
    sctp_cause_code: Option<f64>,
    received_alert: Option<f64>,
    sent_alert: Option<f64>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcErrorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCError", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcErrorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCError",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "errorDetail", get_error_detail)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "sdpLineNumber",
        get_sdp_line_number,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "httpRequestStatusCode",
        get_http_request_status_code,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "sctpCauseCode",
        get_sctp_cause_code,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "receivedAlert", get_received_alert)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sentAlert", get_sent_alert)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::dom_exception::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcErrorStore>()
        .ok_or_else(|| "RTCError state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCError': 1 argument required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCError': The provided value is not of type 'RTCErrorInit'.",
        );
        return;
    };
    let Some(detail_value) = property(scope, init, "errorDetail") else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCError': Failed to read the 'errorDetail' property from 'RTCErrorInit': Required member is undefined.",
        );
        return;
    };
    if detail_value.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCError': Failed to read the 'errorDetail' property from 'RTCErrorInit': Required member is undefined.",
        );
        return;
    }
    let error_detail = crate::webidl::value_to_string(scope, detail_value);
    if !valid_error_detail(&error_detail) {
        crate::webidl::throw_type_error(scope, "The provided errorDetail is not valid");
        return;
    }
    let message = if arguments.get(1).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(1))
    };
    super::dom_exception::attach(
        scope,
        arguments.this(),
        "OperationError".to_owned(),
        message,
        0,
    );
    let record = RtcErrorRecord {
        error_detail,
        sdp_line_number: optional_number(scope, init, "sdpLineNumber"),
        http_request_status_code: optional_number(scope, init, "httpRequestStatusCode"),
        sctp_cause_code: optional_number(scope, init, "sctpCauseCode"),
        received_alert: optional_number(scope, init, "receivedAlert"),
        sent_alert: optional_number(scope, init, "sentAlert"),
    };
    scope
        .get_slot_mut::<RtcErrorStore>()
        .expect("RTCError state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn valid_error_detail(value: &str) -> bool {
    matches!(
        value,
        "data-channel-failure"
            | "dtls-failure"
            | "fingerprint-failure"
            | "hardware-encoder-error"
            | "hardware-encoder-not-available"
            | "sctp-failure"
            | "sdp-syntax-error"
    )
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn optional_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    let value = property(scope, object, name)?;
    if value.is_null_or_undefined() {
        None
    } else {
        value.number_value(scope)
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RtcErrorRecord> {
    scope
        .get_slot::<RtcErrorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

fn get_error_detail(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.error_detail) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_optional_number(
    scope: &mut v8::PinScope<'_, '_>,
    value: Option<f64>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value {
        result.set(v8::Number::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn get_sdp_line_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_number(scope, record.sdp_line_number, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_http_request_status_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_number(scope, record.http_request_status_code, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_sctp_cause_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_number(scope, record.sctp_cause_code, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_received_alert(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_number(scope, record.received_alert, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_sent_alert(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_optional_number(scope, record.sent_alert, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
