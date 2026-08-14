use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlVideoElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, VideoRecord>,
}

#[derive(Clone)]
pub(crate) struct VideoRecord {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) video_width: u32,
    pub(crate) video_height: u32,
    pub(crate) poster: String,
    pub(crate) decoded_frames: u64,
    pub(crate) dropped_frames: u64,
    pub(crate) plays_inline: bool,
    pub(crate) disable_picture_in_picture: bool,
    pub(crate) on_enter_picture_in_picture: Option<v8::Global<v8::Value>>,
    pub(crate) on_leave_picture_in_picture: Option<v8::Global<v8::Value>>,
    pub(crate) callbacks: HashMap<u32, v8::Global<v8::Function>>,
    pub(crate) next_callback_id: u32,
    pub(crate) ms_video_processing: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlVideoElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLVideoElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlVideoElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::html_media_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLVideoElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_video_element_width_property::define(scope, prototype)?;
    super::html_video_element_height_property::define(scope, prototype)?;
    super::html_video_element_video_width_property::define(scope, prototype)?;
    super::html_video_element_video_height_property::define(scope, prototype)?;
    super::html_video_element_poster_property::define(scope, prototype)?;
    super::html_video_element_webkit_decoded_frame_count_property::define(scope, prototype)?;
    super::html_video_element_webkit_dropped_frame_count_property::define(scope, prototype)?;
    super::html_video_element_plays_inline_property::define(scope, prototype)?;
    super::html_video_element_onenterpictureinpicture_property::define(scope, prototype)?;
    super::html_video_element_onleavepictureinpicture_property::define(scope, prototype)?;
    super::html_video_element_disable_picture_in_picture_property::define(scope, prototype)?;
    super::html_video_element_cancel_video_frame_callback::define(scope, prototype)?;
    super::html_video_element_get_video_playback_quality::define(scope, prototype)?;
    super::html_video_element_request_picture_in_picture::define(scope, prototype)?;
    super::html_video_element_request_video_frame_callback::define(scope, prototype)?;
    super::html_video_element_ms_video_processing_property::define(scope, prototype)?;
    super::html_video_element_ms_get_video_processing_types::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlVideoElementStore>()
        .ok_or_else(|| "HTMLVideoElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    source: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLVideoElement".to_owned());
    }
    super::html_media_element::attach_with_tag(scope, object, source, "VIDEO");
    scope
        .get_slot_mut::<HtmlVideoElementStore>()
        .ok_or_else(|| "HTMLVideoElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            VideoRecord {
                width: 0,
                height: 0,
                video_width: 0,
                video_height: 0,
                poster: String::new(),
                decoded_frames: 0,
                dropped_frames: 0,
                plays_inline: false,
                disable_picture_in_picture: false,
                on_enter_picture_in_picture: None,
                on_leave_picture_in_picture: None,
                callbacks: HashMap::new(),
                next_callback_id: 1,
                ms_video_processing: "default".to_owned(),
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
) -> Option<VideoRecord> {
    scope
        .get_slot::<HtmlVideoElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut VideoRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlVideoElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        operation(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&VideoRecord) -> u32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |x| x.width);
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |x| x.height);
}
pub(crate) fn get_video_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |x| x.video_width);
}
pub(crate) fn get_video_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |x| x.video_height);
}
pub(crate) fn get_decoded_frame_count(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.decoded_frames as f64).into());
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}
pub(crate) fn get_dropped_frame_count(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.dropped_frames as f64).into());
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    }
}

pub(crate) fn set_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |x| x.width = value);
}
pub(crate) fn set_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |x| x.height = value);
}

pub(crate) fn get_poster(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.poster) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_poster(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |x| x.poster = value);
}

pub(crate) fn get_plays_inline(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.plays_inline).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_plays_inline(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |x| x.plays_inline = value);
}
pub(crate) fn get_disable_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.disable_picture_in_picture).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_disable_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |x| {
        x.disable_picture_in_picture = value
    });
}

pub(crate) fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&VideoRecord) -> &Option<v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(handler) = select(&record) {
        result.set(v8::Local::new(scope, handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_on_enter_picture_in_picture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| &x.on_enter_picture_in_picture);
}
pub(crate) fn get_on_leave_picture_in_picture(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |x| &x.on_leave_picture_in_picture);
}
pub(crate) fn set_on_enter_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    update(scope, arguments.this(), |x| {
        x.on_enter_picture_in_picture = handler
    });
}
pub(crate) fn set_on_leave_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    update(scope, arguments.this(), |x| {
        x.on_leave_picture_in_picture = handler
    });
}

pub(crate) fn cancel_video_frame_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let id = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| {
        record.callbacks.remove(&id);
    });
}

pub(crate) fn request_video_frame_callback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(callback) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Callback must be callable");
        return;
    };
    let callback = v8::Global::new(scope, callback);
    let identity = arguments.this().get_identity_hash().get();
    let id = if let Some(record) = scope
        .get_slot_mut::<HtmlVideoElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        let id = record.next_callback_id;
        record.next_callback_id = record.next_callback_id.wrapping_add(1).max(1);
        record.callbacks.insert(id, callback);
        Some(id)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        None
    };
    if let Some(id) = id {
        result.set(v8::Integer::new_from_unsigned(scope, id).into());
    }
}

pub(crate) fn get_video_playback_quality(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    match super::video_playback_quality::create(
        scope,
        0.0,
        record.decoded_frames,
        record.dropped_frames,
        0,
    ) {
        Ok(value) => result.set(value.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn request_picture_in_picture(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "HTMLVideoElement",
            "requestPictureInPicture",
            result,
        );
        return;
    };
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        return;
    };
    if record.disable_picture_in_picture {
        let message = v8::String::new(scope, "Picture-in-Picture is disabled")
            .map(v8::Local::<v8::Value>::from)
            .unwrap_or_else(|| v8::undefined(scope).into());
        let _ = resolver.reject(scope, message);
    } else if let Ok(window) =
        super::picture_in_picture_window::create(scope, record.width as i32, record.height as i32)
    {
        let _ = resolver.resolve(scope, window.into());
    }
    result.set(resolver.get_promise(scope).into());
}

pub(crate) fn get_ms_video_processing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.ms_video_processing) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_ms_video_processing(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let valid = matches!(
        value.as_str(),
        "bicubic"
            | "lanczos"
            | "cas"
            | "default"
            | "msSuperResolution"
            | "msGraphicsDriverEnhancement"
    );
    if !valid {
        crate::webidl::throw_type_error(scope, "Invalid video processing type");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.ms_video_processing = value
    });
}

pub(crate) fn ms_get_video_processing_types(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let values = v8::Array::new(scope, 6);
    let bicubic = v8::String::new(scope, "bicubic").expect("string");
    let lanczos = v8::String::new(scope, "lanczos").expect("string");
    let cas = v8::String::new(scope, "cas").expect("string");
    let default_value = v8::String::new(scope, "default").expect("string");
    let super_resolution = v8::String::new(scope, "msSuperResolution").expect("string");
    let graphics_driver = v8::String::new(scope, "msGraphicsDriverEnhancement").expect("string");
    let _ = values.set_index(scope, 0, bicubic.into());
    let _ = values.set_index(scope, 1, lanczos.into());
    let _ = values.set_index(scope, 2, cas.into());
    let _ = values.set_index(scope, 3, default_value.into());
    let _ = values.set_index(scope, 4, super_resolution.into());
    let _ = values.set_index(scope, 5, graphics_driver.into());
    result.set(values.into());
}
