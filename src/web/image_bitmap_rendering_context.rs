use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ImageBitmapRenderingContextStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ImageBitmapRenderingContextRecord>,
}

#[derive(Clone)]
struct ImageBitmapRenderingContextRecord {
    canvas: v8::Global<v8::Object>,
    bitmap_width: u32,
    bitmap_height: u32,
    pixels: Vec<u8>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ImageBitmapRenderingContextStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ImageBitmapRenderingContext", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ImageBitmapRenderingContextStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ImageBitmapRenderingContext",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "canvas", get_canvas)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "transferFromImageBitmap",
        1,
        transfer_from_image_bitmap,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ImageBitmapRenderingContextStore>()
        .ok_or_else(|| "ImageBitmapRenderingContext state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    canvas: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ImageBitmapRenderingContext".to_owned());
    }
    let canvas = v8::Global::new(scope, canvas);
    scope
        .get_slot_mut::<ImageBitmapRenderingContextStore>()
        .ok_or_else(|| "ImageBitmapRenderingContext state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ImageBitmapRenderingContextRecord {
                canvas,
                bitmap_width: 0,
                bitmap_height: 0,
                pixels: Vec::new(),
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
        "Failed to construct 'ImageBitmapRenderingContext': Illegal constructor",
    );
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ImageBitmapRenderingContextRecord> {
    scope
        .get_slot::<ImageBitmapRenderingContextStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    canvas_width: u32,
    canvas_height: u32,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = record(scope, object)?;
    let mut output =
        vec![0_u8; canvas_width.saturating_mul(canvas_height).saturating_mul(4) as usize];
    let copy_width = canvas_width.min(record.bitmap_width);
    let copy_height = canvas_height.min(record.bitmap_height);
    for y in 0..copy_height {
        let source = y as usize * record.bitmap_width as usize * 4;
        let destination = y as usize * canvas_width as usize * 4;
        let length = copy_width as usize * 4;
        output[destination..destination + length]
            .copy_from_slice(&record.pixels[source..source + length]);
    }
    Some((canvas_width, canvas_height, output))
}

pub(crate) fn take_snapshot_and_reset(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    canvas_width: u32,
    canvas_height: u32,
) -> Option<(u32, u32, Vec<u8>)> {
    let snapshot = snapshot(scope, object, canvas_width, canvas_height)?;
    let record = scope
        .get_slot_mut::<ImageBitmapRenderingContextStore>()?
        .records
        .get_mut(&object.get_identity_hash().get())?;
    record.bitmap_width = 0;
    record.bitmap_height = 0;
    record.pixels.clear();
    Some(snapshot)
}

fn get_canvas(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.canvas).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn transfer_from_image_bitmap(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let transferred = if arguments.get(0).is_null() {
        Some((0, 0, Vec::new()))
    } else {
        let Ok(bitmap) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
            crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'ImageBitmap'");
            return;
        };
        super::image_bitmap::take_pixels(scope, bitmap)
    };
    let Some((bitmap_width, bitmap_height, pixels)) = transferred else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'ImageBitmap'");
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<ImageBitmapRenderingContextStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.bitmap_width = bitmap_width;
        record.bitmap_height = bitmap_height;
        record.pixels = pixels;
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ImageBitmapRenderingContextStore>() {
        store.constructor.remove(realm_id);
    }
}
