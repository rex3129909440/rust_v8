use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssNumericArrayStore {
    constructor: crate::webidl::RealmConstructor,
    lengths: HashMap<i32, u32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssNumericArrayStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSNumericArray", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssNumericArrayStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSNumericArray",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
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
        .ok_or_else(|| "CSSNumericArray.values is unavailable".to_owned())?;
    let iterator = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator.into(),
        values_function,
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define CSSNumericArray iterator".to_owned());
    }
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssNumericArrayStore>()
        .ok_or_else(|| "CSSNumericArray state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    values: &[v8::Local<'_, v8::Object>],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CSSNumericArray".to_owned());
    }
    for (index, value) in values.iter().enumerate() {
        let _ = object.set_index(scope, index as u32, (*value).into());
    }
    scope
        .get_slot_mut::<CssNumericArrayStore>()
        .ok_or_else(|| "CSSNumericArray state was not prepared".to_owned())?
        .lengths
        .insert(object.get_identity_hash().get(), values.len() as u32);
    Ok(object)
}

fn length(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<u32> {
    scope
        .get_slot::<CssNumericArrayStore>()?
        .lengths
        .get(&object.get_identity_hash().get())
        .copied()
}

fn array<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    let length = length(scope, object)?;
    let array = v8::Array::new(scope, length as i32);
    for index in 0..length {
        let value = object.get_index(scope, index)?;
        let _ = array.set_index(scope, index, value);
    }
    Some(array)
}

fn array_method<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    array: v8::Local<'s, v8::Array>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let function = array
        .get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Function>::try_from(value).ok())?;
    function.call(scope, array.into(), &[])
}

fn return_iterator(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(array) = array(scope, object)
        && let Some(iterator) = array_method(scope, array, name)
    {
        result.set(iterator);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn entries(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_iterator(s, a.this(), "entries", r);
}
fn keys(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_iterator(s, a.this(), "keys", r);
}
fn values(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_iterator(s, a.this(), "values", r);
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
        let key = v8::Integer::new_from_unsigned(scope, index);
        let _ = callback.call(
            scope,
            receiver,
            &[value, key.into(), arguments.this().into()],
        );
    }
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
