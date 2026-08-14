#[derive(Default)]
pub(crate) struct MediaCapabilitiesStore {
    constructor: crate::webidl::RealmConstructor,
    identities: std::collections::HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaCapabilitiesStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaCapabilities", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<MediaCapabilitiesStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaCapabilities",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "decodingInfo", 1, decoding_info)?;
    crate::webidl::define_method(scope, prototype, "encodingInfo", 1, encoding_info)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaCapabilitiesStore>()
        .ok_or_else(|| "MediaCapabilities state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaCapabilities".to_owned());
    }
    scope
        .get_slot_mut::<MediaCapabilitiesStore>()
        .ok_or_else(|| "MediaCapabilities state was not prepared".to_owned())?
        .identities
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<MediaCapabilitiesStore>()
        .is_some_and(|store| store.identities.contains(&object.get_identity_hash().get()))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'MediaCapabilities': Illegal constructor",
    );
}

fn decoding_info(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !is_instance(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "MediaCapabilities",
            "decodingInfo",
            result,
        );
        return;
    }
    let Some(configuration) = configuration(scope, arguments.get(0), "MediaDecodingConfiguration")
    else {
        return;
    };
    if configuration != "file" && configuration != "media-source" && configuration != "webrtc" {
        crate::webidl::throw_type_error(
            scope,
            "The provided value is not a valid MediaDecodingType",
        );
        return;
    }
    let requested = requested_content_types(scope, arguments.get(0));
    let configured = &crate::fingerprint::edge(scope).media;
    let supported = all_types_match(&configured.decoding_supported_types, &requested);
    let smooth = supported && all_types_match(&configured.decoding_smooth_types, &requested);
    let power_efficient =
        supported && all_types_match(&configured.decoding_power_efficient_types, &requested);
    let output = information(scope, power_efficient, smooth, supported);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, output.into()) {
        result.set(promise.into());
    }
}

fn encoding_info(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !is_instance(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "MediaCapabilities",
            "encodingInfo",
            result,
        );
        return;
    }
    let Some(configuration) = configuration(scope, arguments.get(0), "MediaEncodingConfiguration")
    else {
        return;
    };
    if configuration != "webrtc" {
        crate::webidl::throw_type_error(
            scope,
            "The provided value is not a valid enum value of type MediaEncodingType",
        );
        return;
    }
    let requested = requested_content_types(scope, arguments.get(0));
    let configured = &crate::fingerprint::edge(scope).media;
    let supported = all_types_match(&configured.encoding_supported_types, &requested);
    let smooth = supported && all_types_match(&configured.encoding_smooth_types, &requested);
    let power_efficient =
        supported && all_types_match(&configured.encoding_power_efficient_types, &requested);
    let output = information(scope, power_efficient, smooth, supported);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, output.into()) {
        result.set(promise.into());
    }
}

fn configuration(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    dictionary_name: &str,
) -> Option<String> {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "The configuration must be an object");
        return None;
    };
    let Some(key) = v8::String::new(scope, "type") else {
        return None;
    };
    let Some(value) = object.get(scope, key.into()) else {
        return None;
    };
    if value.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            &format!(
                "Failed to read the 'type' property from '{dictionary_name}': Required member is undefined."
            ),
        );
        return None;
    }
    Some(crate::webidl::value_to_string(scope, value))
}

fn information<'s>(
    scope: &v8::PinScope<'s, '_>,
    power_efficient: bool,
    smooth: bool,
    supported: bool,
) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    define(
        scope,
        object,
        "powerEfficient",
        v8::Boolean::new(scope, power_efficient).into(),
    );
    define(
        scope,
        object,
        "smooth",
        v8::Boolean::new(scope, smooth).into(),
    );
    define(
        scope,
        object,
        "supported",
        v8::Boolean::new(scope, supported).into(),
    );
    define(scope, object, "keySystemAccess", v8::null(scope).into());
    object
}

fn requested_content_types(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Vec<String> {
    let Ok(configuration) = v8::Local::<v8::Object>::try_from(value) else {
        return Vec::new();
    };
    let mut output = Vec::new();
    for member in ["audio", "video"] {
        let Some(key) = v8::String::new(scope, member) else {
            continue;
        };
        let Some(value) = configuration.get(scope, key.into()) else {
            continue;
        };
        let Ok(section) = v8::Local::<v8::Object>::try_from(value) else {
            continue;
        };
        let Some(content_type_key) = v8::String::new(scope, "contentType") else {
            continue;
        };
        let Some(content_type) = section.get(scope, content_type_key.into()) else {
            continue;
        };
        if !content_type.is_undefined() {
            output.push(crate::webidl::value_to_string(scope, content_type));
        }
    }
    output
}

fn all_types_match(patterns: &[String], requested: &[String]) -> bool {
    !patterns.is_empty()
        && (requested.is_empty()
            || requested.iter().all(|media_type| {
                crate::fingerprint_environment::media_type_matches(patterns, media_type)
            }))
}

fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<MediaCapabilitiesStore>() {
        store.constructor.remove(realm_id);
    }
}
