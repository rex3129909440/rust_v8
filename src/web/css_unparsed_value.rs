use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssUnparsedValueStore {
    constructor: crate::webidl::RealmConstructor,
    lengths: HashMap<i32, u32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssUnparsedValueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSUnparsedValue", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssUnparsedValueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSUnparsedValue",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, keys)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let values_key = crate::webidl::string(scope, "values")?;
    let values_function = prototype
        .get(scope, values_key.into())
        .ok_or_else(|| "CSSUnparsedValue.values is unavailable".to_owned())?;
    let iterator = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator.into(),
        values_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define CSSUnparsedValue iterator".to_owned());
    }
    let parent = super::css_style_value::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssUnparsedValueStore>()
        .ok_or_else(|| "CSSUnparsedValue state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "CSSUnparsedValue requires a sequence");
        return;
    }
    let Ok(sequence) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "CSSUnparsedValue sequence must be an object");
        return;
    };
    let length = sequence
        .get(
            scope,
            v8::String::new(scope, "length")
                .expect("short string")
                .into(),
        )
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    for index in 0..length {
        let value = sequence
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        if !value.is_string()
            && v8::Local::<v8::Object>::try_from(value)
                .ok()
                .and_then(|object| super::css_variable_reference_value::record(scope, object))
                .is_none()
        {
            crate::webidl::throw_type_error(
                scope,
                "CSSUnparsedValue entries must be strings or variable references",
            );
            return;
        }
        let _ = arguments.this().set_index(scope, index, value);
    }
    scope
        .get_slot_mut::<CssUnparsedValueStore>()
        .expect("CSSUnparsedValue state")
        .lengths
        .insert(arguments.this().get_identity_hash().get(), length);
    result.set(arguments.this().into());
}

fn length(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<u32> {
    scope
        .get_slot::<CssUnparsedValueStore>()?
        .lengths
        .get(&object.get_identity_hash().get())
        .copied()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(length) = length(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn value_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    length: u32,
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, length as i32);
    for index in 0..length {
        if let Some(value) = object.get_index(scope, index) {
            let _ = array.set_index(scope, index, value);
        }
    }
    array
}

fn iterator<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    array: v8::Local<'s, v8::Array>,
) -> Option<v8::Local<'s, v8::Value>> {
    let function = array
        .get(scope, v8::Symbol::get_iterator(scope).into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())?;
    function.call(scope, array.into(), &[])
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(length) = length(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = value_array(scope, arguments.this(), length);
    if let Some(iterator) = iterator(scope, array) {
        result.set(iterator);
    }
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(length) = length(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, length as i32);
    for index in 0..length {
        let _ = array.set_index(
            scope,
            index,
            v8::Integer::new_from_unsigned(scope, index).into(),
        );
    }
    if let Some(iterator) = iterator(scope, array) {
        result.set(iterator);
    }
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(length) = length(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, length as i32);
    for index in 0..length {
        let pair = v8::Array::new(scope, 2);
        let _ = pair.set_index(
            scope,
            0,
            v8::Integer::new_from_unsigned(scope, index).into(),
        );
        if let Some(value) = arguments.this().get_index(scope, index) {
            let _ = pair.set_index(scope, 1, value);
        }
        let _ = array.set_index(scope, index, pair.into());
    }
    if let Some(iterator) = iterator(scope, array) {
        result.set(iterator);
    }
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(length) = length(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "forEach requires a callback");
        return;
    };
    let receiver = if arguments.get(1).is_undefined() {
        v8::undefined(scope).into()
    } else {
        arguments.get(1)
    };
    for index in 0..length {
        let value = arguments
            .this()
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let _ = callback.call(
            scope,
            receiver,
            &[
                value,
                v8::Integer::new_from_unsigned(scope, index).into(),
                arguments.this().into(),
            ],
        );
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let length = length(scope, object)?;
    let mut output = String::new();
    for index in 0..length {
        let value = object.get_index(scope, index)?;
        if let Ok(variable) = v8::Local::<v8::Object>::try_from(value)
            && let Some(record) = super::css_variable_reference_value::record(scope, variable)
        {
            if !output.is_empty() && !output.chars().last().is_some_and(char::is_whitespace) {
                output.push_str("/**/");
            }
            output.push_str("var(");
            output.push_str(&record.variable);
            if let Some(fallback) = record.fallback {
                let fallback = v8::Local::new(scope, &fallback);
                if let Some(text) = serialize(scope, fallback) {
                    output.push_str(", ");
                    output.push_str(&text);
                }
            }
            output.push(')');
        } else {
            output.push_str(&crate::webidl::value_to_string(scope, value));
        }
    }
    Some(output)
}
