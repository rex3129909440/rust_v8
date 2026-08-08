use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ResizeObserverSizeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SizeRecord>,
}

#[derive(Clone, Copy)]
struct SizeRecord {
    inline_size: f64,
    block_size: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ResizeObserverSizeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ResizeObserverSize", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ResizeObserverSizeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ResizeObserverSize",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "inlineSize", get_inline_size)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "blockSize", get_block_size)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ResizeObserverSizeStore>()
        .ok_or_else(|| "ResizeObserverSize state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    inline_size: f64,
    block_size: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ResizeObserverSize".to_owned());
    }
    scope
        .get_slot_mut::<ResizeObserverSizeStore>()
        .ok_or_else(|| "ResizeObserverSize state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            SizeRecord {
                inline_size,
                block_size,
            },
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ResizeObserverSize': Illegal constructor",
    );
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<SizeRecord> {
    scope
        .get_slot::<ResizeObserverSizeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

fn get_inline_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.inline_size).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_block_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.block_size).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
