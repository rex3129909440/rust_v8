pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "createContextualFragment",
        1,
        create_contextual_fragment,
    )
}

fn create_contextual_fragment(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(range) = super::abstract_range::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let markup = crate::webidl::value_to_string(scope, arguments.get(0));
    let start = v8::Local::new(scope, &range.start_container);
    let context = if super::node::record(scope, start)
        .is_some_and(|record| record.node_type == super::node::ELEMENT_NODE)
    {
        start
    } else {
        super::node::parent(scope, start)
            .filter(|parent| {
                super::node::record(scope, *parent)
                    .is_some_and(|record| record.node_type == super::node::ELEMENT_NODE)
            })
            .unwrap_or(start)
    };
    let parsed = match super::dom_html::parse_fragment(scope, context, &markup) {
        Ok(parsed) => parsed,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let fragment = match super::document_fragment::create(scope) {
        Ok(fragment) => fragment,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    if let Some(document) = super::node::record(scope, start)
        .and_then(|record| {
            if record.node_type == super::node::DOCUMENT_NODE {
                Some(v8::Global::new(scope, start))
            } else {
                record.owner_document
            }
        })
        .map(|document| v8::Local::new(scope, &document))
    {
        super::node::set_owner_document(scope, fragment, document);
    }
    for (index, child) in parsed.iter().enumerate() {
        let child = v8::Local::new(scope, child);
        if let Err((name, message)) = super::node::insert_node(scope, fragment, child, index) {
            super::node::throw_dom_exception(scope, name, message);
            return;
        }
    }
    result.set(fragment.into());
}
