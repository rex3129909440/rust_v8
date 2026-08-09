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
    device_width: f64,
    device_height: f64,
    device_pixel_ratio: f64,
    color_depth: u32,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MediaQueryList".to_owned());
    }
    super::event_target::attach(scope, object);
    let fingerprint = crate::fingerprint::edge(scope);
    let preferences = fingerprint.media_preferences.clone();
    let device_posture_folded = fingerprint.hardware_devices.device_posture == "folded";
    let matches = evaluate_query(
        &media,
        MediaEnvironment {
            viewport_width,
            viewport_height,
            device_width,
            device_height,
            device_pixel_ratio,
            color_depth,
            device_posture_folded,
        },
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

#[derive(Clone, Copy)]
struct MediaEnvironment {
    viewport_width: f64,
    viewport_height: f64,
    device_width: f64,
    device_height: f64,
    device_pixel_ratio: f64,
    color_depth: u32,
    device_posture_folded: bool,
}

impl MediaEnvironment {
    fn query_viewport(self) -> (f64, f64) {
        // `width`, `height`, `aspect-ratio`, orientation and viewport units
        // describe the viewport even when it is zero-sized.  Falling back to
        // the physical screen here lets min/max-width probes recover
        // `screen.width` from an intentionally hidden Window.  The deprecated
        // device-* features have their own explicit device_width/device_height
        // path below.
        (self.viewport_width, self.viewport_height)
    }
}

fn evaluate_query(
    query: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    let normalized = query.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    split_top_level_character(&normalized, ',')
        .into_iter()
        .any(|branch| evaluate_branch(branch, environment, preferences))
}

fn evaluate_branch(
    query: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    let mut query = query.trim();
    if let Some(rest) = strip_keyword_prefix(query, "only") {
        query = rest;
    }
    let (negated, query) = if let Some(rest) = strip_keyword_prefix(query, "not") {
        (true, rest)
    } else {
        (false, query)
    };
    let matches = evaluate_condition(query, environment, preferences);
    if negated { !matches } else { matches }
}

fn evaluate_condition(
    query: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return false;
    }
    let alternatives = split_top_level_keyword(query, "or");
    if alternatives.len() > 1 {
        return alternatives
            .into_iter()
            .any(|part| evaluate_condition(part, environment, preferences));
    }
    let requirements = split_top_level_keyword(query, "and");
    if requirements.len() > 1 {
        return requirements
            .into_iter()
            .all(|part| evaluate_condition(part, environment, preferences));
    }
    if let Some(rest) = strip_keyword_prefix(query, "not") {
        return !evaluate_condition(rest, environment, preferences);
    }
    if let Some(inner) = fully_parenthesized(query) {
        if has_top_level_keyword(inner, "and")
            || has_top_level_keyword(inner, "or")
            || strip_keyword_prefix(inner, "not").is_some()
            || fully_parenthesized(inner).is_some()
        {
            return evaluate_condition(inner, environment, preferences);
        }
        return evaluate_feature(inner.trim(), environment, preferences);
    }
    match query {
        "all" | "screen" => true,
        "print" | "speech" => false,
        _ => false,
    }
}

fn evaluate_feature(
    feature: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    let feature = feature.trim();
    if feature.is_empty() {
        return false;
    }
    if let Some(matches) = evaluate_range_feature(feature, environment, preferences) {
        return matches;
    }
    if let Some((name, value)) = feature.split_once(':') {
        return evaluate_colon_feature(name.trim(), value.trim(), environment, preferences);
    }
    evaluate_boolean_feature(feature, environment, preferences)
}

fn evaluate_colon_feature(
    requested_name: &str,
    value: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    if value.is_empty() {
        return false;
    }
    let (comparison, name) = if let Some(name) = requested_name.strip_prefix("min-") {
        (Comparison::GreaterOrEqual, name)
    } else if let Some(name) = requested_name.strip_prefix("max-") {
        (Comparison::LessOrEqual, name)
    } else {
        (Comparison::Equal, requested_name)
    };
    if let Some(actual) = numeric_feature_value(name, environment, preferences) {
        let Some(requested) = parse_numeric_feature_value(name, value, environment) else {
            return false;
        };
        return compare(actual, requested, comparison);
    }
    if comparison != Comparison::Equal {
        return false;
    }
    evaluate_discrete_feature(name, value, environment, preferences)
}

fn evaluate_boolean_feature(
    name: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    if matches!(name, "aspect-ratio" | "device-aspect-ratio") {
        return numeric_feature_value(name, environment, preferences).is_some();
    }
    if let Some(value) = numeric_feature_value(name, environment, preferences) {
        return value.is_finite() && value > 0.0;
    }
    match name {
        "orientation" => {
            let (width, height) = environment.query_viewport();
            width.is_finite() && height.is_finite()
        }
        "prefers-color-scheme" => true,
        "prefers-contrast" => preferences.contrast != "no-preference",
        "prefers-reduced-motion" => preferences.reduced_motion,
        "prefers-reduced-data" => preferences.reduced_data,
        "prefers-reduced-transparency" => preferences.reduced_transparency,
        "forced-colors" => preferences.forced_colors,
        "inverted-colors" => preferences.inverted_colors,
        "pointer" => preferences.pointer != "none",
        "any-pointer" => preferences.any_pointer != "none",
        "hover" => preferences.hover != "none",
        "any-hover" => preferences.any_hover != "none",
        "color-gamut" | "video-color-gamut" => !preferences.color_gamut.is_empty(),
        "display-mode"
        | "dynamic-range"
        | "video-dynamic-range"
        | "scripting"
        | "update"
        | "overflow-block"
        | "overflow-inline"
        | "environment-blending"
        | "device-posture"
        | "shape"
        | "nav-controls" => true,
        _ => false,
    }
}

fn evaluate_discrete_feature(
    name: &str,
    value: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> bool {
    match name {
        "orientation" => match value {
            "landscape" => {
                let (width, height) = environment.query_viewport();
                width.is_finite() && height.is_finite() && width > height
            }
            "portrait" => {
                let (width, height) = environment.query_viewport();
                width.is_finite() && height.is_finite() && height >= width
            }
            _ => false,
        },
        "prefers-color-scheme" => value == preferences.color_scheme,
        "prefers-contrast" => value == preferences.contrast,
        "prefers-reduced-motion" => match value {
            "reduce" => preferences.reduced_motion,
            "no-preference" => !preferences.reduced_motion,
            _ => false,
        },
        "prefers-reduced-data" => match value {
            "reduce" => preferences.reduced_data,
            "no-preference" => !preferences.reduced_data,
            _ => false,
        },
        "prefers-reduced-transparency" => match value {
            "reduce" => preferences.reduced_transparency,
            "no-preference" => !preferences.reduced_transparency,
            _ => false,
        },
        "forced-colors" => match value {
            "active" => preferences.forced_colors,
            "none" => !preferences.forced_colors,
            _ => false,
        },
        "inverted-colors" => match value {
            "inverted" => preferences.inverted_colors,
            "none" => !preferences.inverted_colors,
            _ => false,
        },
        "color-gamut" | "video-color-gamut" => gamut_at_least(&preferences.color_gamut, value),
        "pointer" => value == preferences.pointer,
        "any-pointer" => value == preferences.any_pointer,
        "hover" => value == preferences.hover,
        "any-hover" => value == preferences.any_hover,
        "display-mode" => value == preferences.display_mode,
        "dynamic-range" => value == preferences.dynamic_range,
        "video-dynamic-range" => value == preferences.video_dynamic_range,
        "scripting" => value == preferences.scripting,
        "update" => value == "fast",
        "overflow-block" => value == "scroll",
        "overflow-inline" => value == "scroll",
        "environment-blending" => value == "opaque",
        "device-posture" => match value {
            "folded" => environment.device_posture_folded,
            "continuous" => !environment.device_posture_folded,
            _ => false,
        },
        "shape" => value == "rect",
        "nav-controls" => value == "none",
        "scan" => false,
        _ => false,
    }
}

fn numeric_feature_value(
    name: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> Option<f64> {
    let (viewport_width, viewport_height) = environment.query_viewport();
    match name {
        "width" => Some(viewport_width),
        "height" => Some(viewport_height),
        "device-width" => Some(environment.device_width),
        "device-height" => Some(environment.device_height),
        "aspect-ratio" => ratio(viewport_width, viewport_height),
        "device-aspect-ratio" => ratio(environment.device_width, environment.device_height),
        "resolution" | "-webkit-device-pixel-ratio" => Some(environment.device_pixel_ratio),
        "color" => Some(f64::from(environment.color_depth / 3)),
        "color-index" | "grid" => Some(0.0),
        "monochrome" => Some(f64::from(preferences.monochrome_bits)),
        "horizontal-viewport-segments" | "vertical-viewport-segments" => Some(1.0),
        "-webkit-transform-3d" => Some(1.0),
        _ => None,
    }
}

fn parse_numeric_feature_value(
    name: &str,
    value: &str,
    environment: MediaEnvironment,
) -> Option<f64> {
    match name {
        "width" | "height" | "device-width" | "device-height" => parse_length(value, environment),
        "aspect-ratio" | "device-aspect-ratio" => parse_ratio(value),
        "resolution" => parse_resolution(value),
        "-webkit-device-pixel-ratio"
        | "color"
        | "color-index"
        | "grid"
        | "monochrome"
        | "horizontal-viewport-segments"
        | "vertical-viewport-segments"
        | "-webkit-transform-3d" => parse_nonnegative_number(value),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Comparison {
    Less,
    LessOrEqual,
    Equal,
    GreaterOrEqual,
    Greater,
}

fn evaluate_range_feature(
    feature: &str,
    environment: MediaEnvironment,
    preferences: &crate::MediaPreferencesFingerprint,
) -> Option<bool> {
    let (parts, operators) = split_comparisons(feature)?;
    match operators.as_slice() {
        [operator] => {
            let left_name = canonical_numeric_feature(parts[0]);
            let right_name = canonical_numeric_feature(parts[1]);
            match (left_name, right_name) {
                (Some(name), None) => {
                    let actual = numeric_feature_value(name, environment, preferences)?;
                    let expected = parse_numeric_feature_value(name, parts[1], environment)?;
                    Some(compare(actual, expected, *operator))
                }
                (None, Some(name)) => {
                    let expected = parse_numeric_feature_value(name, parts[0], environment)?;
                    let actual = numeric_feature_value(name, environment, preferences)?;
                    Some(compare(expected, actual, *operator))
                }
                _ => Some(false),
            }
        }
        [first, second] => {
            let Some(name) = canonical_numeric_feature(parts[1]) else {
                return Some(false);
            };
            let actual = numeric_feature_value(name, environment, preferences)?;
            let lower = parse_numeric_feature_value(name, parts[0], environment)?;
            let upper = parse_numeric_feature_value(name, parts[2], environment)?;
            Some(compare(lower, actual, *first) && compare(actual, upper, *second))
        }
        _ => Some(false),
    }
}

fn split_comparisons(feature: &str) -> Option<(Vec<&str>, Vec<Comparison>)> {
    let bytes = feature.as_bytes();
    let mut parts = Vec::new();
    let mut operators = Vec::new();
    let mut start = 0usize;
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let operator = match bytes[cursor] {
            b'<' if bytes.get(cursor + 1) == Some(&b'=') => Some((Comparison::LessOrEqual, 2)),
            b'>' if bytes.get(cursor + 1) == Some(&b'=') => Some((Comparison::GreaterOrEqual, 2)),
            b'<' => Some((Comparison::Less, 1)),
            b'>' => Some((Comparison::Greater, 1)),
            b'=' => Some((Comparison::Equal, 1)),
            _ => None,
        };
        let Some((operator, length)) = operator else {
            cursor += 1;
            continue;
        };
        let part = feature[start..cursor].trim();
        if part.is_empty() {
            return Some((Vec::new(), Vec::new()));
        }
        parts.push(part);
        operators.push(operator);
        cursor += length;
        start = cursor;
    }
    if operators.is_empty() {
        return None;
    }
    let tail = feature[start..].trim();
    if tail.is_empty() {
        return Some((Vec::new(), Vec::new()));
    }
    parts.push(tail);
    if operators.len() > 2 || parts.len() != operators.len() + 1 {
        return Some((Vec::new(), Vec::new()));
    }
    Some((parts, operators))
}

fn canonical_numeric_feature(name: &str) -> Option<&str> {
    match name.trim() {
        "width"
        | "height"
        | "device-width"
        | "device-height"
        | "aspect-ratio"
        | "device-aspect-ratio"
        | "resolution"
        | "-webkit-device-pixel-ratio"
        | "color"
        | "color-index"
        | "grid"
        | "monochrome"
        | "horizontal-viewport-segments"
        | "vertical-viewport-segments"
        | "-webkit-transform-3d" => Some(name.trim()),
        _ => None,
    }
}

fn compare(left: f64, right: f64, comparison: Comparison) -> bool {
    match comparison {
        Comparison::Less => left < right && !approximately_equal(left, right),
        Comparison::LessOrEqual => left < right || approximately_equal(left, right),
        Comparison::Equal => approximately_equal(left, right),
        Comparison::GreaterOrEqual => left > right || approximately_equal(left, right),
        Comparison::Greater => left > right && !approximately_equal(left, right),
    }
}

fn ratio(numerator: f64, denominator: f64) -> Option<f64> {
    if !numerator.is_finite() || !denominator.is_finite() || numerator < 0.0 || denominator < 0.0 {
        return None;
    }
    if denominator == 0.0 {
        return Some(if numerator == 0.0 { 0.0 } else { f64::INFINITY });
    }
    Some(numerator / denominator)
}

fn parse_ratio(value: &str) -> Option<f64> {
    let value = value.trim();
    if let Some((numerator, denominator)) = value.split_once('/') {
        let numerator = parse_nonnegative_number(numerator)?;
        let denominator = parse_nonnegative_number(denominator)?;
        if denominator == 0.0 {
            return (numerator > 0.0).then_some(f64::INFINITY);
        }
        return Some(numerator / denominator);
    }
    parse_nonnegative_number(value)
}

fn parse_length(value: &str, environment: MediaEnvironment) -> Option<f64> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    let (viewport_width, viewport_height) = environment.query_viewport();
    let units = [
        ("vmin", viewport_width.min(viewport_height) / 100.0),
        ("vmax", viewport_width.max(viewport_height) / 100.0),
        ("dvw", viewport_width / 100.0),
        ("dvh", viewport_height / 100.0),
        ("svw", viewport_width / 100.0),
        ("svh", viewport_height / 100.0),
        ("lvw", viewport_width / 100.0),
        ("lvh", viewport_height / 100.0),
        ("rem", 16.0),
        ("px", 1.0),
        ("em", 16.0),
        ("vw", viewport_width / 100.0),
        ("vh", viewport_height / 100.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("q", 96.0 / 101.6),
        ("in", 96.0),
        ("pc", 16.0),
        ("pt", 96.0 / 72.0),
    ];
    for (unit, factor) in units {
        if let Some(number) = value.strip_suffix(unit) {
            return parse_nonnegative_number(number).map(|number| number * factor);
        }
    }
    None
}

fn parse_resolution(value: &str) -> Option<f64> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    if let Some(value) = value.strip_suffix("dppx") {
        return parse_nonnegative_number(value);
    }
    if let Some(value) = value.strip_suffix("x") {
        return parse_nonnegative_number(value);
    }
    if let Some(value) = value.strip_suffix("dpi") {
        return parse_nonnegative_number(value).map(|value| value / 96.0);
    }
    if let Some(value) = value.strip_suffix("dpcm") {
        return parse_nonnegative_number(value).map(|value| value * 2.54 / 96.0);
    }
    None
}

fn parse_nonnegative_number(value: &str) -> Option<f64> {
    let value = value.trim().parse::<f64>().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

fn strip_keyword_prefix<'a>(value: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = value.strip_prefix(keyword)?;
    rest.strip_prefix(char::is_whitespace).map(str::trim_start)
}

fn split_top_level_character(value: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if character == separator && depth == 0 => {
                parts.push(value[start..index].trim());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(value[start..].trim());
    parts
}

fn split_top_level_keyword<'a>(value: &'a str, keyword: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let bytes = value.as_bytes();
    let keyword = keyword.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0
            && bytes[cursor..].starts_with(keyword)
            && cursor > 0
            && bytes[cursor - 1].is_ascii_whitespace()
            && bytes
                .get(cursor + keyword.len())
                .is_some_and(u8::is_ascii_whitespace)
        {
            parts.push(value[start..cursor].trim());
            cursor += keyword.len();
            start = cursor;
            continue;
        }
        cursor += 1;
    }
    parts.push(value[start..].trim());
    parts
}

fn has_top_level_keyword(value: &str, keyword: &str) -> bool {
    split_top_level_keyword(value, keyword).len() > 1
}

fn fully_parenthesized(value: &str) -> Option<&str> {
    let value = value.trim();
    if !value.starts_with('(') || !value.ends_with(')') {
        return None;
    }
    let mut depth = 0usize;
    for (index, character) in value.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 && index + 1 != value.len() {
                    return None;
                }
            }
            _ => {}
        }
    }
    (depth == 0).then(|| &value[1..value.len() - 1])
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

fn approximately_equal(left: f64, right: f64) -> bool {
    if left == right {
        return true;
    }
    if !left.is_finite() || !right.is_finite() {
        return false;
    }
    (left - right).abs() <= f64::EPSILON * left.abs().max(right.abs()).max(1.0) * 16.0
}

#[cfg(test)]
mod tests {
    use super::{MediaEnvironment, evaluate_query};

    fn environment() -> MediaEnvironment {
        MediaEnvironment {
            viewport_width: 1440.0,
            viewport_height: 900.0,
            device_width: 1512.0,
            device_height: 982.0,
            device_pixel_ratio: 2.0,
            color_depth: 30,
            device_posture_folded: false,
        }
    }

    #[test]
    fn viewport_dimensions_ratios_and_level_four_ranges_are_evaluated() {
        let preferences = crate::MediaPreferencesFingerprint::default();
        let environment = environment();
        for query in [
            "(width: 1440px)",
            "(min-width: 90em)",
            "(height >= 900px)",
            "(800px < width < 1600px)",
            "(aspect-ratio: 8/5)",
            "(min-aspect-ratio: 1/4)",
            "(max-aspect-ratio: 4/1)",
            "(0.250 <= aspect-ratio <= 4.000)",
            "screen and (orientation: landscape)",
        ] {
            assert!(
                evaluate_query(query, environment, &preferences),
                "{query} should match"
            );
        }
        for query in [
            "(width: 900px)",
            "(aspect-ratio: 16/9)",
            "(aspect-ratio > 2/1)",
            "(orientation: portrait)",
        ] {
            assert!(
                !evaluate_query(query, environment, &preferences),
                "{query} should not match"
            );
        }
    }

    #[test]
    fn deprecated_device_queries_read_screen_instead_of_viewport() {
        let preferences = crate::MediaPreferencesFingerprint::default();
        let environment = environment();
        for query in [
            "(device-width: 1512px)",
            "(device-height: 982px)",
            "(min-device-height: 982px)",
            "(max-device-height: 982px)",
            "(device-aspect-ratio: 1512/982)",
            "(900px <= device-height <= 1000px)",
        ] {
            assert!(
                evaluate_query(query, environment, &preferences),
                "{query} should match the physical screen"
            );
        }
        assert!(!evaluate_query(
            "(device-height: 900px)",
            environment,
            &preferences
        ));
    }

    #[test]
    fn zero_sized_viewport_does_not_fall_back_to_the_configured_screen() {
        let preferences = crate::MediaPreferencesFingerprint::default();
        let environment = MediaEnvironment {
            viewport_width: 0.0,
            viewport_height: 0.0,
            ..environment()
        };
        assert!(evaluate_query("(width: 0px)", environment, &preferences));
        assert!(evaluate_query("(height: 0px)", environment, &preferences));
        assert!(!evaluate_query("(width)", environment, &preferences));
        assert!(!evaluate_query("(height)", environment, &preferences));
        assert!(!evaluate_query(
            "(min-width: 1px)",
            environment,
            &preferences
        ));
        assert!(!evaluate_query(
            "(width: 1512px) and (height: 982px)",
            environment,
            &preferences
        ));
        assert!(evaluate_query(
            "(aspect-ratio: 0/1)",
            environment,
            &preferences
        ));
        assert!(evaluate_query(
            "(orientation: portrait)",
            environment,
            &preferences
        ));
        assert!(!evaluate_query(
            "(orientation: landscape)",
            environment,
            &preferences
        ));
        assert!(evaluate_query(
            "(device-height: 982px)",
            environment,
            &preferences
        ));
        assert!(evaluate_query(
            "(device-width: 1512px)",
            environment,
            &preferences
        ));
    }

    #[test]
    fn zero_axis_aspect_ratios_follow_edge_ratio_semantics() {
        let preferences = crate::MediaPreferencesFingerprint::default();
        let horizontal = MediaEnvironment {
            viewport_width: 100.0,
            viewport_height: 0.0,
            ..environment()
        };
        assert!(evaluate_query(
            "(aspect-ratio: 1/0)",
            horizontal,
            &preferences
        ));
        assert!(evaluate_query(
            "(orientation: landscape)",
            horizontal,
            &preferences
        ));

        let vertical = MediaEnvironment {
            viewport_width: 0.0,
            viewport_height: 100.0,
            ..environment()
        };
        assert!(evaluate_query(
            "(aspect-ratio: 0/1)",
            vertical,
            &preferences
        ));
        assert!(evaluate_query(
            "(orientation: portrait)",
            vertical,
            &preferences
        ));
    }

    #[test]
    fn resolution_color_preferences_and_logical_conditions_are_evaluated() {
        let mut preferences = crate::MediaPreferencesFingerprint::default();
        preferences.color_scheme = "dark".to_owned();
        preferences.color_gamut = "p3".to_owned();
        let environment = environment();
        for query in [
            "(resolution: 2dppx)",
            "(resolution: 192dpi)",
            "(-webkit-device-pixel-ratio: 2)",
            "(color: 10)",
            "(color-gamut: srgb)",
            "(color-gamut: p3)",
            "(prefers-color-scheme: dark)",
            "((prefers-color-scheme: light) or (resolution >= 2dppx))",
            "not print and (pointer: fine)",
        ] {
            assert!(
                evaluate_query(query, environment, &preferences),
                "{query} should match"
            );
        }
        for query in [
            "(resolution: 1dppx)",
            "(color: 8)",
            "(color-gamut: rec2020)",
            "(prefers-color-scheme: invalid)",
            "(unknown-feature: value)",
        ] {
            assert!(
                !evaluate_query(query, environment, &preferences),
                "{query} should not match"
            );
        }
    }
}
