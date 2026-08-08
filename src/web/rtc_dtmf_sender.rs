use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcDtmfSenderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DtmfSenderRecord>,
}

#[derive(Clone)]
struct DtmfSenderRecord {
    on_tone_change: Option<v8::Global<v8::Value>>,
    can_insert_dtmf: bool,
    tone_buffer: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcDtmfSenderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCDTMFSender", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcDtmfSenderStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCDTMFSender",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ontonechange",
        get_on_tone_change,
        set_on_tone_change,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "canInsertDTMF",
        get_can_insert_dtmf,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "toneBuffer", get_tone_buffer)?;
    crate::webidl::define_method(scope, prototype, "insertDTMF", 1, insert_dtmf)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcDtmfSenderStore>()
        .ok_or_else(|| "RTCDTMFSender state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCDTMFSender': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    can_insert_dtmf: bool,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let sender = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, sender, prototype.into()) != Some(true) {
        return Err("cannot create RTCDTMFSender".to_owned());
    }
    super::event_target::attach(scope, sender);
    scope
        .get_slot_mut::<RtcDtmfSenderStore>()
        .ok_or_else(|| "RTCDTMFSender state was not prepared".to_owned())?
        .records
        .insert(
            sender.get_identity_hash().get(),
            DtmfSenderRecord {
                on_tone_change: None,
                can_insert_dtmf,
                tone_buffer: String::new(),
            },
        );
    Ok(sender)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DtmfSenderRecord> {
    scope
        .get_slot::<RtcDtmfSenderStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_on_tone_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.on_tone_change {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_on_tone_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, arguments.get(0)))
    };
    if let Some(record) = scope
        .get_slot_mut::<RtcDtmfSenderStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.on_tone_change = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_can_insert_dtmf(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.can_insert_dtmf).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_tone_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.tone_buffer) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn insert_dtmf(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'insertDTMF': 1 argument required",
        );
        return;
    }
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if !snapshot.can_insert_dtmf {
        match super::dom_exception::create(
            scope,
            "The canInsertDTMF attribute is false".to_owned(),
            "InvalidStateError".to_owned(),
        ) {
            Ok(exception) => {
                scope.throw_exception(exception.into());
            }
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    let tones = crate::webidl::value_to_string(scope, arguments.get(0)).to_ascii_uppercase();
    if !tones
        .chars()
        .all(|tone| matches!(tone, '0'..='9' | 'A'..='D' | '#' | '*' | ','))
    {
        if let Some(message) = v8::String::new(scope, "The tones argument contains an invalid tone")
        {
            scope.throw_exception(v8::Exception::syntax_error(scope, message));
        }
        return;
    }
    let duration = arguments.get(1).number_value(scope).unwrap_or(100.0);
    let inter_tone_gap = arguments.get(2).number_value(scope).unwrap_or(70.0);
    if !(40.0..=6000.0).contains(&duration) || inter_tone_gap < 30.0 {
        if let Some(message) = v8::String::new(scope, "DTMF timing is outside the valid range") {
            scope.throw_exception(v8::Exception::range_error(scope, message));
        }
        return;
    }
    if let Some(record) = scope
        .get_slot_mut::<RtcDtmfSenderStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.tone_buffer = tones;
    }
}
