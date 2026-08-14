use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgAElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) target: v8::Global<v8::Object>,
    pub(crate) rel_list: v8::Global<v8::Object>,
    pub(crate) href: v8::Global<v8::Object>,
    pub(crate) interest_for_element: Option<v8::Global<v8::Object>>,
    pub(crate) download: String,
    pub(crate) ping: String,
    pub(crate) hreflang: String,
    pub(crate) link_type: String,
    pub(crate) referrer_policy: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgAElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGAElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgAElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGAElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_a_element_target_property::define(scope, prototype)?;
    super::svg_a_element_rel_property::define(scope, prototype)?;
    super::svg_a_element_rel_list_property::define(scope, prototype)?;
    super::svg_a_element_href_property::define(scope, prototype)?;
    super::svg_a_element_interest_for_element_property::define(scope, prototype)?;
    super::svg_a_element_download_property::define(scope, prototype)?;
    super::svg_a_element_ping_property::define(scope, prototype)?;
    super::svg_a_element_hreflang_property::define(scope, prototype)?;
    super::svg_a_element_type_property::define(scope, prototype)?;
    super::svg_a_element_referrer_policy_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_graphics_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgAElementStore>()
        .ok_or_else(|| "SVGAElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object =
        super::svg_graphics_element::create_with_constructor(scope, constructor, "a", owner)?;
    let target = super::svg_animated_string::create(scope, "")?;
    let rel_list = super::dom_token_list::create_with_support(
        scope,
        "",
        super::dom_token_list::DomTokenSupport::HyperlinkRel,
    )?;
    let href = super::svg_animated_string::create(scope, "")?;
    let record = Record {
        target: v8::Global::new(scope, target),
        rel_list: v8::Global::new(scope, rel_list),
        href: v8::Global::new(scope, href),
        interest_for_element: None,
        download: String::new(),
        ping: String::new(),
        hreflang: String::new(),
        link_type: String::new(),
        referrer_policy: String::new(),
    };
    scope
        .get_slot_mut::<SvgAElementStore>()
        .ok_or_else(|| "SVGAElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGAElement': Illegal constructor",
    );
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgAElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) -> bool {
    if let Some(record) = scope
        .get_slot_mut::<SvgAElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
        true
    } else {
        false
    }
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) {
    if name == "interestfor" {
        update(scope, object, |record| record.interest_for_element = None);
    }
}
pub(crate) fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into())
    }
}
pub(crate) fn return_object(
    scope: &v8::PinScope<'_, '_>,
    value: &v8::Global<v8::Object>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, value).into())
}
pub(crate) fn get_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.target, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_href(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.href, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_rel_list(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.rel_list, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_rel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(s, &v.rel_list);
    if let Some(value) = super::dom_token_list::string_value(s, list) {
        return_string(s, &value, r)
    }
}
pub(crate) fn set_rel(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let list = v8::Local::new(s, &v.rel_list);
    let _ = super::dom_token_list::set_string_value(s, list, &value);
}
pub(crate) fn set_rel_list(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    set_rel(s, a, r)
}
pub(crate) fn get_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(object) = v.interest_for_element {
            r.set(v8::Local::new(s, &object).into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_interest_for_element(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0);
    let object = if value.is_null() {
        None
    } else {
        v8::Local::<v8::Object>::try_from(value)
            .ok()
            .map(|o| v8::Global::new(s, o))
    };
    if !update(s, a.this(), |v| v.interest_for_element = object) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_download(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.download, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_download(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !update(s, a.this(), |v| v.download = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_ping(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.ping, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_ping(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !update(s, a.this(), |v| v.ping = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_hreflang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.hreflang, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_hreflang(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !update(s, a.this(), |v| v.hreflang = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.link_type, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if !update(s, a.this(), |v| v.link_type = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_string(s, &v.referrer_policy, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn set_referrer_policy(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    let valid = matches!(
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
    let value = if valid { value } else { String::new() };
    if !update(s, a.this(), |v| v.referrer_policy = value) {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
