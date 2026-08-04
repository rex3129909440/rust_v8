use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlPreElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) widths: HashMap<i32, i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlPreElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLPreElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlPreElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLPreElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_pre_element_width_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlPreElementStore>()
        .ok_or_else(|| "HTMLPreElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_with_tag(scope, "PRE")
}

pub(crate) fn create_with_tag<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    tag_name: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create HTMLPreElement".to_owned());
    }
    super::html_element::attach(scope, object, tag_name);
    scope
        .get_slot_mut::<HtmlPreElementStore>()
        .ok_or_else(|| "HTMLPreElement state was not prepared".to_owned())?
        .widths
        .insert(object.get_identity_hash().get(), 0);
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn get_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(width) = scope
        .get_slot::<HtmlPreElementStore>()
        .and_then(|store| {
            store
                .widths
                .get(&arguments.this().get_identity_hash().get())
        })
        .copied()
    {
        result.set(v8::Integer::new(scope, width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let width = arguments.get(0).int32_value(scope).unwrap_or(0);
    if let Some(current) = scope
        .get_slot_mut::<HtmlPreElementStore>()
        .and_then(|store| {
            store
                .widths
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = width;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
