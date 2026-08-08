use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct XrPlaneSetStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrPlaneSetStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRPlaneSet", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrPlaneSetStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRPlaneSet",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_method(scope, prototype, "entries", 0, entries)?;
    crate::webidl::define_method(scope, prototype, "forEach", 1, for_each)?;
    crate::webidl::define_method(scope, prototype, "has", 1, has)?;
    crate::webidl::define_method(scope, prototype, "keys", 0, values)?;
    crate::webidl::define_method(scope, prototype, "values", 0, values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "values")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrPlaneSetStore>()
        .ok_or_else(|| "XRPlaneSet state missing".to_owned())?
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
    planes: Vec<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRPlaneSet".to_owned());
    }
    let planes = planes
        .into_iter()
        .map(|plane| v8::Global::new(scope, plane))
        .collect();
    scope
        .get_slot_mut::<XrPlaneSetStore>()
        .ok_or_else(|| "XRPlaneSet state missing".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), planes);
    Ok(object)
}

fn planes(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<XrPlaneSetStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(values) = planes(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, values.len() as i32).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn has(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = planes(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let wanted = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .map(|object| object.get_identity_hash().get());
    let found = wanted.is_some_and(|wanted| {
        values.iter().any(|value| {
            let value = v8::Local::new(scope, value);
            value.get_identity_hash().get() == wanted
        })
    });
    result.set(v8::Boolean::new(scope, found).into());
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = planes(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let value = v8::Local::new(scope, value);
        let _ = output.set_index(scope, index as u32, value.into());
    }
    result.set(output.into());
}

fn entries(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = planes(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().enumerate() {
        let value = v8::Local::new(scope, value);
        let pair = v8::Array::new(scope, 2);
        let _ = pair.set_index(scope, 0, value.into());
        let _ = pair.set_index(scope, 1, value.into());
        let _ = output.set_index(scope, index as u32, pair.into());
    }
    result.set(output.into());
}

fn for_each(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(values) = planes(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "callback must be a function");
        return;
    };
    let this_arg = arguments.get(1);
    for value in values {
        let value = v8::Local::new(scope, &value);
        let callback_arguments = [value.into(), value.into(), arguments.this().into()];
        let _ = callback.call(scope, this_arg, &callback_arguments);
    }
    result.set(v8::undefined(scope).into());
}
