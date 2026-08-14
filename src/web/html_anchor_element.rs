use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct AnchorRecord {
    pub(crate) target: String,
    pub(crate) download: String,
    pub(crate) ping: String,
    pub(crate) rel_list: v8::Global<v8::Object>,
    pub(crate) hreflang: String,
    pub(crate) link_type: String,
    pub(crate) referrer_policy: String,
    pub(crate) text: String,
    pub(crate) coords: String,
    pub(crate) charset: String,
    pub(crate) name: String,
    pub(crate) rev: String,
    pub(crate) shape: String,
    pub(crate) href: String,
    pub(crate) url: Option<::url::Url>,
    pub(crate) interest_for: Option<v8::Global<v8::Object>>,
    pub(crate) href_translate: String,
    pub(crate) attribution_src: String,
}

#[derive(Default)]
pub(crate) struct HtmlAnchorElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, AnchorRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlAnchorElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLAnchorElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlAnchorElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLAnchorElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_anchor_element_target_property::define(scope, p)?;
    super::html_anchor_element_download_property::define(scope, p)?;
    super::html_anchor_element_ping_property::define(scope, p)?;
    super::html_anchor_element_rel_property::define(scope, p)?;
    super::html_anchor_element_rel_list_property::define(scope, p)?;
    super::html_anchor_element_hreflang_property::define(scope, p)?;
    super::html_anchor_element_type_property::define(scope, p)?;
    super::html_anchor_element_referrer_policy_property::define(scope, p)?;
    super::html_anchor_element_text_property::define(scope, p)?;
    super::html_anchor_element_coords_property::define(scope, p)?;
    super::html_anchor_element_charset_property::define(scope, p)?;
    super::html_anchor_element_name_property::define(scope, p)?;
    super::html_anchor_element_rev_property::define(scope, p)?;
    super::html_anchor_element_shape_property::define(scope, p)?;
    super::html_anchor_element_origin_property::define(scope, p)?;
    super::html_anchor_element_protocol_property::define(scope, p)?;
    super::html_anchor_element_username_property::define(scope, p)?;
    super::html_anchor_element_password_property::define(scope, p)?;
    super::html_anchor_element_host_property::define(scope, p)?;
    super::html_anchor_element_hostname_property::define(scope, p)?;
    super::html_anchor_element_port_property::define(scope, p)?;
    super::html_anchor_element_pathname_property::define(scope, p)?;
    super::html_anchor_element_search_property::define(scope, p)?;
    super::html_anchor_element_hash_property::define(scope, p)?;
    super::html_anchor_element_href_property::define(scope, p)?;
    super::html_anchor_element_interest_for_element_property::define(scope, p)?;
    super::html_anchor_element_to_string::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    super::html_anchor_element_href_translate_property::define(scope, p)?;
    super::html_anchor_element_attribution_src_property::define(scope, p)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlAnchorElementStore>()
        .ok_or_else(|| "HTMLAnchorElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(c)
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create HTMLAnchorElement".to_owned());
    }
    super::html_element::attach(scope, o, "A");
    let rel_list = super::dom_token_list::create_bound_with_support(
        scope,
        "",
        o,
        "rel",
        super::dom_token_list::DomTokenSupport::HyperlinkRel,
    )?;
    let rel_list = v8::Global::new(scope, rel_list);
    scope
        .get_slot_mut::<HtmlAnchorElementStore>()
        .ok_or_else(|| "HTMLAnchorElement state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            AnchorRecord {
                target: String::new(),
                download: String::new(),
                ping: String::new(),
                rel_list,
                hreflang: String::new(),
                link_type: String::new(),
                referrer_policy: String::new(),
                text: String::new(),
                coords: String::new(),
                charset: String::new(),
                name: String::new(),
                rev: String::new(),
                shape: String::new(),
                href: String::new(),
                url: None,
                interest_for: None,
                href_translate: String::new(),
                attribution_src: String::new(),
            },
        );
    Ok(o)
}
pub(crate) fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<AnchorRecord> {
    s.get_slot::<HtmlAnchorElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    f: impl FnOnce(&mut AnchorRecord),
) {
    if let Some(x) = s
        .get_slot_mut::<HtmlAnchorElementStore>()
        .and_then(|q| q.records.get_mut(&o.get_identity_hash().get()))
    {
        f(x)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) {
    if name.eq_ignore_ascii_case("interestfor")
        && let Some(record) = scope
            .get_slot_mut::<HtmlAnchorElementStore>()
            .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        record.interest_for = None;
    }
}
pub(crate) fn get_string(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(&AnchorRecord) -> String,
) {
    if let Some(x) = record(s, a.this()) {
        if let Some(v) = v8::String::new(s, &f(&x)) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_string(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    f: impl FnOnce(&mut AnchorRecord, String),
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| f(x, v))
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
pub(crate) fn get_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.target.clone())
}
pub(crate) fn set_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.target = v)
}
pub(crate) fn get_download(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.download.clone())
}
pub(crate) fn set_download(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.download = v)
}
pub(crate) fn get_ping(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.ping.clone())
}
pub(crate) fn set_ping(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.ping = v)
}
pub(crate) fn get_rel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(s, &x.rel_list);
    let value = super::dom_token_list::string_value(s, list).unwrap_or_default();
    if let Some(value) = v8::String::new(s, &value) {
        r.set(value.into())
    }
}
pub(crate) fn set_rel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if let Some(x) = record(s, a.this()) {
        let list = v8::Local::new(s, &x.rel_list);
        super::dom_token_list::set_string_value(s, list, &value);
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_rel_list(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.rel_list).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_rel_list(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let rel_list = v8::Local::new(scope, &record.rel_list);
        super::dom_token_list::set_string_value(scope, rel_list, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_hreflang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.hreflang.clone())
}
pub(crate) fn set_hreflang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.hreflang = v)
}
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.link_type.clone())
}
pub(crate) fn set_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.link_type = v)
}
pub(crate) fn get_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.referrer_policy.clone())
}
pub(crate) fn set_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.referrer_policy = v)
}
pub(crate) fn get_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
    } else if let Some(value) = v8::String::new(s, &super::node::node_text(s, a.this())) {
        let mut r = r;
        r.set(value.into());
    }
}
pub(crate) fn set_text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    for child in super::node::children(s, a.this()) {
        super::node::detach(s, child);
    }
    if !value.is_empty()
        && let Ok(text) = super::text::create(s, value.clone())
    {
        if let Some(document) = super::node::owner_document(s, a.this()) {
            super::node::set_owner_document(s, text, document);
        }
        let _ = super::node::insert_node(s, a.this(), text, 0);
    }
    update(s, a.this(), |record| record.text = value);
}
pub(crate) fn get_coords(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.coords.clone())
}
pub(crate) fn set_coords(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.coords = v)
}
pub(crate) fn get_charset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.charset.clone())
}
pub(crate) fn set_charset(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.charset = v)
}
pub(crate) fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.name.clone())
}
pub(crate) fn set_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.name = v)
}
pub(crate) fn get_rev(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.rev.clone())
}
pub(crate) fn set_rev(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.rev = v)
}
pub(crate) fn get_shape(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.shape.clone())
}
pub(crate) fn set_shape(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.shape = v)
}
pub(crate) fn get_origin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(s, a, r, |url| url.origin().ascii_serialization(), "")
}
pub(crate) fn get_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(s, a, r, |url| format!("{}:", url.scheme()), ":")
}
pub(crate) fn set_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0))
        .trim_end_matches(':')
        .to_owned();
    update_url_attribute(s, a.this(), |url| {
        let _ = url.set_scheme(&v);
    })
}
pub(crate) fn get_username(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(s, a, r, |url| url.username().to_owned(), "")
}
pub(crate) fn set_username(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        let _ = url.set_username(&v);
    })
}
pub(crate) fn get_password(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(s, a, r, |url| url.password().unwrap_or("").to_owned(), "")
}
pub(crate) fn set_password(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        let _ = url.set_password(Some(&v));
    })
}
pub(crate) fn get_host(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(
        s,
        a,
        r,
        |url| match (url.host_str(), url.port()) {
            (Some(host), Some(port)) => format!("{host}:{port}"),
            (Some(host), None) => host.to_owned(),
            _ => String::new(),
        },
        "",
    )
}
pub(crate) fn set_host(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        if let Ok(parsed) = ::url::Url::parse(&format!("{}://{v}/", url.scheme())) {
            let _ = url.set_host(parsed.host_str());
            let _ = url.set_port(parsed.port());
        }
    })
}
pub(crate) fn get_hostname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(s, a, r, |url| url.host_str().unwrap_or("").to_owned(), "")
}
pub(crate) fn set_hostname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        let _ = url.set_host(Some(&v));
    })
}
pub(crate) fn get_port(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(
        s,
        a,
        r,
        |url| url.port().map(|port| port.to_string()).unwrap_or_default(),
        "",
    )
}
pub(crate) fn set_port(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        let port = if v.is_empty() { None } else { v.parse().ok() };
        let _ = url.set_port(port);
    })
}
pub(crate) fn get_pathname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(s, a, r, |url| url.path().to_owned(), "")
}
pub(crate) fn set_pathname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        url.set_path(&v);
    })
}
pub(crate) fn get_search(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(
        s,
        a,
        r,
        |url| {
            url.query()
                .map(|query| format!("?{query}"))
                .unwrap_or_default()
        },
        "",
    )
}
pub(crate) fn set_search(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        url.set_query(if v.is_empty() {
            None
        } else {
            Some(v.trim_start_matches('?'))
        });
    })
}
pub(crate) fn get_hash(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_url_component(
        s,
        a,
        r,
        |url| {
            url.fragment()
                .map(|fragment| format!("#{fragment}"))
                .unwrap_or_default()
        },
        "",
    )
}
pub(crate) fn set_hash(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update_url_attribute(s, a.this(), |url| {
        url.set_fragment(if v.is_empty() {
            None
        } else {
            Some(v.trim_start_matches('#'))
        });
    })
}
pub(crate) fn get_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = super::element::resolved_url_attribute(s, a.this(), "href").unwrap_or_default();
    if let Some(value) = v8::String::new(s, &value) {
        let mut r = r;
        r.set(value.into());
    }
}
pub(crate) fn set_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = crate::webidl::value_to_string(s, a.get(0));
    super::element::set_reflected_string(s, a.this(), "href", value);
}

fn get_url_component(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    component: impl FnOnce(&::url::Url) -> String,
    fallback: &str,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = super::element::resolved_url_attribute(scope, arguments.this(), "href")
        .and_then(|value| ::url::Url::parse(&value).ok())
        .map(|url| component(&url))
        .unwrap_or_else(|| fallback.to_owned());
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn update_url_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    update: impl FnOnce(&mut ::url::Url),
) {
    if record(scope, object).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(value) = super::element::resolved_url_attribute(scope, object, "href") else {
        return;
    };
    let Ok(mut url) = ::url::Url::parse(&value) else {
        return;
    };
    update(&mut url);
    super::element::set_reflected_string(scope, object, "href", url.as_str().to_owned());
}
pub(crate) fn get_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    match record(s, a.this()) {
        Some(x) => match x.interest_for {
            Some(v) => r.set(v8::Local::new(s, &v).into()),
            None => r.set(v8::null(s).into()),
        },
        None => crate::webidl::throw_type_error(s, "Illegal invocation"),
    }
}
pub(crate) fn set_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if a.get(0).is_null_or_undefined() {
        None
    } else if let Ok(o) = v8::Local::<v8::Object>::try_from(a.get(0)) {
        Some(v8::Global::new(s, o))
    } else {
        crate::webidl::throw_type_error(s, "The value must be an Element or null");
        return;
    };
    update(s, a.this(), |x| x.interest_for = value)
}
pub(crate) fn to_string(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_href(s, a, r)
}
pub(crate) fn get_href_translate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.href_translate.clone())
}
pub(crate) fn set_href_translate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.href_translate = v)
}

pub(crate) fn get_attribution_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |record| record.attribution_src.clone())
}

pub(crate) fn set_attribution_src(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |record, value| record.attribution_src = value)
}
