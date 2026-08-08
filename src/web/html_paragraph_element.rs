use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlParagraphElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) alignments: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlParagraphElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLParagraphElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlParagraphElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLParagraphElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_paragraph_element_align_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlParagraphElementStore>()
        .ok_or_else(|| "HTMLParagraphElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLParagraphElement".to_owned());
    }
    super::html_element::attach(scope, object, "P");
    scope
        .get_slot_mut::<HtmlParagraphElementStore>()
        .ok_or_else(|| "HTMLParagraphElement state was not prepared".to_owned())?
        .alignments
        .insert(object.get_identity_hash().get(), String::new());
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn get_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(align) = scope
        .get_slot::<HtmlParagraphElementStore>()
        .and_then(|store| {
            store
                .alignments
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        if let Some(value) = v8::String::new(scope, align) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn set_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(align) = scope
        .get_slot_mut::<HtmlParagraphElementStore>()
        .and_then(|store| {
            store
                .alignments
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *align = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
