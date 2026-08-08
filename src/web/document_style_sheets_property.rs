pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(s, p, "styleSheets", get_style_sheets)
}
fn get_style_sheets(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !super::document_property_support::ensure(s, a.this()) {
        return;
    }
    if super::document_property_support::return_stored(s, a.this(), "styleSheets", r) {
        refresh(s, a.this());
        return;
    }
    match super::style_sheet_list::create(s, Vec::new()) {
        Ok(value) => {
            super::document::remember_value(s, a.this(), "styleSheets", value.into());
            refresh(s, a.this());
            r.set(value.into());
        }
        Err(message) => crate::webidl::throw_type_error(s, &message),
    }
}

pub(crate) fn sheets<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Vec<v8::Local<'s, v8::Object>> {
    super::document::document_descendants(scope, document)
        .into_iter()
        .filter_map(|element| {
            super::html_style_element::sheet(scope, element)
                .or_else(|| super::html_link_element::sheet(scope, element))
        })
        .collect()
}

pub(crate) fn refresh(scope: &mut v8::PinScope<'_, '_>, document: v8::Local<'_, v8::Object>) {
    let Some(value) = super::document::stored_value(scope, document, "styleSheets") else {
        return;
    };
    let value = v8::Local::new(scope, &value);
    let Ok(list) = v8::Local::<v8::Object>::try_from(value) else {
        return;
    };
    let values = sheets(scope, document);
    super::style_sheet_list::replace_values(scope, list, values);
}

pub(crate) fn refresh_for_node(scope: &mut v8::PinScope<'_, '_>, node: v8::Local<'_, v8::Object>) {
    if super::document::is_document(scope, node) {
        refresh(scope, node);
    } else if let Some(document) = super::node::owner_document(scope, node) {
        refresh(scope, document);
    }
}
