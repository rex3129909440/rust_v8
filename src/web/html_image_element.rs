use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ImageRequestState {
    #[default]
    Idle,
    Pending,
    Loaded,
    Broken,
}

struct PendingImageRequest {
    context: v8::Global<v8::Context>,
    element: v8::Global<v8::Object>,
    generation: u64,
    url: String,
    density: f64,
    start_time: f64,
}

#[derive(Default)]
pub(crate) struct HtmlImageElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, ImageRecord>,
    pending: VecDeque<PendingImageRequest>,
}

#[derive(Clone, Default)]
pub(crate) struct ImageRecord {
    pub(crate) alt: String,
    pub(crate) src: String,
    pub(crate) srcset: String,
    pub(crate) sizes: String,
    pub(crate) cross_origin: Option<String>,
    pub(crate) use_map: String,
    pub(crate) is_map: bool,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) referrer_policy: String,
    pub(crate) decoding: String,
    pub(crate) fetch_priority: String,
    pub(crate) loading: String,
    pub(crate) name: String,
    pub(crate) low_src: String,
    pub(crate) align: String,
    pub(crate) hspace: u32,
    pub(crate) vspace: u32,
    pub(crate) long_desc: String,
    pub(crate) border: String,
    pub(crate) browsing_topics: bool,
    pub(crate) attribution_src: String,
    pub(crate) shared_storage_writable: bool,
    pub(crate) request_state: ImageRequestState,
    pub(crate) current_src: String,
    pub(crate) natural_width: u32,
    pub(crate) natural_height: u32,
    pub(crate) intrinsic_width: u32,
    pub(crate) intrinsic_height: u32,
    pub(crate) generation: u64,
    decode_waiters: Vec<v8::Global<v8::PromiseResolver>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlImageElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLImageElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<HtmlImageElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLImageElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let parent = super::html_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;

    super::html_image_element_alt_property::define(scope, prototype)?;
    super::html_image_element_src_property::define(scope, prototype)?;
    super::html_image_element_srcset_property::define(scope, prototype)?;
    super::html_image_element_sizes_property::define(scope, prototype)?;
    super::html_image_element_cross_origin_property::define(scope, prototype)?;
    super::html_image_element_use_map_property::define(scope, prototype)?;
    super::html_image_element_is_map_property::define(scope, prototype)?;
    super::html_image_element_width_property::define(scope, prototype)?;
    super::html_image_element_height_property::define(scope, prototype)?;
    super::html_image_element_natural_width_property::define(scope, prototype)?;
    super::html_image_element_natural_height_property::define(scope, prototype)?;
    super::html_image_element_complete_property::define(scope, prototype)?;
    super::html_image_element_current_src_property::define(scope, prototype)?;
    super::html_image_element_referrer_policy_property::define(scope, prototype)?;
    super::html_image_element_decoding_property::define(scope, prototype)?;
    super::html_image_element_fetch_priority_property::define(scope, prototype)?;
    super::html_image_element_loading_property::define(scope, prototype)?;
    super::html_image_element_name_property::define(scope, prototype)?;
    super::html_image_element_lowsrc_property::define(scope, prototype)?;
    super::html_image_element_align_property::define(scope, prototype)?;
    super::html_image_element_hspace_property::define(scope, prototype)?;
    super::html_image_element_vspace_property::define(scope, prototype)?;
    super::html_image_element_long_desc_property::define(scope, prototype)?;
    super::html_image_element_border_property::define(scope, prototype)?;
    super::html_image_element_x_property::define(scope, prototype)?;
    super::html_image_element_y_property::define(scope, prototype)?;
    super::html_image_element_decode::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let tag = v8::Symbol::get_to_string_tag(scope);
    let _ = prototype.delete(scope, tag.into());
    super::html_image_element_browsing_topics_property::define(scope, prototype)?;
    super::html_image_element_attribution_src_property::define(scope, prototype)?;
    super::html_image_element_shared_storage_writable_property::define(scope, prototype)?;
    crate::webidl::define_to_string_tag(scope, prototype, "HTMLImageElement")?;

    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlImageElementStore>()
        .ok_or_else(|| "HTMLImageElement state was not prepared".to_owned())?
        .constructor
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
        return Err("cannot create HTMLImageElement".to_owned());
    }
    super::html_element::attach(scope, object, "IMG");
    let record = ImageRecord {
        width,
        height,
        decoding: "auto".to_owned(),
        fetch_priority: "auto".to_owned(),
        loading: "auto".to_owned(),
        ..ImageRecord::default()
    };
    scope
        .get_slot_mut::<HtmlImageElementStore>()
        .expect("HTMLImageElement state")
        .records
        .insert(object.get_identity_hash().get(), record);
    if width > 0 {
        super::element::set_reflected_string(scope, object, "width", width.to_string());
    }
    if height > 0 {
        super::element::set_reflected_string(scope, object, "height", height.to_string());
    }
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
) -> Option<ImageRecord> {
    scope
        .get_slot::<HtmlImageElementStore>()
        .and_then(|store| store.records.get(&object.get_identity_hash().get()))
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut ImageRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlImageElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(
            scope,
            "Illegal invocation: receiver is not an HTMLImageElement",
        );
    }
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: Option<&str>,
) {
    let normalized_name = name.to_ascii_lowercase();
    let text = value.unwrap_or_default();
    {
        let Some(record) = scope
            .get_slot_mut::<HtmlImageElementStore>()
            .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
        else {
            return;
        };
        match normalized_name.as_str() {
            "alt" => record.alt = text.to_owned(),
            "src" => record.src = text.to_owned(),
            "srcset" => record.srcset = text.to_owned(),
            "sizes" => record.sizes = text.to_owned(),
            "crossorigin" => record.cross_origin = value.map(str::to_owned),
            "usemap" => record.use_map = text.to_owned(),
            "ismap" => record.is_map = value.is_some(),
            "width" => record.width = text.parse().unwrap_or(0),
            "height" => record.height = text.parse().unwrap_or(0),
            "referrerpolicy" => record.referrer_policy = text.to_owned(),
            "decoding" => record.decoding = text.to_owned(),
            "fetchpriority" => record.fetch_priority = text.to_owned(),
            "loading" => record.loading = text.to_owned(),
            "name" => record.name = text.to_owned(),
            "lowsrc" => record.low_src = text.to_owned(),
            "align" => record.align = text.to_owned(),
            "hspace" => record.hspace = text.parse().unwrap_or(0),
            "vspace" => record.vspace = text.parse().unwrap_or(0),
            "longdesc" => record.long_desc = text.to_owned(),
            "border" => record.border = text.to_owned(),
            "browsingtopics" => record.browsing_topics = value.is_some(),
            "attributionsrc" => record.attribution_src = text.to_owned(),
            "sharedstoragewritable" => record.shared_storage_writable = value.is_some(),
            _ => {}
        }
    }
    if matches!(normalized_name.as_str(), "src" | "srcset" | "sizes") {
        schedule_image_update(scope, object);
    }
}

pub(crate) fn get_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::reflected_string(scope, arguments.this(), name).unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn set_reflected_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::element::set_reflected_string(scope, arguments.this(), name, value);
}

pub(crate) fn get_reflected_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    if let Some(value) = record(scope, arguments.this())
        .and_then(|_| super::element::reflected_boolean(scope, arguments.this(), name))
    {
        result.set(v8::Boolean::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_reflected_boolean(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).boolean_value(scope);
    super::element::set_reflected_boolean(scope, arguments.this(), name, value);
}

pub(crate) fn get_reflected_unsigned(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::attribute_value(scope, arguments.this(), name)
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0);
    result.set(v8::Integer::new_from_unsigned(scope, value).into());
}

pub(crate) fn set_reflected_unsigned(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    name: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    super::element::set_reflected_string(scope, arguments.this(), name, value.to_string());
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

pub(crate) fn get_alt(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.alt);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_alt(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.alt = value);
}

pub(crate) fn get_browsing_topics(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.browsing_topics).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_browsing_topics(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.browsing_topics = value
    });
}

pub(crate) fn get_attribution_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.attribution_src);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_attribution_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        record.attribution_src = value
    });
}

pub(crate) fn get_shared_storage_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.shared_storage_writable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_shared_storage_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.shared_storage_writable = value
    });
}

pub(crate) fn get_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value =
        super::element::resolved_url_attribute(scope, arguments.this(), "src").unwrap_or_default();
    return_string(scope, &mut result, &value);
}

pub(crate) fn set_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    super::element::set_reflected_string(scope, arguments.this(), "src", value);
}

pub(crate) fn get_srcset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.srcset);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_srcset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.srcset = value);
}

pub(crate) fn get_sizes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.sizes);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_sizes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.sizes = value);
}

pub(crate) fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        match record.cross_origin {
            Some(value) => return_string(scope, &mut result, &value),
            None => result.set(v8::null(scope).into()),
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
    let value = arguments.get(0);
    let normalized = if value.is_null() {
        None
    } else {
        let value = crate::webidl::value_to_string(scope, value);
        Some(if value == "use-credentials" {
            value
        } else {
            "anonymous".to_owned()
        })
    };
    update(scope, arguments.this(), |record| {
        record.cross_origin = normalized
    });
}

pub(crate) fn get_use_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.use_map);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_use_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.use_map = value);
}

pub(crate) fn get_is_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.is_map).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_is_map(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.is_map = value);
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
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| record.width = value);
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
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| record.height = value);
}

pub(crate) fn get_natural_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.natural_width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_natural_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.natural_height).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_complete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(
            v8::Boolean::new(scope, record.request_state != ImageRequestState::Pending).into(),
        );
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
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

pub(crate) fn get_referrer_policy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.referrer_policy);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_referrer_policy(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let accepted = matches!(
        value.as_str(),
        "" | "no-referrer"
            | "origin"
            | "no-referrer-when-downgrade"
            | "origin-when-cross-origin"
            | "unsafe-url"
            | "same-origin"
            | "strict-origin"
            | "strict-origin-when-cross-origin"
    );
    update(scope, arguments.this(), |record| {
        record.referrer_policy = if accepted { value } else { String::new() }
    });
}

pub(crate) fn get_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.decoding);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_decoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if matches!(value.as_str(), "sync" | "async" | "auto") {
        value
    } else {
        "auto".to_owned()
    };
    update(scope, arguments.this(), |record| record.decoding = value);
}

pub(crate) fn get_fetch_priority(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.fetch_priority);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_fetch_priority(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let value = if matches!(value.as_str(), "low" | "auto" | "high") {
        value
    } else {
        "auto".to_owned()
    };
    update(scope, arguments.this(), |record| {
        record.fetch_priority = value
    });
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
    let value = if matches!(value.as_str(), "lazy" | "eager" | "auto") {
        value
    } else {
        "auto".to_owned()
    };
    update(scope, arguments.this(), |record| record.loading = value);
}

pub(crate) fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.name);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.name = value);
}

pub(crate) fn get_low_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.low_src);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_low_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.low_src = value);
}

pub(crate) fn get_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.align);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.align = value);
}

pub(crate) fn get_hspace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.hspace).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_hspace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| record.hspace = value);
}

pub(crate) fn get_vspace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.vspace).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_vspace(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).uint32_value(scope).unwrap_or(0);
    update(scope, arguments.this(), |record| record.vspace = value);
}

pub(crate) fn get_long_desc(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.long_desc);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_long_desc(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.long_desc = value);
}

pub(crate) fn get_border(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &mut result, &record.border);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_border(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if arguments.get(0).is_null() {
        String::new()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    update(scope, arguments.this(), |record| record.border = value);
}

pub(crate) fn get_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Integer::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn decode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    decode_promise(scope, arguments.this(), &mut result);
}

pub(crate) fn decode_promise(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    result: &mut v8::ReturnValue<'_>,
) {
    let Some(state) = record(scope, object).map(|record| record.request_state) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        crate::webidl::throw_type_error(scope, "Cannot create decode promise");
        return;
    };
    let promise = resolver.get_promise(scope);
    match state {
        ImageRequestState::Loaded => {
            let _ = resolver.resolve(scope, v8::undefined(scope).into());
        }
        ImageRequestState::Pending => {
            let resolver = v8::Global::new(scope, resolver);
            if let Some(record) = scope
                .get_slot_mut::<HtmlImageElementStore>()
                .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
            {
                record.decode_waiters.push(resolver);
            }
        }
        ImageRequestState::Idle | ImageRequestState::Broken => {
            reject_decode_resolver(scope, resolver);
        }
    }
    result.set(promise.into());
}

pub(crate) fn display_dimensions(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32)> {
    let record = record(scope, object)?;
    dimensions_from_attributes(
        scope,
        object,
        record.intrinsic_width,
        record.intrinsic_height,
    )
}

pub(crate) fn layout_dimensions(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32)> {
    let record = record(scope, object)?;
    dimensions_from_attributes(scope, object, record.natural_width, record.natural_height)
}

fn dimensions_from_attributes(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    intrinsic_width: u32,
    intrinsic_height: u32,
) -> Option<(u32, u32)> {
    let width = super::element::attribute_value(scope, object, "width")
        .and_then(|value| value.parse::<u32>().ok());
    let height = super::element::attribute_value(scope, object, "height")
        .and_then(|value| value.parse::<u32>().ok());
    Some(match (width, height) {
        (Some(width), Some(height)) => (width, height),
        (Some(width), None) if intrinsic_width > 0 => (
            width,
            scale_dimension(width, intrinsic_height, intrinsic_width),
        ),
        (None, Some(height)) if intrinsic_height > 0 => (
            scale_dimension(height, intrinsic_width, intrinsic_height),
            height,
        ),
        (Some(width), None) => (width, 0),
        (None, Some(height)) => (0, height),
        (None, None) => (intrinsic_width, intrinsic_height),
    })
}

fn scale_dimension(specified: u32, natural_other: u32, natural_axis: u32) -> u32 {
    if natural_axis == 0 {
        return 0;
    }
    (f64::from(specified) * f64::from(natural_other) / f64::from(natural_axis))
        .round()
        .clamp(0.0, f64::from(u32::MAX)) as u32
}

fn density_corrected_dimension(value: u32, density: f64) -> u32 {
    if !density.is_finite() || density <= 0.0 {
        return value;
    }
    (f64::from(value) / density)
        .round()
        .clamp(0.0, f64::from(u32::MAX)) as u32
}

fn schedule_image_update(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let candidate = selected_source(scope, object);
    let (generation, abandoned_waiters) = {
        let Some(record) = scope
            .get_slot_mut::<HtmlImageElementStore>()
            .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
        else {
            return;
        };
        record.generation = record.generation.wrapping_add(1);
        record.current_src.clear();
        record.natural_width = 0;
        record.natural_height = 0;
        record.intrinsic_width = 0;
        record.intrinsic_height = 0;
        record.request_state = if candidate.is_some() {
            ImageRequestState::Pending
        } else {
            ImageRequestState::Idle
        };
        (
            record.generation,
            std::mem::take(&mut record.decode_waiters),
        )
    };
    reject_decode_waiters(scope, abandoned_waiters);
    let Some(candidate) = candidate else {
        return;
    };
    let request = PendingImageRequest {
        context: v8::Global::new(scope, scope.get_current_context()),
        element: v8::Global::new(scope, object),
        generation,
        url: candidate.url,
        density: candidate.density,
        start_time: super::performance::now_for_current_realm(scope).unwrap_or(0.0),
    };
    if let Some(store) = scope.get_slot_mut::<HtmlImageElementStore>() {
        store.pending.push_back(request);
    }
}

#[derive(Clone)]
struct SourceCandidate {
    source: String,
    density: f64,
    width: Option<f64>,
}

struct SelectedSource {
    url: String,
    density: f64,
}

fn selected_source(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SelectedSource> {
    let record = record(scope, object)?;
    let fallback = (!record.src.is_empty()).then_some(record.src);
    if record.srcset.trim().is_empty() {
        return fallback
            .and_then(|source| resolve_image_url(scope, object, &source))
            .map(|url| SelectedSource { url, density: 1.0 });
    }
    let mut candidates = parse_srcset(&record.srcset);
    if candidates.is_empty() {
        return fallback
            .and_then(|source| resolve_image_url(scope, object, &source))
            .map(|url| SelectedSource { url, density: 1.0 });
    }
    let has_width_descriptors = candidates.iter().any(|candidate| candidate.width.is_some());
    if !has_width_descriptors && let Some(source) = fallback {
        candidates.push(SourceCandidate {
            source,
            density: 1.0,
            width: None,
        });
    }
    let slot_width = source_size(scope, &record.sizes).max(1.0);
    for candidate in &mut candidates {
        if let Some(width) = candidate.width {
            candidate.density = width / slot_width;
        }
    }
    candidates.retain(|candidate| candidate.density.is_finite() && candidate.density > 0.0);
    candidates.sort_by(|left, right| left.density.total_cmp(&right.density));
    let target = super::window_view_state::device_pixel_ratio(scope).max(0.01);
    let selected = candidates
        .iter()
        .find(|candidate| candidate.density >= target)
        .or_else(|| candidates.last())?;
    resolve_image_url(scope, object, &selected.source).map(|url| SelectedSource {
        url,
        density: selected.density,
    })
}

fn parse_srcset(value: &str) -> Vec<SourceCandidate> {
    srcset_segments(value)
        .into_iter()
        .filter_map(|segment| {
            let mut parts = segment.split_ascii_whitespace();
            let source = parts.next()?.trim().to_owned();
            if source.is_empty() {
                return None;
            }
            let descriptor = parts.next().unwrap_or("1x");
            if parts.next().is_some() {
                return None;
            }
            if let Some(value) = descriptor.strip_suffix('w') {
                let width = value.parse::<f64>().ok()?;
                return (width.is_finite() && width > 0.0).then_some(SourceCandidate {
                    source,
                    density: 1.0,
                    width: Some(width),
                });
            }
            let density = descriptor
                .strip_suffix('x')
                .unwrap_or(descriptor)
                .parse::<f64>()
                .ok()?;
            (density.is_finite() && density > 0.0).then_some(SourceCandidate {
                source,
                density,
                width: None,
            })
        })
        .collect()
}

fn srcset_segments(value: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut data_separator_seen = false;
    for (index, byte) in value.bytes().enumerate() {
        if byte != b',' {
            continue;
        }
        let current = value[start..index].trim_start();
        if current.starts_with("data:") && !data_separator_seen {
            data_separator_seen = true;
            continue;
        }
        let segment = value[start..index].trim();
        if !segment.is_empty() {
            segments.push(segment);
        }
        start = index + 1;
        data_separator_seen = false;
    }
    let segment = value[start..].trim();
    if !segment.is_empty() {
        segments.push(segment);
    }
    segments
}

fn source_size(scope: &v8::PinScope<'_, '_>, sizes: &str) -> f64 {
    for item in sizes.split(',').rev() {
        let length = item
            .split_ascii_whitespace()
            .next_back()
            .unwrap_or_default();
        if let Some(value) = length.strip_suffix("px")
            && let Ok(value) = value.parse::<f64>()
            && value.is_finite()
            && value >= 0.0
        {
            return value;
        }
        if let Some(value) = length.strip_suffix("vw")
            && let Ok(value) = value.parse::<f64>()
            && value.is_finite()
            && value >= 0.0
        {
            return super::window_view_state::inner_width(scope) * value / 100.0;
        }
    }
    super::window_view_state::inner_width(scope)
}

fn resolve_image_url(
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

struct FetchedImage {
    bytes: Vec<u8>,
    content_type: String,
    content_encoding: String,
    response_status: Option<u16>,
}

impl FetchedImage {
    fn status_succeeded(&self) -> bool {
        self.response_status
            .is_none_or(|status| (200..=299).contains(&status))
    }
}

fn fetch_image(scope: &v8::PinScope<'_, '_>, url: &str) -> Result<FetchedImage, String> {
    if url.starts_with("data:") {
        let (content_type, bytes) = super::fetch_global::decode_data_url(url)?;
        return Ok(FetchedImage {
            bytes,
            content_type,
            content_encoding: String::new(),
            response_status: None,
        });
    }
    if url.starts_with("blob:") {
        let (bytes, content_type) = super::url::object_url_snapshot(scope, url)
            .ok_or_else(|| format!("Image object URL '{url}' has been revoked"))?;
        return Ok(FetchedImage {
            bytes,
            content_type,
            content_encoding: String::new(),
            response_status: None,
        });
    }
    let entry = crate::network_replay::lookup(scope, "GET", url)
        .ok_or_else(|| format!("The offline image loader cannot load '{url}'"))?;
    let content_type = header_value(&entry.headers, "content-type")
        .unwrap_or_default()
        .to_owned();
    let content_encoding = header_value(&entry.headers, "content-encoding")
        .unwrap_or_default()
        .to_owned();
    Ok(FetchedImage {
        bytes: entry.body,
        content_type,
        content_encoding,
        response_status: Some(entry.status),
    })
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let pending = scope
        .get_slot_mut::<HtmlImageElementStore>()
        .and_then(|store| store.pending.pop_front());
    let Some(pending) = pending else {
        return false;
    };
    run_image_request(scope, pending);
    true
}

fn run_image_request(scope: &mut v8::PinScope<'_, '_>, request: PendingImageRequest) {
    let context = v8::Local::new(scope, &request.context);
    let request_scope = &mut v8::ContextScope::new(scope, context);
    let element = v8::Local::new(request_scope, &request.element);
    if record(request_scope, element).is_none_or(|record| record.generation != request.generation) {
        return;
    }
    let fetched = fetch_image(request_scope, &request.url);
    let dimensions = fetched.as_ref().ok().and_then(|resource| {
        resource
            .status_succeeded()
            .then(|| image_dimensions(&resource.bytes, &resource.content_type))
            .flatten()
    });
    let succeeded = dimensions.is_some();
    let (natural_width, natural_height) = dimensions.unwrap_or_default();
    let waiters = {
        let Some(record) = request_scope
            .get_slot_mut::<HtmlImageElementStore>()
            .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
        else {
            return;
        };
        if record.generation != request.generation {
            return;
        }
        record.request_state = if succeeded {
            ImageRequestState::Loaded
        } else {
            ImageRequestState::Broken
        };
        record.current_src = request.url.clone();
        record.intrinsic_width = natural_width;
        record.intrinsic_height = natural_height;
        record.natural_width = density_corrected_dimension(natural_width, request.density);
        record.natural_height = density_corrected_dimension(natural_height, request.density);
        std::mem::take(&mut record.decode_waiters)
    };
    if succeeded {
        resolve_decode_waiters(request_scope, waiters);
    } else {
        reject_decode_waiters(request_scope, waiters);
    }
    if let Ok(resource) = &fetched
        && let Some(status) = resource.response_status
    {
        let end_time =
            super::performance::now_for_current_realm(request_scope).unwrap_or(request.start_time);
        if let Ok(entry) = super::performance_resource_timing::create_for_resource(
            request_scope,
            request.url.clone(),
            "img".to_owned(),
            request.start_time,
            (end_time - request.start_time).max(0.0),
            status,
            resource.bytes.len(),
            normalized_media_type(&resource.content_type),
            resource.content_encoding.clone(),
        ) {
            super::performance::add_entry_for_current_realm(request_scope, entry, "resource");
        }
    }
    request_scope.perform_microtask_checkpoint();
    if record(request_scope, element).is_none_or(|record| record.generation != request.generation) {
        return;
    }
    if let Ok(event) = super::event::create(request_scope, if succeeded { "load" } else { "error" })
    {
        super::event_target::dispatch(request_scope, element, event);
    }
}

fn resolve_decode_waiters(
    scope: &mut v8::PinScope<'_, '_>,
    waiters: Vec<v8::Global<v8::PromiseResolver>>,
) {
    for waiter in waiters {
        let waiter = v8::Local::new(scope, waiter);
        let _ = waiter.resolve(scope, v8::undefined(scope).into());
    }
}

fn reject_decode_waiters(
    scope: &mut v8::PinScope<'_, '_>,
    waiters: Vec<v8::Global<v8::PromiseResolver>>,
) {
    if waiters.is_empty() {
        return;
    }
    let exception = decode_exception(scope);
    for waiter in waiters {
        let waiter = v8::Local::new(scope, waiter);
        let _ = waiter.reject(scope, exception);
    }
}

fn reject_decode_resolver(
    scope: &mut v8::PinScope<'_, '_>,
    resolver: v8::Local<'_, v8::PromiseResolver>,
) {
    let exception = decode_exception(scope);
    let _ = resolver.reject(scope, exception);
}

fn decode_exception<'s>(scope: &mut v8::PinScope<'s, '_>) -> v8::Local<'s, v8::Value> {
    if let Ok(exception) = super::dom_exception::create(
        scope,
        "The source image cannot be decoded.".to_owned(),
        "EncodingError".to_owned(),
    ) {
        return exception.into();
    }
    let message = v8::String::new(scope, "The source image cannot be decoded.")
        .expect("static image decode message");
    v8::Exception::error(scope, message)
}

fn normalized_media_type(value: &str) -> String {
    value
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub(crate) fn image_dimensions(bytes: &[u8], content_type: &str) -> Option<(u32, u32)> {
    png_dimensions(bytes)
        .or_else(|| gif_dimensions(bytes))
        .or_else(|| jpeg_dimensions(bytes))
        .or_else(|| webp_dimensions(bytes))
        .or_else(|| bmp_dimensions(bytes))
        .or_else(|| ico_dimensions(bytes))
        .or_else(|| svg_dimensions(bytes, content_type))
        .filter(|(width, height)| *width > 0 && *height > 0)
}

pub(crate) fn bitmap_snapshot(
    scope: &v8::PinScope<'_, '_>,
    image: v8::Local<'_, v8::Object>,
) -> Option<(u32, u32, Vec<u8>)> {
    let record = record(scope, image)?;
    if record.request_state != ImageRequestState::Loaded
        || record.intrinsic_width == 0
        || record.intrinsic_height == 0
    {
        return None;
    }
    let length = usize::try_from(record.intrinsic_width)
        .ok()?
        .checked_mul(usize::try_from(record.intrinsic_height).ok()?)?
        .checked_mul(4)?;
    Some((
        record.intrinsic_width,
        record.intrinsic_height,
        vec![0; length],
    ))
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24
        || bytes[..8] != [137, 80, 78, 71, 13, 10, 26, 10]
        || &bytes[12..16] != b"IHDR"
    {
        return None;
    }
    Some((
        u32::from_be_bytes(bytes[16..20].try_into().ok()?),
        u32::from_be_bytes(bytes[20..24].try_into().ok()?),
    ))
}

fn gif_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 10 || !matches!(&bytes[..6], b"GIF87a" | b"GIF89a") {
        return None;
    }
    Some((
        u16::from_le_bytes(bytes[6..8].try_into().ok()?).into(),
        u16::from_le_bytes(bytes[8..10].try_into().ok()?).into(),
    ))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[..2] != [0xff, 0xd8] {
        return None;
    }
    let mut index = 2;
    while index + 3 < bytes.len() {
        if bytes[index] != 0xff {
            index += 1;
            continue;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        let marker = *bytes.get(index)?;
        index += 1;
        if matches!(marker, 0xd8 | 0xd9) {
            continue;
        }
        let length = usize::from(u16::from_be_bytes(
            bytes.get(index..index + 2)?.try_into().ok()?,
        ));
        if length < 2 || index + length > bytes.len() {
            return None;
        }
        if matches!(
            marker,
            0xc0 | 0xc1
                | 0xc2
                | 0xc3
                | 0xc5
                | 0xc6
                | 0xc7
                | 0xc9
                | 0xca
                | 0xcb
                | 0xcd
                | 0xce
                | 0xcf
        ) {
            let height = u16::from_be_bytes(bytes.get(index + 3..index + 5)?.try_into().ok()?);
            let width = u16::from_be_bytes(bytes.get(index + 5..index + 7)?.try_into().ok()?);
            return Some((width.into(), height.into()));
        }
        index += length;
    }
    None
}

fn webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 30 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    match &bytes[12..16] {
        b"VP8X" => Some((
            1 + little_endian_24(bytes.get(24..27)?),
            1 + little_endian_24(bytes.get(27..30)?),
        )),
        b"VP8L" if bytes.get(20) == Some(&0x2f) => {
            let width = 1 + u32::from(bytes[21]) + (u32::from(bytes[22] & 0x3f) << 8);
            let height = 1
                + u32::from(bytes[22] >> 6)
                + (u32::from(bytes[23]) << 2)
                + (u32::from(bytes[24] & 0x0f) << 10);
            Some((width, height))
        }
        b"VP8 " if bytes.get(23..26) == Some(&[0x9d, 0x01, 0x2a]) => Some((
            u32::from(u16::from_le_bytes(bytes[26..28].try_into().ok()?) & 0x3fff),
            u32::from(u16::from_le_bytes(bytes[28..30].try_into().ok()?) & 0x3fff),
        )),
        _ => None,
    }
}

fn little_endian_24(bytes: &[u8]) -> u32 {
    u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16)
}

fn bmp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 26 || &bytes[..2] != b"BM" {
        return None;
    }
    let width = i32::from_le_bytes(bytes[18..22].try_into().ok()?);
    let height = i32::from_le_bytes(bytes[22..26].try_into().ok()?);
    Some((width.unsigned_abs(), height.unsigned_abs()))
}

fn ico_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 8 || bytes[..4] != [0, 0, 1, 0] {
        return None;
    }
    let width = if bytes[6] == 0 {
        256
    } else {
        u32::from(bytes[6])
    };
    let height = if bytes[7] == 0 {
        256
    } else {
        u32::from(bytes[7])
    };
    Some((width, height))
}

fn svg_dimensions(bytes: &[u8], content_type: &str) -> Option<(u32, u32)> {
    let source = std::str::from_utf8(bytes).ok()?;
    let trimmed = source.trim_start();
    if normalized_media_type(content_type) != "image/svg+xml"
        && !trimmed.starts_with("<svg")
        && !trimmed.starts_with("<?xml")
    {
        return None;
    }
    let svg_start = source.to_ascii_lowercase().find("<svg")?;
    let svg_end = source[svg_start..]
        .find('>')
        .map(|offset| svg_start + offset)?;
    let tag = &source[svg_start..=svg_end];
    let width = svg_numeric_attribute(tag, "width");
    let height = svg_numeric_attribute(tag, "height");
    let view_box = svg_attribute(tag, "viewbox").and_then(|value| {
        let numbers = value
            .split(|character: char| character.is_ascii_whitespace() || character == ',')
            .filter(|value| !value.is_empty())
            .filter_map(|value| value.parse::<f64>().ok())
            .collect::<Vec<_>>();
        (numbers.len() == 4 && numbers[2] > 0.0 && numbers[3] > 0.0)
            .then_some((numbers[2], numbers[3]))
    });
    let (width, height) = match (width, height, view_box) {
        (Some(width), Some(height), _) => (width, height),
        (Some(width), None, Some((view_width, view_height))) => {
            (width, width * view_height / view_width)
        }
        (None, Some(height), Some((view_width, view_height))) => {
            (height * view_width / view_height, height)
        }
        (Some(width), None, None) => (width, 150.0),
        (None, Some(height), None) => (300.0, height),
        (None, None, _) => (300.0, 150.0),
    };
    Some((
        width.round().clamp(0.0, f64::from(u32::MAX)) as u32,
        height.round().clamp(0.0, f64::from(u32::MAX)) as u32,
    ))
}

fn svg_numeric_attribute(tag: &str, name: &str) -> Option<f64> {
    let value = svg_attribute(tag, name)?;
    if value.trim_end().ends_with('%') {
        return None;
    }
    let number = value.trim().trim_end_matches("px").parse::<f64>().ok()?;
    (number.is_finite() && number >= 0.0).then_some(number)
}

fn svg_attribute(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut offset = 0;
    while let Some(found) = lower[offset..].find(name) {
        let start = offset + found;
        let before = lower[..start].chars().next_back();
        let after_name = start + name.len();
        if before.is_some_and(|character| {
            !character.is_ascii_whitespace() && !matches!(character, '<' | '/')
        }) {
            offset = after_name;
            continue;
        }
        let rest = tag.get(after_name..)?.trim_start();
        let rest = rest.strip_prefix('=')?.trim_start();
        let quote = rest.chars().next()?;
        if matches!(quote, '"' | '\'') {
            let value = &rest[quote.len_utf8()..];
            let end = value.find(quote)?;
            return Some(value[..end].to_owned());
        }
        let end = rest
            .find(|character: char| character.is_ascii_whitespace() || character == '>')
            .unwrap_or(rest.len());
        return Some(rest[..end].to_owned());
    }
    None
}
