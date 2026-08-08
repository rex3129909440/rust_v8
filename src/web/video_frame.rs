use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct VideoFrameStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, VideoFrameRecord>,
}

#[derive(Clone)]
struct VideoFrameRecord {
    source: Option<v8::Global<v8::Object>>,
    format: String,
    timestamp: f64,
    duration: Option<f64>,
    coded_width: u32,
    coded_height: u32,
    visible_x: f64,
    visible_y: f64,
    visible_width: f64,
    visible_height: f64,
    rotation: u32,
    flip: bool,
    display_width: u32,
    display_height: u32,
    color_space: v8::Global<v8::Object>,
    closed: bool,
}

#[derive(Clone)]
pub(crate) struct VideoFrameEncodingSnapshot {
    pub(crate) format: String,
    pub(crate) timestamp: f64,
    pub(crate) duration: Option<f64>,
    pub(crate) coded_width: u32,
    pub(crate) coded_height: u32,
    pub(crate) display_width: u32,
    pub(crate) display_height: u32,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(VideoFrameStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "VideoFrame", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<VideoFrameStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "VideoFrame",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "format", get_format)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timestamp", get_timestamp)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "duration", get_duration)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "codedWidth", get_coded_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "codedHeight", get_coded_height)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "codedRect", get_coded_rect)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "visibleRect", get_visible_rect)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rotation", get_rotation)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "flip", get_flip)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "displayWidth", get_display_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "displayHeight", get_display_height)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "colorSpace", get_color_space)?;
    crate::webidl::define_method(scope, prototype, "allocationSize", 0, allocation_size)?;
    crate::webidl::define_method(scope, prototype, "clone", 0, clone_frame)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "copyTo", 1, copy_to)?;
    crate::webidl::define_method(scope, prototype, "metadata", 0, metadata)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<VideoFrameStore>()
        .ok_or_else(|| "VideoFrame state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(scope, "VideoFrame requires a source");
        return;
    }
    let source_value = arguments.get(0);
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let cloned = v8::Local::<v8::Object>::try_from(source_value)
        .ok()
        .and_then(|source| record(scope, source));
    let record = if let Some(mut record) = cloned {
        record.timestamp = options
            .and_then(|options| optional_number(scope, options, "timestamp"))
            .unwrap_or(record.timestamp);
        record.duration = options
            .and_then(|options| optional_number(scope, options, "duration"))
            .or(record.duration);
        record.closed = false;
        record
    } else {
        let Some(options) = options else {
            crate::webidl::throw_type_error(
                scope,
                "VideoFrame buffer sources require initialization options",
            );
            return;
        };
        let Some(format) = optional_string(scope, options, "format") else {
            crate::webidl::throw_type_error(scope, "VideoFrame format is required");
            return;
        };
        let coded_width = optional_u32(scope, options, "codedWidth").unwrap_or(0);
        let coded_height = optional_u32(scope, options, "codedHeight").unwrap_or(0);
        let Some(timestamp) = optional_number(scope, options, "timestamp") else {
            crate::webidl::throw_type_error(scope, "VideoFrame timestamp is required");
            return;
        };
        if coded_width == 0 || coded_height == 0 {
            crate::webidl::throw_type_error(
                scope,
                "VideoFrame codedWidth and codedHeight must be non-zero",
            );
            return;
        }
        let source = v8::Local::<v8::Object>::try_from(source_value)
            .ok()
            .map(|source| v8::Global::new(scope, source));
        let visible = object_property(scope, options, "visibleRect");
        let visible_x = visible
            .and_then(|visible| optional_number(scope, visible, "x"))
            .unwrap_or(0.0);
        let visible_y = visible
            .and_then(|visible| optional_number(scope, visible, "y"))
            .unwrap_or(0.0);
        let visible_width = visible
            .and_then(|visible| optional_number(scope, visible, "width"))
            .unwrap_or(coded_width as f64);
        let visible_height = visible
            .and_then(|visible| optional_number(scope, visible, "height"))
            .unwrap_or(coded_height as f64);
        let rotation = optional_u32(scope, options, "rotation").unwrap_or(0);
        let flip = optional_boolean(scope, options, "flip").unwrap_or(false);
        let display_width =
            optional_u32(scope, options, "displayWidth").unwrap_or(visible_width as u32);
        let display_height =
            optional_u32(scope, options, "displayHeight").unwrap_or(visible_height as u32);
        let color_space_init = object_property(scope, options, "colorSpace");
        let color_space = create_color_space(scope, color_space_init);
        let Ok(color_space) = color_space else {
            crate::webidl::throw_type_error(scope, "cannot create VideoFrame colorSpace");
            return;
        };
        VideoFrameRecord {
            source,
            format,
            timestamp,
            duration: optional_number(scope, options, "duration"),
            coded_width,
            coded_height,
            visible_x,
            visible_y,
            visible_width,
            visible_height,
            rotation,
            flip,
            display_width,
            display_height,
            color_space: v8::Global::new(scope, color_space),
            closed: false,
        }
    };
    attach(scope, arguments.this(), record);
    result.set(arguments.this().into());
}

fn create_color_space<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    init: Option<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let primaries = init.and_then(|init| optional_string(scope, init, "primaries"));
    let transfer = init.and_then(|init| optional_string(scope, init, "transfer"));
    let matrix = init.and_then(|init| optional_string(scope, init, "matrix"));
    let full_range = init.and_then(|init| optional_boolean(scope, init, "fullRange"));
    super::video_color_space::create(scope, primaries, transfer, matrix, full_range)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    record: VideoFrameRecord,
) {
    if let Some(store) = scope.get_slot_mut::<VideoFrameStore>() {
        store
            .records
            .insert(object.get_identity_hash().get(), record);
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<VideoFrameRecord> {
    scope
        .get_slot::<VideoFrameStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_video_frame(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<VideoFrameStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn bitmap_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = record(scope, object)?;
    if record.closed || record.display_width == 0 || record.display_height == 0 {
        return None;
    }
    let length = usize::try_from(record.display_width)
        .ok()?
        .checked_mul(usize::try_from(record.display_height).ok()?)?
        .checked_mul(4)?;
    Some((record.display_width, record.display_height, vec![0; length]))
}

pub(crate) fn encoding_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<VideoFrameEncodingSnapshot> {
    let record = live_record(scope, object)?;
    let (_, _, bytes) = bitmap_snapshot(scope, object)?;
    Some(VideoFrameEncodingSnapshot {
        format: record.format,
        timestamp: record.timestamp,
        duration: record.duration,
        coded_width: record.coded_width,
        coded_height: record.coded_height,
        display_width: record.display_width,
        display_height: record.display_height,
        bytes,
    })
}

pub(crate) fn create_from_encoding_snapshot<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    snapshot: VideoFrameEncodingSnapshot,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create decoded VideoFrame".to_owned());
    }
    let byte_length = snapshot.bytes.len();
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(snapshot.bytes.clone()).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    let source = v8::Uint8Array::new(scope, buffer, 0, byte_length)
        .ok_or_else(|| "cannot create decoded VideoFrame storage".to_owned())?;
    let source: v8::Local<'_, v8::Object> = source.into();
    let color_space = create_color_space(scope, None)?;
    attach(
        scope,
        object,
        VideoFrameRecord {
            source: Some(v8::Global::new(scope, source)),
            format: snapshot.format,
            timestamp: snapshot.timestamp,
            duration: snapshot.duration,
            coded_width: snapshot.coded_width,
            coded_height: snapshot.coded_height,
            visible_x: 0.0,
            visible_y: 0.0,
            visible_width: snapshot.coded_width as f64,
            visible_height: snapshot.coded_height as f64,
            rotation: 0,
            flip: false,
            display_width: snapshot.display_width,
            display_height: snapshot.display_height,
            color_space: v8::Global::new(scope, color_space),
            closed: false,
        },
    );
    Ok(object)
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut VideoFrameRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<VideoFrameStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn live_record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<VideoFrameRecord> {
    record(scope, object).filter(|record| !record.closed)
}

fn get_format(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if v.closed {
        r.set(v8::null(s).into())
    } else if let Some(value) = v8::String::new(s, &v.format) {
        r.set(value.into())
    }
}
fn get_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = live_record(s, a.this()) {
        r.set(v8::Number::new(s, v.timestamp).into())
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}
fn get_duration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = live_record(s, a.this()) {
        if let Some(value) = v.duration {
            r.set(v8::Number::new(s, value).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}

fn return_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&VideoFrameRecord) -> u32,
) {
    if let Some(record) = live_record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, select(&record)).into())
    } else {
        crate::webidl::throw_type_error(scope, "VideoFrame is closed")
    }
}
fn get_coded_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |v| v.coded_width)
}
fn get_coded_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |v| v.coded_height)
}
fn get_rotation(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |v| v.rotation)
}
fn get_display_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |v| v.display_width)
}
fn get_display_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |v| v.display_height)
}

fn get_flip(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = live_record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.flip).into())
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}

fn define_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}
fn rect<'s>(
    scope: &v8::PinScope<'s, '_>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    let x_value = v8::Number::new(scope, x);
    define_data(scope, object, "x", x_value.into());
    let y_value = v8::Number::new(scope, y);
    define_data(scope, object, "y", y_value.into());
    let width_value = v8::Number::new(scope, width);
    define_data(scope, object, "width", width_value.into());
    let height_value = v8::Number::new(scope, height);
    define_data(scope, object, "height", height_value.into());
    object
}
fn get_coded_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = live_record(s, a.this()) {
        r.set(rect(s, 0.0, 0.0, v.coded_width as f64, v.coded_height as f64).into())
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}
fn get_visible_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = live_record(s, a.this()) {
        r.set(
            rect(
                s,
                v.visible_x,
                v.visible_y,
                v.visible_width,
                v.visible_height,
            )
            .into(),
        )
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}

fn get_color_space(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = live_record(s, a.this()) {
        r.set(v8::Local::new(s, &v.color_space).into())
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}

fn byte_size(record: &VideoFrameRecord) -> u32 {
    let pixels = record.coded_width.saturating_mul(record.coded_height);
    match record.format.as_str() {
        "I420" | "I420A" | "NV12" => pixels.saturating_mul(3).saturating_div(2),
        "RGBA" | "RGBX" | "BGRA" | "BGRX" => pixels.saturating_mul(4),
        _ => pixels.saturating_mul(4),
    }
}
fn allocation_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = live_record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, byte_size(&v)).into())
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}

fn clone_frame(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = live_record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed");
        return;
    };
    let Ok(constructor) = ensure_constructor(s) else {
        return;
    };
    let Ok(prototype) = crate::webidl::prototype(s, constructor) else {
        return;
    };
    let object = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, object, prototype.into()) == Some(true) {
        attach(s, object, v);
        r.set(object.into())
    }
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(s, a.this(), |v| {
        v.closed = true;
        v.source = None
    })
}

fn copy_to(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = live_record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed");
        return;
    };
    let Ok(destination) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "copyTo requires a destination buffer");
        return;
    };
    let size = byte_size(&v);
    let source = v.source.as_ref().map(|source| v8::Local::new(s, source));
    for index in 0..size {
        let value = source
            .and_then(|source| source.get_index(s, index))
            .unwrap_or_else(|| v8::Integer::new(s, 0).into());
        let _ = destination.set_index(s, index, value);
    }
    let layouts = v8::Array::new(s, 1);
    let layout = v8::Object::new(s);
    let offset = v8::Integer::new(s, 0);
    define_data(s, layout, "offset", offset.into());
    let stride = v8::Integer::new_from_unsigned(s, v.coded_width);
    define_data(s, layout, "stride", stride.into());
    let _ = layouts.set_index(s, 0, layout.into());
    let Some(resolver) = v8::PromiseResolver::new(s) else {
        return;
    };
    let promise = resolver.get_promise(s);
    let _ = resolver.resolve(s, layouts.into());
    r.set(promise.into())
}
fn metadata(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if live_record(s, a.this()).is_some() {
        r.set(v8::Object::new(s).into())
    } else {
        crate::webidl::throw_type_error(s, "VideoFrame is closed")
    }
}

fn optional_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_null() && !value.is_undefined()).then_some(value)
}
fn optional_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    optional_value(scope, object, name).map(|value| crate::webidl::value_to_string(scope, value))
}
fn optional_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    optional_value(scope, object, name)?.number_value(scope)
}
fn optional_u32(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u32> {
    optional_value(scope, object, name)?.uint32_value(scope)
}
fn optional_boolean(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<bool> {
    Some(optional_value(scope, object, name)?.boolean_value(scope))
}
fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    v8::Local::<v8::Object>::try_from(optional_value(scope, object, name)?).ok()
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<VideoFrameStore>() {
        store.constructor.remove(realm_id);
    }
}
