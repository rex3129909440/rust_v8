use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaErrorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MediaErrorRecord>,
}

#[derive(Clone)]
struct MediaErrorRecord {
    code: u32,
    message: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaErrorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaError", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaErrorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaError",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "code", get_code)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "message", get_message)?;
    crate::webidl::define_constant(scope, prototype, "MEDIA_ERR_ABORTED", 1)?;
    crate::webidl::define_constant(scope, prototype, "MEDIA_ERR_NETWORK", 2)?;
    crate::webidl::define_constant(scope, prototype, "MEDIA_ERR_DECODE", 3)?;
    crate::webidl::define_constant(scope, prototype, "MEDIA_ERR_SRC_NOT_SUPPORTED", 4)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_constant(scope, constructor.into(), "MEDIA_ERR_ABORTED", 1)?;
    crate::webidl::define_constant(scope, constructor.into(), "MEDIA_ERR_NETWORK", 2)?;
    crate::webidl::define_constant(scope, constructor.into(), "MEDIA_ERR_DECODE", 3)?;
    crate::webidl::define_constant(scope, constructor.into(), "MEDIA_ERR_SRC_NOT_SUPPORTED", 4)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaErrorStore>()
        .ok_or_else(|| "MediaError state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'MediaError': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    code: u32,
    message: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaError".to_owned());
    }
    scope
        .get_slot_mut::<MediaErrorStore>()
        .ok_or_else(|| "MediaError state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            MediaErrorRecord { code, message },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MediaErrorRecord> {
    scope
        .get_slot::<MediaErrorStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_code(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.code).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_message(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.message) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
