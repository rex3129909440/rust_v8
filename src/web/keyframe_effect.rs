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

enum KeyframeValidationError {
    Composite(String),
    Easing(String),
    OffsetOrder,
    OffsetRange,
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
    if arguments.length() == 1 {
        let source = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
        let source_record = source.and_then(|source| {
            super::structured_clone::inherits_platform_interface(scope, source, "KeyframeEffect")
                .then(|| record(scope, source))
                .flatten()
        });
        let (Some(source), Some(source_record)) = (source, source_record) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'KeyframeEffect': parameter 1 is not of type 'KeyframeEffect'.",
            );
            return;
        };
        if !super::animation_effect::copy_timing(scope, source, arguments.this()) {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'KeyframeEffect': parameter 1 is not of type 'KeyframeEffect'.",
            );
            return;
        }
        scope
            .get_slot_mut::<KeyframeEffectStore>()
            .expect("KeyframeEffect state")
            .records
            .insert(arguments.this().get_identity_hash().get(), source_record);
        result.set(arguments.this().into());
        return;
    }
    let target = object_or_null(arguments.get(0));
    if let Some(target) = target
        && super::element::record(scope, target).is_none()
    {
        if arguments.length() == 1 {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'KeyframeEffect': parameter 1 is not of type 'KeyframeEffect'.",
            );
        } else {
            crate::webidl::throw_type_error(scope, "target is not an Element");
        }
        return;
    }
    let keyframes = match normalize_keyframes(scope, arguments.get(1)) {
        Ok(keyframes) => keyframes,
        Err(error) => {
            throw_keyframe_validation(scope, true, error);
            return;
        }
    };
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
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
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
        let stored = v8::Local::new(scope, &record.keyframes);
        result.set(clone_keyframes(scope, stored).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_keyframes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let keyframes = match normalize_keyframes(scope, arguments.get(0)) {
        Ok(keyframes) => keyframes,
        Err(error) => {
            throw_keyframe_validation(scope, false, error);
            return;
        }
    };
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
) -> Result<v8::Local<'s, v8::Array>, KeyframeValidationError> {
    let input = if let Ok(input) = v8::Local::<v8::Array>::try_from(value) {
        input
    } else if let Ok(input) = v8::Local::<v8::Object>::try_from(value) {
        property_indexed_keyframes(scope, input)
    } else {
        return Ok(v8::Array::new(scope, 0));
    };
    let length = input.length();
    let output = v8::Array::new(scope, length as i32);
    let mut explicit_offsets = Vec::with_capacity(length as usize);
    for index in 0..input.length() {
        let frame = input
            .get_index(scope, index)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok());
        let normalized = v8::Object::new(scope);
        let offset = frame.and_then(|frame| object_value(scope, frame, "offset"));
        let explicit_offset = offset
            .filter(|value| !value.is_null_or_undefined())
            .and_then(|value| value.number_value(scope));
        if explicit_offset
            .is_some_and(|offset| !offset.is_finite() || !(0.0..=1.0).contains(&offset))
        {
            return Err(KeyframeValidationError::OffsetRange);
        }
        explicit_offsets.push(explicit_offset);
        define_value(
            scope,
            normalized,
            "offset",
            explicit_offset
                .map(|value| v8::Number::new(scope, value).into())
                .unwrap_or_else(|| v8::null(scope).into()),
        );
        let easing = frame
            .and_then(|frame| object_value(scope, frame, "easing"))
            .filter(|value| !value.is_undefined())
            .map(|value| crate::webidl::value_to_string(scope, value))
            .unwrap_or_else(|| "linear".to_owned());
        if !valid_easing(&easing) {
            return Err(KeyframeValidationError::Easing(easing));
        }
        define_string(scope, normalized, "easing", &easing);
        let composite = frame
            .and_then(|frame| object_value(scope, frame, "composite"))
            .filter(|value| !value.is_undefined())
            .map(|value| crate::webidl::value_to_string(scope, value))
            .unwrap_or_else(|| "auto".to_owned());
        if !matches!(
            composite.as_str(),
            "auto" | "replace" | "add" | "accumulate"
        ) {
            return Err(KeyframeValidationError::Composite(composite));
        }
        define_string(scope, normalized, "composite", &composite);
        if let Some(frame) = frame
            && let Some(names) =
                frame.get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
        {
            for name_index in 0..names.length() {
                let Some(name) = names.get_index(scope, name_index) else {
                    continue;
                };
                let name_text = crate::webidl::value_to_string(scope, name);
                if matches!(name_text.as_str(), "offset" | "easing" | "composite") {
                    continue;
                }
                if let Some(value) = frame.get(scope, name)
                    && let Some(text) = value.to_string(scope)
                {
                    let _ = normalized.set(scope, name, text.into());
                }
            }
        }
        let _ = output.set_index(scope, index, normalized.into());
    }
    if explicit_offsets
        .iter()
        .flatten()
        .try_fold(f64::NEG_INFINITY, |previous, current| {
            (*current >= previous).then_some(*current)
        })
        .is_none()
    {
        return Err(KeyframeValidationError::OffsetOrder);
    }
    let computed_offsets = computed_offsets(&explicit_offsets);
    for (index, computed) in computed_offsets.into_iter().enumerate() {
        if let Some(frame) = output
            .get_index(scope, index as u32)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        {
            define_number(scope, frame, "computedOffset", computed);
        }
    }
    Ok(output)
}

fn throw_keyframe_validation(
    scope: &mut v8::PinScope<'_, '_>,
    constructing: bool,
    error: KeyframeValidationError,
) {
    let prefix = if constructing {
        "Failed to construct 'KeyframeEffect':"
    } else {
        "Failed to execute 'setKeyframes' on 'KeyframeEffect':"
    };
    let message = match error {
        KeyframeValidationError::Composite(value) => format!(
            "{prefix} Failed to read the 'composite' property from 'BaseKeyframe': The provided value '{value}' is not a valid enum value of type CompositeOperationOrAuto."
        ),
        KeyframeValidationError::Easing(value) => {
            format!("{prefix} '{value}' is not a valid value for easing")
        }
        KeyframeValidationError::OffsetOrder => {
            format!("{prefix} Offsets must be monotonically non-decreasing.")
        }
        KeyframeValidationError::OffsetRange => {
            format!("{prefix} Offsets must be null or in the range [0,1].")
        }
    };
    crate::webidl::throw_type_error(scope, &message);
}

fn valid_easing(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    if matches!(
        value.as_str(),
        "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out" | "step-start" | "step-end"
    ) {
        return true;
    }
    if let Some(body) = value
        .strip_prefix("cubic-bezier(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let values = body
            .split(',')
            .map(|value| value.trim().parse::<f64>())
            .collect::<Result<Vec<_>, _>>();
        return values.is_ok_and(|values| {
            values.len() == 4
                && values.iter().all(|value| value.is_finite())
                && (0.0..=1.0).contains(&values[0])
                && (0.0..=1.0).contains(&values[2])
        });
    }
    if let Some(body) = value
        .strip_prefix("steps(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let mut values = body.split(',').map(str::trim);
        let count = values
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .is_some_and(|value| value > 0);
        let position = values.next().is_none_or(|value| {
            matches!(
                value,
                "jump-start" | "jump-end" | "jump-none" | "jump-both" | "start" | "end"
            )
        });
        return count && position && values.next().is_none();
    }
    value
        .strip_prefix("linear(")
        .and_then(|value| value.strip_suffix(')'))
        .is_some_and(|body| !body.trim().is_empty())
}

fn property_indexed_keyframes<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    input: v8::Local<'_, v8::Object>,
) -> v8::Local<'s, v8::Array> {
    let Some(names) = input.get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
    else {
        return v8::Array::new(scope, 0);
    };
    let mut properties = Vec::with_capacity(names.length() as usize);
    let mut frame_count = 0_u32;
    for name_index in 0..names.length() {
        let Some(name) = names.get_index(scope, name_index) else {
            continue;
        };
        let Some(value) = input.get(scope, name) else {
            continue;
        };
        let value_length = v8::Local::<v8::Array>::try_from(value)
            .map(|values| values.length())
            .unwrap_or(1);
        frame_count = frame_count.max(value_length);
        properties.push((
            crate::webidl::value_to_string(scope, name),
            v8::Global::new(scope, value),
            value_length,
        ));
    }
    if frame_count == 0 {
        return v8::Array::new(scope, 0);
    }
    let frames = v8::Array::new(scope, frame_count as i32);
    for index in 0..frame_count {
        let _ = frames.set_index(scope, index, v8::Object::new(scope).into());
    }
    for (name, value, value_length) in properties {
        let value = v8::Local::new(scope, &value);
        if let Ok(values) = v8::Local::<v8::Array>::try_from(value) {
            if matches!(name.as_str(), "easing" | "composite") && value_length > 0 {
                for frame_index in 0..frame_count {
                    if let Some(value) = values.get_index(scope, frame_index % value_length) {
                        set_frame_property(scope, frames, frame_index, &name, value);
                    }
                }
                continue;
            }
            for value_index in 0..value_length {
                let Some(value) = values.get_index(scope, value_index) else {
                    continue;
                };
                let frame_index = if value_length <= 1 {
                    frame_count.saturating_sub(1)
                } else {
                    ((value_index as f64 * (frame_count - 1) as f64 / (value_length - 1) as f64)
                        .round()) as u32
                };
                set_frame_property(scope, frames, frame_index, &name, value);
            }
        } else if matches!(name.as_str(), "easing" | "composite") {
            for frame_index in 0..frame_count {
                set_frame_property(scope, frames, frame_index, &name, value);
            }
        } else {
            set_frame_property(scope, frames, frame_count.saturating_sub(1), &name, value);
        }
    }
    frames
}

fn set_frame_property(
    scope: &v8::PinScope<'_, '_>,
    frames: v8::Local<'_, v8::Array>,
    index: u32,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    let Some(frame) = frames
        .get_index(scope, index)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
    else {
        return;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let _ = frame.set(scope, key.into(), value);
}

fn computed_offsets(explicit: &[Option<f64>]) -> Vec<f64> {
    if explicit.is_empty() {
        return Vec::new();
    }
    if explicit.len() == 1 {
        return vec![explicit[0].unwrap_or(1.0)];
    }
    let mut output = explicit.to_vec();
    if output[0].is_none() {
        output[0] = Some(0.0);
    }
    let last = output.len() - 1;
    if output[last].is_none() {
        output[last] = Some(1.0);
    }
    let mut anchor = 0;
    while anchor < last {
        let mut next = anchor + 1;
        while next < output.len() && output[next].is_none() {
            next += 1;
        }
        let start = output[anchor].unwrap_or(0.0);
        let end = output[next].unwrap_or(start);
        let span = (next - anchor) as f64;
        for (index, slot) in output.iter_mut().enumerate().take(next).skip(anchor + 1) {
            *slot = Some(start + (end - start) * (index - anchor) as f64 / span);
        }
        anchor = next;
    }
    output
        .into_iter()
        .map(|value| value.unwrap_or(0.0))
        .collect()
}

fn clone_keyframes<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    input: v8::Local<'_, v8::Array>,
) -> v8::Local<'s, v8::Array> {
    let output = v8::Array::new(scope, input.length() as i32);
    for index in 0..input.length() {
        let Some(source) = input
            .get_index(scope, index)
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        else {
            continue;
        };
        let target = v8::Object::new(scope);
        if let Some(names) =
            source.get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
        {
            for name_index in 0..names.length() {
                if let Some(name) = names.get_index(scope, name_index)
                    && let Some(value) = source.get(scope, name)
                {
                    let _ = target.set(scope, name, value);
                }
            }
        }
        let _ = output.set_index(scope, index, target.into());
    }
    output
}

fn object_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), value);
    }
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define_value(scope, object, name, value.into());
    }
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
