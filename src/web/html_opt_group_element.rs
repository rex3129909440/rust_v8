use std::collections::HashMap;

#[derive(Clone, Default)]
pub(crate) struct OptGroupRecord {
    pub(crate) disabled: bool,
    pub(crate) label: String,
}

#[derive(Default)]
pub(crate) struct HtmlOptGroupElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, OptGroupRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlOptGroupElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLOptGroupElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlOptGroupElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLOptGroupElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_opt_group_element_disabled_property::define(scope, prototype)?;
    super::html_opt_group_element_label_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlOptGroupElementStore>()
        .ok_or_else(|| "HTMLOptGroupElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLOptGroupElement".to_owned());
    }
    super::html_element::attach(scope, object, "OPTGROUP");
    scope
        .get_slot_mut::<HtmlOptGroupElementStore>()
        .ok_or_else(|| "HTMLOptGroupElement state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), OptGroupRecord::default());
    Ok(object)
}

pub(crate) fn is_opt_group(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<HtmlOptGroupElementStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&object.get_identity_hash().get())
        })
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<OptGroupRecord> {
    scope
        .get_slot::<HtmlOptGroupElementStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
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
    if let Some(record) = scope
        .get_slot_mut::<HtmlOptGroupElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.disabled = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.label) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_label(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlOptGroupElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.label = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
