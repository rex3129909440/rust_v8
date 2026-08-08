use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct OffscreenCanvasStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, OffscreenCanvasRecord>,
}

#[derive(Clone)]
struct OffscreenCanvasRecord {
    width: u32,
    height: u32,
    oncontextlost: Option<v8::Global<v8::Value>>,
    oncontextrestored: Option<v8::Global<v8::Value>>,
    context_2d: Option<v8::Global<v8::Object>>,
    bitmap_renderer: Option<v8::Global<v8::Object>>,
    webgl: Option<v8::Global<v8::Object>>,
    webgl2: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OffscreenCanvasStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "OffscreenCanvas", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<OffscreenCanvasStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "OffscreenCanvas",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "width", get_width, set_width)?;
    crate::webidl::define_accessor(scope, prototype, "height", get_height, set_height)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncontextlost",
        get_oncontextlost,
        set_oncontextlost,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "oncontextrestored",
        get_oncontextrestored,
        set_oncontextrestored,
    )?;
    crate::webidl::define_method(scope, prototype, "convertToBlob", 0, convert_to_blob)?;
    crate::webidl::define_method(scope, prototype, "getContext", 1, get_context)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "transferToImageBitmap",
        0,
        transfer_to_image_bitmap,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<OffscreenCanvasStore>()
        .ok_or_else(|| "OffscreenCanvas state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    width: u32,
    height: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create OffscreenCanvas".to_owned());
    }
    super::event_target::attach(scope, object);
    attach_external(scope, object, width, height);
    Ok(object)
}

pub(crate) fn attach_external(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    width: u32,
    height: u32,
) {
    if let Some(store) = scope.get_slot_mut::<OffscreenCanvasStore>() {
        store.records.insert(
            object.get_identity_hash().get(),
            OffscreenCanvasRecord {
                width,
                height,
                oncontextlost: None,
                oncontextrestored: None,
                context_2d: None,
                bitmap_renderer: None,
                webgl: None,
                webgl2: None,
            },
        );
    }
}

pub(crate) fn resize_external(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    width: u32,
    height: u32,
) {
    resize(scope, object, Some(width), Some(height));
}

fn dimension(scope: &mut v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> Option<u32> {
    let number = value.number_value(scope).unwrap_or(f64::NAN);
    if !number.is_finite() || number < 0.0 || number > u32::MAX as f64 {
        crate::webidl::throw_type_error(
            scope,
            "The canvas dimension is outside the unsigned long range",
        );
        None
    } else {
        Some(number.floor() as u32)
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'OffscreenCanvas': 2 arguments required",
        );
        return;
    }
    let Some(width) = dimension(scope, arguments.get(0)) else {
        return;
    };
    let Some(height) = dimension(scope, arguments.get(1)) else {
        return;
    };
    super::event_target::attach(scope, arguments.this());
    scope
        .get_slot_mut::<OffscreenCanvasStore>()
        .expect("OffscreenCanvas state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            OffscreenCanvasRecord {
                width,
                height,
                oncontextlost: None,
                oncontextrestored: None,
                context_2d: None,
                bitmap_renderer: None,
                webgl: None,
                webgl2: None,
            },
        );
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<OffscreenCanvasRecord> {
    scope
        .get_slot::<OffscreenCanvasStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn dimensions(
    scope: &v8::PinScope<'_, '_>,
    canvas: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32)> {
    record(scope, canvas).map(|record| (record.width, record.height))
}

fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.width).into())
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
        result.set(v8::Integer::new_from_unsigned(scope, record.height).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn resize(
    scope: &mut v8::PinScope<'_, '_>,
    canvas: v8::Local<'_, v8::Object>,
    width: Option<u32>,
    height: Option<u32>,
) {
    let context = if let Some(record) = scope
        .get_slot_mut::<OffscreenCanvasStore>()
        .and_then(|store| store.records.get_mut(&canvas.get_identity_hash().get()))
    {
        if let Some(width) = width {
            record.width = width;
        }
        if let Some(height) = height {
            record.height = height;
        }
        record.context_2d.clone()
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(context) = context {
        let context = v8::Local::new(scope, &context);
        super::offscreen_canvas_rendering_context_2d::reset_for_resize(scope, context);
    }
}

fn set_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(width) = dimension(scope, arguments.get(0)) {
        resize(scope, arguments.this(), Some(width), None)
    }
}
fn set_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(height) = dimension(scope, arguments.get(0)) {
        resize(scope, arguments.this(), None, Some(height))
    }
}

fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&OffscreenCanvasRecord) -> Option<v8::Global<v8::Value>>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match select(&record) {
            Some(value) => result.set(v8::Local::new(scope, &value)),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
fn get_oncontextlost(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |v| v.oncontextlost.clone())
}
fn get_oncontextrestored(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_handler(s, a, r, |v| v.oncontextrestored.clone())
}

fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    select: fn(&mut OffscreenCanvasRecord) -> &mut Option<v8::Global<v8::Value>>,
) {
    let value = if arguments.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, arguments.get(0)))
    };
    if let Some(record) = scope
        .get_slot_mut::<OffscreenCanvasStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *select(record) = value
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn set_oncontextlost(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |v| &mut v.oncontextlost)
}
fn set_oncontextrestored(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, |v| &mut v.oncontextrestored)
}

fn get_context(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let context_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let context_type = if context_type == "experimental-webgl" {
        "webgl"
    } else {
        context_type.as_str()
    };
    if !matches!(context_type, "2d" | "bitmaprenderer" | "webgl" | "webgl2") {
        result.set(v8::null(scope).into());
        return;
    }
    if context_type == "2d" {
        if snapshot.bitmap_renderer.is_some()
            || snapshot.webgl.is_some()
            || snapshot.webgl2.is_some()
        {
            result.set(v8::null(scope).into());
            return;
        }
        if let Some(context) = snapshot.context_2d {
            result.set(v8::Local::new(scope, &context).into());
            return;
        }
        let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
        match super::offscreen_canvas_rendering_context_2d::create(scope, arguments.this(), options)
        {
            Ok(context) => {
                let context_global = v8::Global::new(scope, context);
                if let Some(record) =
                    scope
                        .get_slot_mut::<OffscreenCanvasStore>()
                        .and_then(|store| {
                            store
                                .records
                                .get_mut(&arguments.this().get_identity_hash().get())
                        })
                {
                    record.context_2d = Some(context_global);
                }
                result.set(context.into());
            }
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    if context_type == "webgl" {
        if snapshot.context_2d.is_some()
            || snapshot.bitmap_renderer.is_some()
            || snapshot.webgl2.is_some()
        {
            result.set(v8::null(scope).into());
            return;
        }
        if let Some(context) = snapshot.webgl {
            result.set(v8::Local::new(scope, &context).into());
            return;
        }
        match super::webgl_rendering_context::create(
            scope,
            Some(arguments.this()),
            snapshot.width,
            snapshot.height,
        ) {
            Ok(context) => {
                let context_global = v8::Global::new(scope, context);
                if let Some(record) =
                    scope
                        .get_slot_mut::<OffscreenCanvasStore>()
                        .and_then(|store| {
                            store
                                .records
                                .get_mut(&arguments.this().get_identity_hash().get())
                        })
                {
                    record.webgl = Some(context_global);
                }
                result.set(context.into());
            }
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    if context_type == "webgl2" {
        if snapshot.context_2d.is_some()
            || snapshot.bitmap_renderer.is_some()
            || snapshot.webgl.is_some()
        {
            result.set(v8::null(scope).into());
            return;
        }
        if let Some(context) = snapshot.webgl2 {
            result.set(v8::Local::new(scope, &context).into());
            return;
        }
        match super::webgl2_rendering_context::create(
            scope,
            Some(arguments.this()),
            snapshot.width,
            snapshot.height,
        ) {
            Ok(context) => {
                let context_global = v8::Global::new(scope, context);
                if let Some(record) =
                    scope
                        .get_slot_mut::<OffscreenCanvasStore>()
                        .and_then(|store| {
                            store
                                .records
                                .get_mut(&arguments.this().get_identity_hash().get())
                        })
                {
                    record.webgl2 = Some(context_global);
                }
                result.set(context.into());
            }
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    if snapshot.context_2d.is_some() || snapshot.webgl.is_some() || snapshot.webgl2.is_some() {
        result.set(v8::null(scope).into());
        return;
    }
    if let Some(context) = snapshot.bitmap_renderer {
        result.set(v8::Local::new(scope, &context).into());
        return;
    }
    match super::image_bitmap_rendering_context::create(scope, arguments.this()) {
        Ok(context) => {
            let context_global = v8::Global::new(scope, context);
            if let Some(record) = scope
                .get_slot_mut::<OffscreenCanvasStore>()
                .and_then(|store| {
                    store
                        .records
                        .get_mut(&arguments.this().get_identity_hash().get())
                })
            {
                record.bitmap_renderer = Some(context_global);
            }
            result.set(context.into());
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn pixel_snapshot(
    scope: &v8::PinScope<'_, '_>,
    canvas: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = record(scope, canvas)?;
    if let Some(context) = record.context_2d {
        let context = v8::Local::new(scope, &context);
        super::offscreen_canvas_rendering_context_2d::snapshot_pixels(scope, context)
    } else if let Some(context) = record.bitmap_renderer {
        let context = v8::Local::new(scope, &context);
        super::image_bitmap_rendering_context::snapshot(scope, context, record.width, record.height)
    } else {
        Some((
            record.width,
            record.height,
            vec![0_u8; record.width as usize * record.height as usize * 4],
        ))
    }
}

fn convert_to_blob(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some((width, height, pixels)) = pixel_snapshot(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let options = v8::Local::<v8::Object>::try_from(arguments.get(0)).ok();
    let media_type = options
        .and_then(|options| {
            let key = v8::String::new(scope, "type")?;
            options.get(scope, key.into())
        })
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value).to_ascii_lowercase())
        .unwrap_or_else(|| "image/png".to_owned());
    let media_type = if matches!(
        media_type.as_str(),
        "image/png" | "image/jpeg" | "image/webp"
    ) {
        media_type
    } else {
        "image/png".to_owned()
    };
    let encoded = if media_type == "image/png" {
        super::html_canvas_element::fingerprinted_png_bytes(scope, width, height, Some(&pixels))
            .unwrap_or_default()
    } else {
        let mut encoded = b"OFFSCREEN".to_vec();
        encoded.extend_from_slice(&width.to_le_bytes());
        encoded.extend_from_slice(&height.to_le_bytes());
        encoded.extend_from_slice(
            crate::fingerprint::edge(scope)
                .rendering
                .canvas
                .data_url_salt
                .as_bytes(),
        );
        encoded.extend_from_slice(&pixels);
        encoded
    };
    match super::blob::create(scope, encoded, &media_type) {
        Ok(blob) => {
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, blob.into()) {
                result.set(promise.into())
            }
        }
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn transfer_to_image_bitmap(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let snapshot = record(scope, arguments.this());
    let pixels = snapshot
        .as_ref()
        .and_then(|record| record.context_2d.as_ref())
        .and_then(|context| {
            let context = v8::Local::new(scope, context);
            super::offscreen_canvas_rendering_context_2d::take_pixel_snapshot_and_reset(
                scope, context,
            )
        })
        .or_else(|| {
            snapshot
                .as_ref()
                .and_then(|record| {
                    record
                        .bitmap_renderer
                        .as_ref()
                        .map(|context| (record, context))
                })
                .and_then(|(record, context)| {
                    let context = v8::Local::new(scope, context);
                    super::image_bitmap_rendering_context::take_snapshot_and_reset(
                        scope,
                        context,
                        record.width,
                        record.height,
                    )
                })
        })
        .or_else(|| pixel_snapshot(scope, arguments.this()));
    let Some((width, height, pixels)) = pixels else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match super::image_bitmap::create(scope, width, height, pixels) {
        Ok(bitmap) => result.set(bitmap.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<OffscreenCanvasStore>() {
        store.constructors.remove(&realm_id);
    }
}
