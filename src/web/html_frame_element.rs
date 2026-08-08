use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct FrameRecord {
    pub(crate) name: String,
    pub(crate) scrolling: String,
    pub(crate) src: String,
    pub(crate) frame_border: String,
    pub(crate) long_desc: String,
    pub(crate) no_resize: bool,
    pub(crate) margin_height: String,
    pub(crate) margin_width: String,
}
#[derive(Default)]
pub(crate) struct HtmlFrameElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, FrameRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlFrameElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLFrameElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlFrameElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLFrameElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_frame_element_name_property::define(scope, prototype)?;
    super::html_frame_element_scrolling_property::define(scope, prototype)?;
    super::html_frame_element_src_property::define(scope, prototype)?;
    super::html_frame_element_frame_border_property::define(scope, prototype)?;
    super::html_frame_element_long_desc_property::define(scope, prototype)?;
    super::html_frame_element_no_resize_property::define(scope, prototype)?;
    super::html_frame_element_content_document_property::define(scope, prototype)?;
    super::html_frame_element_content_window_property::define(scope, prototype)?;
    super::html_frame_element_margin_height_property::define(scope, prototype)?;
    super::html_frame_element_margin_width_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlFrameElementStore>()
        .ok_or_else(|| "HTMLFrameElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLFrameElement".to_owned());
    }
    super::html_element::attach(scope, object, "FRAME");
    scope
        .get_slot_mut::<HtmlFrameElementStore>()
        .ok_or_else(|| "HTMLFrameElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), FrameRecord::default());
    Ok(object)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<FrameRecord> {
    scope
        .get_slot::<HtmlFrameElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut FrameRecord),
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlFrameElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&FrameRecord) -> &str,
) {
    if let Some(record) = record(scope, a.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            r.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    change: impl FnOnce(&mut FrameRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    update(scope, a.this(), |record| change(record, value))
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
pub(crate) fn get_no_resize(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, record.no_resize).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_no_resize(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    update(scope, a.this(), |x| x.no_resize = value)
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
