use std::collections::HashMap;

#[derive(Clone)]
struct FontFaceRecord {
    family: String,
    style: String,
    weight: String,
    stretch: String,
    unicode_range: String,
    variant: String,
    feature_settings: String,
    display: String,
    ascent_override: String,
    descent_override: String,
    line_gap_override: String,
    size_adjust: String,
    variation_settings: String,
    status: String,
    loaded: v8::Global<v8::Promise>,
    resolver: Option<v8::Global<v8::PromiseResolver>>,
}

#[derive(Default)]
pub(crate) struct FontFaceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, FontFaceRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FontFaceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FontFace", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<FontFaceStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FontFace",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "family", get_family, set_family)?;
    crate::webidl::define_accessor(scope, prototype, "style", get_style, set_style)?;
    crate::webidl::define_accessor(scope, prototype, "weight", get_weight, set_weight)?;
    crate::webidl::define_accessor(scope, prototype, "stretch", get_stretch, set_stretch)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "unicodeRange",
        get_unicode_range,
        set_unicode_range,
    )?;
    crate::webidl::define_accessor(scope, prototype, "variant", get_variant, set_variant)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "featureSettings",
        get_feature_settings,
        set_feature_settings,
    )?;
    crate::webidl::define_accessor(scope, prototype, "display", get_display, set_display)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "ascentOverride",
        get_ascent_override,
        set_ascent_override,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "descentOverride",
        get_descent_override,
        set_descent_override,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "lineGapOverride",
        get_line_gap_override,
        set_line_gap_override,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "sizeAdjust",
        get_size_adjust,
        set_size_adjust,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "status", get_status)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "loaded", get_loaded)?;
    crate::webidl::define_method(scope, prototype, "load", 0, load)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "variationSettings",
        get_variation_settings,
        set_variation_settings,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FontFaceStore>()
        .ok_or_else(|| "FontFace state was not prepared".to_owned())?
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
            "Failed to construct 'FontFace': 2 arguments required",
        );
        return;
    }
    let family = crate::webidl::value_to_string(scope, arguments.get(0));
    if family.trim().is_empty() {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The font family is empty".to_owned(),
            "SyntaxError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let descriptors = v8::Local::<v8::Object>::try_from(arguments.get(2)).ok();
    let resolver = match v8::PromiseResolver::new(scope) {
        Some(value) => value,
        None => return,
    };
    let loaded = resolver.get_promise(scope);
    let binary_source = arguments.get(1).is_array_buffer()
        || v8::Local::<v8::ArrayBufferView>::try_from(arguments.get(1)).is_ok();
    let status = if binary_source { "loaded" } else { "unloaded" };
    if binary_source {
        let _ = resolver.resolve(scope, arguments.this().into());
    }
    let record = FontFaceRecord {
        family,
        style: option_string(scope, descriptors, "style", "normal"),
        weight: option_string(scope, descriptors, "weight", "normal"),
        stretch: option_string(scope, descriptors, "stretch", "normal"),
        unicode_range: option_string(scope, descriptors, "unicodeRange", "U+0-10FFFF"),
        variant: option_string(scope, descriptors, "variant", "normal"),
        feature_settings: option_string(scope, descriptors, "featureSettings", "normal"),
        display: option_string(scope, descriptors, "display", "auto"),
        ascent_override: option_string(scope, descriptors, "ascentOverride", "normal"),
        descent_override: option_string(scope, descriptors, "descentOverride", "normal"),
        line_gap_override: option_string(scope, descriptors, "lineGapOverride", "normal"),
        size_adjust: option_string(scope, descriptors, "sizeAdjust", "100%"),
        variation_settings: option_string(scope, descriptors, "variationSettings", "normal"),
        status: status.to_owned(),
        loaded: v8::Global::new(scope, loaded),
        resolver: (!binary_source).then(|| v8::Global::new(scope, resolver)),
    };
    scope
        .get_slot_mut::<FontFaceStore>()
        .expect("FontFace state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn option_string(
    scope: &mut v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    fallback: &str,
) -> String {
    let Some(options) = options else {
        return fallback.to_owned();
    };
    let Some(key) = v8::String::new(scope, name) else {
        return fallback.to_owned();
    };
    let Some(value) = options.get(scope, key.into()) else {
        return fallback.to_owned();
    };
    if value.is_undefined() {
        fallback.to_owned()
    } else {
        crate::webidl::value_to_string(scope, value)
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<FontFaceRecord> {
    scope
        .get_slot::<FontFaceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn string_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&FontFaceRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn string_set(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    assign: impl FnOnce(&mut FontFaceRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope.get_slot_mut::<FontFaceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        assign(record, value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

macro_rules! string_property {
    ($getter:ident, $setter:ident, $field:ident) => {
        fn $getter(
            s: &mut v8::PinScope<'_, '_>,
            a: v8::FunctionCallbackArguments<'_>,
            r: v8::ReturnValue<'_>,
        ) {
            string_get(s, a, r, |record| &record.$field);
        }
        fn $setter(
            s: &mut v8::PinScope<'_, '_>,
            a: v8::FunctionCallbackArguments<'_>,
            _: v8::ReturnValue<'_>,
        ) {
            string_set(s, a, |record, value| record.$field = value);
        }
    };
}

string_property!(get_family, set_family, family);
string_property!(get_style, set_style, style);
string_property!(get_weight, set_weight, weight);
string_property!(get_stretch, set_stretch, stretch);
string_property!(get_unicode_range, set_unicode_range, unicode_range);
string_property!(get_variant, set_variant, variant);
string_property!(get_feature_settings, set_feature_settings, feature_settings);
string_property!(get_display, set_display, display);
string_property!(get_ascent_override, set_ascent_override, ascent_override);
string_property!(get_descent_override, set_descent_override, descent_override);
string_property!(
    get_line_gap_override,
    set_line_gap_override,
    line_gap_override
);
string_property!(get_size_adjust, set_size_adjust, size_adjust);
string_property!(
    get_variation_settings,
    set_variation_settings,
    variation_settings
);

fn get_status(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    string_get(s, a, r, |record| &record.status);
}

fn get_loaded(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.loaded).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn load(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let id = arguments.this().get_identity_hash().get();
    let Some(mut record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.status != "loaded" {
        record.status = "loaded".to_owned();
        if let Some(resolver) = record.resolver.take() {
            let resolver = v8::Local::new(scope, &resolver);
            let _ = resolver.resolve(scope, arguments.this().into());
        }
        if let Some(stored) = scope
            .get_slot_mut::<FontFaceStore>()
            .and_then(|store| store.records.get_mut(&id))
        {
            *stored = record.clone();
        }
    }
    result.set(v8::Local::new(scope, &record.loaded).into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<FontFaceStore>() {
        store.constructor.remove(realm_id);
    }
}
