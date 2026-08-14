use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct AreaRecord {
    pub(crate) alt: String,
    pub(crate) coords: String,
    pub(crate) download: String,
    pub(crate) shape: String,
    pub(crate) target: String,
    pub(crate) ping: String,
    pub(crate) rel_list: v8::Global<v8::Object>,
    pub(crate) referrer_policy: String,
    pub(crate) no_href: bool,
    pub(crate) href: String,
    pub(crate) url: Option<::url::Url>,
    pub(crate) interest_for: Option<v8::Global<v8::Object>>,
    pub(crate) attribution_src: String,
}

#[derive(Default)]
pub(crate) struct HtmlAreaElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, AreaRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlAreaElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLAreaElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlAreaElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLAreaElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_area_element_alt_property::define(scope, p)?;
    super::html_area_element_coords_property::define(scope, p)?;
    super::html_area_element_download_property::define(scope, p)?;
    super::html_area_element_shape_property::define(scope, p)?;
    super::html_area_element_target_property::define(scope, p)?;
    super::html_area_element_ping_property::define(scope, p)?;
    super::html_area_element_rel_property::define(scope, p)?;
    super::html_area_element_rel_list_property::define(scope, p)?;
    super::html_area_element_referrer_policy_property::define(scope, p)?;
    super::html_area_element_no_href_property::define(scope, p)?;
    super::html_area_element_origin_property::define(scope, p)?;
    super::html_area_element_protocol_property::define(scope, p)?;
    super::html_area_element_username_property::define(scope, p)?;
    super::html_area_element_password_property::define(scope, p)?;
    super::html_area_element_host_property::define(scope, p)?;
    super::html_area_element_hostname_property::define(scope, p)?;
    super::html_area_element_port_property::define(scope, p)?;
    super::html_area_element_pathname_property::define(scope, p)?;
    super::html_area_element_search_property::define(scope, p)?;
    super::html_area_element_hash_property::define(scope, p)?;
    super::html_area_element_href_property::define(scope, p)?;
    super::html_area_element_interest_for_element_property::define(scope, p)?;
    super::html_area_element_to_string::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    super::html_area_element_attribution_src_property::define(scope, p)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlAreaElementStore>()
        .ok_or_else(|| "HTMLAreaElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLAreaElement".to_owned());
    }
    super::html_element::attach(scope, o, "AREA");
    let rel = super::dom_token_list::create_with_support(
        scope,
        "",
        super::dom_token_list::DomTokenSupport::HyperlinkRel,
    )?;
    let rel = v8::Global::new(scope, rel);
    scope
        .get_slot_mut::<HtmlAreaElementStore>()
        .ok_or_else(|| "HTMLAreaElement state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            AreaRecord {
                alt: String::new(),
                coords: String::new(),
                download: String::new(),
                shape: String::new(),
                target: String::new(),
                ping: String::new(),
                rel_list: rel,
                referrer_policy: String::new(),
                no_href: false,
                href: String::new(),
                url: None,
                interest_for: None,
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
pub(crate) fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<AreaRecord> {
    s.get_slot::<HtmlAreaElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    f: impl FnOnce(&mut AreaRecord),
) {
    if let Some(x) = s
        .get_slot_mut::<HtmlAreaElementStore>()
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
    if record(scope, object).is_none() {
        return;
    }
    if name.eq_ignore_ascii_case("interestfor") {
        update(scope, object, |record| record.interest_for = None);
    }
}
pub(crate) fn get_string(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(&AreaRecord) -> String,
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
    f: impl FnOnce(&mut AreaRecord, String),
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| f(x, v))
}
pub(crate) fn get_alt(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.alt.clone())
}
pub(crate) fn set_alt(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.alt = v)
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
    let o = v8::Local::new(s, &x.rel_list);
    let v = super::dom_token_list::string_value(s, o).unwrap_or_default();
    if let Some(v) = v8::String::new(s, &v) {
        r.set(v.into())
    }
}
pub(crate) fn set_rel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    if let Some(x) = record(s, a.this()) {
        let o = v8::Local::new(s, &x.rel_list);
        super::dom_token_list::set_string_value(s, o, &v);
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
pub(crate) fn get_no_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, x.no_href).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_no_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = a.get(0).boolean_value(s);
    update(s, a.this(), |x| x.no_href = v)
}
pub(crate) fn get_origin(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| u.origin().ascii_serialization())
            .unwrap_or_default()
    })
}
pub(crate) fn get_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| format!("{}:", u.scheme()))
            .unwrap_or_else(|| ":".to_owned())
    })
}
pub(crate) fn set_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0))
        .trim_end_matches(':')
        .to_owned();
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            let _ = u.set_scheme(&v);
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_username(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| u.username().to_owned())
            .unwrap_or_default()
    })
}
pub(crate) fn set_username(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            let _ = u.set_username(&v);
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_password(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .and_then(|u| u.password())
            .unwrap_or("")
            .to_owned()
    })
}
pub(crate) fn set_password(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            let _ = u.set_password(Some(&v));
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_host(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| match (u.host_str(), u.port()) {
                (Some(h), Some(p)) => format!("{h}:{p}"),
                (Some(h), None) => h.to_owned(),
                _ => String::new(),
            })
            .unwrap_or_default()
    })
}
pub(crate) fn set_host(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            if let Ok(q) = ::url::Url::parse(&format!("{}://{v}/", u.scheme())) {
                let _ = u.set_host(q.host_str());
                let _ = u.set_port(q.port());
                x.href = u.as_str().to_owned();
            }
        }
    })
}
pub(crate) fn get_hostname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .and_then(|u| u.host_str())
            .unwrap_or("")
            .to_owned()
    })
}
pub(crate) fn set_hostname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            let _ = u.set_host(Some(&v));
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_port(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .and_then(|u| u.port())
            .map(|p| p.to_string())
            .unwrap_or_default()
    })
}
pub(crate) fn set_port(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            let _ = u.set_port(if v.is_empty() { None } else { v.parse().ok() });
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_pathname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .map(|u| u.path().to_owned())
            .unwrap_or_default()
    })
}
pub(crate) fn set_pathname(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            u.set_path(&v);
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_search(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .and_then(|u| u.query())
            .map(|q| format!("?{q}"))
            .unwrap_or_default()
    })
}
pub(crate) fn set_search(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            u.set_query(if v.is_empty() {
                None
            } else {
                Some(v.trim_start_matches('?'))
            });
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_hash(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| {
        x.url
            .as_ref()
            .and_then(|u| u.fragment())
            .map(|q| format!("#{q}"))
            .unwrap_or_default()
    })
}
pub(crate) fn set_hash(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        if let Some(u) = x.url.as_mut() {
            u.set_fragment(if v.is_empty() {
                None
            } else {
                Some(v.trim_start_matches('#'))
            });
            x.href = u.as_str().to_owned()
        }
    })
}
pub(crate) fn get_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| x.href.clone())
}
pub(crate) fn set_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(s, a.get(0));
    update(s, a.this(), |x| {
        x.url = ::url::Url::parse(&v).ok();
        x.href = x.url.as_ref().map(|u| u.as_str().to_owned()).unwrap_or(v)
    })
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
    let v = if a.get(0).is_null_or_undefined() {
        None
    } else if let Ok(o) = v8::Local::<v8::Object>::try_from(a.get(0)) {
        Some(v8::Global::new(s, o))
    } else {
        crate::webidl::throw_type_error(s, "The value must be an Element or null");
        return;
    };
    update(s, a.this(), |x| x.interest_for = v)
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
pub(crate) fn to_string(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_href(s, a, r)
}
