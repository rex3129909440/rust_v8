use std::collections::HashMap;

#[derive(Clone)]
struct ByteLengthStrategyRecord {
    high_water_mark: f64,
    size: v8::Global<v8::Function>,
}

#[derive(Default)]
pub(crate) struct ByteLengthQueuingStrategyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ByteLengthStrategyRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ByteLengthQueuingStrategyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ByteLengthQueuingStrategy", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<ByteLengthQueuingStrategyStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ByteLengthQueuingStrategy",
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
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ByteLengthQueuingStrategyStore>()
        .ok_or_else(|| "ByteLengthQueuingStrategy state was not prepared".to_owned())?
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
            "ByteLengthQueuingStrategy requires an options object",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "ByteLengthQueuingStrategy options must be an object",
        );
        return;
    };
    let Some(key) = v8::String::new(scope, "highWaterMark") else {
        return;
    };
    let value = options
        .get(scope, key.into())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(f64::NAN);
    if value.is_nan() {
        crate::webidl::throw_type_error(scope, "highWaterMark must be a number");
        return;
    }
    let size = match crate::webidl::create_function(
        scope,
        "size",
        1,
        v8::ConstructorBehavior::Throw,
        size_algorithm,
    ) {
        Ok(size) => v8::Global::new(scope, size),
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    scope
        .get_slot_mut::<ByteLengthQueuingStrategyStore>()
        .expect("ByteLengthQueuingStrategy state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            ByteLengthStrategyRecord {
                high_water_mark: value,
                size,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ByteLengthStrategyRecord> {
    scope
        .get_slot::<ByteLengthQueuingStrategyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_high_water_mark(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.high_water_mark).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.size).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn size_algorithm(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Ok(chunk) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        result.set(v8::Number::new(scope, 0.0).into());
        return;
    };
    let Some(key) = v8::String::new(scope, "byteLength") else {
        return;
    };
    let value = chunk
        .get(scope, key.into())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(0.0);
    result.set(v8::Number::new(scope, value).into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ByteLengthQueuingStrategyStore>() {
        store.constructor.remove(realm_id);
    }
}
