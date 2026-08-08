use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PictureInPictureWindowStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PictureInPictureWindowRecord>,
}

#[derive(Clone)]
struct PictureInPictureWindowRecord {
    width: i32,
    height: i32,
    on_resize: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PictureInPictureWindowStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PictureInPictureWindow", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PictureInPictureWindowStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PictureInPictureWindow",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "width", get_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "height", get_height)?;
    crate::webidl::define_accessor(scope, prototype, "onresize", get_on_resize, set_on_resize)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PictureInPictureWindowStore>()
        .ok_or_else(|| "PictureInPictureWindow state was not prepared".to_owned())?
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
        "Failed to construct 'PictureInPictureWindow': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    width: i32,
    height: i32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let window = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, window, prototype.into()) != Some(true) {
        return Err("cannot create PictureInPictureWindow".to_owned());
    }
    super::event_target::attach(scope, window);
    scope
        .get_slot_mut::<PictureInPictureWindowStore>()
        .ok_or_else(|| "PictureInPictureWindow state was not prepared".to_owned())?
        .records
        .insert(
            window.get_identity_hash().get(),
            PictureInPictureWindowRecord {
                width,
                height,
                on_resize: None,
            },
        );
    Ok(window)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    record(scope, object).is_some()
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PictureInPictureWindowRecord> {
    scope
        .get_slot::<PictureInPictureWindowStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.height).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_on_resize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = record.on_resize {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_on_resize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, arguments.get(0)))
    };
    if let Some(record) = scope
        .get_slot_mut::<PictureInPictureWindowStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.on_resize = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
