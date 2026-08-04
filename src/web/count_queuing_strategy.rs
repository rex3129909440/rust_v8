use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CountQueuingStrategyStore {
    constructor: crate::webidl::RealmConstructor,
    size_function: Option<v8::Global<v8::Function>>,
    high_water_marks: HashMap<i32, f64>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CountQueuingStrategyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CountQueuingStrategy", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CountQueuingStrategyStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CountQueuingStrategy",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "highWaterMark",
        get_high_water_mark,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let size_function =
        crate::webidl::create_function(scope, "size", 0, v8::ConstructorBehavior::Throw, size)?;
    let constructor_global = v8::Global::new(scope, constructor);
    let size_function_global = v8::Global::new(scope, size_function);
    let store = scope
        .get_slot_mut::<CountQueuingStrategyStore>()
        .ok_or_else(|| "CountQueuingStrategy state was not prepared".to_owned())?;
    store.constructor.insert(realm_id, constructor_global);
    store.size_function = Some(size_function_global);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CountQueuingStrategy requires an init object");
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "CountQueuingStrategy init must be an object");
        return;
    };
    let Some(key) = v8::String::new(scope, "highWaterMark") else {
        return;
    };
    let high_water_mark = options
        .get(scope, key.into())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(f64::NAN);
    if high_water_mark.is_nan() || high_water_mark < 0.0 {
        crate::webidl::throw_type_error(scope, "highWaterMark must be non-negative");
        return;
    }
    scope
        .get_slot_mut::<CountQueuingStrategyStore>()
        .expect("CountQueuingStrategy state")
        .high_water_marks
        .insert(arguments.this().get_identity_hash().get(), high_water_mark);
    result.set(arguments.this().into());
}

fn get_high_water_mark(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<CountQueuingStrategyStore>()
        .and_then(|store| {
            store
                .high_water_marks
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Number::new(scope, *value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<CountQueuingStrategyStore>()
        .is_some_and(|store| {
            store
                .high_water_marks
                .contains_key(&arguments.this().get_identity_hash().get())
        })
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(function) = scope
        .get_slot::<CountQueuingStrategyStore>()
        .and_then(|store| store.size_function.as_ref())
        .cloned()
    {
        result.set(v8::Local::new(scope, &function).into());
    }
}

fn size(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Integer::new(scope, 1).into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CountQueuingStrategyStore>() {
        store.constructor.remove(realm_id);
    }
}
