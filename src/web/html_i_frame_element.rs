use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct IFrameRecord {
    sequence: u64,
    element: v8::Global<v8::Object>,
    pub(crate) src: String,
    pub(crate) srcdoc: String,
    srcdoc_present: bool,
    pub(crate) name: String,
    pub(crate) sandbox: v8::Global<v8::Object>,
    pub(crate) allow_fullscreen: bool,
    pub(crate) width: String,
    pub(crate) height: String,
    pub(crate) referrer_policy: String,
    pub(crate) csp: String,
    pub(crate) allow: String,
    pub(crate) feature_policy: v8::Global<v8::Object>,
    pub(crate) loading: String,
    pub(crate) align: String,
    pub(crate) scrolling: String,
    pub(crate) frame_border: String,
    pub(crate) long_desc: String,
    pub(crate) margin_height: String,
    pub(crate) margin_width: String,
    pub(crate) credentialless: bool,
    pub(crate) allow_payment_request: bool,
    pub(crate) private_token: String,
    pub(crate) browsing_topics: bool,
    pub(crate) ad_auction_headers: bool,
    pub(crate) shared_storage_writable: bool,
    parent_context: v8::Global<v8::Context>,
    parent_window: v8::Global<v8::Object>,
    top_window: v8::Global<v8::Object>,
    global_template: Option<v8::Global<v8::ObjectTemplate>>,
    handler_holder: Option<v8::Global<v8::Object>>,
    context: Option<v8::Global<v8::Context>>,
    content_window: Option<v8::Global<v8::Object>>,
    content_document: Option<v8::Global<v8::Object>>,
    location: Option<v8::Global<v8::Object>>,
    cross_origin_location: Option<v8::Global<v8::Object>>,
    cross_origin_descriptors: Option<v8::Global<v8::Object>>,
    cross_origin_ancestor_location: Option<v8::Global<v8::Object>>,
    cross_origin_ancestor_descriptors: Option<v8::Global<v8::Object>>,
    loaded_srcdoc: Option<String>,
    loaded_src: Option<String>,
    same_origin: bool,
    installing_context: bool,
    exposed_parent_names: Vec<String>,
}

#[derive(Default)]
pub(crate) struct HtmlIFrameElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, IFrameRecord>,
    next_sequence: u64,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlIFrameElementStore::default());
}

pub(crate) fn enable_native_trace_for_existing_realms(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let realms = scope
        .get_slot::<HtmlIFrameElementStore>()
        .map(|store| {
            store
                .records
                .iter()
                .filter_map(|(id, record)| {
                    Some((
                        *id,
                        record.context.clone()?,
                        record.content_window.clone()?,
                        record.content_document.clone(),
                        record.location.clone(),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for (id, context, window, document, location) in realms {
        let context = v8::Local::new(scope, &context);
        let child_scope = &mut v8::ContextScope::new(scope, context);
        let window = v8::Local::new(child_scope, &window);
        let label = format!("iframe[{id}]");
        crate::trace::label_native_value(child_scope, window.into(), &label);
        if let Some(document) = document {
            let document = v8::Local::new(child_scope, &document);
            crate::trace::label_native_value(
                child_scope,
                document.into(),
                &format!("{label}.document"),
            );
        }
        if let Some(location) = location {
            let location = v8::Local::new(child_scope, &location);
            crate::trace::label_native_value(
                child_scope,
                location.into(),
                &format!("{label}.location"),
            );
        }
    }
    Ok(())
}

pub(crate) fn disable_native_trace_for_existing_realms(_: &mut v8::OwnedIsolate) {}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLIFrameElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLIFrameElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_i_frame_element_src_property::define(scope, prototype)?;
    super::html_i_frame_element_srcdoc_property::define(scope, prototype)?;
    super::html_i_frame_element_name_property::define(scope, prototype)?;
    super::html_i_frame_element_sandbox_property::define(scope, prototype)?;
    super::html_i_frame_element_allow_fullscreen_property::define(scope, prototype)?;
    super::html_i_frame_element_width_property::define(scope, prototype)?;
    super::html_i_frame_element_height_property::define(scope, prototype)?;
    super::html_i_frame_element_content_document_property::define(scope, prototype)?;
    super::html_i_frame_element_content_window_property::define(scope, prototype)?;
    super::html_i_frame_element_referrer_policy_property::define(scope, prototype)?;
    super::html_i_frame_element_csp_property::define(scope, prototype)?;
    super::html_i_frame_element_allow_property::define(scope, prototype)?;
    super::html_i_frame_element_feature_policy_property::define(scope, prototype)?;
    super::html_i_frame_element_loading_property::define(scope, prototype)?;
    super::html_i_frame_element_align_property::define(scope, prototype)?;
    super::html_i_frame_element_scrolling_property::define(scope, prototype)?;
    super::html_i_frame_element_frame_border_property::define(scope, prototype)?;
    super::html_i_frame_element_long_desc_property::define(scope, prototype)?;
    super::html_i_frame_element_margin_height_property::define(scope, prototype)?;
    super::html_i_frame_element_margin_width_property::define(scope, prototype)?;
    super::html_i_frame_element_get_svg_document::define(scope, prototype)?;
    super::html_i_frame_element_credentialless_property::define(scope, prototype)?;
    super::html_i_frame_element_allow_payment_request_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::html_i_frame_element_private_token_property::define(scope, prototype)?;
    super::html_i_frame_element_browsing_topics_property::define(scope, prototype)?;
    super::html_i_frame_element_ad_auction_headers_property::define(scope, prototype)?;
    super::html_i_frame_element_shared_storage_writable_property::define(scope, prototype)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .ok_or_else(|| "HTMLIFrameElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLIFrameElement".to_owned());
    }
    super::html_element::attach(scope, object, "IFRAME");
    let sandbox = super::dom_token_list::create(scope, "")?;
    let feature_policy = super::feature_policy::create(scope)?;
    let sandbox = v8::Global::new(scope, sandbox);
    let feature_policy = v8::Global::new(scope, feature_policy);
    let parent_context = scope.get_current_context();
    let parent_window = parent_context.global(scope);
    let top_window = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| {
            store.records.values().find_map(|record| {
                record.content_window.as_ref().and_then(|window| {
                    v8::Local::new(scope, window)
                        .strict_equals(parent_window.into())
                        .then(|| v8::Local::new(scope, &record.top_window))
                })
            })
        })
        .unwrap_or(parent_window);
    let element = v8::Global::new(scope, object);
    let parent_context = v8::Global::new(scope, parent_context);
    let parent_window = v8::Global::new(scope, parent_window);
    let top_window = v8::Global::new(scope, top_window);
    let sequence = {
        let store = scope
            .get_slot_mut::<HtmlIFrameElementStore>()
            .ok_or_else(|| "HTMLIFrameElement state was not prepared".to_owned())?;
        let sequence = store.next_sequence;
        store.next_sequence = store.next_sequence.saturating_add(1);
        sequence
    };
    if scope
        .get_slot::<HtmlIFrameElementStore>()
        .is_some_and(|store| store.records.len() >= 64)
    {
        return Err("The iframe browsing-context limit of 64 was exceeded".to_owned());
    }
    scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .ok_or_else(|| "HTMLIFrameElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            IFrameRecord {
                sequence,
                element,
                src: String::new(),
                srcdoc: String::new(),
                srcdoc_present: false,
                name: String::new(),
                sandbox,
                allow_fullscreen: false,
                width: String::new(),
                height: String::new(),
                referrer_policy: String::new(),
                csp: String::new(),
                allow: String::new(),
                feature_policy,
                loading: "auto".to_owned(),
                align: String::new(),
                scrolling: String::new(),
                frame_border: String::new(),
                long_desc: String::new(),
                margin_height: String::new(),
                margin_width: String::new(),
                credentialless: false,
                allow_payment_request: false,
                private_token: String::new(),
                browsing_topics: false,
                ad_auction_headers: false,
                shared_storage_writable: false,
                parent_context,
                parent_window,
                top_window,
                global_template: None,
                handler_holder: None,
                context: None,
                content_window: None,
                content_document: None,
                location: None,
                cross_origin_location: None,
                cross_origin_descriptors: None,
                cross_origin_ancestor_location: None,
                cross_origin_ancestor_descriptors: None,
                loaded_srcdoc: None,
                loaded_src: None,
                same_origin: true,
                installing_context: false,
                exposed_parent_names: Vec::new(),
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
) -> Option<IFrameRecord> {
    scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut IFrameRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
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
    if record(scope, object).is_none() {
        return;
    }
    let normalized = name.to_ascii_lowercase();
    let present = value.is_some();
    let value = value.unwrap_or("").to_owned();
    update(scope, object, |record| match normalized.as_str() {
        "src" => {
            record.src = value;
            record.loaded_src = None;
        }
        "srcdoc" => {
            record.srcdoc = value;
            record.srcdoc_present = present;
            record.loaded_srcdoc = None;
        }
        "name" => record.name = value,
        "width" => record.width = value,
        "height" => record.height = value,
        "referrerpolicy" => record.referrer_policy = value,
        "csp" => record.csp = value,
        "allow" => record.allow = value,
        "loading" => {
            record.loading = if value.eq_ignore_ascii_case("lazy") {
                "lazy".to_owned()
            } else {
                "auto".to_owned()
            };
        }
        "align" => record.align = value,
        "scrolling" => record.scrolling = value,
        "frameborder" => record.frame_border = value,
        "longdesc" => record.long_desc = value,
        "marginheight" => record.margin_height = value,
        "marginwidth" => record.margin_width = value,
        "allowfullscreen" => record.allow_fullscreen = present,
        "credentialless" => record.credentialless = present,
        "allowpaymentrequest" => record.allow_payment_request = present,
        "browsingtopics" => record.browsing_topics = present,
        "adauctionheaders" => record.ad_auction_headers = present,
        "sharedstoragewritable" => record.shared_storage_writable = present,
        _ => {}
    });
    if matches!(normalized.as_str(), "name" | "id") && super::node::is_connected(scope, object) {
        let _ = expose_child_window_on_parent(scope, object);
    }
    if matches!(normalized.as_str(), "src" | "srcdoc")
        && super::node::is_connected(scope, object)
        && let Err(message) =
            ensure_browsing_context(scope, object).and_then(|_| load_selected_source(scope, object))
    {
        crate::webidl::throw_type_error(scope, &message);
    }
}

fn current_iframe_record(scope: &v8::PinScope<'_, '_>) -> Option<IFrameRecord> {
    let window = scope.get_current_context().global(scope);
    scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .values()
        .find(|record| {
            record.content_window.as_ref().is_some_and(|candidate| {
                v8::Local::new(scope, candidate).strict_equals(window.into())
            })
        })
        .cloned()
}

fn iframe_record_for_handler(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::PropertyCallbackArguments<'_>,
) -> Option<IFrameRecord> {
    let identity = arguments.data().int32_value(scope)?;
    scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .get(&identity)
        .cloned()
}

pub(crate) fn current_parent_window<'s>(
    scope: &v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = current_iframe_record(scope)?;
    Some(v8::Local::new(scope, &record.parent_window))
}

pub(crate) fn current_top_window<'s>(
    scope: &v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = current_iframe_record(scope)?;
    Some(v8::Local::new(scope, &record.top_window))
}

pub(crate) fn current_frame_element<'s>(
    scope: &v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = current_iframe_record(scope)?;
    record
        .same_origin
        .then(|| v8::Local::new(scope, &record.element))
}

pub(crate) fn current_frame_element_for_layout<'s>(
    scope: &v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = current_iframe_record(scope)?;
    Some(v8::Local::new(scope, &record.element))
}

pub(crate) fn current_content_document<'s>(
    scope: &v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = current_iframe_record(scope)?;
    Some(v8::Local::new(scope, record.content_document.as_ref()?))
}

pub(crate) fn current_location<'s>(
    scope: &v8::PinScope<'s, '_>,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = current_iframe_record(scope)?;
    Some(v8::Local::new(scope, record.location.as_ref()?))
}

pub(crate) fn current_name(scope: &v8::PinScope<'_, '_>) -> Option<String> {
    Some(current_iframe_record(scope)?.name)
}

pub(crate) fn set_current_name(scope: &mut v8::PinScope<'_, '_>, value: String) -> bool {
    let window = scope.get_current_context().global(scope);
    let record_id = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| {
            store.records.iter().find_map(|(id, record)| {
                record
                    .content_window
                    .as_ref()
                    .is_some_and(|candidate| {
                        v8::Local::new(scope, candidate).strict_equals(window.into())
                    })
                    .then_some(*id)
            })
        });
    let Some(record) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&record_id?))
    else {
        return false;
    };
    record.name = value;
    true
}

pub(crate) fn get_private_token(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &value.private_token) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_private_token(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| {
        record.private_token = value
    });
}

pub(crate) fn get_browsing_topics(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_flag(scope, arguments, result, |record| record.browsing_topics);
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

pub(crate) fn get_ad_auction_headers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_flag(scope, arguments, result, |record| record.ad_auction_headers);
}

pub(crate) fn set_ad_auction_headers(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| {
        record.ad_auction_headers = value
    });
}

pub(crate) fn get_shared_storage_writable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    get_flag(scope, arguments, result, |record| {
        record.shared_storage_writable
    });
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

pub(crate) fn get_flag(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: fn(&IFrameRecord) -> bool,
) {
    if let Some(value) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, select(&value)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&IFrameRecord) -> &str,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            r.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut IFrameRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    update(scope, a.this(), |record| change(record, value));
}
pub(crate) fn get_bool(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&IFrameRecord) -> bool,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_bool(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut IFrameRecord, bool),
) {
    let value = a.get(0).boolean_value(scope);
    update(scope, a.this(), |record| change(record, value));
}
pub(crate) fn get_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.src)
}
pub(crate) fn set_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.src = v)
}
pub(crate) fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.name)
}
pub(crate) fn set_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.name = v)
}
pub(crate) fn get_sandbox(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &record.sandbox).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn set_sandbox(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let sandbox = v8::Local::new(scope, &record.sandbox);
        super::dom_token_list::set_string_value(scope, sandbox, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_allow_fullscreen(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.allow_fullscreen)
}
pub(crate) fn set_allow_fullscreen(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.allow_fullscreen = v)
}
pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.width)
}
pub(crate) fn set_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.width = v)
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.height)
}
pub(crate) fn set_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.height = v)
}
pub(crate) fn null_value(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        r.set(v8::null(scope).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn content_window<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<Option<v8::Local<'s, v8::Object>>, String> {
    if record(scope, object).is_none() {
        return Err("Illegal invocation".to_owned());
    }
    if !super::node::is_connected(scope, object) {
        return Ok(None);
    }
    ensure_browsing_context(scope, object)?;
    Ok(record(scope, object)
        .and_then(|record| record.content_window)
        .map(|window| v8::Local::new(scope, &window)))
}

pub(crate) fn content_document<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<Option<v8::Local<'s, v8::Object>>, String> {
    if content_window(scope, object)?.is_none() {
        return Ok(None);
    }
    if record(scope, object).is_some_and(|record| !record.same_origin) {
        return Ok(None);
    }
    Ok(record(scope, object)
        .and_then(|record| record.content_document)
        .map(|document| v8::Local::new(scope, &document)))
}

pub(crate) fn notify_connected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if record(scope, root).is_some() && super::node::is_connected(scope, root) {
        let _ =
            ensure_browsing_context(scope, root).and_then(|_| load_selected_source(scope, root));
    }
    for child in super::node::children(scope, root) {
        notify_connected_tree(scope, child);
    }
}

fn install_iframe_interface_prefix(
    scope: &mut v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Context>,
) -> Result<crate::intrinsics::LateIntrinsics, String> {
    let late_intrinsics = crate::intrinsics::LateIntrinsics::detach(scope, context)?;
    super::install_window_interfaces(scope)?;
    Ok(late_intrinsics)
}

fn install_iframe_window_globals(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    // Keep this order identical to install_window_globals().  The document
    // and location objects are iframe-local, but every remaining global is
    // installed by the same realm-aware module used by the root Window.
    super::window_global::install(scope)?;
    super::self_global::install(scope)?;
    super::document_global::install_existing(scope)?;
    super::window_name::install(scope)?;
    super::location_global::install_existing(scope)?;
    super::custom_elements_global::install_for_document(scope, document)?;
    super::history_global::install(scope)?;
    super::install_context_window_globals(scope)
}

fn install_iframe_late_globals(
    scope: &mut v8::PinScope<'_, '_>,
    late_intrinsics: &crate::intrinsics::LateIntrinsics,
) -> Result<(), String> {
    let temporal = v8::Local::new(scope, &late_intrinsics.temporal);
    super::temporal_global::install(scope, temporal)?;
    let suppressed_error = v8::Local::new(scope, &late_intrinsics.suppressed_error);
    super::suppressed_error_global::install(scope, suppressed_error)?;
    let disposable_stack = v8::Local::new(scope, &late_intrinsics.disposable_stack);
    super::disposable_stack_global::install(scope, disposable_stack)?;
    let async_disposable_stack = v8::Local::new(scope, &late_intrinsics.async_disposable_stack);
    super::async_disposable_stack_global::install(scope, async_disposable_stack)?;
    let float16_array = v8::Local::new(scope, &late_intrinsics.float16_array);
    super::float16_array_global::install(scope, float16_array)?;
    super::install_after_late_intrinsics(scope)?;
    let web_assembly = v8::Local::new(scope, &late_intrinsics.web_assembly);
    super::web_assembly_global::install(scope, web_assembly)?;
    super::install_after_webassembly(scope)
}

pub(crate) fn ensure_browsing_context(
    scope: &mut v8::PinScope<'_, '_>,
    iframe: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let snapshot = record(scope, iframe).ok_or_else(|| "Illegal invocation".to_owned())?;
    if snapshot.context.is_some() {
        return Ok(());
    }
    let inherited_base_url = parent_document_base_url(scope, &snapshot);
    let global_template = v8::ObjectTemplate::new(scope);
    let handler_data = v8::Integer::new(scope, iframe.get_identity_hash().get());
    global_template.set_indexed_property_handler(
        v8::IndexedPropertyHandlerConfiguration::new()
            .getter(child_window_indexed_getter)
            .setter(child_window_indexed_setter)
            .query(child_window_indexed_query)
            .deleter(child_window_indexed_deleter)
            .enumerator(child_window_indexed_enumerator)
            .definer(child_window_indexed_definer)
            .descriptor(child_window_indexed_descriptor)
            .data(handler_data.into()),
    );
    global_template.set_named_property_handler(
        v8::NamedPropertyHandlerConfiguration::new()
            .getter(child_window_named_getter)
            .setter(child_window_named_setter)
            .query(child_window_named_query)
            .deleter(child_window_named_deleter)
            .enumerator(child_window_named_enumerator)
            .definer(child_window_named_definer)
            .descriptor(child_window_named_descriptor)
            .data(handler_data.into()),
    );
    if let Some(stored) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&iframe.get_identity_hash().get()))
    {
        stored.installing_context = true;
    }
    let context = v8::Context::new(
        scope,
        v8::ContextOptions {
            global_template: Some(global_template),
            ..Default::default()
        },
    );
    let parent_context = v8::Local::new(scope, &snapshot.parent_context);
    let security_token = parent_context.get_security_token(scope);
    context.set_security_token(security_token);
    let child_window = context.global(scope);
    let handler_holder = child_window
        .get_prototype(scope)
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .ok_or_else(|| "iframe global target is unavailable".to_owned())?;
    let context_global = v8::Global::new(scope, context);
    let child_window_global = v8::Global::new(scope, child_window);
    let cross_origin_location =
        super::cross_origin_location::create(scope, iframe.get_identity_hash().get())?;
    let cross_origin_location = v8::Global::new(scope, cross_origin_location);
    let cross_origin_descriptors =
        super::cross_origin_window_descriptors::create(scope, iframe.get_identity_hash().get())?;
    let cross_origin_descriptors = v8::Global::new(scope, cross_origin_descriptors);
    let global_template_global = v8::Global::new(scope, global_template);
    let handler_holder_global = v8::Global::new(scope, handler_holder);
    {
        let stored = scope
            .get_slot_mut::<HtmlIFrameElementStore>()
            .and_then(|store| store.records.get_mut(&iframe.get_identity_hash().get()))
            .ok_or_else(|| "iframe state disappeared".to_owned())?;
        stored.context = Some(context_global);
        stored.content_window = Some(child_window_global);
        stored.cross_origin_location = Some(cross_origin_location);
        stored.cross_origin_descriptors = Some(cross_origin_descriptors);
        stored.global_template = Some(global_template_global);
        stored.handler_holder = Some(handler_holder_global);
    }

    let setup = {
        let child_scope = &mut v8::ContextScope::new(scope, context);
        let late_intrinsics = install_iframe_interface_prefix(child_scope, context)?;
        let location = super::location::create(child_scope, "about:blank")?;
        let document = super::document_global::create_document(child_scope, "about:blank")?;
        super::document::set_string_value(
            child_scope,
            document,
            "fallbackBaseURL",
            &inherited_base_url,
        );
        super::document::set_object_value(child_scope, document, "defaultView", child_window);
        install_iframe_window_globals(child_scope, document)?;
        super::performance::replace_navigation_entry(
            child_scope,
            "about:blank".to_owned(),
            0,
            0,
            "text/html".to_owned(),
        );
        install_iframe_late_globals(child_scope, &late_intrinsics)?;
        super::event_target::install(child_scope)?;
        super::event::install(child_scope)?;
        super::custom_event::install(child_scope)?;
        super::message_event::install(child_scope)?;
        super::error_event::install(child_scope)?;
        super::promise_rejection_event::install(child_scope)?;
        super::dom_exception::install(child_scope)?;
        super::close_event::install(child_scope)?;
        super::progress_event::install(child_scope)?;
        super::security_policy_violation_event::install(child_scope)?;
        super::task_priority_change_event::install(child_scope)?;
        super::webgl_context_event::install(child_scope)?;
        super::abort_signal::install(child_scope)?;
        super::abort_controller::install(child_scope)?;
        super::message_port::install(child_scope)?;
        super::message_channel::install(child_scope)?;
        super::broadcast_channel::install(child_scope)?;
        super::node::install(child_scope)?;
        super::element::install(child_scope)?;
        super::html_element::install(child_scope)?;
        super::character_data::install(child_scope)?;
        super::text::install(child_scope)?;
        super::comment::install(child_scope)?;
        super::document_fragment::install(child_scope)?;
        super::attr::install(child_scope)?;
        super::document_type::install(child_scope)?;
        super::cdata_section::install(child_scope)?;
        super::processing_instruction::install(child_scope)?;
        super::document::install(child_scope)?;
        super::html_document::install(child_scope)?;
        super::html_html_element::install(child_scope)?;
        super::html_head_element::install(child_scope)?;
        super::html_body_element::install(child_scope)?;
        super::html_div_element::install(child_scope)?;
        super::html_anchor_element::install(child_scope)?;
        super::html_area_element::install(child_scope)?;
        super::html_media_element::install(child_scope)?;
        super::html_audio_element::install(child_scope)?;
        super::html_video_element::install(child_scope)?;
        super::html_button_element::install(child_scope)?;
        super::html_canvas_element::install(child_scope)?;
        super::html_form_element::install(child_scope)?;
        super::html_i_frame_element::install(child_scope)?;
        super::html_image_element::install(child_scope)?;
        super::html_input_element::install(child_scope)?;
        super::html_link_element::install(child_scope)?;
        super::html_meta_element::install(child_scope)?;
        super::html_script_element::install(child_scope)?;
        super::html_select_element::install(child_scope)?;
        super::html_span_element::install(child_scope)?;
        super::html_style_element::install(child_scope)?;
        super::html_table_element::install(child_scope)?;
        super::html_text_area_element::install(child_scope)?;
        super::html_base_element::install(child_scope)?;
        super::html_br_element::install(child_scope)?;
        super::html_d_list_element::install(child_scope)?;
        super::html_data_element::install(child_scope)?;
        super::html_data_list_element::install(child_scope)?;
        super::html_details_element::install(child_scope)?;
        super::html_dialog_element::install(child_scope)?;
        super::html_directory_element::install(child_scope)?;
        super::html_embed_element::install(child_scope)?;
        super::html_fenced_frame_element::install(child_scope)?;
        super::html_field_set_element::install(child_scope)?;
        super::html_font_element::install(child_scope)?;
        super::html_frame_element::install(child_scope)?;
        super::html_frame_set_element::install(child_scope)?;
        super::html_geolocation_element::install(child_scope)?;
        super::html_heading_element::install(child_scope)?;
        super::html_hr_element::install(child_scope)?;
        super::html_label_element::install(child_scope)?;
        super::html_legend_element::install(child_scope)?;
        super::html_li_element::install(child_scope)?;
        super::html_map_element::install(child_scope)?;
        super::html_marquee_element::install(child_scope)?;
        super::html_menu_element::install(child_scope)?;
        super::html_meter_element::install(child_scope)?;
        super::html_mod_element::install(child_scope)?;
        super::html_o_list_element::install(child_scope)?;
        super::html_object_element::install(child_scope)?;
        super::html_opt_group_element::install(child_scope)?;
        super::html_option_element::install(child_scope)?;
        super::html_output_element::install(child_scope)?;
        super::html_paragraph_element::install(child_scope)?;
        super::html_param_element::install(child_scope)?;
        super::html_picture_element::install(child_scope)?;
        super::html_pre_element::install(child_scope)?;
        super::html_progress_element::install(child_scope)?;
        super::html_quote_element::install(child_scope)?;
        super::html_selected_content_element::install(child_scope)?;
        super::html_slot_element::install(child_scope)?;
        super::html_source_element::install(child_scope)?;
        super::html_table_caption_element::install(child_scope)?;
        super::html_table_cell_element::install(child_scope)?;
        super::html_table_col_element::install(child_scope)?;
        super::html_table_row_element::install(child_scope)?;
        super::html_table_section_element::install(child_scope)?;
        super::html_template_element::install(child_scope)?;
        super::html_time_element::install(child_scope)?;
        super::html_title_element::install(child_scope)?;
        super::html_track_element::install(child_scope)?;
        super::html_u_list_element::install(child_scope)?;
        super::html_unknown_element::install(child_scope)?;
        super::location::install(child_scope)?;
        super::history::install(child_scope)?;
        super::custom_element_registry::install(child_scope)?;
        super::cookie_store::install(child_scope)?;
        super::scheduler::install(child_scope)?;
        super::trusted_type_policy_factory::install(child_scope)?;
        super::cache::install(child_scope)?;
        super::cache_storage::install(child_scope)?;
        super::idb_factory::install(child_scope)?;
        super::storage::install(child_scope)?;
        super::url::install_standard_name(child_scope)?;
        super::url_search_params::install_global(child_scope)?;
        super::url_pattern::install(child_scope)?;
        super::blob::install(child_scope)?;
        super::file::install(child_scope)?;
        super::file_reader::install(child_scope)?;
        super::headers::install(child_scope)?;
        super::request::install(child_scope)?;
        super::response::install(child_scope)?;
        super::form_data::install(child_scope)?;
        super::xml_http_request_upload::install(child_scope)?;
        super::xml_http_request_event_target::install(child_scope)?;
        super::xml_http_request::install(child_scope)?;
        super::shared_worker::install(child_scope)?;
        super::speech_synthesis_event::install(child_scope)?;
        super::speech_synthesis_error_event::install(child_scope)?;
        super::speech_synthesis_utterance::install(child_scope)?;
        super::speech_synthesis_voice::install(child_scope)?;
        super::speech_synthesis::install(child_scope)?;
        super::offscreen_canvas::install(child_scope)?;
        super::offscreen_canvas_rendering_context_2d::install(child_scope)?;
        super::webgl_rendering_context::install(child_scope)?;
        super::webgl2_rendering_context::install(child_scope)?;
        super::window::install(child_scope)?;
        super::event_target::attach(child_scope, child_window);
        super::navigator::install(child_scope)?;
        super::screen::install(child_scope)?;
        super::subtle_crypto::install(child_scope)?;
        super::crypto_key::install(child_scope)?;
        super::crypto::install(child_scope)?;
        super::performance::install(child_scope)?;
        crate::locale_runtime::install(child_scope)?;
        crate::determinism::install(child_scope)?;
        crate::iframe_hook::run_for_current_iframe(child_scope, "about:blank")?;
        if crate::trace::is_enabled(child_scope) {
            let label = format!("iframe[{}]", iframe.get_identity_hash().get());
            crate::trace::label_native_value(child_scope, child_window.into(), &label);
            crate::trace::label_native_value(
                child_scope,
                document.into(),
                &format!("{label}.document"),
            );
            crate::trace::label_native_value(
                child_scope,
                location.into(),
                &format!("{label}.location"),
            );
        }
        Ok::<_, String>((
            v8::Global::new(child_scope, document),
            v8::Global::new(child_scope, location),
        ))
    };
    match setup {
        Ok((document, location)) => {
            let stored = scope
                .get_slot_mut::<HtmlIFrameElementStore>()
                .and_then(|store| store.records.get_mut(&iframe.get_identity_hash().get()))
                .ok_or_else(|| "iframe state disappeared".to_owned())?;
            stored.content_document = Some(document);
            stored.location = Some(location);
            stored.loaded_src = Some("about:blank".to_owned());
            stored.loaded_srcdoc = None;
            stored.same_origin = true;
            stored.installing_context = false;
            expose_child_window_on_parent(scope, iframe)?;
            Ok(())
        }
        Err(error) => {
            if let Some(stored) = scope
                .get_slot_mut::<HtmlIFrameElementStore>()
                .and_then(|store| store.records.get_mut(&iframe.get_identity_hash().get()))
            {
                stored.context = None;
                stored.content_window = None;
                stored.installing_context = false;
            }
            Err(error)
        }
    }
}

struct IFrameNavigation {
    url: String,
    fallback_base_url: String,
    html: String,
    content_type: String,
    response_status: u16,
    same_origin: bool,
    loaded_srcdoc: Option<String>,
    loaded_src: Option<String>,
}

pub(crate) fn load_selected_source(
    scope: &mut v8::PinScope<'_, '_>,
    iframe: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let snapshot = record(scope, iframe).ok_or_else(|| "Illegal invocation".to_owned())?;
    let parent_url = parent_document_url(scope, &snapshot);
    let parent_base_url = parent_document_base_url(scope, &snapshot);
    let resource_start_time = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
    let mut timing_replay = None;
    let navigation = if snapshot.srcdoc_present {
        if snapshot.loaded_srcdoc.as_ref() == Some(&snapshot.srcdoc) {
            return Ok(());
        }
        IFrameNavigation {
            url: "about:srcdoc".to_owned(),
            fallback_base_url: parent_base_url.clone(),
            html: snapshot.srcdoc.clone(),
            content_type: "text/html".to_owned(),
            response_status: 0,
            same_origin: true,
            loaded_srcdoc: Some(snapshot.srcdoc),
            loaded_src: None,
        }
    } else if !snapshot.src.is_empty() {
        let base = url::Url::parse(&parent_base_url)
            .map_err(|_| "iframe parent base URL is invalid".to_owned())?;
        let resolved = base
            .join(&snapshot.src)
            .map_err(|_| "iframe src is invalid".to_owned())?;
        if !matches!(resolved.scheme(), "http" | "https") {
            return Err("iframe src must resolve to HTTP(S)".to_owned());
        }
        let resolved = resolved.as_str().to_owned();
        if snapshot.loaded_src.as_deref() == Some(&resolved) {
            return Ok(());
        }
        let replay = crate::network_replay::lookup(scope, "GET", &resolved)
            .ok_or_else(|| format!("no deterministic network replay entry for {resolved}"))?;
        timing_replay = Some(replay.clone());
        let content_type = replay
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
            .map(|(_, value)| {
                value
                    .split_once(';')
                    .map_or(value.as_str(), |(essence, _)| essence)
                    .trim()
                    .to_ascii_lowercase()
            })
            .unwrap_or_else(|| "text/html".to_owned());
        if !matches!(content_type.as_str(), "text/html" | "application/xhtml+xml") {
            return Err(format!(
                "iframe replay content type {content_type} is not an HTML document"
            ));
        }
        IFrameNavigation {
            same_origin: urls_share_origin(&parent_url, &resolved),
            url: resolved.clone(),
            fallback_base_url: resolved.clone(),
            html: String::from_utf8_lossy(&replay.body).into_owned(),
            content_type,
            response_status: replay.status,
            loaded_srcdoc: None,
            loaded_src: Some(resolved),
        }
    } else {
        if snapshot.loaded_src.as_deref() == Some("about:blank") && snapshot.loaded_srcdoc.is_none()
        {
            return Ok(());
        }
        IFrameNavigation {
            url: "about:blank".to_owned(),
            fallback_base_url: parent_base_url,
            html: String::new(),
            content_type: "text/html".to_owned(),
            response_status: 0,
            same_origin: true,
            loaded_srcdoc: None,
            loaded_src: Some("about:blank".to_owned()),
        }
    };
    let result = navigate_browsing_context(scope, iframe, &parent_url, navigation);
    if result.is_ok()
        && let Some(replay) = timing_replay.as_ref()
    {
        super::performance_resource_timing::record_network_replay(
            scope,
            replay,
            "iframe",
            resource_start_time,
        );
    }
    result
}

pub(crate) fn navigate_cross_origin_location(
    scope: &mut v8::PinScope<'_, '_>,
    iframe_id: i32,
    value: String,
) -> Result<(), String> {
    let iframe = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get(&iframe_id))
        .map(|record| record.element.clone())
        .ok_or_else(|| "Cross-origin WindowProxy is detached".to_owned())?;
    if let Some(record) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&iframe_id))
    {
        record.src = value;
        record.srcdoc_present = false;
        record.loaded_src = None;
        record.loaded_srcdoc = None;
    }
    let iframe = v8::Local::new(scope, &iframe);
    match load_selected_source(scope, iframe) {
        Ok(()) => Ok(()),
        Err(error) if error.starts_with("no deterministic network replay entry for ") => {
            let snapshot = record(scope, iframe)
                .ok_or_else(|| "iframe state disappeared during navigation".to_owned())?;
            let parent_url = parent_document_url(scope, &snapshot);
            let parent_base_url = parent_document_base_url(scope, &snapshot);
            let base = url::Url::parse(&parent_base_url)
                .map_err(|_| "iframe parent base URL is invalid".to_owned())?;
            let resolved = base
                .join(&snapshot.src)
                .map_err(|_| "iframe src is invalid".to_owned())?;
            if urls_share_origin(&parent_url, resolved.as_str()) {
                return Err(error);
            }
            if let Some(record) = scope
                .get_slot_mut::<HtmlIFrameElementStore>()
                .and_then(|store| store.records.get_mut(&iframe_id))
            {
                // The sandbox never performs an unconfigured external
                // request.  It still commits the observable WindowProxy
                // origin transition so the embedding realm immediately loses
                // same-origin access, matching a browser navigation commit.
                record.same_origin = false;
                record.loaded_src = Some(resolved.as_str().to_owned());
                record.loaded_srcdoc = None;
            }
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn navigate_for_location_object(
    scope: &mut v8::PinScope<'_, '_>,
    location: v8::Local<'_, v8::Object>,
    value: String,
) -> Option<Result<(), String>> {
    let iframe_id = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .iter()
        .find_map(|(id, record)| {
            record.location.as_ref().and_then(|candidate| {
                v8::Local::new(scope, candidate)
                    .strict_equals(location.into())
                    .then_some(*id)
            })
        })?;
    Some(navigate_cross_origin_location(scope, iframe_id, value))
}

fn navigate_browsing_context(
    scope: &mut v8::PinScope<'_, '_>,
    iframe: v8::Local<'_, v8::Object>,
    parent_url: &str,
    navigation: IFrameNavigation,
) -> Result<(), String> {
    let document_referrer =
        iframe_document_referrer(parent_url, &navigation.url, navigation.same_origin);
    let snapshot = record(scope, iframe).ok_or_else(|| "Illegal invocation".to_owned())?;
    let global_template = snapshot
        .global_template
        .as_ref()
        .map(|template| v8::Local::new(scope, template))
        .ok_or_else(|| "iframe global template is unavailable".to_owned())?;
    let previous_window = snapshot
        .content_window
        .as_ref()
        .map(|window| v8::Local::new(scope, window))
        .ok_or_else(|| "iframe WindowProxy is unavailable".to_owned())?;
    if let Some(stored) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&iframe.get_identity_hash().get()))
    {
        stored.installing_context = true;
    }
    let context = v8::Context::new(
        scope,
        v8::ContextOptions {
            global_template: Some(global_template),
            global_object: Some(previous_window.into()),
            ..Default::default()
        },
    );
    let parent_context = v8::Local::new(scope, &snapshot.parent_context);
    let security_token = parent_context.get_security_token(scope);
    context.set_security_token(security_token);
    let child_window = context.global(scope);
    if !child_window.strict_equals(previous_window.into()) {
        return Err("iframe navigation did not preserve WindowProxy identity".to_owned());
    }
    let prepared = {
        let child_scope = &mut v8::ContextScope::new(scope, context);
        let late_intrinsics = install_iframe_interface_prefix(child_scope, context)?;
        let location = super::location::create(child_scope, &navigation.url)?;
        let document = super::document_global::create_document(child_scope, &navigation.url)?;
        super::document::set_string_value(
            child_scope,
            document,
            "fallbackBaseURL",
            &navigation.fallback_base_url,
        );
        if !navigation.html.is_empty() {
            super::document_html_parser::parse_page(child_scope, document, &navigation.html)?;
        }
        super::document::set_content_type(child_scope, document, navigation.content_type.clone());
        super::document::set_string_value(child_scope, document, "referrer", &document_referrer);
        let domain = url::Url::parse(&navigation.url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_owned))
            .or_else(|| {
                url::Url::parse(parent_url)
                    .ok()
                    .and_then(|url| url.host_str().map(str::to_owned))
            })
            .unwrap_or_default();
        super::document::set_string_value(child_scope, document, "domain", &domain);
        super::document::set_object_value(child_scope, document, "defaultView", child_window);
        let (ancestor_location, ancestor_descriptors) = if navigation.same_origin {
            (None, None)
        } else {
            let ancestor_location = super::cross_origin_ancestor_location::create(child_scope)?;
            let ancestor_descriptors = super::cross_origin_window_descriptors::create_ancestor(
                child_scope,
                iframe.get_identity_hash().get(),
            )?;
            (
                Some(v8::Global::new(child_scope, ancestor_location)),
                Some(v8::Global::new(child_scope, ancestor_descriptors)),
            )
        };
        install_iframe_window_globals(child_scope, document)?;
        super::performance::replace_navigation_entry(
            child_scope,
            navigation.url.clone(),
            navigation.response_status,
            navigation.html.len(),
            navigation.content_type.clone(),
        );
        install_iframe_late_globals(child_scope, &late_intrinsics)?;
        super::event_target::install(child_scope)?;
        super::event::install(child_scope)?;
        super::custom_event::install(child_scope)?;
        super::message_event::install(child_scope)?;
        super::error_event::install(child_scope)?;
        super::promise_rejection_event::install(child_scope)?;
        super::dom_exception::install(child_scope)?;
        super::close_event::install(child_scope)?;
        super::progress_event::install(child_scope)?;
        super::security_policy_violation_event::install(child_scope)?;
        super::task_priority_change_event::install(child_scope)?;
        super::webgl_context_event::install(child_scope)?;
        super::abort_signal::install(child_scope)?;
        super::abort_controller::install(child_scope)?;
        super::message_port::install(child_scope)?;
        super::message_channel::install(child_scope)?;
        super::broadcast_channel::install(child_scope)?;
        super::node::install(child_scope)?;
        super::element::install(child_scope)?;
        super::html_element::install(child_scope)?;
        super::character_data::install(child_scope)?;
        super::text::install(child_scope)?;
        super::comment::install(child_scope)?;
        super::document_fragment::install(child_scope)?;
        super::attr::install(child_scope)?;
        super::document_type::install(child_scope)?;
        super::cdata_section::install(child_scope)?;
        super::processing_instruction::install(child_scope)?;
        super::document::install(child_scope)?;
        super::html_document::install(child_scope)?;
        super::html_html_element::install(child_scope)?;
        super::html_head_element::install(child_scope)?;
        super::html_body_element::install(child_scope)?;
        super::html_div_element::install(child_scope)?;
        super::html_anchor_element::install(child_scope)?;
        super::html_area_element::install(child_scope)?;
        super::html_media_element::install(child_scope)?;
        super::html_audio_element::install(child_scope)?;
        super::html_video_element::install(child_scope)?;
        super::html_button_element::install(child_scope)?;
        super::html_canvas_element::install(child_scope)?;
        super::html_form_element::install(child_scope)?;
        super::html_i_frame_element::install(child_scope)?;
        super::html_image_element::install(child_scope)?;
        super::html_input_element::install(child_scope)?;
        super::html_link_element::install(child_scope)?;
        super::html_meta_element::install(child_scope)?;
        super::html_script_element::install(child_scope)?;
        super::html_select_element::install(child_scope)?;
        super::html_span_element::install(child_scope)?;
        super::html_style_element::install(child_scope)?;
        super::html_table_element::install(child_scope)?;
        super::html_text_area_element::install(child_scope)?;
        super::html_base_element::install(child_scope)?;
        super::html_br_element::install(child_scope)?;
        super::html_d_list_element::install(child_scope)?;
        super::html_data_element::install(child_scope)?;
        super::html_data_list_element::install(child_scope)?;
        super::html_details_element::install(child_scope)?;
        super::html_dialog_element::install(child_scope)?;
        super::html_directory_element::install(child_scope)?;
        super::html_embed_element::install(child_scope)?;
        super::html_fenced_frame_element::install(child_scope)?;
        super::html_field_set_element::install(child_scope)?;
        super::html_font_element::install(child_scope)?;
        super::html_frame_element::install(child_scope)?;
        super::html_frame_set_element::install(child_scope)?;
        super::html_geolocation_element::install(child_scope)?;
        super::html_heading_element::install(child_scope)?;
        super::html_hr_element::install(child_scope)?;
        super::html_label_element::install(child_scope)?;
        super::html_legend_element::install(child_scope)?;
        super::html_li_element::install(child_scope)?;
        super::html_map_element::install(child_scope)?;
        super::html_marquee_element::install(child_scope)?;
        super::html_menu_element::install(child_scope)?;
        super::html_meter_element::install(child_scope)?;
        super::html_mod_element::install(child_scope)?;
        super::html_o_list_element::install(child_scope)?;
        super::html_object_element::install(child_scope)?;
        super::html_opt_group_element::install(child_scope)?;
        super::html_option_element::install(child_scope)?;
        super::html_output_element::install(child_scope)?;
        super::html_paragraph_element::install(child_scope)?;
        super::html_param_element::install(child_scope)?;
        super::html_picture_element::install(child_scope)?;
        super::html_pre_element::install(child_scope)?;
        super::html_progress_element::install(child_scope)?;
        super::html_quote_element::install(child_scope)?;
        super::html_selected_content_element::install(child_scope)?;
        super::html_slot_element::install(child_scope)?;
        super::html_source_element::install(child_scope)?;
        super::html_table_caption_element::install(child_scope)?;
        super::html_table_cell_element::install(child_scope)?;
        super::html_table_col_element::install(child_scope)?;
        super::html_table_row_element::install(child_scope)?;
        super::html_table_section_element::install(child_scope)?;
        super::html_template_element::install(child_scope)?;
        super::html_time_element::install(child_scope)?;
        super::html_title_element::install(child_scope)?;
        super::html_track_element::install(child_scope)?;
        super::html_u_list_element::install(child_scope)?;
        super::html_unknown_element::install(child_scope)?;
        super::location::install(child_scope)?;
        super::history::install(child_scope)?;
        super::custom_element_registry::install(child_scope)?;
        super::cookie_store::install(child_scope)?;
        super::scheduler::install(child_scope)?;
        super::trusted_type_policy_factory::install(child_scope)?;
        super::cache::install(child_scope)?;
        super::cache_storage::install(child_scope)?;
        super::idb_factory::install(child_scope)?;
        super::storage::install(child_scope)?;
        super::url::install_standard_name(child_scope)?;
        super::url_search_params::install_global(child_scope)?;
        super::url_pattern::install(child_scope)?;
        super::blob::install(child_scope)?;
        super::file::install(child_scope)?;
        super::file_reader::install(child_scope)?;
        super::headers::install(child_scope)?;
        super::request::install(child_scope)?;
        super::response::install(child_scope)?;
        super::form_data::install(child_scope)?;
        super::xml_http_request_upload::install(child_scope)?;
        super::xml_http_request_event_target::install(child_scope)?;
        super::xml_http_request::install(child_scope)?;
        super::shared_worker::install(child_scope)?;
        super::speech_synthesis_event::install(child_scope)?;
        super::speech_synthesis_error_event::install(child_scope)?;
        super::speech_synthesis_utterance::install(child_scope)?;
        super::speech_synthesis_voice::install(child_scope)?;
        super::speech_synthesis::install(child_scope)?;
        super::offscreen_canvas::install(child_scope)?;
        super::offscreen_canvas_rendering_context_2d::install(child_scope)?;
        super::webgl_rendering_context::install(child_scope)?;
        super::webgl2_rendering_context::install(child_scope)?;
        super::window::install(child_scope)?;
        super::event_target::reset(child_scope, child_window);
        super::navigator::install(child_scope)?;
        super::screen::install(child_scope)?;
        super::subtle_crypto::install(child_scope)?;
        super::crypto_key::install(child_scope)?;
        super::crypto::install(child_scope)?;
        super::performance::install(child_scope)?;

        crate::locale_runtime::install(child_scope)?;
        crate::determinism::install(child_scope)?;
        crate::iframe_hook::run_for_current_iframe(child_scope, &navigation.url)?;
        if crate::trace::is_enabled(child_scope) {
            let label = format!("iframe[{}]", iframe.get_identity_hash().get());
            crate::trace::label_native_value(child_scope, child_window.into(), &label);
            crate::trace::label_native_value(
                child_scope,
                document.into(),
                &format!("{label}.document"),
            );
            crate::trace::label_native_value(
                child_scope,
                location.into(),
                &format!("{label}.location"),
            );
        }
        Ok::<_, String>((
            v8::Global::new(child_scope, context),
            v8::Global::new(child_scope, child_window),
            v8::Global::new(child_scope, document),
            v8::Global::new(child_scope, location),
            ancestor_location,
            ancestor_descriptors,
        ))
    }?;

    {
        let stored = scope
            .get_slot_mut::<HtmlIFrameElementStore>()
            .and_then(|store| store.records.get_mut(&iframe.get_identity_hash().get()))
            .ok_or_else(|| "iframe state disappeared during navigation".to_owned())?;
        stored.context = Some(prepared.0);
        stored.content_window = Some(prepared.1);
        stored.content_document = Some(prepared.2);
        stored.location = Some(prepared.3);
        stored.cross_origin_ancestor_location = prepared.4;
        stored.cross_origin_ancestor_descriptors = prepared.5;
        stored.loaded_srcdoc = navigation.loaded_srcdoc;
        stored.loaded_src = navigation.loaded_src;
        stored.same_origin = navigation.same_origin;
        stored.installing_context = false;
    }

    let current = record(scope, iframe)
        .ok_or_else(|| "iframe state disappeared after navigation".to_owned())?;
    let context = v8::Local::new(
        scope,
        current
            .context
            .as_ref()
            .ok_or_else(|| "navigated iframe context is unavailable".to_owned())?,
    );
    {
        let child_scope = &mut v8::ContextScope::new(scope, context);
        v8::tc_scope!(let try_catch, child_scope);
        let _user_execution = crate::trace::enter_user_execution(try_catch);
        let document = v8::Local::new(
            try_catch,
            current
                .content_document
                .as_ref()
                .ok_or_else(|| "navigated iframe document is unavailable".to_owned())?,
        );
        super::html_script_element::execute_parser_inserted_tree(try_catch, document);
        try_catch.perform_microtask_checkpoint();
    }
    if let Ok(event) = super::event::create(scope, "load") {
        super::event_target::dispatch(scope, iframe, event);
    }
    Ok(())
}

fn parent_document_url(scope: &v8::PinScope<'_, '_>, record: &IFrameRecord) -> String {
    let parent_window = v8::Local::new(scope, &record.parent_window);
    let parent_record = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| {
            store.records.values().find(|candidate| {
                candidate.content_window.as_ref().is_some_and(|window| {
                    v8::Local::new(scope, window).strict_equals(parent_window.into())
                })
            })
        })
        .cloned();
    if let Some(location) = parent_record.and_then(|record| record.location) {
        let location = v8::Local::new(scope, &location);
        if let Some(href) =
            property(scope, location, "href").and_then(|value| value.to_string(scope))
        {
            return href.to_rust_string_lossy(scope);
        }
    }
    crate::page_init::base_url(scope)
}

fn parent_document_base_url(scope: &v8::PinScope<'_, '_>, record: &IFrameRecord) -> String {
    let element = v8::Local::new(scope, &record.element);
    super::node::owner_document(scope, element)
        .map(|document| super::document::base_url(scope, document))
        .unwrap_or_else(|| parent_document_url(scope, record))
}

fn urls_share_origin(parent: &str, child: &str) -> bool {
    let (Ok(parent), Ok(child)) = (url::Url::parse(parent), url::Url::parse(child)) else {
        return false;
    };
    parent.origin() == child.origin()
}

fn iframe_document_referrer(parent_url: &str, child_url: &str, same_origin: bool) -> String {
    if same_origin && !matches!(child_url, "about:srcdoc" | "about:blank") {
        return parent_url.to_owned();
    }
    let Ok(parent) = url::Url::parse(parent_url) else {
        return String::new();
    };
    let origin = parent.origin().ascii_serialization();
    if origin == "null" {
        String::new()
    } else {
        format!("{}/", origin.trim_end_matches('/'))
    }
}

fn script_sources(scope: &v8::PinScope<'_, '_>, root: v8::Local<'_, v8::Object>) -> Vec<String> {
    let mut output = Vec::new();
    collect_script_sources(scope, root, &mut output);
    output
}

fn collect_script_sources(
    scope: &v8::PinScope<'_, '_>,
    node: v8::Local<'_, v8::Object>,
    output: &mut Vec<String>,
) {
    if super::element::record(scope, node)
        .is_some_and(|record| record.tag_name.eq_ignore_ascii_case("SCRIPT"))
    {
        output.push(super::node::text_content(scope, node));
    }
    for child in super::node::children(scope, node) {
        collect_script_sources(scope, child, output);
    }
}

fn is_cross_origin_parent_access(scope: &v8::PinScope<'_, '_>, record: &IFrameRecord) -> bool {
    if record.same_origin {
        return false;
    }
    !current_iframe_record(scope).is_some_and(|current| {
        v8::Local::new(scope, &current.element)
            .strict_equals(v8::Local::new(scope, &record.element).into())
    })
}

fn cross_origin_record_for_window(
    scope: &v8::PinScope<'_, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<IFrameRecord> {
    let child_target = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .values()
        .find(|record| {
            !record.installing_context
                && is_cross_origin_parent_access(scope, record)
                && record.content_window.as_ref().is_some_and(|candidate| {
                    v8::Local::new(scope, candidate).strict_equals(window.into())
                })
        })
        .cloned();
    if child_target.is_some() {
        return child_target;
    }
    let current = current_iframe_record(scope)?;
    if current.installing_context || current.same_origin {
        return None;
    }
    let parent = v8::Local::new(scope, &current.parent_window);
    let top = v8::Local::new(scope, &current.top_window);
    (window.strict_equals(parent.into()) || window.strict_equals(top.into())).then_some(current)
}

pub(crate) fn is_cross_origin_window_proxy(
    scope: &v8::PinScope<'_, '_>,
    window: v8::Local<'_, v8::Object>,
) -> bool {
    cross_origin_record_for_window(scope, window).is_some()
}

pub(crate) fn cross_origin_window_string_keys<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    cross_origin_record_for_window(scope, window)?;
    let child_count = cross_origin_child_records(scope, window).len();
    let mut keys = Vec::<v8::Local<v8::Value>>::with_capacity(child_count + 14);
    for index in 0..child_count {
        keys.push(v8::String::new(scope, &index.to_string())?.into());
    }
    keys.push(v8::String::new(scope, "window")?.into());
    keys.push(v8::String::new(scope, "self")?.into());
    keys.push(v8::String::new(scope, "location")?.into());
    keys.push(v8::String::new(scope, "closed")?.into());
    keys.push(v8::String::new(scope, "frames")?.into());
    keys.push(v8::String::new(scope, "length")?.into());
    keys.push(v8::String::new(scope, "top")?.into());
    keys.push(v8::String::new(scope, "opener")?.into());
    keys.push(v8::String::new(scope, "parent")?.into());
    keys.push(v8::String::new(scope, "blur")?.into());
    keys.push(v8::String::new(scope, "close")?.into());
    keys.push(v8::String::new(scope, "focus")?.into());
    keys.push(v8::String::new(scope, "postMessage")?.into());
    keys.push(v8::String::new(scope, "then")?.into());
    Some(v8::Array::new_with_elements(scope, &keys))
}

pub(crate) fn cross_origin_window_index_keys<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    cross_origin_record_for_window(scope, window)?;
    let child_count = cross_origin_child_records(scope, window).len();
    let mut keys = Vec::<v8::Local<v8::Value>>::with_capacity(child_count);
    for index in 0..child_count {
        keys.push(v8::String::new(scope, &index.to_string())?.into());
    }
    Some(v8::Array::new_with_elements(scope, &keys))
}

pub(crate) fn cross_origin_window_index_values<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    cross_origin_record_for_window(scope, window)?;
    let children = cross_origin_child_records(scope, window);
    let mut values = Vec::<v8::Local<v8::Value>>::with_capacity(children.len());
    for child in children {
        values.push(v8::Local::new(scope, &child.content_window?).into());
    }
    Some(v8::Array::new_with_elements(scope, &values))
}

pub(crate) fn cross_origin_window_index_entries<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    cross_origin_record_for_window(scope, window)?;
    let children = cross_origin_child_records(scope, window);
    let mut entries = Vec::<v8::Local<v8::Value>>::with_capacity(children.len());
    for (index, child) in children.into_iter().enumerate() {
        let key = v8::String::new(scope, &index.to_string())?;
        let value = v8::Local::new(scope, &child.content_window?);
        let entry = v8::Array::new_with_elements(scope, &[key.into(), value.into()]);
        entries.push(entry.into());
    }
    Some(v8::Array::new_with_elements(scope, &entries))
}

pub(crate) fn cross_origin_window_symbol_keys<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    cross_origin_record_for_window(scope, window)?;
    let keys = [
        v8::Symbol::get_to_string_tag(scope).into(),
        v8::Symbol::get_has_instance(scope).into(),
        v8::Symbol::get_is_concat_spreadable(scope).into(),
    ];
    Some(v8::Array::new_with_elements(scope, &keys))
}

pub(crate) fn cross_origin_window_all_keys<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Array>> {
    cross_origin_record_for_window(scope, window)?;
    let child_count = cross_origin_child_records(scope, window).len();
    let mut keys = Vec::<v8::Local<v8::Value>>::with_capacity(child_count + 17);
    for index in 0..child_count {
        keys.push(v8::String::new(scope, &index.to_string())?.into());
    }
    keys.push(v8::String::new(scope, "window")?.into());
    keys.push(v8::String::new(scope, "self")?.into());
    keys.push(v8::String::new(scope, "location")?.into());
    keys.push(v8::String::new(scope, "closed")?.into());
    keys.push(v8::String::new(scope, "frames")?.into());
    keys.push(v8::String::new(scope, "length")?.into());
    keys.push(v8::String::new(scope, "top")?.into());
    keys.push(v8::String::new(scope, "opener")?.into());
    keys.push(v8::String::new(scope, "parent")?.into());
    keys.push(v8::String::new(scope, "blur")?.into());
    keys.push(v8::String::new(scope, "close")?.into());
    keys.push(v8::String::new(scope, "focus")?.into());
    keys.push(v8::String::new(scope, "postMessage")?.into());
    keys.push(v8::String::new(scope, "then")?.into());
    keys.push(v8::Symbol::get_to_string_tag(scope).into());
    keys.push(v8::Symbol::get_has_instance(scope).into());
    keys.push(v8::Symbol::get_is_concat_spreadable(scope).into());
    Some(v8::Array::new_with_elements(scope, &keys))
}

pub(crate) fn throw_cross_origin_window_security_error(scope: &mut v8::PinScope<'_, '_>) {
    let window = scope.get_current_context().global(scope);
    let origin = origin_for_window(scope, window);
    let message =
        format!("Blocked a frame with origin \"{origin}\" from accessing a cross-origin frame.");
    match super::dom_exception::create(scope, message.clone(), "SecurityError".to_owned()) {
        Ok(exception) => {
            scope.throw_exception(exception.into());
        }
        Err(_) => {
            let message = v8::String::new(scope, &message).expect("short SecurityError message");
            scope.throw_exception(v8::Exception::error(scope, message));
        }
    }
}

fn is_cross_origin_window_property(name: &str) -> bool {
    matches!(
        name,
        "window"
            | "self"
            | "location"
            | "closed"
            | "frames"
            | "length"
            | "top"
            | "opener"
            | "parent"
            | "blur"
            | "close"
            | "focus"
            | "postMessage"
            | "then"
    )
}

fn cross_origin_property_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: &IFrameRecord,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    match name {
        "window" | "self" | "frames" => record
            .content_window
            .as_ref()
            .map(|window| v8::Local::new(scope, window).into()),
        "parent" => Some(v8::Local::new(scope, &record.parent_window).into()),
        "top" => Some(v8::Local::new(scope, &record.top_window).into()),
        "location" => record
            .cross_origin_location
            .as_ref()
            .map(|location| v8::Local::new(scope, location).into()),
        "closed" => Some(v8::Boolean::new(scope, false).into()),
        "length" => {
            let window = v8::Local::new(scope, record.content_window.as_ref()?);
            Some(
                v8::Integer::new_from_unsigned(
                    scope,
                    direct_child_records(scope, window).len() as u32,
                )
                .into(),
            )
        }
        "opener" => Some(v8::null(scope).into()),
        "then" => Some(v8::undefined(scope).into()),
        "blur" | "close" | "focus" | "postMessage" => {
            let key = v8::String::new(scope, name)?;
            let descriptors = v8::Local::new(scope, record.cross_origin_descriptors.as_ref()?);
            let descriptor = descriptors
                .get(scope, key.into())
                .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())?;
            let value_key = v8::String::new(scope, "value")?;
            descriptor.get(scope, value_key.into())
        }
        _ => {
            super::cross_origin_location::throw_security_error(scope, name, "Window");
            None
        }
    }
}

pub(crate) fn cross_origin_property_value_for_iframe<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let record = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .get(&iframe_id)?
        .clone();
    cross_origin_property_value(scope, &record, name)
}

pub(crate) fn cross_origin_ancestor_iframe_id_for_target(
    scope: &v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    let current = current_iframe_record(scope)?;
    if current.same_origin || current.installing_context {
        return None;
    }
    let parent = v8::Local::new(scope, &current.parent_window);
    let top = v8::Local::new(scope, &current.top_window);
    (target.strict_equals(parent.into()) || target.strict_equals(top.into())).then(|| {
        v8::Local::new(scope, &current.element)
            .get_identity_hash()
            .get()
    })
}

pub(crate) fn cross_origin_ancestor_property_value_for_iframe<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    iframe_id: i32,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let record = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .get(&iframe_id)?
        .clone();
    let target = v8::Local::new(scope, &record.parent_window);
    match name {
        "window" | "self" | "frames" => Some(target.into()),
        "parent" => Some(parent_for_window(scope, target).unwrap_or(target).into()),
        "top" => Some(top_for_window(scope, target).unwrap_or(target).into()),
        "location" => record
            .cross_origin_ancestor_location
            .as_ref()
            .map(|location| v8::Local::new(scope, location).into()),
        "closed" => Some(v8::Boolean::new(scope, false).into()),
        "length" => Some(
            v8::Integer::new_from_unsigned(
                scope,
                cross_origin_child_records(scope, target).len() as u32,
            )
            .into(),
        ),
        "opener" => Some(v8::null(scope).into()),
        "then" => Some(v8::undefined(scope).into()),
        "blur" | "close" | "focus" | "postMessage" => {
            let key = v8::String::new(scope, name)?;
            let descriptors =
                v8::Local::new(scope, record.cross_origin_ancestor_descriptors.as_ref()?);
            let descriptor = descriptors
                .get(scope, key.into())
                .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())?;
            let value_key = v8::String::new(scope, "value")?;
            descriptor.get(scope, value_key.into())
        }
        _ => {
            super::cross_origin_location::throw_security_error(scope, name, "Window");
            None
        }
    }
}

pub(crate) fn cross_origin_ancestor_descriptor_for_iframe<'s>(
    scope: &v8::PinScope<'s, '_>,
    iframe_id: i32,
    key: v8::Local<'_, v8::Name>,
) -> Option<v8::Local<'s, v8::Value>> {
    let record = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .get(&iframe_id)?;
    let descriptors = v8::Local::new(scope, record.cross_origin_ancestor_descriptors.as_ref()?);
    descriptors.get(scope, key.into())
}

pub(crate) fn indexed_window_for_target<'s>(
    scope: &v8::PinScope<'s, '_>,
    target: v8::Local<'_, v8::Object>,
    index: usize,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = cross_origin_child_records(scope, target)
        .into_iter()
        .nth(index)?;
    Some(v8::Local::new(scope, record.content_window.as_ref()?))
}

pub(crate) fn cross_origin_message_windows<'s>(
    scope: &v8::PinScope<'s, '_>,
    iframe_id: i32,
) -> Option<(v8::Local<'s, v8::Object>, v8::Local<'s, v8::Object>)> {
    let record = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .get(&iframe_id)?;
    Some((
        v8::Local::new(scope, record.content_window.as_ref()?),
        v8::Local::new(scope, &record.parent_window),
    ))
}

fn parent_for_window<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .values()
        .find(|record| {
            record.content_window.as_ref().is_some_and(|candidate| {
                v8::Local::new(scope, candidate).strict_equals(window.into())
            })
        })
        .map(|record| v8::Local::new(scope, &record.parent_window))
}

fn top_for_window<'s>(
    scope: &v8::PinScope<'s, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .values()
        .find(|record| {
            record.content_window.as_ref().is_some_and(|candidate| {
                v8::Local::new(scope, candidate).strict_equals(window.into())
            })
        })
        .map(|record| v8::Local::new(scope, &record.top_window))
}

fn child_window_named_getter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "get", key, None);
    let Some(handler_record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if handler_record.installing_context {
        return v8::Intercepted::kNo;
    }
    if key.is_symbol() && is_cross_origin_parent_access(scope, &handler_record) {
        if is_allowed_cross_origin_symbol(scope, key) {
            result.set(v8::undefined(scope).into());
        } else {
            super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        }
        return v8::Intercepted::kYes;
    }
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if is_cross_origin_parent_access(scope, &handler_record) {
        match cross_origin_property_value(scope, &handler_record, &name) {
            Some(value) => result.set(value),
            None => result.set(v8::undefined(scope).into()),
        }
        return v8::Intercepted::kYes;
    }
    if is_child_own_property(scope, &handler_record, key) {
        return v8::Intercepted::kNo;
    }
    if is_child_local_property(scope, &name) {
        return v8::Intercepted::kNo;
    }
    let Some(content_window) = handler_record.content_window.as_ref() else {
        // V8 may invoke the interceptor while it is still creating the
        // context global.  The WindowProxy is not published until Genesis
        // finishes, so there is deliberately nothing to intercept yet.
        return v8::Intercepted::kNo;
    };
    let current_window = v8::Local::new(scope, content_window);
    if let Some(window) = named_child_window(scope, current_window, &name) {
        result.set(window.into());
        return v8::Intercepted::kYes;
    }
    v8::Intercepted::kNo
}

fn child_window_named_setter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "set", key, Some(value));
    let Some(handler_record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if handler_record.installing_context {
        return v8::Intercepted::kNo;
    }
    if !is_cross_origin_parent_access(scope, &handler_record) {
        return v8::Intercepted::kNo;
    }
    let Some(name) = property_name(scope, key) else {
        super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        result.set_bool(false);
        return v8::Intercepted::kYes;
    };
    if name == "location" {
        let value = crate::webidl::value_to_string(scope, value);
        if let Err(message) = navigate_cross_origin_location(
            scope,
            arguments.data().int32_value(scope).unwrap_or(0),
            value,
        ) {
            crate::webidl::throw_type_error(scope, &message);
        }
        result.set_bool(true);
        return v8::Intercepted::kYes;
    }
    super::cross_origin_location::throw_security_error(scope, &name, "Window");
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn child_window_named_query(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "has", key, None);
    let Some(handler_record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if handler_record.installing_context {
        return v8::Intercepted::kNo;
    }
    if key.is_symbol() && is_cross_origin_parent_access(scope, &handler_record) {
        if is_allowed_cross_origin_symbol(scope, key) {
            result.set_int32(1);
        } else {
            super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        }
        return v8::Intercepted::kYes;
    }
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if is_cross_origin_parent_access(scope, &handler_record) {
        if is_cross_origin_window_property(&name) {
            result.set_int32(0);
        } else {
            super::cross_origin_location::throw_security_error(scope, &name, "Window");
        }
        return v8::Intercepted::kYes;
    }
    if is_child_own_property(scope, &handler_record, key) {
        return v8::Intercepted::kNo;
    }
    if is_child_local_property(scope, &name) {
        return v8::Intercepted::kNo;
    }
    let Some(content_window) = handler_record.content_window.as_ref() else {
        return v8::Intercepted::kNo;
    };
    let current_window = v8::Local::new(scope, content_window);
    if named_child_window(scope, current_window, &name).is_some() {
        result.set_int32(0);
        return v8::Intercepted::kYes;
    }
    v8::Intercepted::kNo
}

fn child_window_named_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "delete", key, None);
    let Some(record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if record.installing_context || !is_cross_origin_parent_access(scope, &record) {
        return v8::Intercepted::kNo;
    }
    throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn child_window_named_definer(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    _: &v8::PropertyDescriptor,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(scope, &arguments, "defineProperty", key, None);
    let Some(record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if record.installing_context || !is_cross_origin_parent_access(scope, &record) {
        return v8::Intercepted::kNo;
    }
    throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn child_window_named_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    if let Some(record) = iframe_record_for_handler(scope, &arguments) {
        if record.installing_context {
            result.set(v8::Array::new(scope, 0));
            return;
        }
        if is_cross_origin_parent_access(scope, &record) {
            let names = [
                v8::String::new(scope, "window")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "self")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "location")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "closed")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "frames")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "length")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "top")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "opener")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "parent")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "blur")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "close")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "focus")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "postMessage")
                    .expect("short Window property")
                    .into(),
                v8::String::new(scope, "then")
                    .expect("short Window property")
                    .into(),
            ];
            result.set(v8::Array::new_with_elements(scope, &names));
            return;
        }
    }
    let mut names = Vec::new();
    if let Some(window) =
        iframe_record_for_handler(scope, &arguments).and_then(|record| record.content_window)
    {
        let window = v8::Local::new(scope, &window);
        for child in direct_child_records(scope, window) {
            for name in [
                child.name,
                super::element::attribute_value(scope, v8::Local::new(scope, &child.element), "id")
                    .unwrap_or_default(),
            ] {
                if !name.is_empty() && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }
    let names = names
        .into_iter()
        .filter_map(|name| v8::String::new(scope, &name).map(|name| name.into()))
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &names));
}

fn child_window_named_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_named_native_intercept(
        scope,
        &arguments,
        "getOwnPropertyDescriptor",
        key,
        None,
    );
    let Some(handler_record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if handler_record.installing_context {
        return v8::Intercepted::kNo;
    }
    if key.is_symbol() && is_cross_origin_parent_access(scope, &handler_record) {
        if is_allowed_cross_origin_symbol(scope, key) {
            let descriptor = super::cross_origin_window_descriptors::data_descriptor(
                scope,
                v8::undefined(scope).into(),
                false,
                false,
                true,
            );
            result.set(descriptor.into());
        } else {
            super::html_i_frame_element::throw_cross_origin_window_security_error(scope);
        }
        return v8::Intercepted::kYes;
    }
    let Some(name) = property_name(scope, key) else {
        return v8::Intercepted::kNo;
    };
    if is_cross_origin_parent_access(scope, &handler_record) {
        if !is_cross_origin_window_property(&name) {
            super::cross_origin_location::throw_security_error(scope, &name, "Window");
            return v8::Intercepted::kYes;
        }
        let Some(descriptors) = handler_record.cross_origin_descriptors.as_ref() else {
            return v8::Intercepted::kYes;
        };
        let descriptors = v8::Local::new(scope, descriptors);
        let Some(descriptor) = descriptors.get(scope, key.into()) else {
            return v8::Intercepted::kYes;
        };
        result.set(descriptor);
        return v8::Intercepted::kYes;
    }
    if is_child_own_property(scope, &handler_record, key) {
        return v8::Intercepted::kNo;
    }
    if is_child_local_property(scope, &name) {
        return v8::Intercepted::kNo;
    }
    let Some(content_window) = handler_record.content_window.as_ref() else {
        return v8::Intercepted::kNo;
    };
    let current_window = v8::Local::new(scope, content_window);
    if let Some(window) = named_child_window(scope, current_window, &name) {
        let descriptor = v8::Object::new(scope);
        let (Some(value_key), Some(writable_key), Some(enumerable_key), Some(configurable_key)) = (
            v8::String::new(scope, "value"),
            v8::String::new(scope, "writable"),
            v8::String::new(scope, "enumerable"),
            v8::String::new(scope, "configurable"),
        ) else {
            return v8::Intercepted::kNo;
        };
        let true_value = v8::Boolean::new(scope, true);
        let _ = descriptor.set(scope, value_key.into(), window.into());
        let _ = descriptor.set(scope, writable_key.into(), true_value.into());
        let _ = descriptor.set(scope, enumerable_key.into(), true_value.into());
        let _ = descriptor.set(scope, configurable_key.into(), true_value.into());
        result.set(descriptor.into());
        return v8::Intercepted::kYes;
    }
    v8::Intercepted::kNo
}

fn child_window_indexed_getter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "get", index, None);
    let Some(handler_record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    let Some(content_window) = handler_record.content_window.as_ref() else {
        return v8::Intercepted::kNo;
    };
    let current_window = v8::Local::new(scope, content_window);
    let Some(window) = indexed_child_window(scope, current_window, index as usize) else {
        return v8::Intercepted::kNo;
    };
    result.set(window.into());
    v8::Intercepted::kYes
}

fn child_window_indexed_setter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "set", index, Some(value));
    let Some(record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if record.installing_context || !is_cross_origin_parent_access(scope, &record) {
        return v8::Intercepted::kNo;
    }
    throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn child_window_indexed_query(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "has", index, None);
    let Some(handler_record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    let Some(content_window) = handler_record.content_window.as_ref() else {
        return v8::Intercepted::kNo;
    };
    let current_window = v8::Local::new(scope, content_window);
    if indexed_child_window(scope, current_window, index as usize).is_some() {
        result.set_int32(0);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn child_window_indexed_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "delete", index, None);
    let Some(record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if record.installing_context || !is_cross_origin_parent_access(scope, &record) {
        return v8::Intercepted::kNo;
    }
    throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn child_window_indexed_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let count = iframe_record_for_handler(scope, &arguments)
        .and_then(|record| record.content_window)
        .map(|window| direct_child_records(scope, v8::Local::new(scope, &window)).len())
        .unwrap_or_default();
    let indices = (0..count)
        .map(|index| v8::Integer::new_from_unsigned(scope, index as u32).into())
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &indices));
}

fn child_window_indexed_definer(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    _: &v8::PropertyDescriptor,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "defineProperty", index, None);
    let Some(record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if record.installing_context || !is_cross_origin_parent_access(scope, &record) {
        return v8::Intercepted::kNo;
    }
    throw_cross_origin_window_security_error(scope);
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn child_window_indexed_descriptor(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(
        scope,
        &arguments,
        "getOwnPropertyDescriptor",
        index,
        None,
    );
    let Some(handler_record) = iframe_record_for_handler(scope, &arguments) else {
        return v8::Intercepted::kNo;
    };
    if handler_record.installing_context {
        return v8::Intercepted::kNo;
    }
    let Some(content_window) = handler_record.content_window.as_ref() else {
        return v8::Intercepted::kNo;
    };
    let current_window = v8::Local::new(scope, content_window);
    let Some(window) = indexed_child_window(scope, current_window, index as usize) else {
        return v8::Intercepted::kNo;
    };
    let descriptor = super::cross_origin_window_descriptors::data_descriptor(
        scope,
        window.into(),
        false,
        true,
        true,
    );
    result.set(descriptor.into());
    v8::Intercepted::kYes
}

pub(crate) fn direct_child_count(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
) -> usize {
    direct_child_records(scope, parent).len()
}

pub(crate) fn is_child_window(
    scope: &v8::PinScope<'_, '_>,
    window: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<HtmlIFrameElementStore>()
        .is_some_and(|store| {
            store.records.values().any(|record| {
                record.content_window.as_ref().is_some_and(|candidate| {
                    v8::Local::new(scope, candidate).strict_equals(window.into())
                })
            })
        })
}

pub(crate) fn suspend_child_window_interceptor(
    scope: &mut v8::PinScope<'_, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<(i32, bool)> {
    let iframe_id = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .iter()
        .find_map(|(iframe_id, record)| {
            record.content_window.as_ref().and_then(|candidate| {
                v8::Local::new(scope, candidate)
                    .strict_equals(window.into())
                    .then_some(*iframe_id)
            })
        })?;
    let record = scope
        .get_slot_mut::<HtmlIFrameElementStore>()?
        .records
        .get_mut(&iframe_id)?;
    let was_suspended = record.installing_context;
    record.installing_context = true;
    Some((iframe_id, was_suspended))
}

pub(crate) fn restore_child_window_interceptor(
    scope: &mut v8::PinScope<'_, '_>,
    suspension: Option<(i32, bool)>,
) {
    let Some((iframe_id, was_suspended)) = suspension else {
        return;
    };
    if let Some(record) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&iframe_id))
    {
        record.installing_context = was_suspended;
    }
}

pub(crate) fn context_for_window(
    scope: &v8::PinScope<'_, '_>,
    window: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Context>> {
    if let Some(context) = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| {
            store.records.values().find_map(|record| {
                record.content_window.as_ref().and_then(|candidate| {
                    v8::Local::new(scope, candidate)
                        .strict_equals(window.into())
                        .then(|| record.context.clone())?
                })
            })
        })
    {
        return Some(context);
    }
    let context = window.get_creation_context(scope)?;
    context
        .global(scope)
        .strict_equals(window.into())
        .then(|| v8::Global::new(scope, context))
}

pub(crate) fn origin_for_window(
    scope: &v8::PinScope<'_, '_>,
    window: v8::Local<'_, v8::Object>,
) -> String {
    origin_for_window_inner(scope, window, 0)
}

fn origin_for_window_inner(
    scope: &v8::PinScope<'_, '_>,
    window: v8::Local<'_, v8::Object>,
    depth: usize,
) -> String {
    if depth > 64 {
        return "null".to_owned();
    }
    let record = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| {
            store.records.values().find(|record| {
                record.content_window.as_ref().is_some_and(|candidate| {
                    v8::Local::new(scope, candidate).strict_equals(window.into())
                })
            })
        })
        .cloned();
    let Some(record) = record else {
        return crate::page_init::origin(scope);
    };
    let href = record.loaded_src.as_deref().unwrap_or("about:srcdoc");
    if let Ok(url) = url::Url::parse(&href)
        && matches!(url.scheme(), "http" | "https")
    {
        return url.origin().ascii_serialization();
    }
    origin_for_window_inner(
        scope,
        v8::Local::new(scope, &record.parent_window),
        depth + 1,
    )
}

fn direct_child_records(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
) -> Vec<IFrameRecord> {
    let mut records = scope
        .get_slot::<HtmlIFrameElementStore>()
        .map(|store| {
            store
                .records
                .values()
                .filter(|record| {
                    v8::Local::new(scope, &record.parent_window).strict_equals(parent.into())
                        && belongs_to_active_parent_document(scope, record)
                        && super::node::is_connected(scope, v8::Local::new(scope, &record.element))
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    records.sort_by_key(|record| record.sequence);
    records
}

fn cross_origin_child_records(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
) -> Vec<IFrameRecord> {
    let mut records = scope
        .get_slot::<HtmlIFrameElementStore>()
        .map(|store| {
            store
                .records
                .values()
                .filter(|record| {
                    record.content_window.is_some()
                        && v8::Local::new(scope, &record.parent_window).strict_equals(parent.into())
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    records.sort_by_key(|record| record.sequence);
    records
}

fn belongs_to_active_parent_document(scope: &v8::PinScope<'_, '_>, child: &IFrameRecord) -> bool {
    let parent_window = v8::Local::new(scope, &child.parent_window);
    let owner_document = super::node::owner_document(scope, v8::Local::new(scope, &child.element));
    let parent_record = scope
        .get_slot::<HtmlIFrameElementStore>()
        .and_then(|store| {
            store.records.values().find(|candidate| {
                candidate.content_window.as_ref().is_some_and(|window| {
                    v8::Local::new(scope, window).strict_equals(parent_window.into())
                })
            })
        });
    let Some(parent_record) = parent_record else {
        return true;
    };
    match (owner_document, parent_record.content_document.as_ref()) {
        (Some(owner_document), Some(active_document)) => {
            v8::Local::new(scope, active_document).strict_equals(owner_document.into())
        }
        _ => false,
    }
}

fn indexed_child_window<'s>(
    scope: &v8::PinScope<'s, '_>,
    parent: v8::Local<'_, v8::Object>,
    index: usize,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = direct_child_records(scope, parent).into_iter().nth(index)?;
    Some(v8::Local::new(scope, record.content_window.as_ref()?))
}

fn named_child_window<'s>(
    scope: &v8::PinScope<'s, '_>,
    parent: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let record = direct_child_records(scope, parent)
        .into_iter()
        .find(|record| {
            if record.name == name {
                return true;
            }
            let element = v8::Local::new(scope, &record.element);
            super::element::attribute_value(scope, element, "id").as_deref() == Some(name)
                || super::element::attribute_value(scope, element, "name").as_deref() == Some(name)
        })?;
    Some(v8::Local::new(scope, record.content_window.as_ref()?))
}

fn containing_parent_window<'s>(
    scope: &v8::PinScope<'s, '_>,
    child: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    let child_id = child.get_identity_hash().get();
    let parent = scope
        .get_slot::<HtmlIFrameElementStore>()?
        .records
        .values()
        .find(|record| {
            record.content_window.as_ref().is_some_and(|window| {
                v8::Local::new(scope, window).get_identity_hash().get() == child_id
            })
        })?
        .parent_window
        .clone();
    Some(v8::Local::new(scope, &parent))
}

fn property_name(scope: &v8::PinScope<'_, '_>, key: v8::Local<'_, v8::Name>) -> Option<String> {
    if key.is_symbol() {
        return None;
    }
    key.to_string(scope)
        .map(|name| name.to_rust_string_lossy(scope))
}

fn is_allowed_cross_origin_symbol(
    scope: &v8::PinScope<'_, '_>,
    key: v8::Local<'_, v8::Name>,
) -> bool {
    key.strict_equals(v8::Symbol::get_to_string_tag(scope).into())
        || key.strict_equals(v8::Symbol::get_has_instance(scope).into())
        || key.strict_equals(v8::Symbol::get_is_concat_spreadable(scope).into())
}

fn is_child_own_property(
    scope: &mut v8::PinScope<'_, '_>,
    record: &IFrameRecord,
    key: v8::Local<'_, v8::Name>,
) -> bool {
    let Some(window) = record.content_window.as_ref().cloned() else {
        return false;
    };
    let iframe_id = v8::Local::new(scope, &record.element)
        .get_identity_hash()
        .get();
    if let Some(stored) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&iframe_id))
    {
        stored.installing_context = true;
    }
    let window = v8::Local::new(scope, &window);
    let own = window.has_own_property(scope, key).unwrap_or(false);
    if let Some(stored) = scope
        .get_slot_mut::<HtmlIFrameElementStore>()
        .and_then(|store| store.records.get_mut(&iframe_id))
    {
        stored.installing_context = false;
    }
    own
}

fn is_child_local_property(_scope: &v8::PinScope<'_, '_>, name: &str) -> bool {
    matches!(
        name,
        "window"
            | "self"
            | "frames"
            | "parent"
            | "top"
            | "frameElement"
            | "document"
            | "location"
            | "name"
            | "length"
            | "closed"
            | "Temporal"
    )
}

fn expose_child_window_on_parent(
    scope: &mut v8::PinScope<'_, '_>,
    iframe: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let snapshot = record(scope, iframe).ok_or_else(|| "iframe state disappeared".to_owned())?;
    let parent = v8::Local::new(scope, &snapshot.parent_window);
    // Child WindowProxy objects resolve indices and names through their
    // interceptors.  The root Window has no such interceptor and needs its
    // actual named properties refreshed.
    if containing_parent_window(scope, parent).is_some() {
        return Ok(());
    }
    refresh_root_frame_bindings(scope, parent);
    Ok(())
}

fn refresh_root_frame_bindings(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
) {
    let all_children = scope
        .get_slot::<HtmlIFrameElementStore>()
        .map(|store| {
            store
                .records
                .values()
                .filter(|record| {
                    v8::Local::new(scope, &record.parent_window).strict_equals(parent.into())
                })
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for index in 0..all_children.len() {
        if let Some(key) = v8::String::new(scope, &index.to_string()) {
            let _ = parent.delete(scope, key.into());
        }
    }
    for child in &all_children {
        let child_window = child
            .content_window
            .as_ref()
            .map(|window| v8::Local::new(scope, window));
        let mut previous_names = child.exposed_parent_names.clone();
        previous_names.push(child.name.clone());
        previous_names.push(
            super::element::attribute_value(scope, v8::Local::new(scope, &child.element), "id")
                .unwrap_or_default(),
        );
        for name in previous_names {
            if name.is_empty() {
                continue;
            }
            let Some(key) = v8::String::new(scope, &name) else {
                continue;
            };
            let belongs_to_child = child_window.is_some_and(|window| {
                parent
                    .get(scope, key.into())
                    .is_some_and(|value| value.strict_equals(window.into()))
            });
            if belongs_to_child {
                let _ = parent.delete(scope, key.into());
            }
        }
    }

    let mut exposed_names = Vec::new();
    for (index, child) in direct_child_records(scope, parent).into_iter().enumerate() {
        let Some(window) = child
            .content_window
            .as_ref()
            .map(|window| v8::Local::new(scope, window))
        else {
            continue;
        };
        if let Some(index_key) = v8::String::new(scope, &index.to_string()) {
            let _ = parent.define_own_property(
                scope,
                index_key.into(),
                window.into(),
                v8::PropertyAttribute::NONE,
            );
        }
        let names = [
            child.name,
            super::element::attribute_value(scope, v8::Local::new(scope, &child.element), "id")
                .unwrap_or_default(),
        ];
        for name in &names {
            if name.is_empty() {
                continue;
            }
            let Some(key) = v8::String::new(scope, name) else {
                continue;
            };
            if parent.has_own_property(scope, key.into()) != Some(true) {
                let _ = parent.define_own_property(
                    scope,
                    key.into(),
                    window.into(),
                    v8::PropertyAttribute::NONE,
                );
            }
        }
        exposed_names.push((
            v8::Local::new(scope, &child.element)
                .get_identity_hash()
                .get(),
            names
                .into_iter()
                .filter(|name| !name.is_empty())
                .collect::<Vec<_>>(),
        ));
    }
    if let Some(store) = scope.get_slot_mut::<HtmlIFrameElementStore>() {
        for (identity, names) in exposed_names {
            if let Some(record) = store.records.get_mut(&identity) {
                record.exposed_parent_names = names;
            }
        }
    }
}

pub(crate) fn notify_disconnected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = record(scope, root) {
        let parent = v8::Local::new(scope, &record.parent_window);
        if containing_parent_window(scope, parent).is_none() {
            refresh_root_frame_bindings(scope, parent);
        }
    }
    for child in super::node::children(scope, root) {
        notify_disconnected_tree(scope, child);
    }
}

fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}
pub(crate) fn get_content_document(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    null_value(s, a, r)
}
pub(crate) fn get_content_window(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    null_value(s, a, r)
}
pub(crate) fn get_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.referrer_policy)
}
pub(crate) fn set_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.referrer_policy = v)
}
pub(crate) fn get_csp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.csp)
}
pub(crate) fn set_csp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.csp = v)
}
pub(crate) fn get_allow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.allow)
}
pub(crate) fn set_allow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.allow = v)
}
pub(crate) fn get_feature_policy(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &record.feature_policy).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_loading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.loading)
}
pub(crate) fn set_loading(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    let value = if value == "lazy" { "lazy" } else { "auto" }.to_owned();
    update(s, a.this(), |x| x.loading = value)
}
pub(crate) fn get_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.align)
}
pub(crate) fn set_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.align = v)
}
pub(crate) fn get_scrolling(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.scrolling)
}
pub(crate) fn set_scrolling(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.scrolling = v)
}
pub(crate) fn get_frame_border(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.frame_border)
}
pub(crate) fn set_frame_border(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.frame_border = v)
}
pub(crate) fn get_long_desc(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.long_desc)
}
pub(crate) fn set_long_desc(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.long_desc = v)
}
pub(crate) fn get_margin_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.margin_height)
}
pub(crate) fn set_margin_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.margin_height = v)
}
pub(crate) fn get_margin_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.margin_width)
}
pub(crate) fn set_margin_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.margin_width = v)
}
pub(crate) fn get_svg_document(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    null_value(s, a, r)
}
pub(crate) fn get_credentialless(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.credentialless)
}
pub(crate) fn set_credentialless(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.credentialless = v)
}
pub(crate) fn get_allow_payment_request(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_bool(s, a, r, |x| x.allow_payment_request)
}
pub(crate) fn set_allow_payment_request(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_bool(s, a, |x, v| x.allow_payment_request = v)
}
