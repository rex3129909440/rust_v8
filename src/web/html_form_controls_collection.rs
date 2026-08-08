#[derive(Default)]
pub(crate) struct HtmlFormControlsCollectionStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlFormControlsCollectionStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLFormControlsCollection", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<HtmlFormControlsCollectionStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::html_collection::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "HTMLFormControlsCollection",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "namedItem", 1, named_item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<HtmlFormControlsCollectionStore>()
        .ok_or_else(|| "HTMLFormControlsCollection state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    items: Vec<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let collection = super::html_collection::create(scope, items)?;
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    if crate::webidl::set_platform_prototype(scope, collection, prototype.into()) != Some(true) {
        return Err("cannot create HTMLFormControlsCollection".to_owned());
    }
    Ok(collection)
}

pub(crate) fn replace(
    scope: &mut v8::PinScope<'_, '_>,
    collection: v8::Local<'_, v8::Object>,
    items: Vec<v8::Local<'_, v8::Object>>,
) -> bool {
    super::html_collection::replace(scope, collection, items)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'HTMLFormControlsCollection': Illegal constructor",
    );
}

fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    super::html_collection::refresh_live(scope, arguments.this());
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(items) = super::html_collection::items(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if name.is_empty() {
        result.set(v8::null(scope).into());
        return;
    }
    let mut matches = Vec::new();
    for item in items {
        let item = v8::Local::new(scope, &item);
        let attribute_match = super::element::record(scope, item).is_some_and(|element| {
            element.attributes.iter().any(|(attribute, value)| {
                (attribute.eq_ignore_ascii_case("id") || attribute.eq_ignore_ascii_case("name"))
                    && value == &name
            })
        });
        let property_match = v8::String::new(scope, "name")
            .and_then(|key| item.get(scope, key.into()))
            .is_some_and(|value| crate::webidl::value_to_string(scope, value) == name);
        if attribute_match || property_match {
            matches.push(item);
        }
    }
    match matches.len() {
        0 => result.set(v8::null(scope).into()),
        1 => result.set(matches[0].into()),
        _ => match super::radio_node_list::create(scope, matches) {
            Ok(list) => result.set(list.into()),
            Err(message) => crate::webidl::throw_type_error(scope, &message),
        },
    }
}
