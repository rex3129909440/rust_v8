use std::collections::{HashMap, HashSet};

const FORMATS: &[&str] = &[
    "aztec",
    "code_128",
    "code_39",
    "code_93",
    "codabar",
    "data_matrix",
    "ean_13",
    "ean_8",
    "itf",
    "pdf417",
    "qr_code",
    "unknown",
    "upc_a",
    "upc_e",
];

#[derive(Default)]
pub(crate) struct BarcodeDetectorStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
    selected: HashMap<i32, Vec<String>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(BarcodeDetectorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "BarcodeDetector", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<BarcodeDetectorStore>()
        .and_then(|store| store.constructor.get(realm))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "BarcodeDetector",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "detect", 1, detect)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "getSupportedFormats",
        0,
        get_supported_formats,
    )?;
    super::android_api_support::set_tag(scope, prototype, "BarcodeDetector")?;
    let stored_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<BarcodeDetectorStore>()
        .ok_or_else(|| "BarcodeDetector state was not prepared".to_owned())?
        .constructor
        .insert(realm, stored_constructor);
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
            "Failed to construct 'BarcodeDetector': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    let mut selected = Vec::new();
    if let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0))
        && let Some(value) = super::android_api_support::property(scope, options, "formats")
        && !value.is_undefined()
    {
        let Ok(values) = v8::Local::<v8::Array>::try_from(value) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'BarcodeDetector': formats must be a sequence.",
            );
            return;
        };
        for index in 0..values.length() {
            let value = values
                .get_index(scope, index)
                .unwrap_or_else(|| v8::undefined(scope).into());
            let value = crate::webidl::value_to_string(scope, value);
            if !FORMATS.contains(&value.as_str()) {
                crate::webidl::throw_type_error(
                    scope,
                    &format!(
                        "Failed to construct 'BarcodeDetector': Failed to read the 'formats' property from 'BarcodeDetectorOptions': The provided value '{value}' is not a valid enum value of type BarcodeFormat."
                    ),
                );
                return;
            }
            selected.push(value);
        }
    }
    let id = arguments.this().get_identity_hash().get();
    let store = scope
        .get_slot_mut::<BarcodeDetectorStore>()
        .expect("BarcodeDetector state");
    store.instances.insert(id);
    store.selected.insert(id, selected);
    result.set(arguments.this().into());
}

fn get_supported_formats(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let values = v8::Array::new(scope, FORMATS.len() as i32);
    for (index, value) in FORMATS.iter().enumerate() {
        if let Some(value) = v8::String::new(scope, value) {
            let _ = values.set_index(scope, index as u32, value.into());
        }
    }
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, values.into()) {
        result.set(promise.into());
    }
}

fn detect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let valid = scope
        .get_slot::<BarcodeDetectorStore>()
        .expect("BarcodeDetector state")
        .instances
        .contains(&arguments.this().get_identity_hash().get());
    if !super::android_api_support::require_brand(scope, valid, "BarcodeDetector", "detect") {
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'detect' on 'BarcodeDetector': 1 argument required, but only 0 present.",
        );
        return;
    }
    let detections = v8::Array::new(scope, 0);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, detections.into()) {
        result.set(promise.into());
    }
}
