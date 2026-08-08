use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct SvgStyleElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}

#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) style_type: String,
    pub(crate) media: String,
    pub(crate) title: String,
    pub(crate) sheet: v8::Global<v8::Object>,
    pub(crate) disabled: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgStyleElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGStyleElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgStyleElementStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGStyleElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::svg_style_element_type_property::define(scope, prototype)?;
    super::svg_style_element_media_property::define(scope, prototype)?;
    super::svg_style_element_title_property::define(scope, prototype)?;
    super::svg_style_element_sheet_property::define(scope, prototype)?;
    super::svg_style_element_disabled_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::svg_element::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgStyleElementStore>()
        .ok_or_else(|| "SVGStyleElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let object = super::svg_element::create_with_constructor(scope, constructor, "style", owner)?;
    let media = crate::webidl::string(scope, "all")?;
    let sheet = super::style_sheet::create(scope, None, None, media.into())?;
    let record = Record {
        style_type: "text/css".to_owned(),
        media: "all".to_owned(),
        title: String::new(),
        sheet: v8::Global::new(scope, sheet),
        disabled: false,
    };
    scope
        .get_slot_mut::<SvgStyleElementStore>()
        .ok_or_else(|| "SVGStyleElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGStyleElement': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Record> {
    scope
        .get_slot::<SvgStyleElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_string(
    scope: &v8::PinScope<'_, '_>,
    value: &str,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = v8::String::new(scope, value) {
        result.set(value.into());
    }
}

pub(crate) fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.style_type, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.style_type = value);
}

pub(crate) fn get_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.media, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_media(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.media = value);
}

pub(crate) fn get_title(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        return_string(scope, &record.title, result);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_title(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    update(scope, arguments.this(), |record| record.title = value);
}

pub(crate) fn get_sheet(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.sheet).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.disabled).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).boolean_value(scope);
    update(scope, arguments.this(), |record| record.disabled = value);
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Record),
) {
    if let Some(record) = scope
        .get_slot_mut::<SvgStyleElementStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    {
        change(record);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
