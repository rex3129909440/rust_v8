use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct HtmlFencedFrameElementRecord {
    pub(crate) width: String,
    pub(crate) height: String,
    pub(crate) sandbox: v8::Global<v8::Object>,
    pub(crate) config: Option<v8::Global<v8::Object>>,
    pub(crate) allow: String,
}

#[derive(Default)]
pub(crate) struct HtmlFencedFrameElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, HtmlFencedFrameElementRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlFencedFrameElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLFencedFrameElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlFencedFrameElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLFencedFrameElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_fenced_frame_element_width_property::define(scope, prototype)?;
    super::html_fenced_frame_element_height_property::define(scope, prototype)?;
    super::html_fenced_frame_element_sandbox_property::define(scope, prototype)?;
    super::html_fenced_frame_element_config_property::define(scope, prototype)?;
    super::html_fenced_frame_element_allow_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::html_fenced_frame_element_can_load_opaque_url::define(scope, constructor.into())?;
    let parent = super::html_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlFencedFrameElementStore>()
        .ok_or_else(|| "HTMLFencedFrameElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let element = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, element, prototype.into()) != Some(true) {
        return Err("cannot create HTMLFencedFrameElement".to_owned());
    }
    super::html_element::attach(scope, element, "FENCEDFRAME");
    let sandbox = super::dom_token_list::create(scope, "")?;
    let record = HtmlFencedFrameElementRecord {
        width: String::new(),
        height: String::new(),
        sandbox: v8::Global::new(scope, sandbox),
        config: None,
        allow: String::new(),
    };
    scope
        .get_slot_mut::<HtmlFencedFrameElementStore>()
        .ok_or_else(|| "HTMLFencedFrameElement state was not prepared".to_owned())?
        .records
        .insert(element.get_identity_hash().get(), record);
    Ok(element)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<HtmlFencedFrameElementRecord> {
    scope
        .get_slot::<HtmlFencedFrameElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut HtmlFencedFrameElementRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlFencedFrameElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'HTMLFencedFrameElement': Illegal constructor",
    );
}

pub(crate) fn get_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&HtmlFencedFrameElementRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

pub(crate) fn set_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut HtmlFencedFrameElementRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| change(record, value));
}

pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_text(s, a, r, |x| &x.width)
}
pub(crate) fn set_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_text(s, a, |x, v| x.width = v)
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_text(s, a, r, |x| &x.height)
}
pub(crate) fn set_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_text(s, a, |x, v| x.height = v)
}
pub(crate) fn get_allow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_text(s, a, r, |x| &x.allow)
}
pub(crate) fn set_allow(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_text(s, a, |x, v| x.allow = v)
}

pub(crate) fn get_sandbox(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.sandbox).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_sandbox(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let sandbox = v8::Local::new(scope, &record.sandbox);
    let _ = super::dom_token_list::set_string_value(scope, sandbox, &value);
}

pub(crate) fn get_config(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    match record(scope, arguments.this()) {
        Some(record) => match record.config {
            Some(config) => result.set(v8::Local::new(scope, &config).into()),
            None => result.set(v8::null(scope).into()),
        },
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn set_config(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0);
    let config = if value.is_null() {
        None
    } else {
        let Ok(config) = v8::Local::<v8::Object>::try_from(value) else {
            crate::webidl::throw_type_error(
                scope,
                "Failed to set 'config' on 'HTMLFencedFrameElement': value is not a FencedFrameConfig.",
            );
            return;
        };
        if !super::fenced_frame_config::is_instance(scope, config) {
            crate::webidl::throw_type_error(
                scope,
                "Failed to set 'config' on 'HTMLFencedFrameElement': value is not a FencedFrameConfig.",
            );
            return;
        }
        Some(v8::Global::new(scope, config))
    };
    update(scope, arguments.this(), |record| record.config = config);
}

pub(crate) fn can_load_opaque_url(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Boolean::new(scope, false).into());
}
