pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "surroundContents", 1, surround_contents)
}

fn surround_contents(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if super::abstract_range::record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(new_parent) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    let Some(node_record) = super::node::record(scope, new_parent) else {
        crate::webidl::throw_type_error(scope, "The argument is not a Node");
        return;
    };
    if matches!(node_record.node_type, 9..=11) {
        super::node::throw_dom_exception(
            scope,
            "InvalidNodeTypeError",
            "The new parent has an invalid node type",
        );
        return;
    }
    if super::range_contents::has_partially_contained_non_text(scope, arguments.this()) {
        super::node::throw_dom_exception(
            scope,
            "InvalidStateError",
            "Failed to execute 'surroundContents' on 'Range': The Range has partially selected a non-Text node.",
        );
        return;
    }
    let fragment = match super::range_contents::extract_contents(scope, arguments.this()) {
        Ok(fragment) => fragment,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    for child in super::node::children(scope, new_parent) {
        super::node::detach(scope, child);
    }
    if let Err((name, message)) =
        super::range_insert_node::insert_node(scope, arguments.this(), new_parent)
    {
        if name == "TypeError" {
            crate::webidl::throw_type_error(scope, &message);
        } else {
            super::node::throw_dom_exception(scope, name, &message);
        }
        return;
    }
    if let Err((name, message)) = super::node::insert_node(scope, new_parent, fragment, 0) {
        super::node::throw_dom_exception(scope, name, message);
        return;
    }
    let Some(parent) = super::node::parent(scope, new_parent) else {
        return;
    };
    let index = super::node::children(scope, parent)
        .iter()
        .position(|candidate| candidate.strict_equals(new_parent.into()))
        .unwrap_or(0) as u32;
    let parent = v8::Global::new(scope, parent);
    super::abstract_range::update(scope, arguments.this(), |value| {
        value.start_container = parent.clone();
        value.start_offset = index;
        value.end_container = parent;
        value.end_offset = index + 1;
    });
}
