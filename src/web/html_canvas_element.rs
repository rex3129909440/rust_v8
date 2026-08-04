use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct CanvasRecord {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) context_kind: Option<String>,
    pub(crate) context_2d: Option<v8::Global<v8::Object>>,
    pub(crate) context_webgl: Option<v8::Global<v8::Object>>,
    pub(crate) context_webgl2: Option<v8::Global<v8::Object>>,
    pub(crate) context_bitmap: Option<v8::Global<v8::Object>>,
    pub(crate) transferred: bool,
    pub(crate) transferred_canvas: Option<v8::Global<v8::Object>>,
}

#[derive(Default)]
pub(crate) struct HtmlCanvasElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, CanvasRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlCanvasElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLCanvasElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlCanvasElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLCanvasElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_canvas_element_width_property::define(scope, prototype)?;
    super::html_canvas_element_height_property::define(scope, prototype)?;
    super::html_canvas_element_capture_stream::define(scope, prototype)?;
    super::html_canvas_element_get_context::define(scope, prototype)?;
    super::html_canvas_element_to_blob::define(scope, prototype)?;
    super::html_canvas_element_to_data_url::define(scope, prototype)?;
    super::html_canvas_element_transfer_control_to_offscreen::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlCanvasElementStore>()
        .ok_or_else(|| "HTMLCanvasElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLCanvasElement".to_owned());
    }
    super::html_element::attach(scope, object, "CANVAS");
    super::offscreen_canvas::attach_external(scope, object, 300, 150);
    scope
        .get_slot_mut::<HtmlCanvasElementStore>()
        .ok_or_else(|| "HTMLCanvasElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CanvasRecord {
                width: 300,
                height: 150,
                context_kind: None,
                context_2d: None,
                context_webgl: None,
                context_webgl2: None,
                context_bitmap: None,
                transferred: false,
                transferred_canvas: None,
            },
        );
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CanvasRecord> {
    scope
        .get_slot::<HtmlCanvasElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn throw_dom_exception(scope: &mut v8::PinScope<'_, '_>, name: &str, message: &str) {
    match super::dom_exception::create(scope, message.to_owned(), name.to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(_) => crate::webidl::throw_type_error(scope, message),
    }
}

pub(crate) fn dimension(scope: &mut v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> u32 {
    value.uint32_value(scope).unwrap_or(0)
}

pub(crate) fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = dimension(scope, arguments.get(0));
    resize(scope, arguments.this(), Some(width), None);
}

pub(crate) fn get_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.height).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let height = dimension(scope, arguments.get(0));
    resize(scope, arguments.this(), None, Some(height));
}

pub(crate) fn resize(
    scope: &mut v8::PinScope<'_, '_>,
    canvas: v8::Local<'_, v8::Object>,
    width: Option<u32>,
    height: Option<u32>,
) {
    let snapshot = if let Some(record) = scope
        .get_slot_mut::<HtmlCanvasElementStore>()
        .and_then(|store| store.records.get_mut(&canvas.get_identity_hash().get()))
    {
        if let Some(width) = width {
            record.width = width;
        }
        if let Some(height) = height {
            record.height = height;
        }
        record.clone()
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::offscreen_canvas::resize_external(scope, canvas, snapshot.width, snapshot.height);
    if let Some(context) = snapshot.context_2d {
        let context = v8::Local::new(scope, &context);
        super::offscreen_canvas_rendering_context_2d::reset_for_resize(scope, context);
    }
    if let Some(offscreen) = snapshot.transferred_canvas {
        let offscreen = v8::Local::new(scope, &offscreen);
        super::offscreen_canvas::resize_external(scope, offscreen, snapshot.width, snapshot.height);
    }
}

pub(crate) fn capture_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.transferred {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Cannot capture a canvas after control has been transferred",
        );
        return;
    }
    if !arguments.get(0).is_undefined() && arguments.get(0).number_value(scope).unwrap_or(0.0) < 0.0
    {
        throw_dom_exception(
            scope,
            "NotSupportedError",
            "The frame rate cannot be negative",
        );
        return;
    }
    let track = match super::canvas_capture_media_stream_track::create(scope, arguments.this()) {
        Ok(track) => track,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    match super::media_stream::create_with_tracks(scope, &[track]) {
        Ok(stream) => result.set(stream.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn existing_context<'s>(
    scope: &v8::PinScope<'s, '_>,
    record: &CanvasRecord,
    kind: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let context = match kind {
        "2d" => record.context_2d.as_ref(),
        "webgl" => record.context_webgl.as_ref(),
        "webgl2" => record.context_webgl2.as_ref(),
        "bitmaprenderer" => record.context_bitmap.as_ref(),
        _ => None,
    }?;
    Some(v8::Local::new(scope, context))
}

pub(crate) fn get_context(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.transferred {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Cannot get a context after control has been transferred",
        );
        return;
    }
    let requested = crate::webidl::value_to_string(scope, arguments.get(0));
    let kind = match requested.as_str() {
        "2d" => "2d",
        "webgl" | "experimental-webgl" => "webgl",
        "webgl2" => "webgl2",
        "bitmaprenderer" => "bitmaprenderer",
        _ => {
            result.set(v8::null(scope).into());
            return;
        }
    };
    if snapshot
        .context_kind
        .as_ref()
        .is_some_and(|current| current != kind)
    {
        result.set(v8::null(scope).into());
        return;
    }
    if let Some(context) = existing_context(scope, &snapshot, kind) {
        result.set(context.into());
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let created = match kind {
        "2d" => super::canvas_rendering_context_2d::create(scope, arguments.this(), options),
        "webgl" => super::webgl_rendering_context::create(
            scope,
            Some(arguments.this()),
            snapshot.width,
            snapshot.height,
        ),
        "webgl2" => super::webgl2_rendering_context::create(
            scope,
            Some(arguments.this()),
            snapshot.width,
            snapshot.height,
        ),
        "bitmaprenderer" => super::image_bitmap_rendering_context::create(scope, arguments.this()),
        _ => unreachable!(),
    };
    let context = match created {
        Ok(context) => context,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let stored = v8::Global::new(scope, context);
    if let Some(record) = scope
        .get_slot_mut::<HtmlCanvasElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.context_kind = Some(kind.to_owned());
        match kind {
            "2d" => record.context_2d = Some(stored),
            "webgl" => record.context_webgl = Some(stored),
            "webgl2" => record.context_webgl2 = Some(stored),
            "bitmaprenderer" => record.context_bitmap = Some(stored),
            _ => {}
        }
    }
    result.set(context.into());
}

pub(crate) fn canvas_pixels(
    scope: &v8::PinScope<'_, '_>,
    record: &CanvasRecord,
) -> Option<Vec<u8>> {
    let context = record.context_2d.as_ref()?;
    let context = v8::Local::new(scope, context);
    super::offscreen_canvas_rendering_context_2d::pixel_snapshot(scope, context)
        .map(|(_, _, pixels)| pixels)
}

pub(crate) fn bitmap_snapshot(
    scope: &v8::PinScope<'_, '_>,
    canvas: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = record(scope, canvas)?;
    if record.transferred {
        return None;
    }
    let pixels = if let Some(context) = record.context_2d {
        let context = v8::Local::new(scope, context);
        super::offscreen_canvas_rendering_context_2d::pixel_snapshot(scope, context)?.2
    } else if let Some(context) = record.context_bitmap {
        let context = v8::Local::new(scope, context);
        super::image_bitmap_rendering_context::snapshot(
            scope,
            context,
            record.width,
            record.height,
        )?
        .2
    } else {
        let length = usize::try_from(record.width)
            .ok()?
            .checked_mul(usize::try_from(record.height).ok()?)?
            .checked_mul(4)?;
        vec![0; length]
    };
    Some((record.width, record.height, pixels))
}

pub(crate) fn transfer_control_to_offscreen(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(snapshot) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if snapshot.transferred || snapshot.context_kind.is_some() {
        throw_dom_exception(
            scope,
            "InvalidStateError",
            "Cannot transfer control from a canvas that has a context or was already transferred",
        );
        return;
    }
    let offscreen = match super::offscreen_canvas::create(scope, snapshot.width, snapshot.height) {
        Ok(offscreen) => offscreen,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let stored = v8::Global::new(scope, offscreen);
    if let Some(record) = scope
        .get_slot_mut::<HtmlCanvasElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.transferred = true;
        record.transferred_canvas = Some(stored);
    }
    result.set(offscreen.into());
}

pub(crate) fn png_bytes(width: u32, height: u32, pixels: Option<&[u8]>) -> Option<Vec<u8>> {
    let row_bytes = (width as usize).checked_mul(4)?;
    let raw_length = row_bytes.checked_add(1)?.checked_mul(height as usize)?;
    if raw_length > 128 * 1024 * 1024 {
        return None;
    }
    let expected_pixels = row_bytes.checked_mul(height as usize)?;
    let pixels = pixels.filter(|pixels| pixels.len() >= expected_pixels);
    let mut raw = Vec::with_capacity(raw_length);
    for row in 0..height as usize {
        raw.push(0);
        if let Some(pixels) = pixels {
            let start = row * row_bytes;
            raw.extend_from_slice(&pixels[start..start + row_bytes]);
        } else {
            raw.resize(raw.len() + row_bytes, 0);
        }
    }

    let mut compressed = Vec::with_capacity(raw.len() + raw.len() / 65_535 * 5 + 16);
    compressed.extend_from_slice(&[0x78, 0x01]);
    let mut offset = 0;
    while offset < raw.len() {
        let length = (raw.len() - offset).min(65_535);
        let final_block = offset + length == raw.len();
        compressed.push(if final_block { 1 } else { 0 });
        compressed.extend_from_slice(&(length as u16).to_le_bytes());
        compressed.extend_from_slice(&(!(length as u16)).to_le_bytes());
        compressed.extend_from_slice(&raw[offset..offset + length]);
        offset += length;
    }
    compressed.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::new();
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    append_chunk(&mut png, b"IHDR", &header);
    append_chunk(&mut png, b"IDAT", &compressed);
    append_chunk(&mut png, b"IEND", &[]);
    Some(png)
}

pub(crate) fn fingerprinted_png_bytes(
    scope: &v8::PinScope<'_, '_>,
    width: u32,
    height: u32,
    pixels: Option<&[u8]>,
) -> Option<Vec<u8>> {
    let mut png = png_bytes(width, height, pixels)?;
    let salt = &crate::fingerprint::edge(scope)
        .rendering
        .canvas
        .data_url_salt;
    if salt.is_empty() {
        return Some(png);
    }
    if png.len() < 12 {
        return None;
    }
    png.truncate(png.len() - 12);
    let mut data = b"edge-fingerprint\0".to_vec();
    data.extend_from_slice(salt.as_bytes());
    append_chunk(&mut png, b"tEXt", &data);
    append_chunk(&mut png, b"IEND", &[]);
    Some(png)
}

pub(crate) fn append_chunk(output: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_be_bytes());
    output.extend_from_slice(kind);
    output.extend_from_slice(data);
    let mut checksum_input = Vec::with_capacity(4 + data.len());
    checksum_input.extend_from_slice(kind);
    checksum_input.extend_from_slice(data);
    output.extend_from_slice(&crc32(&checksum_input).to_be_bytes());
}

pub(crate) fn adler32(data: &[u8]) -> u32 {
    let mut a = 1_u32;
    let mut b = 0_u32;
    for byte in data {
        a = (a + *byte as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

pub(crate) fn encode_base64(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut offset = 0;
    while offset < data.len() {
        let first = data[offset];
        let second = data.get(offset + 1).copied();
        let third = data.get(offset + 2).copied();
        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 0x03) << 4) | second.unwrap_or(0) >> 4) as usize] as char);
        if let Some(second) = second {
            output.push(
                ALPHABET[(((second & 0x0f) << 2) | third.unwrap_or(0) >> 6) as usize] as char,
            );
        } else {
            output.push('=');
        }
        if let Some(third) = third {
            output.push(ALPHABET[(third & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
        offset += 3;
    }
    output
}
