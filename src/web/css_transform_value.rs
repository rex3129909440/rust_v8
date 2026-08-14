use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssTransformValueStore {
    constructor: crate::webidl::RealmConstructor,
    lengths: HashMap<i32, u32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssTransformValueStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSTransformValue", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssTransformValueStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSTransformValue",
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
    crate::webidl::define_readonly_accessor(scope, prototype, "is2D", get_is_2d)?;
    crate::webidl::define_method(scope, prototype, "toMatrix", 0, to_matrix)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let values_key = crate::webidl::string(scope, "values")?;
    let values_function = prototype
        .get(scope, values_key.into())
        .ok_or_else(|| "CSSTransformValue.values is unavailable".to_owned())?;
    let iterator = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator.into(),
        values_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define CSSTransformValue iterator".to_owned());
    }
    let parent = super::css_style_value::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssTransformValueStore>()
        .ok_or_else(|| "CSSTransformValue state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "CSSTransformValue requires a sequence");
        return;
    }
    if !arguments.get(0).is_object() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSTransformValue': The provided value cannot be converted to a sequence.",
        );
        return;
    }
    let values = match crate::webidl::sequence_values(scope, arguments.get(0)) {
        Ok(values) => values,
        Err(_) => {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'CSSTransformValue': The object must have a callable @@iterator property.",
            );
            return;
        }
    };
    if values.is_empty() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSTransformValue': CSSTransformValue must have at least one component",
        );
        return;
    }
    let length = values.len() as u32;
    for (index, value) in values.into_iter().enumerate() {
        let value = v8::Local::new(scope, &value);
        let Some(object) = v8::Local::<v8::Object>::try_from(value).ok() else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'CSSTransformValue': Failed to convert value to 'CSSTransformComponent'.",
            );
            return;
        };
        if !super::css_transform_component::is_component(scope, object) {
            crate::webidl::throw_type_error(
                scope,
                "Failed to construct 'CSSTransformValue': Failed to convert value to 'CSSTransformComponent'.",
            );
            return;
        }
        let _ = arguments
            .this()
            .set_index(scope, index as u32, object.into());
    }
    scope
        .get_slot_mut::<CssTransformValueStore>()
        .expect("CSSTransformValue state")
        .lengths
        .insert(arguments.this().get_identity_hash().get(), length);
    result.set(arguments.this().into());
}

fn length(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<u32> {
    scope
        .get_slot::<CssTransformValueStore>()?
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

fn array_iterator<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    array: v8::Local<'s, v8::Array>,
) -> Option<v8::Local<'s, v8::Value>> {
    let iterator_key = v8::Symbol::get_iterator(scope);
    let function = array
        .get(scope, iterator_key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())?;
    function.call(scope, array.into(), &[])
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    crate::webidl::return_array_like_iterator(
        scope,
        arguments.this(),
        crate::webidl::ArrayLikeIteratorKind::Values,
        result,
    );
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    crate::webidl::return_array_like_iterator(
        scope,
        arguments.this(),
        crate::webidl::ArrayLikeIteratorKind::Keys,
        result,
    );
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    crate::webidl::return_array_like_iterator(
        scope,
        arguments.this(),
        crate::webidl::ArrayLikeIteratorKind::Entries,
        result,
    );
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::array_like_for_each(scope, arguments)
}

fn get_is_2d(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(length) = length(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut is_2d = true;
    for index in 0..length {
        let Some(value) = arguments.this().get_index(scope, index) else {
            is_2d = false;
            break;
        };
        let Some(object) = v8::Local::<v8::Object>::try_from(value).ok() else {
            is_2d = false;
            break;
        };
        if super::css_transform_component::is_2d(scope, object) != Some(true) {
            is_2d = false;
            break;
        }
    }
    result.set(v8::Boolean::new(scope, is_2d).into());
}

fn combined_matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let length = length(scope, object)?;
    let mut matrix = super::dom_matrix::identity();
    for index in 0..length {
        let component = object.get_index(scope, index)?;
        let component = v8::Local::<v8::Object>::try_from(component).ok()?;
        let next = super::css_transform_component::matrix(scope, component)?;
        matrix = super::dom_matrix::multiply(matrix, next);
    }
    Some(matrix)
}

fn to_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(matrix) = combined_matrix(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match super::dom_matrix::create_from_matrix(scope, matrix) {
        Ok(matrix) => result.set(matrix.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let length = length(scope, object)?;
    let mut output = String::new();
    for index in 0..length {
        let component = object.get_index(scope, index)?;
        let component = v8::Local::<v8::Object>::try_from(component).ok()?;
        let text = super::css_transform_component::serialize(scope, component)?;
        if !output.is_empty() {
            output.push(' ');
        }
        output.push_str(&text);
    }
    Some(output)
}
