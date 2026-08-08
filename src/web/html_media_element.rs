use std::collections::{HashMap, VecDeque};

struct PendingMediaRequest {
    context: v8::Global<v8::Context>,
    element: v8::Global<v8::Object>,
    generation: u64,
    url: String,
}

#[derive(Clone)]
pub(crate) struct MediaRecord {
    pub(crate) src: String,
    pub(crate) current_src: String,
    pub(crate) cross_origin: Option<String>,
    pub(crate) network_state: u32,
    pub(crate) preload: String,
    pub(crate) ready_state: u32,
    pub(crate) seeking: bool,
    pub(crate) current_time: f64,
    pub(crate) duration: f64,
    pub(crate) error: Option<v8::Global<v8::Object>>,
    pub(crate) paused: bool,
    pub(crate) default_playback_rate: f64,
    pub(crate) playback_rate: f64,
    pub(crate) has_played: bool,
    pub(crate) ended: bool,
    pub(crate) autoplay: bool,
    pub(crate) loop_enabled: bool,
    pub(crate) preserves_pitch: bool,
    pub(crate) controls: bool,
    pub(crate) controls_list: Option<v8::Global<v8::Object>>,
    pub(crate) volume: f64,
    pub(crate) muted: bool,
    pub(crate) default_muted: bool,
    pub(crate) text_tracks: Option<v8::Global<v8::Object>>,
    pub(crate) on_encrypted: Option<v8::Global<v8::Value>>,
    pub(crate) on_waiting_for_key: Option<v8::Global<v8::Value>>,
    pub(crate) src_object: Option<v8::Global<v8::Object>>,
    pub(crate) loading: String,
    pub(crate) sink_id: String,
    pub(crate) remote: Option<v8::Global<v8::Object>>,
    pub(crate) disable_remote_playback: bool,
    pub(crate) media_keys: Option<v8::Global<v8::Value>>,
    pub(crate) generation: u64,
}

impl Default for MediaRecord {
    fn default() -> Self {
        Self {
            src: String::new(),
            current_src: String::new(),
            cross_origin: None,
            network_state: 0,
            preload: "metadata".to_owned(),
            ready_state: 0,
            seeking: false,
            current_time: 0.0,
            duration: f64::NAN,
            error: None,
            paused: true,
            default_playback_rate: 1.0,
            playback_rate: 1.0,
            has_played: false,
            ended: false,
            autoplay: false,
            loop_enabled: false,
            preserves_pitch: true,
            controls: false,
            controls_list: None,
            volume: 1.0,
            muted: false,
            default_muted: false,
            text_tracks: None,
            on_encrypted: None,
            on_waiting_for_key: None,
            src_object: None,
            loading: "eager".to_owned(),
            sink_id: String::new(),
            remote: None,
            disable_remote_playback: false,
            media_keys: None,
            generation: 0,
        }
    }
}

#[derive(Default)]
pub(crate) struct HtmlMediaElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MediaRecord>,
    pending: VecDeque<PendingMediaRequest>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlMediaElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLMediaElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlMediaElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLMediaElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_media_element_error_property::define(scope, prototype)?;
    super::html_media_element_src_property::define(scope, prototype)?;
    super::html_media_element_current_src_property::define(scope, prototype)?;
    super::html_media_element_cross_origin_property::define(scope, prototype)?;
    super::html_media_element_network_state_property::define(scope, prototype)?;
    super::html_media_element_preload_property::define(scope, prototype)?;
    super::html_media_element_buffered_property::define(scope, prototype)?;
    super::html_media_element_ready_state_property::define(scope, prototype)?;
    super::html_media_element_seeking_property::define(scope, prototype)?;
    super::html_media_element_current_time_property::define(scope, prototype)?;
    super::html_media_element_duration_property::define(scope, prototype)?;
    super::html_media_element_paused_property::define(scope, prototype)?;
    super::html_media_element_default_playback_rate_property::define(scope, prototype)?;
    super::html_media_element_playback_rate_property::define(scope, prototype)?;
    super::html_media_element_played_property::define(scope, prototype)?;
    super::html_media_element_seekable_property::define(scope, prototype)?;
    super::html_media_element_ended_property::define(scope, prototype)?;
    super::html_media_element_autoplay_property::define(scope, prototype)?;
    super::html_media_element_loop_property::define(scope, prototype)?;
    super::html_media_element_preserves_pitch_property::define(scope, prototype)?;
    super::html_media_element_controls_property::define(scope, prototype)?;
    super::html_media_element_controls_list_property::define(scope, prototype)?;
    super::html_media_element_volume_property::define(scope, prototype)?;
    super::html_media_element_muted_property::define(scope, prototype)?;
    super::html_media_element_default_muted_property::define(scope, prototype)?;
    super::html_media_element_text_tracks_property::define(scope, prototype)?;
    super::html_media_element_webkit_audio_decoded_byte_count_property::define(scope, prototype)?;
    super::html_media_element_webkit_video_decoded_byte_count_property::define(scope, prototype)?;
    super::html_media_element_onencrypted_property::define(scope, prototype)?;
    super::html_media_element_onwaitingforkey_property::define(scope, prototype)?;
    super::html_media_element_src_object_property::define(scope, prototype)?;
    define_constant(scope, prototype, "NETWORK_EMPTY", 0)?;
    define_constant(scope, prototype, "NETWORK_IDLE", 1)?;
    define_constant(scope, prototype, "NETWORK_LOADING", 2)?;
    define_constant(scope, prototype, "NETWORK_NO_SOURCE", 3)?;
    define_constant(scope, prototype, "HAVE_NOTHING", 0)?;
    define_constant(scope, prototype, "HAVE_METADATA", 1)?;
    define_constant(scope, prototype, "HAVE_CURRENT_DATA", 2)?;
    define_constant(scope, prototype, "HAVE_FUTURE_DATA", 3)?;
    define_constant(scope, prototype, "HAVE_ENOUGH_DATA", 4)?;
    super::html_media_element_add_text_track::define(scope, prototype)?;
    super::html_media_element_can_play_type::define(scope, prototype)?;
    super::html_media_element_capture_stream::define(scope, prototype)?;
    super::html_media_element_load::define(scope, prototype)?;
    super::html_media_element_pause::define(scope, prototype)?;
    super::html_media_element_play::define(scope, prototype)?;
    super::html_media_element_loading_property::define(scope, prototype)?;
    super::html_media_element_sink_id_property::define(scope, prototype)?;
    super::html_media_element_remote_property::define(scope, prototype)?;
    super::html_media_element_disable_remote_playback_property::define(scope, prototype)?;
    super::html_media_element_set_sink_id::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::html_media_element_media_keys_property::define(scope, prototype)?;
    super::html_media_element_set_media_keys::define(scope, prototype)?;
    define_constant(scope, constructor.into(), "NETWORK_EMPTY", 0)?;
    define_constant(scope, constructor.into(), "NETWORK_IDLE", 1)?;
    define_constant(scope, constructor.into(), "NETWORK_LOADING", 2)?;
    define_constant(scope, constructor.into(), "NETWORK_NO_SOURCE", 3)?;
    define_constant(scope, constructor.into(), "HAVE_NOTHING", 0)?;
    define_constant(scope, constructor.into(), "HAVE_METADATA", 1)?;
    define_constant(scope, constructor.into(), "HAVE_CURRENT_DATA", 2)?;
    define_constant(scope, constructor.into(), "HAVE_FUTURE_DATA", 3)?;
    define_constant(scope, constructor.into(), "HAVE_ENOUGH_DATA", 4)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlMediaElementStore>()
        .ok_or_else(|| "HTMLMediaElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    source: String,
) {
    attach_with_tag(scope, object, source, "AUDIO");
}

pub(crate) fn attach_with_tag(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    source: String,
    tag_name: &str,
) {
    super::html_element::attach(scope, object, tag_name);
    let controls_list = super::dom_token_list::create(scope, "").ok();
    let text_tracks = super::text_track_list::create(scope).ok();
    let remote = super::remote_playback::create(scope).ok();
    let has_source = !source.is_empty();
    let mut record = MediaRecord {
        src: source.clone(),
        ..MediaRecord::default()
    };
    record.controls_list = controls_list.map(|object| v8::Global::new(scope, object));
    record.text_tracks = text_tracks.map(|object| v8::Global::new(scope, object));
    record.remote = remote.map(|object| v8::Global::new(scope, object));
    scope
        .get_slot_mut::<HtmlMediaElementStore>()
        .expect("HTMLMediaElement state")
        .records
        .insert(object.get_identity_hash().get(), record);
    if has_source {
        super::element::set_reflected_string(scope, object, "src", source);
    }
}

pub(crate) fn is_media_element(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    record(scope, object).is_some()
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn define_constant(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    name: &str,
    value: i32,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if target.define_own_property(
        scope,
        key.into(),
        v8::Integer::new(scope, value).into(),
        v8::PropertyAttribute::READ_ONLY | v8::PropertyAttribute::DONT_DELETE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define HTMLMediaElement.{name}"))
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MediaRecord> {
    scope
        .get_slot::<HtmlMediaElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: impl FnOnce(&mut MediaRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlMediaElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        operation(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<&str>,
) {
    let normalized = name.to_ascii_lowercase();
    let text = value.unwrap_or_default();
    let Some(record) = scope
        .get_slot_mut::<HtmlMediaElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return;
    };
    match normalized.as_str() {
        "src" => record.src = text.to_owned(),
        "crossorigin" => record.cross_origin = value.map(str::to_owned),
        "preload" => record.preload = text.to_owned(),
        "autoplay" => record.autoplay = value.is_some(),
        "loop" => record.loop_enabled = value.is_some(),
        "controls" => record.controls = value.is_some(),
        "muted" => record.default_muted = value.is_some(),
        _ => {}
    }
    if normalized == "src" {
        schedule_media_update(scope, object);
    }
}

fn schedule_media_update(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let source = record(scope, object)
        .map(|record| record.src)
        .unwrap_or_default();
    let url = resolve_media_url(scope, object, &source);
    let generation = {
        let Some(record) = scope
            .get_slot_mut::<HtmlMediaElementStore>()
            .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
        else {
            return;
        };
        record.generation = record.generation.wrapping_add(1);
        record.current_src.clear();
        record.network_state = if url.is_some() { 2 } else { 0 };
        record.ready_state = 0;
        record.seeking = false;
        record.current_time = 0.0;
        record.duration = f64::NAN;
        record.error = None;
        record.ended = false;
        record.has_played = false;
        record.generation
    };
    let Some(url) = url else {
        return;
    };
    let request = PendingMediaRequest {
        context: v8::Global::new(scope, scope.get_current_context()),
        element: v8::Global::new(scope, object),
        generation,
        url,
    };
    if let Some(store) = scope.get_slot_mut::<HtmlMediaElementStore>() {
        store.pending.push_back(request);
    }
}

fn resolve_media_url(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    source: &str,
) -> Option<String> {
    if source.is_empty() {
        return None;
    }
    let base = super::element::element_base_url(scope, object);
    ::url::Url::parse(source)
        .or_else(|_| ::url::Url::parse(&base).and_then(|base| base.join(source)))
        .map(|url| url.to_string())
        .ok()
        .or_else(|| Some(source.to_owned()))
}

pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    result: &mut v8::ReturnValue<'_>,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

pub(crate) fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(error) = record.error {
            result.set(v8::Local::new(scope, error).into());
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.src);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    super::element::set_reflected_string(scope, arguments.this(), "src", value);
}

pub(crate) fn get_current_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.current_src);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(cross_origin) = record.cross_origin {
            return_string(scope, &mut result, &cross_origin);
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, arguments.get(0));
        Some(if value == "use-credentials" {
            value
        } else {
            "anonymous".to_owned()
        })
    };
    update(scope, arguments.this(), |record| {
        record.cross_origin = value
    });
}

pub(crate) fn get_network_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.network_state).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_preload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.preload);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_preload(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if matches!(value.as_str(), "none" | "metadata" | "auto" | "") {
        value
    } else {
        "metadata".to_owned()
    };
    update(scope, arguments.this(), |record| record.preload = value);
}

pub(crate) fn empty_ranges(
    scope: &mut v8::PinScope<'_, '_>,
    mut result: v8::ReturnValue<'_>,
    ranges: Vec<(f64, f64)>,
) {
    if let Ok(object) = super::time_ranges::create(scope, ranges) {
        result.set(object.into());
    }
}

pub(crate) fn get_buffered(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        empty_ranges(scope, result, Vec::new());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_ready_state(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.ready_state).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_seeking(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.seeking).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.current_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_current_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(0.0);
    update(scope, arguments.this(), |record| {
        record.seeking = true;
        record.current_time = value.max(0.0);
        record.seeking = false;
    });
}

pub(crate) fn get_duration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.duration).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_paused(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.paused).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_default_playback_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.default_playback_rate).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_default_playback_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(1.0);
    update(scope, arguments.this(), |record| {
        record.default_playback_rate = value
    });
}

pub(crate) fn get_playback_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.playback_rate).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_playback_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(1.0);
    update(scope, arguments.this(), |record| {
        record.playback_rate = value
    });
}

pub(crate) fn get_played(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let ranges = if record.has_played && record.current_time > 0.0 {
            vec![(0.0, record.current_time)]
        } else {
            Vec::new()
        };
        empty_ranges(scope, result, ranges);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_seekable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let ranges = if record.duration.is_finite() && record.duration > 0.0 {
            vec![(0.0, record.duration)]
        } else {
            Vec::new()
        };
        empty_ranges(scope, result, ranges);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_ended(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.ended).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_autoplay(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.autoplay).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_autoplay(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.autoplay = value);
}

pub(crate) fn get_loop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.loop_enabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_loop(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.loop_enabled = value
    });
}

pub(crate) fn get_preserves_pitch(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.preserves_pitch).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_preserves_pitch(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.preserves_pitch = value
    });
}

pub(crate) fn get_controls(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.controls).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_controls(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.controls = value);
}

pub(crate) fn get_controls_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(list) = record.controls_list {
            result.set(v8::Local::new(scope, &list).into());
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_controls_list(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(list) = record.controls_list {
            let list = v8::Local::new(scope, &list);
            super::dom_token_list::set_string_value(scope, list, &value);
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_volume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.volume).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_volume(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(1.0);
    if !(0.0..=1.0).contains(&value) {
        if let Some(message) =
            v8::String::new(scope, "The volume provided is outside the range [0, 1]")
        {
            let exception = v8::Exception::range_error(scope, message);
            scope.throw_exception(exception);
        }
        return;
    }
    update(scope, arguments.this(), |record| record.volume = value);
}

pub(crate) fn get_muted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.muted).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_muted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.muted = value);
}

pub(crate) fn get_default_muted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.default_muted).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_default_muted(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.default_muted = value
    });
}

pub(crate) fn get_text_tracks(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(list) = record.text_tracks {
            result.set(v8::Local::new(scope, &list).into());
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn decoded_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new_from_unsigned(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_audio_decoded_byte_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    decoded_count(scope, arguments, result);
}

pub(crate) fn get_video_decoded_byte_count(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    decoded_count(scope, arguments, result);
}

pub(crate) fn handler(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Value>> {
    if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, value))
    }
}

pub(crate) fn return_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MediaRecord) -> Option<v8::Global<v8::Value>>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = select(&record) {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn get_on_encrypted(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.on_encrypted.clone());
}
pub(crate) fn set_on_encrypted(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(s, a.get(0));
    update(s, a.this(), |record| record.on_encrypted = value);
}
pub(crate) fn get_on_waiting_for_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_handler(s, a, r, |record| record.on_waiting_for_key.clone());
}
pub(crate) fn set_on_waiting_for_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = handler(s, a.get(0));
    update(s, a.this(), |record| record.on_waiting_for_key = value);
}

pub(crate) fn get_src_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(object) = record.src_object {
            result.set(v8::Local::new(scope, &object).into());
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_src_object(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let object = if value.is_null() {
        None
    } else {
        let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
            crate::webidl::throw_type_error(scope, "srcObject must be a MediaStream or null");
            return;
        };
        if !super::media_stream::is_stream(scope, object) {
            crate::webidl::throw_type_error(scope, "srcObject must be a MediaStream or null");
            return;
        }
        Some(v8::Global::new(scope, object))
    };
    update(scope, arguments.this(), |record| record.src_object = object);
}

pub(crate) fn add_text_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let kind = crate::webidl::value_to_string(scope, arguments.get(0));
    let label = if arguments.length() > 1 {
        crate::webidl::value_to_string(scope, arguments.get(1))
    } else {
        String::new()
    };
    let language = if arguments.length() > 2 {
        crate::webidl::value_to_string(scope, arguments.get(2))
    } else {
        String::new()
    };
    let Ok(track) = super::text_track::create(scope, kind, label, language, String::new()) else {
        return;
    };
    if let Some(list) = record.text_tracks {
        let list = v8::Local::new(scope, &list);
        let _ = super::text_track_list::append(scope, list, track);
    }
    result.set(track.into());
}

pub(crate) fn can_play_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let support = if media_type.starts_with("audio/mpeg")
        || media_type.starts_with("audio/ogg")
        || media_type.starts_with("audio/wav")
        || media_type.starts_with("audio/webm")
        || media_type.starts_with("video/mp4")
        || media_type.starts_with("video/webm")
    {
        "probably"
    } else if media_type.starts_with("audio/") || media_type.starts_with("video/") {
        "maybe"
    } else {
        ""
    };
    return_string(scope, &mut result, support);
}

pub(crate) fn capture_stream(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(stream) = super::media_stream::create_with_tracks(scope, &[]) {
        result.set(stream.into());
    }
}

pub(crate) fn load(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    update(scope, arguments.this(), |record| record.paused = true);
    schedule_media_update(scope, arguments.this());
}

pub(crate) fn pause(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update(scope, arguments.this(), |record| record.paused = true);
}

pub(crate) fn play(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    update(scope, arguments.this(), |record| {
        record.paused = false;
        record.has_played = true;
        record.ended = false;
    });
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn get_loading(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.loading);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_loading(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if value == "lazy" { "lazy" } else { "eager" }.to_owned();
    update(scope, arguments.this(), |record| record.loading = value);
}

pub(crate) fn get_sink_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.sink_id);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_remote(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(remote) = record.remote {
            result.set(v8::Local::new(scope, &remote).into());
        } else {
            result.set(v8::null(scope).into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_disable_remote_playback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.disable_remote_playback).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_disable_remote_playback(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.disable_remote_playback = value
    });
}

pub(crate) fn set_sink_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let sink_id = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.sink_id = sink_id);
    let value = v8::undefined(scope);
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value.into()) {
        result.set(promise.into());
    }
}

pub(crate) fn get_media_keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(media_keys) = record.media_keys {
        result.set(v8::Local::new(scope, &media_keys));
    } else {
        result.set(v8::null(scope).into());
    }
}

pub(crate) fn set_media_keys(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let value = arguments.get(0);
    let media_keys =
        (!value.is_null() && !value.is_undefined()).then(|| v8::Global::new(scope, value));
    if let Some(record) = scope
        .get_slot_mut::<HtmlMediaElementStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        record.media_keys = media_keys;
        let undefined = v8::undefined(scope);
        if let Ok(promise) = super::writable_stream::resolved_promise(scope, undefined.into()) {
            result.set(promise.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

struct MediaLoadError {
    code: u32,
    message: String,
}

fn fetch_media(
    scope: &v8::PinScope<'_, '_>,
    url: &str,
) -> Result<(Vec<u8>, String), MediaLoadError> {
    if url.starts_with("data:") {
        return super::fetch_global::decode_data_url(url)
            .map(|(content_type, bytes)| (bytes, content_type))
            .map_err(|_| MediaLoadError {
                code: 4,
                message: "MEDIA_ELEMENT_ERROR: Format error".to_owned(),
            });
    }
    if url.starts_with("blob:") {
        return super::url::object_url_snapshot(scope, url).ok_or_else(|| MediaLoadError {
            code: 4,
            message: "MEDIA_ELEMENT_ERROR: The blob URL has been revoked".to_owned(),
        });
    }
    let entry = crate::network_replay::lookup(scope, "GET", url).ok_or_else(|| MediaLoadError {
        code: 2,
        message: "MEDIA_ELEMENT_ERROR: Network error".to_owned(),
    })?;
    if !(200..=299).contains(&entry.status) {
        return Err(MediaLoadError {
            code: 2,
            message: format!("MEDIA_ELEMENT_ERROR: HTTP status {}", entry.status),
        });
    }
    let content_type = entry
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    Ok((entry.body, content_type))
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let pending = scope
        .get_slot_mut::<HtmlMediaElementStore>()
        .and_then(|store| store.pending.pop_front());
    let Some(pending) = pending else {
        return false;
    };
    run_media_request(scope, pending);
    true
}

fn run_media_request(scope: &mut v8::PinScope<'_, '_>, request: PendingMediaRequest) {
    let context = v8::Local::new(scope, &request.context);
    let request_scope = &mut v8::ContextScope::new(scope, context);
    let element = v8::Local::new(request_scope, &request.element);
    if !request_is_current(request_scope, element, request.generation) {
        return;
    }
    dispatch_media_event(request_scope, element, "loadstart");
    if !request_is_current(request_scope, element, request.generation) {
        return;
    }
    let loaded = fetch_media(request_scope, &request.url).and_then(|(bytes, content_type)| {
        super::audio_metadata::parse(&bytes, &content_type).ok_or_else(|| MediaLoadError {
            code: 4,
            message: "MEDIA_ELEMENT_ERROR: Format error".to_owned(),
        })
    });
    match loaded {
        Ok(metadata) => {
            let Some(record) = request_scope
                .get_slot_mut::<HtmlMediaElementStore>()
                .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
            else {
                return;
            };
            if record.generation != request.generation {
                return;
            }
            record.current_src = request.url;
            record.network_state = 1;
            record.ready_state = 1;
            record.duration = metadata.duration;
            record.error = None;
            dispatch_media_event(request_scope, element, "durationchange");
            if request_is_current(request_scope, element, request.generation) {
                dispatch_media_event(request_scope, element, "loadedmetadata");
            }
        }
        Err(failure) => {
            let error = super::media_error::create(request_scope, failure.code, failure.message)
                .ok()
                .map(|error| v8::Global::new(request_scope, error));
            let Some(record) = request_scope
                .get_slot_mut::<HtmlMediaElementStore>()
                .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
            else {
                return;
            };
            if record.generation != request.generation {
                return;
            }
            record.current_src = request.url;
            record.network_state = 3;
            record.ready_state = 0;
            record.duration = f64::NAN;
            record.error = error;
            dispatch_media_event(request_scope, element, "error");
        }
    }
}

fn request_is_current(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    generation: u64,
) -> bool {
    record(scope, element).is_some_and(|record| record.generation == generation)
}

fn dispatch_media_event(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if let Ok(event) = super::event::create(scope, event_type) {
        super::event_target::dispatch(scope, element, event);
    }
}
