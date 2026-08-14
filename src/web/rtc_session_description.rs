use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcSessionDescriptionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DescriptionRecord>,
}

#[derive(Clone)]
struct DescriptionRecord {
    description_type: Option<String>,
    sdp: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcSessionDescriptionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCSessionDescription", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcSessionDescriptionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCSessionDescription",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "type", get_type, set_type)?;
    crate::webidl::define_accessor(scope, prototype, "sdp", get_sdp, set_sdp)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcSessionDescriptionStore>()
        .ok_or_else(|| "RTCSessionDescription state was not prepared".to_owned())?
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
            "Failed to construct 'RTCSessionDescription': use new",
        );
        return;
    }
    let mut description_type = None;
    let mut sdp = String::new();
    if let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) {
        if let Some(value) = property(scope, init, "type") {
            if !value.is_undefined() {
                let candidate = crate::webidl::value_to_string(scope, value);
                if !valid_type(&candidate) {
                    crate::webidl::throw_type_error(
                        scope,
                        "The provided value is not a valid RTCSdpType",
                    );
                    return;
                }
                description_type = Some(candidate);
            }
        }
        if let Some(value) = property(scope, init, "sdp") {
            if !value.is_undefined() {
                sdp = crate::webidl::value_to_string(scope, value);
            }
        }
    }
    attach(scope, arguments.this(), description_type, sdp);
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    description_type: Option<String>,
    sdp: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create RTCSessionDescription".to_owned());
    }
    attach(scope, object, description_type, sdp);
    Ok(object)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    description_type: Option<String>,
    sdp: String,
) {
    scope
        .get_slot_mut::<RtcSessionDescriptionStore>()
        .expect("RTCSessionDescription state")
        .records
        .insert(
            object.get_identity_hash().get(),
            DescriptionRecord {
                description_type,
                sdp,
            },
        );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DescriptionRecord> {
    scope
        .get_slot::<RtcSessionDescriptionStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut DescriptionRecord),
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<RtcSessionDescriptionStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
        true
    } else {
        false
    }
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.description_type {
        if let Some(value) = v8::String::new(scope, &value) {
            result.set(value.into());
        }
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !valid_type(&value) {
        crate::webidl::throw_type_error(scope, "The provided value is not a valid RTCSdpType");
        return;
    }
    if !update(scope, arguments.this(), |record| {
        record.description_type = Some(value)
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_sdp(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.sdp) {
        result.set(value.into());
    }
}

fn set_sdp(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if !update(scope, arguments.this(), |record| record.sdp = value) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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
    let type_value = if let Some(value) = record.description_type {
        v8::String::new(scope, &value)
            .map(Into::into)
            .unwrap_or_else(|| v8::null(scope).into())
    } else {
        v8::null(scope).into()
    };
    define_data(scope, object, "type", type_value);
    let sdp = v8::String::new(scope, &record.sdp)
        .map(Into::into)
        .unwrap_or_else(|| v8::undefined(scope).into());
    define_data(scope, object, "sdp", sdp);
    result.set(object.into());
}

fn valid_type(value: &str) -> bool {
    matches!(value, "offer" | "pranswer" | "answer" | "rollback")
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}
