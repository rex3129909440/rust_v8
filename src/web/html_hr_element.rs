use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct HorizontalRuleRecord {
    pub(crate) align: String,
    pub(crate) color: String,
    pub(crate) no_shade: bool,
    pub(crate) size: String,
    pub(crate) width: String,
}
#[derive(Default)]
pub(crate) struct HtmlHrElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, HorizontalRuleRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlHrElementStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLHRElement", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlHrElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLHRElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_hr_element_align_property::define(scope, prototype)?;
    super::html_hr_element_color_property::define(scope, prototype)?;
    super::html_hr_element_no_shade_property::define(scope, prototype)?;
    super::html_hr_element_size_property::define(scope, prototype)?;
    super::html_hr_element_width_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlHrElementStore>()
        .ok_or_else(|| "HTMLHRElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLHRElement".to_owned());
    }
    super::html_element::attach(scope, object, "HR");
    scope
        .get_slot_mut::<HtmlHrElementStore>()
        .ok_or_else(|| "HTMLHRElement state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            HorizontalRuleRecord::default(),
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
) -> Option<HorizontalRuleRecord> {
    scope
        .get_slot::<HtmlHrElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_string(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    select: impl FnOnce(&HorizontalRuleRecord) -> &str,
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
    change: impl FnOnce(&mut HorizontalRuleRecord, String),
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlHrElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        change(record, value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.align);
}
pub(crate) fn set_align(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.align = v);
}
pub(crate) fn get_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.color);
}
pub(crate) fn set_color(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.color = v);
}
pub(crate) fn get_no_shade(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, a.this()) {
        r.set(v8::Boolean::new(scope, record.no_shade).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn set_no_shade(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = a.get(0).boolean_value(scope);
    if let Some(record) = scope
        .get_slot_mut::<HtmlHrElementStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        record.no_shade = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
pub(crate) fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.size);
}
pub(crate) fn set_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.size = v);
}
pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    get_string(s, a, r, |x| &x.width);
}
pub(crate) fn set_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_string(s, a, |x, v| x.width = v);
}
