use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct XrInputSourceArrayStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrInputSourceArrayStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRInputSourceArray", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrInputSourceArrayStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRInputSourceArray",
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
    crate::webidl::define_iterator_alias(scope, prototype, "values")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrInputSourceArrayStore>()
        .ok_or_else(|| "XRInputSourceArray state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    sources: Vec<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRInputSourceArray".to_owned());
    }
    let sources = sources
        .into_iter()
        .map(|source| v8::Global::new(scope, source))
        .collect();
    scope
        .get_slot_mut::<XrInputSourceArrayStore>()
        .ok_or_else(|| "XRInputSourceArray state missing".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), sources);
    Ok(object)
}

fn sources(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<XrInputSourceArrayStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn array<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    let sources = sources(scope, object)?;
    let array = v8::Array::new(scope, sources.len() as i32);
    for (index, source) in sources.iter().enumerate() {
        let source = v8::Local::new(scope, source);
        let _ = array.set_index(scope, index as u32, source.into());
    }
    Some(array)
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(sources) = sources(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, sources.len() as i32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = array(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, values.length() as i32);
    for index in 0..values.length() {
        let pair = v8::Array::new(scope, 2);
        let index_value = v8::Integer::new_from_unsigned(scope, index);
        let source = values
            .get_index(scope, index)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let _ = pair.set_index(scope, 0, index_value.into());
        let _ = pair.set_index(scope, 1, source);
        let _ = output.set_index(scope, index, pair.into());
    }
    result.set(output.into());
}

fn keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = sources(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, values.len() as i32);
    for index in 0..values.len() {
        let index_value = v8::Integer::new_from_unsigned(scope, index as u32);
        let _ = output.set_index(scope, index as u32, index_value.into());
    }
    result.set(output.into());
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = array(scope, arguments.this()) {
        result.set(values.into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = sources(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let this_arg = arguments.get(1);
    for (index, source) in values.iter().enumerate() {
        let source = v8::Local::new(scope, source);
        let index_value = v8::Integer::new_from_unsigned(scope, index as u32);
        let callback_arguments = [source.into(), index_value.into(), arguments.this().into()];
        let _ = callback.call(scope, this_arg, &callback_arguments);
    }
    result.set(v8::undefined(scope).into());
}
