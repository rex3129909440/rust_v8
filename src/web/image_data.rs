use std::collections::HashMap;

// Keep allocation failure in the Web API error channel. Chromium rejects
// dimensions outside its supported backing-store range instead of allowing an
// allocator abort to terminate the renderer process. The sandbox worker has a
// smaller fixed memory envelope, so it applies that limit before allocation.
const MAX_IMAGE_DATA_BYTES: usize = 512 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct ImageDataStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ImageDataRecord>,
}

#[derive(Clone)]
struct ImageDataRecord {
    width: u32,
    height: u32,
    color_space: String,
    pixel_format: String,
    data: v8::Global<v8::Uint8ClampedArray>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ImageDataStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ImageData", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<ImageDataStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ImageData",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "width", get_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "height", get_height)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "colorSpace", get_color_space)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "data", get_data)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pixelFormat", get_pixel_format)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ImageDataStore>()
        .ok_or_else(|| "ImageData state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'ImageData': 2 arguments required",
        );
        return;
    }
    if let Some(message) = crate::webidl::number_conversion_error(arguments.get(0)) {
        crate::webidl::throw_type_error(scope, &message);
        return;
    }
    if let Ok(array) = v8::Local::<v8::Uint8ClampedArray>::try_from(arguments.get(0)) {
        let Some(width) = arguments.get(1).uint32_value(scope) else {
            return;
        };
        if width == 0 {
            throw_index_size(
                scope,
                "Failed to construct 'ImageData': The source width is zero or not a number.",
            );
            return;
        }
        let length = array.length();
        let height = if arguments.get(2).is_undefined() {
            let row_length = width as usize * 4;
            if length % row_length != 0 {
                throw_index_size(
                    scope,
                    "The input data length is not a multiple of (4 * width).",
                );
                return;
            }
            (length / row_length) as u32
        } else {
            let Some(height) = arguments.get(2).uint32_value(scope) else {
                return;
            };
            height
        };
        if height == 0 {
            throw_index_size(scope, "The source height is zero or not a number.");
            return;
        }
        if supported_byte_length(width, height).is_none() {
            throw_supported_range(scope);
            return;
        }
        if length != width as usize * height as usize * 4 {
            throw_index_size(
                scope,
                "The input data length does not match width and height.",
            );
            return;
        }
        let settings = v8::Local::<v8::Object>::try_from(arguments.get(3)).ok();
        let Some(color_space) = validated_setting(scope, settings, "colorSpace", "srgb") else {
            return;
        };
        let Some(pixel_format) = validated_setting(scope, settings, "pixelFormat", "rgba-unorm8")
        else {
            return;
        };
        if pixel_format != "rgba-unorm8" {
            crate::webidl::throw_type_error(
                scope,
                "ImageData with Uint8ClampedArray requires pixelFormat 'rgba-unorm8'",
            );
            return;
        }
        let data = v8::Global::new(scope, array);
        match attach_array(
            scope,
            arguments.this(),
            width,
            height,
            color_space,
            pixel_format,
            data,
        ) {
            Ok(()) => result.set(arguments.this().into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        }
        return;
    }
    let Some(width) = arguments.get(0).uint32_value(scope) else {
        return;
    };
    let Some(height) = arguments.get(1).uint32_value(scope) else {
        return;
    };
    if width == 0 {
        throw_index_size(
            scope,
            "Failed to construct 'ImageData': The source width is zero or not a number.",
        );
        return;
    }
    if height == 0 {
        throw_index_size(scope, "The source height is zero or not a number.");
        return;
    }
    let Some(byte_length) = supported_byte_length(width, height) else {
        throw_supported_range(scope);
        return;
    };
    let settings = v8::Local::<v8::Object>::try_from(arguments.get(2)).ok();
    let Some(color_space) = validated_setting(scope, settings, "colorSpace", "srgb") else {
        return;
    };
    let Some(pixel_format) = validated_setting(scope, settings, "pixelFormat", "rgba-unorm8")
    else {
        return;
    };
    if pixel_format != "rgba-unorm8" {
        crate::webidl::throw_type_error(
            scope,
            "ImageData without a Float16Array requires pixelFormat 'rgba-unorm8'",
        );
        return;
    }
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(byte_length).is_err() {
        throw_supported_range(scope);
        return;
    }
    bytes.resize(byte_length, 0);
    match attach(scope, arguments.this(), width, height, bytes, color_space) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn supported_byte_length(width: u32, height: u32) -> Option<usize> {
    let bytes = usize::try_from(width)
        .ok()?
        .checked_mul(usize::try_from(height).ok()?)?
        .checked_mul(4)?;
    (bytes <= MAX_IMAGE_DATA_BYTES).then_some(bytes)
}

fn throw_supported_range(scope: &mut v8::PinScope<'_, '_>) {
    throw_index_size(
        scope,
        "Failed to construct 'ImageData': The requested image size exceeds the supported range.",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    color_space: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if width == 0 || height == 0 || bytes.len() != width as usize * height as usize * 4 {
        return Err("ImageData dimensions do not match its storage".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let image_data = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, image_data, prototype.into()) != Some(true) {
        return Err("cannot create ImageData".to_owned());
    }
    attach(
        scope,
        image_data,
        width,
        height,
        bytes,
        color_space.to_owned(),
    )?;
    Ok(image_data)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    color_space: String,
) -> Result<(), String> {
    let length = bytes.len();
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    let data = v8::Uint8ClampedArray::new(scope, buffer, 0, length)
        .ok_or_else(|| "cannot create ImageData pixel storage".to_owned())?;
    let data = v8::Global::new(scope, data);
    attach_array(
        scope,
        object,
        width,
        height,
        color_space,
        "rgba-unorm8".to_owned(),
        data,
    )
}

fn attach_array(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    width: u32,
    height: u32,
    color_space: String,
    pixel_format: String,
    data: v8::Global<v8::Uint8ClampedArray>,
) -> Result<(), String> {
    scope
        .get_slot_mut::<ImageDataStore>()
        .ok_or_else(|| "ImageData state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ImageDataRecord {
                width,
                height,
                color_space,
                pixel_format,
                data,
            },
        );
    Ok(())
}

fn validated_setting(
    scope: &v8::PinScope<'_, '_>,
    settings: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: &str,
) -> Option<String> {
    let Some(settings) = settings else {
        return Some(default.to_owned());
    };
    let Some(key) = v8::String::new(scope, name) else {
        return Some(default.to_owned());
    };
    let value = settings
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_else(|| default.to_owned());
    let valid = match name {
        "colorSpace" => matches!(value.as_str(), "srgb" | "display-p3"),
        "pixelFormat" => matches!(value.as_str(), "rgba-unorm8" | "rgba-float16"),
        _ => true,
    };
    if valid {
        Some(value)
    } else {
        crate::webidl::throw_type_error(
            scope,
            &format!("The provided value '{value}' is not a valid {name} value"),
        );
        None
    }
}

fn throw_index_size(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Ok(exception) =
        super::dom_exception::create(scope, message.to_owned(), "IndexSizeError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ImageDataRecord> {
    scope
        .get_slot::<ImageDataStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = record(scope, object)?;
    let data = v8::Local::new(scope, &record.data);
    let mut bytes = vec![0_u8; data.byte_length()];
    let _ = data.copy_contents(&mut bytes);
    Some((record.width, record.height, bytes))
}

fn get_width(
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
fn get_height(
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
fn get_color_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.color_space) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_pixel_format(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.pixel_format) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.data).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ImageDataStore>() {
        store.constructor.remove(realm_id);
    }
}
