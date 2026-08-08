use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct FeaturePolicyStore {
    constructor: crate::webidl::RealmConstructor,
    objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FeaturePolicyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FeaturePolicy", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FeaturePolicyStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FeaturePolicy",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "allowedFeatures", 0, allowed_features)?;
    crate::webidl::define_method(scope, prototype, "allowsFeature", 1, allows_feature)?;
    crate::webidl::define_method(scope, prototype, "features", 0, features)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getAllowlistForFeature",
        1,
        get_allowlist_for_feature,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FeaturePolicyStore>()
        .ok_or_else(|| "FeaturePolicy state was not prepared".to_owned())?
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
        return Err("cannot create FeaturePolicy".to_owned());
    }
    scope
        .get_slot_mut::<FeaturePolicyStore>()
        .ok_or_else(|| "FeaturePolicy state was not prepared".to_owned())?
        .objects
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn feature_names() -> &'static [&'static str] {
    &[
        "geolocation",
        "ch-ua-full-version-list",
        "cross-origin-isolated",
        "screen-wake-lock",
        "on-device-speech-recognition",
        "translator",
        "publickey-credentials-get",
        "shared-storage-select-url",
        "ch-ua-arch",
        "bluetooth",
        "compute-pressure",
        "ch-prefers-reduced-transparency",
        "deferred-fetch",
        "usb",
        "ch-save-data",
        "publickey-credentials-create",
        "shared-storage",
        "deferred-fetch-minimal",
        "run-ad-auction",
        "ch-downlink",
        "ch-ua-form-factors",
        "otp-credentials",
        "payment",
        "ch-ua",
        "ch-ua-model",
        "ch-ect",
        "autoplay",
        "camera",
        "language-detector",
        "private-state-token-issuance",
        "digital-credentials-get",
        "accelerometer",
        "ch-ua-platform-version",
        "idle-detection",
        "private-aggregation",
        "interest-cohort",
        "ch-viewport-height",
        "captured-surface-control",
        "local-fonts",
        "ch-ua-platform",
        "midi",
        "ch-ua-full-version",
        "xr-spatial-tracking",
        "clipboard-read",
        "gamepad",
        "display-capture",
        "keyboard-map",
        "join-ad-interest-group",
        "aria-notify",
        "local-network",
        "ch-ua-high-entropy-values",
        "ch-width",
        "ch-prefers-reduced-motion",
        "browsing-topics",
        "encrypted-media",
        "local-network-access",
        "gyroscope",
        "serial",
        "ch-rtt",
        "ch-ua-mobile",
        "window-management",
        "unload",
        "ch-dpr",
        "ch-prefers-color-scheme",
        "ch-ua-wow64",
        "attribution-reporting",
        "fullscreen",
        "identity-credentials-get",
        "private-state-token-redemption",
        "hid",
        "summarizer",
        "ch-ua-bitness",
        "storage-access",
        "sync-xhr",
        "ch-device-memory",
        "ch-viewport-width",
        "picture-in-picture",
        "loopback-network",
        "magnetometer",
        "clipboard-write",
        "microphone",
    ]
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<FeaturePolicyStore>()
        .is_some_and(|store| store.objects.contains(&object.get_identity_hash().get()))
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn string_array<'s>(scope: &mut v8::PinScope<'s, '_>, values: &[&str]) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = array.set_index(scope, index as u32, value.into());
        }
    }
    array
}

fn allowed_features(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if valid(scope, arguments.this()) {
        result.set(string_array(scope, feature_names()).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn allows_feature(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let wanted = crate::webidl::value_to_string(scope, arguments.get(0));
    result.set(
        v8::Boolean::new(
            scope,
            feature_names().iter().any(|feature| *feature == wanted),
        )
        .into(),
    );
}

fn features(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    allowed_features(scope, arguments, result);
}

fn get_allowlist_for_feature(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if valid(scope, arguments.this()) {
        result.set(v8::Array::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
