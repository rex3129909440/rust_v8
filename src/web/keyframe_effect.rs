use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct KeyframeEffectStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, KeyframeRecord>,
}

#[derive(Clone)]
struct KeyframeRecord {
    target: Option<v8::Global<v8::Object>>,
    pseudo_element: Option<String>,
    composite: String,
    keyframes: v8::Global<v8::Array>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(KeyframeEffectStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "KeyframeEffect", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<KeyframeEffectStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "KeyframeEffect",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "target", get_target, set_target)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "pseudoElement",
        get_pseudo_element,
        set_pseudo_element,
    )?;
    crate::webidl::define_accessor(scope, prototype, "composite", get_composite, set_composite)?;
    crate::webidl::define_method(scope, prototype, "getKeyframes", 0, get_keyframes)?;
    crate::webidl::define_method(scope, prototype, "setKeyframes", 1, set_keyframes)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::animation_effect::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<KeyframeEffectStore>()
        .ok_or_else(|| "KeyframeEffect state was not prepared".to_owned())?
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
            "Failed to construct 'KeyframeEffect': 1 argument required, but only 0 present.",
        );
        return;
    }
    let target = object_or_null(arguments.get(0));
    if let Some(target) = target
        && super::element::record(scope, target).is_none()
    {
        crate::webidl::throw_type_error(scope, "target is not an Element");
        return;
    }
    let keyframes = normalize_keyframes(scope, arguments.get(1));
    super::animation_effect::attach(scope, arguments.this(), Some(arguments.get(2)));
    let target = target.map(|target| v8::Global::new(scope, target));
    let keyframes = v8::Global::new(scope, keyframes);
    scope
        .get_slot_mut::<KeyframeEffectStore>()
        .expect("KeyframeEffect state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            KeyframeRecord {
                target,
                pseudo_element: None,
                composite: "replace".to_owned(),
                keyframes,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<KeyframeRecord> {
    scope
        .get_slot::<KeyframeEffectStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn target(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    record(scope, object)?.target
}

fn get_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(target) = record.target {
        result.set(v8::Local::new(scope, &target).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let target = object_or_null(arguments.get(0));
    if let Some(target) = target
        && super::element::record(scope, target).is_none()
    {
        crate::webidl::throw_type_error(scope, "target is not an Element");
        return;
    }
    let target = target.map(|target| v8::Global::new(scope, target));
    update(scope, arguments.this(), |record| record.target = target);
}

fn get_pseudo_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.pseudo_element
        && let Some(value) = v8::String::new(scope, &value)
    {
        result.set(value.into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_pseudo_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, arguments.get(0)))
    };
    update(scope, arguments.this(), |record| {
        record.pseudo_element = value
    });
}

fn get_composite(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.composite) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_composite(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if value != "replace" && value != "add" && value != "accumulate" {
        crate::webidl::throw_type_error(scope, "Invalid composite operation");
        return;
    }
    update(scope, arguments.this(), |record| record.composite = value);
}

fn get_keyframes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.keyframes).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_keyframes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let keyframes = normalize_keyframes(scope, arguments.get(0));
    let keyframes = v8::Global::new(scope, keyframes);
    update(scope, arguments.this(), |record| {
        record.keyframes = keyframes
    });
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    update: impl FnOnce(&mut KeyframeRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<KeyframeEffectStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        update(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn object_or_null(value: v8::Local<'_, v8::Value>) -> Option<v8::Local<'_, v8::Object>> {
    if value.is_null() || value.is_undefined() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(value).ok()
    }
}

fn normalize_keyframes<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'_, v8::Value>,
) -> v8::Local<'s, v8::Array> {
    let Ok(input) = v8::Local::<v8::Array>::try_from(value) else {
        return v8::Array::new(scope, 0);
    };
    let output = v8::Array::new(scope, input.length() as i32);
    for index in 0..input.length() {
        let frame = input
            .get_index(scope, index)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok());
        let normalized = v8::Object::new(scope);
        if let Some(frame) = frame
            && let Some(names) =
                frame.get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
        {
            for name_index in 0..names.length() {
                if let Some(name) = names.get_index(scope, name_index)
                    && let Some(value) = frame.get(scope, name)
                {
                    let _ = normalized.set(scope, name, value);
                }
            }
        }
        let computed = if input.length() <= 1 {
            1.0
        } else {
            index as f64 / (input.length() - 1) as f64
        };
        define_number(scope, normalized, "computedOffset", computed);
        let _ = output.set_index(scope, index, normalized.into());
    }
    output
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let value = v8::Number::new(scope, value);
        let _ = object.set(scope, key.into(), value.into());
    }
}
