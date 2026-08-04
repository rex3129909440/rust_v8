pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "splitText", 1, split_text)
}

fn split_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(data) = super::text::data_if_text(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let units: Vec<u16> = data.encode_utf16().collect();
    let offset = arguments.get(0).uint32_value(scope).unwrap_or(0) as usize;
    if offset > units.len() {
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            "The offset is larger than the data length",
        );
        return;
    }
    let left = String::from_utf16_lossy(&units[..offset]);
    let right = String::from_utf16_lossy(&units[offset..]);
    let _ = super::character_data::set_data_without_range_adjustment(scope, arguments.this(), left);
    let new_text = match super::text::create(scope, right) {
        Ok(text) => text,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if let Some(document) = super::node::record(scope, arguments.this())
        .and_then(|record| record.owner_document)
        .map(|document| v8::Local::new(scope, &document))
    {
        super::node::set_owner_document(scope, new_text, document);
    }
    if let Some(parent) = super::node::parent(scope, arguments.this()) {
        let index = super::node::children(scope, parent)
            .iter()
            .position(|candidate| candidate.strict_equals(arguments.this().into()))
            .map(|index| index + 1)
            .unwrap_or_else(|| super::node::children(scope, parent).len());
        if let Err((name, message)) = super::node::insert_node(scope, parent, new_text, index) {
            super::node::throw_dom_exception(scope, name, message);
            return;
        }
    }
    super::abstract_range::adjust_for_split_text(scope, arguments.this(), new_text, offset as u32);
    result.set(new_text.into());
}
