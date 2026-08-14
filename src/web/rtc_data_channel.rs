use std::collections::HashMap;

#[derive(Clone)]
struct RtcDataChannelRecord {
    label: String,
    ordered: bool,
    max_packet_life_time: Option<u16>,
    max_retransmits: Option<u16>,
    protocol: String,
    negotiated: bool,
    id: Option<u16>,
    ready_state: String,
    buffered_amount: u64,
    buffered_amount_low_threshold: u64,
    onopen: Option<v8::Global<v8::Value>>,
    onbufferedamountlow: Option<v8::Global<v8::Value>>,
    onerror: Option<v8::Global<v8::Value>>,
    onclosing: Option<v8::Global<v8::Value>>,
    onclose: Option<v8::Global<v8::Value>>,
    onmessage: Option<v8::Global<v8::Value>>,
    binary_type: String,
}

#[derive(Default)]
pub(crate) struct RtcDataChannelStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RtcDataChannelRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcDataChannelStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCDataChannel", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<RtcDataChannelStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCDataChannel",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "label", get_label)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ordered", get_ordered)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxPacketLifeTime",
        get_max_packet_life_time,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "maxRetransmits",
        get_max_retransmits,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "protocol", get_protocol)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "negotiated", get_negotiated)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "readyState", get_ready_state)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "bufferedAmount",
        get_buffered_amount,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "bufferedAmountLowThreshold",
        get_buffered_amount_low_threshold,
        set_buffered_amount_low_threshold,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onopen", get_onopen, set_onopen)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onbufferedamountlow",
        get_onbufferedamountlow,
        set_onbufferedamountlow,
    )?;
    crate::webidl::define_accessor(scope, prototype, "onerror", get_onerror, set_onerror)?;
    crate::webidl::define_accessor(scope, prototype, "onclosing", get_onclosing, set_onclosing)?;
    crate::webidl::define_accessor(scope, prototype, "onclose", get_onclose, set_onclose)?;
    crate::webidl::define_accessor(scope, prototype, "onmessage", get_onmessage, set_onmessage)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "binaryType",
        get_binary_type,
        set_binary_type,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reliable", get_reliable)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "send", 1, send)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcDataChannelStore>()
        .ok_or_else(|| "RTCDataChannel state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    label: String,
    options: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let mut ordered = true;
    let mut max_packet_life_time = None;
    let mut max_retransmits = None;
    let mut protocol = String::new();
    let mut negotiated = false;
    let mut id = None;
    if let Ok(init) = v8::Local::<v8::Object>::try_from(options) {
        ordered = boolean_property(scope, init, "ordered").unwrap_or(true);
        max_packet_life_time = integer_property(scope, init, "maxPacketLifeTime");
        max_retransmits = integer_property(scope, init, "maxRetransmits");
        protocol = string_property(scope, init, "protocol").unwrap_or_default();
        negotiated = boolean_property(scope, init, "negotiated").unwrap_or(false);
        id = integer_property(scope, init, "id");
    }
    if max_packet_life_time.is_some() && max_retransmits.is_some() {
        return Err(
            "Failed to execute 'createDataChannel': both maxPacketLifeTime and maxRetransmits were specified."
                .to_owned(),
        );
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create RTCDataChannel".to_owned());
    }
    super::event_target::attach(scope, object);
    let record = RtcDataChannelRecord {
        label,
        ordered,
        max_packet_life_time,
        max_retransmits,
        protocol,
        negotiated,
        id,
        ready_state: "connecting".to_owned(),
        buffered_amount: 0,
        buffered_amount_low_threshold: 0,
        onopen: None,
        onbufferedamountlow: None,
        onerror: None,
        onclosing: None,
        onclose: None,
        onmessage: None,
        binary_type: "arraybuffer".to_owned(),
    };
    scope
        .get_slot_mut::<RtcDataChannelStore>()
        .ok_or_else(|| "RTCDataChannel state is unavailable".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn boolean_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<bool> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then(|| value.boolean_value(scope))
}

fn integer_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u16> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() || value.is_null() {
        None
    } else {
        value
            .integer_value(scope)
            .map(|number| number.clamp(0, u16::MAX as i64) as u16)
    }
}

fn string_property(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then(|| crate::webidl::value_to_string(scope, value))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCDataChannel': Illegal constructor",
    )
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RtcDataChannelRecord> {
    scope
        .get_slot::<RtcDataChannelStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut RtcDataChannelRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<RtcDataChannelStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return false;
    };
    change(record);
    true
}

fn text_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&RtcDataChannelRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

fn get_label(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |x| &x.label)
}
fn get_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |x| &x.protocol)
}
fn get_ready_state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |x| &x.ready_state)
}
fn get_binary_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text_get(s, a, r, |x| &x.binary_type)
}

fn get_ordered(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.ordered).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_negotiated(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.negotiated).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_reliable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(
            v8::Boolean::new(
                scope,
                record.max_packet_life_time.is_none() && record.max_retransmits.is_none(),
            )
            .into(),
        );
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn optional_integer_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&RtcDataChannelRecord) -> Option<u16>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match select(&record) {
        Some(value) => result.set(v8::Integer::new_from_unsigned(scope, value as u32).into()),
        None => result.set(v8::null(scope).into()),
    }
}
fn get_max_packet_life_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    optional_integer_get(s, a, r, |x| x.max_packet_life_time)
}
fn get_max_retransmits(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    optional_integer_get(s, a, r, |x| x.max_retransmits)
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    optional_integer_get(s, a, r, |x| x.id)
}

fn get_buffered_amount(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.buffered_amount as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_buffered_amount_low_threshold(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.buffered_amount_low_threshold as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn set_buffered_amount_low_threshold(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).integer_value(scope).unwrap_or(0).max(0) as u64;
    update(scope, arguments.this(), |record| {
        record.buffered_amount_low_threshold = value
    });
}

fn handler_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
    select: impl FnOnce(&RtcDataChannelRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, select(&record), result);
}
fn handler_set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    assign: impl FnOnce(&mut RtcDataChannelRecord, Option<v8::Global<v8::Value>>),
) {
    let value = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    update(scope, arguments.this(), |record| assign(record, value));
}

fn get_onopen(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, |x| x.onopen.clone())
}
fn set_onopen(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, |x, value| x.onopen = value)
}
fn get_onbufferedamountlow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, |x| x.onbufferedamountlow.clone())
}
fn set_onbufferedamountlow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, |x, value| x.onbufferedamountlow = value)
}
fn get_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, |x| x.onerror.clone())
}
fn set_onerror(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, |x, value| x.onerror = value)
}
fn get_onclosing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, |x| x.onclosing.clone())
}
fn set_onclosing(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, |x, value| x.onclosing = value)
}
fn get_onclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, |x| x.onclose.clone())
}
fn set_onclose(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, |x, value| x.onclose = value)
}
fn get_onmessage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler_get(s, a, r, |x| x.onmessage.clone())
}
fn set_onmessage(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    handler_set(s, a, |x, value| x.onmessage = value)
}

fn set_binary_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "blob" && value != "arraybuffer" {
        crate::webidl::throw_type_error(
            scope,
            "The provided value is not a valid enum value of type BinaryType.",
        );
        return;
    }
    update(scope, arguments.this(), |record| record.binary_type = value);
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current.ready_state == "closed" || current.ready_state == "closing" {
        return;
    }
    update(scope, arguments.this(), |record| {
        record.ready_state = "closing".to_owned()
    });
    if let Ok(event) = super::event::create(scope, "closing") {
        super::event_target::dispatch(scope, arguments.this(), event);
    }
}

fn send(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.ready_state != "open" {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "RTCDataChannel.readyState is not 'open'".to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let size = crate::webidl::value_to_string(scope, arguments.get(0))
        .as_bytes()
        .len() as u64;
    update(scope, arguments.this(), |record| {
        record.buffered_amount = record.buffered_amount.saturating_add(size)
    });
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RtcDataChannelStore>() {
        store.constructor.remove(realm_id);
    }
}
