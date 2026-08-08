use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct HtmlDataListElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) options: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlDataListElementStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLDataListElement", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<HtmlDataListElementStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLDataListElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::html_data_list_element_options_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlDataListElementStore>()
        .ok_or_else(|| "HTMLDataListElement state was not prepared".to_owned())?
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
        return Err("cannot create HTMLDataListElement".to_owned());
    }
    super::html_element::attach(scope, object, "DATALIST");
    let options = super::html_collection::create(scope, Vec::new())?;
    super::html_collection::register_data_list_owner(scope, options, object);
    let options = v8::Global::new(scope, options);
    scope
        .get_slot_mut::<HtmlDataListElementStore>()
        .ok_or_else(|| "HTMLDataListElement state was not prepared".to_owned())?
        .options
        .insert(object.get_identity_hash().get(), options);
    Ok(object)
}

pub(crate) fn collect_options<'s>(
    scope: &v8::PinScope<'s, '_>,
    data_list: v8::Local<'s, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    fn descend<'s>(
        scope: &v8::PinScope<'s, '_>,
        root: v8::Local<'s, v8::Object>,
        output: &mut Vec<v8::Local<'s, v8::Object>>,
    ) {
        for child in super::node::children(scope, root) {
            if super::html_option_element::is_option(scope, child) {
                output.push(child);
            }
            descend(scope, child, output);
        }
    }
    let mut options = Vec::new();
    descend(scope, data_list, &mut options);
    options
}

pub(crate) fn is_data_list(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope
        .get_slot::<HtmlDataListElementStore>()
        .is_some_and(|store| {
            store
                .options
                .contains_key(&object.get_identity_hash().get())
        })
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn get_options(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let options = scope
        .get_slot::<HtmlDataListElementStore>()
        .and_then(|store| {
            store
                .options
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    let Some(options) = options else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let options = v8::Local::new(scope, &options);
    super::html_collection::refresh_live(scope, options);
    result.set(options.into());
}
