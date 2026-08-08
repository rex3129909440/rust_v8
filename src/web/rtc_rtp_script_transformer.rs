use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct TransformerRecord {
    options: v8::Global<v8::Value>,
    readable: v8::Global<v8::Object>,
    writable: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct RtcRtpScriptTransformerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TransformerRecord>,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcRtpScriptTransformerStore::default());
}

pub(crate) fn install_in_worker_realm(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "RTCRtpScriptTransformer", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<RtcRtpScriptTransformerStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCRtpScriptTransformer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_rtp_script_transformer_options_property::define(scope, prototype)?;
    super::rtc_rtp_script_transformer_readable_property::define(scope, prototype)?;
    super::rtc_rtp_script_transformer_writable_property::define(scope, prototype)?;
    super::rtc_rtp_script_transformer_generate_key_frame::define(scope, prototype)?;
    super::rtc_rtp_script_transformer_send_key_frame_request::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcRtpScriptTransformerStore>()
        .ok_or_else(|| "RTCRtpScriptTransformer state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    options: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create RTCRtpScriptTransformer".to_owned());
    }
    let readable = super::readable_stream::create_empty(scope)?;
    let writable = super::writable_stream::create_empty(scope)?;
    let record = TransformerRecord {
        options: v8::Global::new(scope, options),
        readable: v8::Global::new(scope, readable),
        writable: v8::Global::new(scope, writable),
    };
    let store = scope
        .get_slot_mut::<RtcRtpScriptTransformerStore>()
        .ok_or_else(|| "RTCRtpScriptTransformer state was not prepared".to_owned())?;
    store.instances.insert(object.get_identity_hash().get());
    store
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) enum ObjectProperty {
    Options,
    Readable,
    Writable,
}

pub(crate) fn get_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    property: ObjectProperty,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope
        .get_slot::<RtcRtpScriptTransformerStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match property {
        ObjectProperty::Options => result.set(v8::Local::new(scope, &record.options)),
        ObjectProperty::Readable => result.set(v8::Local::new(scope, &record.readable).into()),
        ObjectProperty::Writable => result.set(v8::Local::new(scope, &record.writable).into()),
    }
}

pub(crate) fn resolved_action(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let valid = scope
        .get_slot::<RtcRtpScriptTransformerStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains(&arguments.this().get_identity_hash().get())
        });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(promise) =
        super::writable_stream::resolved_promise(scope, v8::undefined(scope).into())
    {
        result.set(promise.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RtcRtpScriptTransformerStore>() {
        store.constructor.remove(realm_id);
    }
}
