use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MediaQueryListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MediaQueryRecord>,
}

#[derive(Clone)]
struct MediaQueryRecord {
    media: String,
    matches: bool,
    onchange: Option<v8::Global<v8::Value>>,
    listeners: Vec<v8::Global<v8::Function>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaQueryListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaQueryList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MediaQueryListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaQueryList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "media", get_media)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "matches", get_matches)?;
    crate::webidl::define_accessor(scope, prototype, "onchange", get_onchange, set_onchange)?;
    crate::webidl::define_method(scope, prototype, "addListener", 1, add_listener)?;
    crate::webidl::define_method(scope, prototype, "removeListener", 1, remove_listener)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MediaQueryListStore>()
        .ok_or_else(|| "MediaQueryList state was not prepared".to_owned())?
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
        "Failed to construct 'MediaQueryList': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    media: String,
    viewport_width: f64,
    viewport_height: f64,
    device_pixel_ratio: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaQueryList".to_owned());
    }
    super::event_target::attach(scope, object);
    let preferences = crate::fingerprint::edge(scope).media_preferences.clone();
    let matches = evaluate_query(
        &media,
        viewport_width,
        viewport_height,
        device_pixel_ratio,
        &preferences,
    );
    scope
        .get_slot_mut::<MediaQueryListStore>()
        .ok_or_else(|| "MediaQueryList state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            MediaQueryRecord {
                media,
                matches,
                onchange: None,
                listeners: Vec::new(),
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MediaQueryRecord> {
    scope
        .get_slot::<MediaQueryListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.media) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_matches(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.matches).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = record.onchange {
        result.set(v8::Local::new(scope, &value));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = arguments
        .get(0)
        .is_function()
        .then(|| v8::Global::new(scope, arguments.get(0)));
    if let Some(record) = scope
        .get_slot_mut::<MediaQueryListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.onchange = handler;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn add_listener(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(function) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        return;
    };
    let identity = function.get_identity_hash().get();
    let function = v8::Global::new(scope, function);
    let Some(mut current) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if current
        .listeners
        .iter()
        .any(|listener| v8::Local::new(scope, listener).get_identity_hash().get() == identity)
    {
        return;
    }
    current.listeners.push(function);
    if let Some(stored) = scope
        .get_slot_mut::<MediaQueryListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        stored.listeners = current.listeners;
    }
}

fn remove_listener(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(function) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        return;
    };
    let identity = function.get_identity_hash().get();
    let Some(mut record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record
        .listeners
        .retain(|listener| v8::Local::new(scope, listener).get_identity_hash().get() != identity);
    if let Some(stored) = scope
        .get_slot_mut::<MediaQueryListStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        stored.listeners = record.listeners;
    }
}

fn evaluate_query(
    query: &str,
    width: f64,
    height: f64,
    device_pixel_ratio: f64,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    let normalized = query.trim().to_ascii_lowercase();
    normalized.split(',').any(|branch| {
        evaluate_branch(
            branch.trim(),
            width,
            height,
            device_pixel_ratio,
            preferences,
        )
    })
}

fn evaluate_branch(
    query: &str,
    width: f64,
    height: f64,
    device_pixel_ratio: f64,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    if let Some(query) = query.strip_prefix("not ") {
        return !evaluate_branch(query, width, height, device_pixel_ratio, preferences);
    }
    query.split(" and ").all(|clause| {
        let clause = clause.trim();
        if clause == "all" || clause == "screen" {
            return true;
        }
        if clause == "print" {
            return false;
        }
        let feature = clause
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .unwrap_or(clause)
            .trim();
        evaluate_feature(feature, width, height, device_pixel_ratio, preferences)
    })
}

fn evaluate_feature(
    feature: &str,
    width: f64,
    height: f64,
    device_pixel_ratio: f64,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    if let Some(value) = pixels_after(feature, "min-width:") {
        return width >= value;
    }
    if let Some(value) = pixels_after(feature, "max-width:") {
        return width <= value;
    }
    if let Some(value) = pixels_after(feature, "width:") {
        return approximately_equal(width, value);
    }
    if let Some(value) = pixels_after(feature, "min-height:") {
        return height >= value;
    }
    if let Some(value) = pixels_after(feature, "max-height:") {
        return height <= value;
    }
    if let Some(value) = pixels_after(feature, "height:") {
        return approximately_equal(height, value);
    }
    if feature == "orientation: landscape" {
        return width >= height;
    }
    if feature == "orientation: portrait" {
        return height > width;
    }
    if let Some(value) = resolution_after(feature, "min-resolution:") {
        return device_pixel_ratio >= value;
    }
    if let Some(value) = resolution_after(feature, "max-resolution:") {
        return device_pixel_ratio <= value;
    }
    if let Some(value) = resolution_after(feature, "resolution:") {
        return approximately_equal(device_pixel_ratio, value);
    }
    if let Some(value) = number_after(feature, "-webkit-min-device-pixel-ratio:") {
        return device_pixel_ratio >= value;
    }
    if let Some(value) = number_after(feature, "-webkit-max-device-pixel-ratio:") {
        return device_pixel_ratio <= value;
    }
    if let Some(value) = number_after(feature, "-webkit-device-pixel-ratio:") {
        return approximately_equal(device_pixel_ratio, value);
    }
    if let Some(value) = value_after(feature, "prefers-color-scheme:") {
        return value == preferences.color_scheme;
    }
    if let Some(value) = value_after(feature, "prefers-contrast:") {
        return value == preferences.contrast;
    }
    if let Some(value) = value_after(feature, "prefers-reduced-motion:") {
        return (value == "reduce") == preferences.reduced_motion;
    }
    if let Some(value) = value_after(feature, "prefers-reduced-data:") {
        return (value == "reduce") == preferences.reduced_data;
    }
    if let Some(value) = value_after(feature, "forced-colors:") {
        return (value == "active") == preferences.forced_colors;
    }
    if let Some(value) = value_after(feature, "inverted-colors:") {
        return (value == "inverted") == preferences.inverted_colors;
    }
    if let Some(value) = value_after(feature, "color-gamut:") {
        return gamut_at_least(&preferences.color_gamut, value);
    }
    if let Some(value) = value_after(feature, "pointer:") {
        return value == preferences.pointer;
    }
    if let Some(value) = value_after(feature, "any-pointer:") {
        return value == preferences.any_pointer;
    }
    if let Some(value) = value_after(feature, "hover:") {
        return value == preferences.hover;
    }
    if let Some(value) = value_after(feature, "any-hover:") {
        return value == preferences.any_hover;
    }
    if let Some(value) = value_after(feature, "display-mode:") {
        return value == preferences.display_mode;
    }
    if let Some(value) = value_after(feature, "dynamic-range:") {
        return value == preferences.dynamic_range;
    }
    if let Some(value) = value_after(feature, "video-dynamic-range:") {
        return value == preferences.dynamic_range;
    }
    if let Some(value) = value_after(feature, "scripting:") {
        return value == preferences.scripting;
    }
    if feature == "monochrome" {
        return preferences.monochrome_bits > 0;
    }
    if let Some(value) = number_after(feature, "monochrome:") {
        return preferences.monochrome_bits == value.max(0.0) as u32;
    }
    false
}

fn value_after<'a>(query: &'a str, marker: &str) -> Option<&'a str> {
    query
        .strip_prefix(marker)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn gamut_at_least(configured: &str, requested: &str) -> bool {
    let rank = |value: &str| match value {
        "srgb" => 1,
        "p3" => 2,
        "rec2020" => 3,
        _ => 0,
    };
    rank(configured) >= rank(requested) && rank(requested) > 0
}

fn pixels_after(query: &str, marker: &str) -> Option<f64> {
    let start = query.find(marker)? + marker.len();
    let tail = query[start..].trim_start();
    let end = tail.find("px")?;
    tail[..end].trim().parse().ok()
}

fn resolution_after(query: &str, marker: &str) -> Option<f64> {
    let start = query.find(marker)? + marker.len();
    let tail = query[start..].trim();
    if let Some(value) = tail.strip_suffix("dppx") {
        return value.trim().parse().ok();
    }
    if let Some(value) = tail.strip_suffix("dpi") {
        return value.trim().parse::<f64>().ok().map(|value| value / 96.0);
    }
    if let Some(value) = tail.strip_suffix("dpcm") {
        return value
            .trim()
            .parse::<f64>()
            .ok()
            .map(|value| value * 2.54 / 96.0);
    }
    None
}

fn number_after(query: &str, marker: &str) -> Option<f64> {
    let start = query.find(marker)? + marker.len();
    query[start..].trim().parse().ok()
}

fn approximately_equal(left: f64, right: f64) -> bool {
    (left - right).abs() <= f64::EPSILON * left.abs().max(right.abs()).max(1.0) * 8.0
}
