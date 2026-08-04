pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "write", 0, write)
}

fn write(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !super::document::is_document(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut markup = String::new();
    for index in 0..arguments.length() {
        markup.push_str(&crate::webidl::value_to_string(scope, arguments.get(index)));
    }
    if let Err(message) = append_markup(scope, arguments.this(), &markup) {
        crate::webidl::throw_type_error(scope, &message);
    }
}

pub(crate) fn append_markup(
    scope: &mut v8::PinScope<'_, '_>,
    document: v8::Local<'_, v8::Object>,
    markup: &str,
) -> Result<(), String> {
    if let Some(source) = super::document::buffer_open_document_write(scope, document, markup) {
        return super::document::parse_source(scope, document, &source);
    }
    let target = super::dom_selector::descendants(scope, document)
        .into_iter()
        .find(|element| {
            super::node::record(scope, *element)
                .is_some_and(|record| record.node_name.eq_ignore_ascii_case("BODY"))
        })
        .or_else(|| {
            super::node::children(scope, document)
                .into_iter()
                .find(|child| {
                    super::node::record(scope, *child)
                        .is_some_and(|record| record.node_type == super::node::ELEMENT_NODE)
                })
        });
    let Some(target) = target else {
        return super::document::parse_source(scope, document, markup);
    };
    let parsed = super::dom_html::parse_fragment(scope, target, markup)?;
    let mut insertion = super::node::children(scope, target).len();
    for child in parsed {
        super::node::insert_node(scope, target, v8::Local::new(scope, &child), insertion)
            .map_err(|(_, message)| message.to_owned())?;
        insertion += 1;
    }
    Ok(())
}
