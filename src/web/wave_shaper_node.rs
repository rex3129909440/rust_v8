use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct WaveShaperNodeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, WaveShaperRecord>,
}

#[derive(Clone)]
struct WaveShaperRecord {
    curve: Option<v8::Global<v8::Object>>,
    oversample: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WaveShaperNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WaveShaperNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<WaveShaperNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "WaveShaperNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "curve", get_curve, set_curve)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oversample",
        get_oversample,
        set_oversample,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let audio_node = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, audio_node)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WaveShaperNodeStore>()
        .ok_or_else(|| "WaveShaperNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    constructor
        .new_instance(scope, &[context.into()])
        .ok_or_else(|| "cannot create WaveShaperNode".to_owned())
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WaveShaperNode': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WaveShaperNode': 1 argument required",
        );
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WaveShaperNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WaveShaperNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let curve = options.and_then(|options| object_property(scope, options, "curve"));
    let oversample = options
        .and_then(|options| string_property(scope, options, "oversample"))
        .unwrap_or_else(|| "none".to_owned());
    if oversample != "none" && oversample != "2x" && oversample != "4x" {
        crate::webidl::throw_type_error(scope, "Invalid WaveShaperNode oversample value");
        return;
    }
    let object = arguments.this();
    super::audio_node::attach(scope, object, Some(context), 1, 1);
    let curve = curve.map(|curve| v8::Global::new(scope, curve));
    if let Some(store) = scope.get_slot_mut::<WaveShaperNodeStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            WaveShaperRecord { curve, oversample },
        );
    }
    result.set(object.into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<WaveShaperRecord> {
    scope
        .get_slot::<WaveShaperNodeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut WaveShaperRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<WaveShaperNodeStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_curve(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(curve) = record.curve {
        result.set(v8::Local::new(scope, &curve).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_curve(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let curve = if arguments.get(0).is_null() {
        None
    } else {
        let Ok(curve) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "curve must be a Float32Array or null");
            return;
        };
        Some(v8::Global::new(scope, curve))
    };
    update(scope, arguments.this(), |record| record.curve = curve);
}

fn get_oversample(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.oversample) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_oversample(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "none" && value != "2x" && value != "4x" {
        crate::webidl::throw_type_error(scope, "Invalid WaveShaperNode oversample value");
        return;
    }
    update(scope, arguments.this(), |record| record.oversample = value);
}

fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() || value.is_null() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(value).ok()
    }
}

fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then(|| crate::webidl::value_to_string(scope, value))
}

pub(crate) fn shape(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    input: f32,
) -> Option<f32> {
    let record = record(scope, object)?;
    let Some(curve) = record.curve else {
        return Some(input);
    };
    let curve = v8::Local::new(scope, &curve);
    let length_key = v8::String::new(scope, "length")?;
    let length = curve
        .get(scope, length_key.into())?
        .uint32_value(scope)
        .unwrap_or(0);
    if length < 2 {
        return Some(input);
    }
    let position = f64::from(input.clamp(-1.0, 1.0) + 1.0) * 0.5 * f64::from(length - 1);
    let lower = position.floor() as u32;
    let upper = (lower + 1).min(length - 1);
    let amount = position - f64::from(lower);
    let lower_value = curve
        .get_index(scope, lower)
        .and_then(|value| value.number_value(scope))
        .unwrap_or(0.0);
    let upper_value = curve
        .get_index(scope, upper)
        .and_then(|value| value.number_value(scope))
        .unwrap_or(lower_value);
    Some((lower_value + (upper_value - lower_value) * amount) as f32)
}
