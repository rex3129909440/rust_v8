pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function = crate::webidl::create_function(
        scope,
        "createImageBitmap",
        1,
        v8::ConstructorBehavior::Throw,
        execute,
    )?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "createImageBitmap")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.createImageBitmap".to_owned())
    }
}

pub(crate) enum CanvasImageSourceError {
    InvalidType,
    Unusable,
    Decode,
}

pub(crate) fn execute(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'createImageBitmap' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(source) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        reject_type_error(scope, invalid_source_message(), result);
        return;
    };
    let (source_width, source_height, source_pixels) = match source_pixels(scope, source) {
        Ok(snapshot) => snapshot,
        Err(CanvasImageSourceError::InvalidType) => {
            reject_type_error(scope, invalid_source_message(), result);
            return;
        }
        Err(CanvasImageSourceError::Unusable) => {
            reject_dom_exception(
                scope,
                "InvalidStateError",
                "Failed to execute 'createImageBitmap' on 'Window': The image source is not usable.",
                result,
            );
            return;
        }
        Err(CanvasImageSourceError::Decode) => {
            reject_dom_exception(
                scope,
                "InvalidStateError",
                "The source image could not be decoded.",
                result,
            );
            return;
        }
    };
    let (mut width, mut height, mut pixels) = if arguments.length() >= 5 {
        let source_x = arguments.get(1).int32_value(scope).unwrap_or(0);
        let source_y = arguments.get(2).int32_value(scope).unwrap_or(0);
        let crop_width = arguments.get(3).int32_value(scope).unwrap_or(0);
        let crop_height = arguments.get(4).int32_value(scope).unwrap_or(0);
        if crop_width == 0 {
            reject_range_error(
                scope,
                "Failed to execute 'createImageBitmap' on 'Window': The crop rect width is 0.",
                result,
            );
            return;
        }
        if crop_height == 0 {
            reject_range_error(
                scope,
                "Failed to execute 'createImageBitmap' on 'Window': The crop rect height is 0.",
                result,
            );
            return;
        }
        crop_pixels(
            source_width,
            source_height,
            &source_pixels,
            source_x,
            source_y,
            crop_width,
            crop_height,
        )
    } else {
        (source_width, source_height, source_pixels)
    };
    let options_index = if arguments.length() >= 5 { 5 } else { 1 };
    if let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(options_index)) {
        let image_orientation =
            enum_property(scope, options, "imageOrientation", &["from-image", "flipY"]);
        let premultiply_alpha = enum_property(
            scope,
            options,
            "premultiplyAlpha",
            &["none", "premultiply", "default"],
        );
        let color_space_conversion =
            enum_property(scope, options, "colorSpaceConversion", &["none", "default"]);
        let resize_quality = enum_property(
            scope,
            options,
            "resizeQuality",
            &["pixelated", "low", "medium", "high"],
        );
        if image_orientation.is_err()
            || premultiply_alpha.is_err()
            || color_space_conversion.is_err()
            || resize_quality.is_err()
        {
            reject_type_error(
                scope,
                "The provided ImageBitmapOptions value is invalid.",
                result,
            );
            return;
        }
        let requested_width = unsigned_property(scope, options, "resizeWidth");
        let requested_height = unsigned_property(scope, options, "resizeHeight");
        let (resize_width, resize_height) = match (requested_width, requested_height) {
            (Some(resize_width), Some(resize_height)) => (resize_width, resize_height),
            (Some(resize_width), None) => (
                resize_width,
                scaled_axis(height, resize_width, width).max(1),
            ),
            (None, Some(resize_height)) => (
                scaled_axis(width, resize_height, height).max(1),
                resize_height,
            ),
            (None, None) => (width, height),
        };
        if resize_width == 0 || resize_height == 0 {
            reject_dom_exception(
                scope,
                "InvalidStateError",
                "Failed to execute 'createImageBitmap' on 'Window': The ImageBitmap could not be allocated.",
                result,
            );
            return;
        }
        if resize_width != width || resize_height != height {
            pixels = resize_pixels(&pixels, width, height, resize_width, resize_height);
            width = resize_width;
            height = resize_height;
        }
        if image_orientation
            .ok()
            .flatten()
            .is_some_and(|value| value == "flipY")
        {
            flip_rows(&mut pixels, width, height);
        }
    }
    match super::image_bitmap::create(scope, width, height, pixels) {
        Ok(bitmap) => {
            if let Ok(promise) = super::writable_stream::resolved_promise(scope, bitmap.into()) {
                result.set(promise.into());
            }
        }
        Err(message) => reject_type_error(scope, &message, result),
    }
}

pub(crate) fn source_pixels(
    scope: &v8::PinScope<'_, '_>,
    source: v8::Local<'_, v8::Object>,
) -> Result<(u32, u32, Vec<u8>), CanvasImageSourceError> {
    if let Some(snapshot) = super::image_data::snapshot(scope, source) {
        return usable_snapshot(snapshot);
    }
    if super::image_bitmap::is_image_bitmap(scope, source) {
        return super::image_bitmap::snapshot(scope, source)
            .ok_or(CanvasImageSourceError::Unusable)
            .and_then(usable_snapshot);
    }
    if super::html_canvas_element::record(scope, source).is_some() {
        return super::html_canvas_element::bitmap_snapshot(scope, source)
            .ok_or(CanvasImageSourceError::Unusable)
            .and_then(usable_snapshot);
    }
    if super::html_image_element::record(scope, source).is_some() {
        return super::html_image_element::bitmap_snapshot(scope, source)
            .ok_or(CanvasImageSourceError::Unusable)
            .and_then(usable_snapshot);
    }
    if super::html_video_element::record(scope, source).is_some() {
        let record = super::html_video_element::record(scope, source)
            .ok_or(CanvasImageSourceError::Unusable)?;
        return transparent_snapshot(record.video_width, record.video_height);
    }
    if super::svg_image_element::record(scope, source).is_some() {
        return Err(CanvasImageSourceError::Unusable);
    }
    if super::video_frame::is_video_frame(scope, source) {
        return super::video_frame::bitmap_snapshot(scope, source)
            .ok_or(CanvasImageSourceError::Unusable)
            .and_then(usable_snapshot);
    }
    if super::offscreen_canvas::dimensions(scope, source).is_some() {
        return super::offscreen_canvas::pixel_snapshot(scope, source)
            .ok_or(CanvasImageSourceError::Unusable)
            .and_then(usable_snapshot);
    }
    if let Some((bytes, media_type)) = super::blob::byte_snapshot(scope, source) {
        if media_type
            .split(';')
            .next()
            .is_some_and(|value| value.trim().eq_ignore_ascii_case("image/svg+xml"))
        {
            return Err(CanvasImageSourceError::Decode);
        }
        let (width, height) = super::html_image_element::image_dimensions(&bytes, &media_type)
            .ok_or(CanvasImageSourceError::Decode)?;
        return transparent_snapshot(width, height);
    }
    Err(CanvasImageSourceError::InvalidType)
}

fn usable_snapshot(
    snapshot: (u32, u32, Vec<u8>),
) -> Result<(u32, u32, Vec<u8>), CanvasImageSourceError> {
    if snapshot.0 == 0 || snapshot.1 == 0 {
        Err(CanvasImageSourceError::Unusable)
    } else {
        Ok(snapshot)
    }
}

fn transparent_snapshot(
    width: u32,
    height: u32,
) -> Result<(u32, u32, Vec<u8>), CanvasImageSourceError> {
    if width == 0 || height == 0 {
        return Err(CanvasImageSourceError::Unusable);
    }
    let length = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|length| length.checked_mul(4))
        .filter(|length| *length <= 256 * 1024 * 1024)
        .ok_or(CanvasImageSourceError::Unusable)?;
    Ok((width, height, vec![0; length]))
}

fn crop_pixels(
    source_width: u32,
    source_height: u32,
    source: &[u8],
    source_x: i32,
    source_y: i32,
    crop_width: i32,
    crop_height: i32,
) -> (u32, u32, Vec<u8>) {
    let width = crop_width.unsigned_abs();
    let height = crop_height.unsigned_abs();
    let start_x = if crop_width < 0 {
        source_x.saturating_add(crop_width)
    } else {
        source_x
    };
    let start_y = if crop_height < 0 {
        source_y.saturating_add(crop_height)
    } else {
        source_y
    };
    let mut destination = vec![0; width.saturating_mul(height).saturating_mul(4) as usize];
    for destination_y in 0..height {
        for destination_x in 0..width {
            let input_x = start_x.saturating_add(destination_x as i32);
            let input_y = start_y.saturating_add(destination_y as i32);
            if input_x < 0
                || input_y < 0
                || input_x as u32 >= source_width
                || input_y as u32 >= source_height
            {
                continue;
            }
            let input_offset = ((input_y as u32 * source_width + input_x as u32) * 4) as usize;
            let output_offset = ((destination_y * width + destination_x) * 4) as usize;
            destination[output_offset..output_offset + 4]
                .copy_from_slice(&source[input_offset..input_offset + 4]);
        }
    }
    (width, height, destination)
}

fn resize_pixels(
    source: &[u8],
    width: u32,
    height: u32,
    new_width: u32,
    new_height: u32,
) -> Vec<u8> {
    let mut destination = vec![0; new_width.saturating_mul(new_height).saturating_mul(4) as usize];
    for output_y in 0..new_height {
        for output_x in 0..new_width {
            let input_x = output_x.saturating_mul(width) / new_width;
            let input_y = output_y.saturating_mul(height) / new_height;
            let input_offset = ((input_y * width + input_x) * 4) as usize;
            let output_offset = ((output_y * new_width + output_x) * 4) as usize;
            destination[output_offset..output_offset + 4]
                .copy_from_slice(&source[input_offset..input_offset + 4]);
        }
    }
    destination
}

fn unsigned_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<u32> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        value.uint32_value(scope)
    }
}

fn enum_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    values: &[&str],
) -> Result<Option<String>, ()> {
    let Some(key) = v8::String::new(scope, name) else {
        return Err(());
    };
    let Some(value) = object.get(scope, key.into()) else {
        return Err(());
    };
    if value.is_undefined() {
        return Ok(None);
    }
    let value = crate::webidl::value_to_string(scope, value);
    values
        .contains(&value.as_str())
        .then_some(Some(value))
        .ok_or(())
}

fn scaled_axis(value: u32, requested_other: u32, original_other: u32) -> u32 {
    if original_other == 0 {
        return 0;
    }
    (f64::from(value) * f64::from(requested_other) / f64::from(original_other))
        .round()
        .clamp(0.0, f64::from(u32::MAX)) as u32
}

fn flip_rows(pixels: &mut [u8], width: u32, height: u32) {
    let row_length = width as usize * 4;
    for top in 0..height / 2 {
        let bottom = height - top - 1;
        let top_offset = top as usize * row_length;
        let bottom_offset = bottom as usize * row_length;
        for offset in 0..row_length {
            pixels.swap(top_offset + offset, bottom_offset + offset);
        }
    }
}

fn invalid_source_message() -> &'static str {
    "Failed to execute 'createImageBitmap' on 'Window': The provided value is not of type '(Blob or HTMLCanvasElement or HTMLImageElement or HTMLVideoElement or ImageBitmap or ImageData or OffscreenCanvas or SVGImageElement or VideoFrame)'."
}

fn reject_dom_exception(
    scope: &mut v8::PinScope<'_, '_>,
    name: &str,
    message: &str,
    result: v8::ReturnValue<'_>,
) {
    let exception = super::dom_exception::create(scope, message.to_owned(), name.to_owned())
        .map(Into::into)
        .unwrap_or_else(|_| v8::undefined(scope).into());
    reject(scope, exception, result);
}

fn reject_range_error(
    scope: &mut v8::PinScope<'_, '_>,
    message: &str,
    result: v8::ReturnValue<'_>,
) {
    let message = v8::String::new(scope, message).expect("short createImageBitmap range error");
    let exception = v8::Exception::range_error(scope, message);
    reject(scope, exception, result);
}

fn reject_type_error(scope: &mut v8::PinScope<'_, '_>, message: &str, result: v8::ReturnValue<'_>) {
    let message = v8::String::new(scope, message).expect("short createImageBitmap error");
    let exception = v8::Exception::type_error(scope, message);
    reject(scope, exception, result);
}

fn reject(
    scope: &mut v8::PinScope<'_, '_>,
    exception: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
        result.set(promise.into());
    }
}
