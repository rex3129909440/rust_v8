#[derive(Default)]
pub(crate) struct DocumentGlobalStore {
    document: Option<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DocumentGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let url = crate::page_init::url(scope);
    let document = create_document(scope, &url)?;
    let html = crate::page_init::html(scope);
    if !html.is_empty() {
        super::document_html_parser::parse_page(scope, document, &html)?;
    }
    super::document::set_content_type(scope, document, crate::page_init::content_type(scope));
    super::document::set_string_value(
        scope,
        document,
        "referrer",
        &crate::page_init::referrer(scope),
    );
    super::document::set_string_value(scope, document, "domain", &crate::page_init::host(scope));
    let base_href = super::document::document_descendants(scope, document)
        .into_iter()
        .find(|node| {
            super::node::record(scope, *node)
                .is_some_and(|record| record.node_name.eq_ignore_ascii_case("BASE"))
        })
        .and_then(|base| super::element::attribute_value(scope, base, "href"));
    crate::page_init::update_base_url(scope, base_href.as_deref());
    let stored_document = v8::Global::new(scope, document);
    scope
        .get_slot_mut::<DocumentGlobalStore>()
        .ok_or_else(|| "document global state was not prepared".to_owned())?
        .document = Some(stored_document);
    install_existing(scope)?;
    super::html_script_element::execute_parser_inserted_tree(scope, document);
    Ok(())
}

pub(crate) fn install_existing(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get document",
        0,
        v8::ConstructorBehavior::Throw,
        get_document,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(false);
    let key = crate::webidl::string(scope, "document")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.document".to_owned())
    }
}

pub(crate) fn create_document<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    url: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let document = super::html_document::create(scope)?;
    let html = super::html_html_element::create(scope)?;
    let head = super::html_head_element::create(scope)?;
    let body = super::html_body_element::create(scope)?;
    super::node::set_owner_document(scope, html, document);
    super::node::set_owner_document(scope, head, document);
    super::node::set_owner_document(scope, body, document);
    if !super::node::insert_child(scope, document, html, 0) {
        return Err("cannot append the document element".to_owned());
    }
    if !super::node::insert_child(scope, html, head, 0) {
        return Err("cannot append the document head".to_owned());
    }
    if !super::node::insert_child(scope, html, body, 1) {
        return Err("cannot append the document body".to_owned());
    }
    super::document::set_string_value(scope, document, "URL", url);
    super::document::set_string_value(scope, document, "documentURI", url);
    super::document::set_string_value(scope, document, "fallbackBaseURL", url);
    super::document::set_string_value(scope, document, "compatMode", "CSS1Compat");
    let global = scope.get_current_context().global(scope);
    super::document::set_object_value(scope, document, "defaultView", global);
    define_location(scope, document)?;
    Ok(document)
}

fn define_location(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get location",
        0,
        v8::ConstructorBehavior::Throw,
        get_location,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set location",
        1,
        v8::ConstructorBehavior::Throw,
        set_location,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(false);
    let key = crate::webidl::string(scope, "location")?;
    if document.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define document.location".to_owned())
    }
}

fn get_location(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let global = scope.get_current_context().global(scope);
    if let Some(key) = v8::String::new(scope, "location")
        && let Some(location) = global.get(scope, key.into())
    {
        result.set(location);
    }
}

fn set_location(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let global = scope.get_current_context().global(scope);
    let Some(key) = v8::String::new(scope, "location") else {
        return;
    };
    let _ = global.set(scope, key.into(), arguments.get(0));
}

fn get_document(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(document) = value(scope) {
        result.set(document.into());
    } else {
        crate::webidl::throw_type_error(scope, "document is unavailable");
    }
}

pub(crate) fn value<'s>(scope: &v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::Object>> {
    if let Some(document) = super::html_i_frame_element::current_content_document(scope) {
        return Some(document);
    }
    let document = scope.get_slot::<DocumentGlobalStore>()?.document.as_ref()?;
    Some(v8::Local::new(scope, document))
}
