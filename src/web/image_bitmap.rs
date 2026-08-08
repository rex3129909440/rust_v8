use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ImageBitmapStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ImageBitmapRecord>,
}

#[derive(Clone)]
struct ImageBitmapRecord {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    closed: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ImageBitmapStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ImageBitmap", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ImageBitmapStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ImageBitmap",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "width", get_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "height", get_height)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ImageBitmapStore>()
        .ok_or_else(|| "ImageBitmap state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    width: u32,
    height: u32,
    pixels: Vec<u8>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let bitmap = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, bitmap, prototype.into()) != Some(true) {
        return Err("cannot create ImageBitmap".to_owned());
    }
    scope
        .get_slot_mut::<ImageBitmapStore>()
        .ok_or_else(|| "ImageBitmap state was not prepared".to_owned())?
        .records
        .insert(
            bitmap.get_identity_hash().get(),
            ImageBitmapRecord {
                width,
                height,
                pixels,
                closed: false,
            },
        );
    Ok(bitmap)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ImageBitmap': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ImageBitmapRecord> {
    scope
        .get_slot::<ImageBitmapStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_image_bitmap(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<ImageBitmapStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn take_pixels(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = scope
        .get_slot_mut::<ImageBitmapStore>()?
        .records
        .get_mut(&object.get_identity_hash().get())?;
    if record.closed {
        return Some((0, 0, Vec::new()));
    }
    record.closed = true;
    Some((
        record.width,
        record.height,
        std::mem::take(&mut record.pixels),
    ))
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = record(scope, object)?;
    if record.closed {
        None
    } else {
        Some((record.width, record.height, record.pixels))
    }
}

fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(
            v8::Integer::new_from_unsigned(scope, if record.closed { 0 } else { record.width })
                .into(),
        )
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(
            v8::Integer::new_from_unsigned(scope, if record.closed { 0 } else { record.height })
                .into(),
        )
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<ImageBitmapStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.closed = true;
        record.pixels.clear()
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ImageBitmapStore>() {
        store.constructor.remove(realm_id);
    }
}
