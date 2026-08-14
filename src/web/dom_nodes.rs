pub(crate) fn arguments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
) -> Result<Vec<v8::Global<v8::Object>>, String> {
    let mut nodes = Vec::new();
    for index in 0..arguments.length() {
        let value = arguments.get(index);
        let node = v8::Local::<v8::Object>::try_from(value)
            .ok()
            .filter(|object| super::node::record(scope, *object).is_some())
            .map(Ok)
            .unwrap_or_else(|| {
                super::text::create(scope, crate::webidl::value_to_string(scope, value))
            })?;
        nodes.push(v8::Global::new(scope, node));
    }
    Ok(nodes)
}

pub(crate) fn insert(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    index: usize,
    nodes: &[v8::Global<v8::Object>],
) -> Result<(), (&'static str, &'static str)> {
    let mut insertion = index;
    for node in nodes {
        super::node::insert_node(scope, parent, v8::Local::new(scope, node), insertion)?;
        insertion += 1;
    }
    Ok(())
}

pub(crate) fn child_index(
    scope: &v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    child: v8::Local<'_, v8::Object>,
) -> Option<usize> {
    super::node::children(scope, parent)
        .iter()
        .position(|candidate| {
            candidate.get_identity_hash().get() == child.get_identity_hash().get()
        })
}

pub(crate) fn ensure_parent_node(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    if super::node::record(scope, object)
        .is_some_and(|record| matches!(record.node_type, 1 | 9 | 11))
    {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn insert_error(scope: &mut v8::PinScope<'_, '_>, error: (&'static str, &'static str)) {
    super::node::throw_dom_exception(scope, error.0, error.1);
}

fn contains_node(
    scope: &v8::PinScope<'_, '_>,
    nodes: &[v8::Global<v8::Object>],
    node: v8::Local<'_, v8::Object>,
) -> bool {
    nodes
        .iter()
        .any(|value| v8::Local::new(scope, value).strict_equals(node.into()))
}

fn insert_before_reference(
    scope: &mut v8::PinScope<'_, '_>,
    parent: v8::Local<'_, v8::Object>,
    nodes: &[v8::Global<v8::Object>],
    reference: Option<v8::Global<v8::Object>>,
) -> Result<(), (&'static str, &'static str)> {
    let fragment = super::document_fragment::create(scope)
        .map_err(|_| ("TypeError", "The DocumentFragment could not be created"))?;
    for node in nodes {
        let index = super::node::children(scope, fragment).len();
        super::node::insert_node(scope, fragment, v8::Local::new(scope, node), index)?;
    }
    let index = reference
        .as_ref()
        .and_then(|reference| child_index(scope, parent, v8::Local::new(scope, reference)))
        .unwrap_or_else(|| super::node::children(scope, parent).len());
    super::node::insert_node(scope, parent, fragment, index)
}

fn reject_symbol_child_argument(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    callback_arguments: &v8::FunctionCallbackArguments<'_>,
    method: &str,
) -> bool {
    let has_symbol = (0..callback_arguments.length()).any(|index| {
        let value = callback_arguments.get(index);
        value.is_symbol()
    });
    if !has_symbol {
        return false;
    }
    let interface = if super::character_data::data_if_character(scope, object).is_some() {
        "CharacterData"
    } else if super::element::record(scope, object).is_some() {
        "Element"
    } else {
        "DocumentType"
    };
    crate::webidl::throw_type_error(
        scope,
        &format!(
            "Failed to execute '{method}' on '{interface}': Cannot convert a Symbol value to a string"
        ),
    );
    true
}

pub(crate) fn child_before(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    callback_arguments: &v8::FunctionCallbackArguments<'_>,
) -> Result<(), (&'static str, &'static str)> {
    if reject_symbol_child_argument(scope, object, callback_arguments, "before") {
        return Ok(());
    }
    let nodes = arguments(scope, callback_arguments)
        .map_err(|_| ("TypeError", "Node conversion failed"))?;
    let Some(parent) = super::node::parent(scope, object) else {
        return Ok(());
    };
    let siblings = super::node::children(scope, parent);
    let Some(index) = siblings
        .iter()
        .position(|node| node.strict_equals(object.into()))
    else {
        return Ok(());
    };
    let previous = siblings[..index]
        .iter()
        .rev()
        .find(|node| !contains_node(scope, &nodes, **node))
        .copied();
    let reference = match previous {
        Some(previous) => {
            let current = super::node::children(scope, parent);
            let index = current
                .iter()
                .position(|node| node.strict_equals(previous.into()))
                .map(|index| index + 1)
                .unwrap_or(0);
            current.get(index).map(|node| v8::Global::new(scope, *node))
        }
        None => super::node::children(scope, parent)
            .first()
            .map(|node| v8::Global::new(scope, *node)),
    };
    insert_before_reference(scope, parent, &nodes, reference)
}

pub(crate) fn child_after(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    callback_arguments: &v8::FunctionCallbackArguments<'_>,
) -> Result<(), (&'static str, &'static str)> {
    if reject_symbol_child_argument(scope, object, callback_arguments, "after") {
        return Ok(());
    }
    let nodes = arguments(scope, callback_arguments)
        .map_err(|_| ("TypeError", "Node conversion failed"))?;
    let Some(parent) = super::node::parent(scope, object) else {
        return Ok(());
    };
    let siblings = super::node::children(scope, parent);
    let Some(index) = siblings
        .iter()
        .position(|node| node.strict_equals(object.into()))
    else {
        return Ok(());
    };
    let reference = siblings[index + 1..]
        .iter()
        .find(|node| !contains_node(scope, &nodes, **node))
        .map(|node| v8::Global::new(scope, *node));
    insert_before_reference(scope, parent, &nodes, reference)
}

pub(crate) fn child_replace_with(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    callback_arguments: &v8::FunctionCallbackArguments<'_>,
) -> Result<(), (&'static str, &'static str)> {
    if reject_symbol_child_argument(scope, object, callback_arguments, "replaceWith") {
        return Ok(());
    }
    let nodes = arguments(scope, callback_arguments)
        .map_err(|_| ("TypeError", "Node conversion failed"))?;
    let Some(parent) = super::node::parent(scope, object) else {
        return Ok(());
    };
    let siblings = super::node::children(scope, parent);
    let Some(index) = siblings
        .iter()
        .position(|node| node.strict_equals(object.into()))
    else {
        return Ok(());
    };
    let reference = siblings[index + 1..]
        .iter()
        .find(|node| !contains_node(scope, &nodes, **node))
        .map(|node| v8::Global::new(scope, *node));
    let retains_object = contains_node(scope, &nodes, object);
    insert_before_reference(scope, parent, &nodes, reference)?;
    if !retains_object
        && super::node::parent(scope, object)
            .is_some_and(|current| current.strict_equals(parent.into()))
    {
        super::node::detach(scope, object);
    }
    Ok(())
}

pub(crate) fn adjacent_location<'s>(
    scope: &v8::PinScope<'s, '_>,
    element: v8::Local<'s, v8::Object>,
    position: &str,
) -> Result<Option<(v8::Local<'s, v8::Object>, usize)>, ()> {
    match position.to_ascii_lowercase().as_str() {
        "beforebegin" => {
            let Some(parent) = super::node::parent(scope, element) else {
                return Ok(None);
            };
            Ok(child_index(scope, parent, element).map(|index| (parent, index)))
        }
        "afterbegin" => Ok(Some((element, 0))),
        "beforeend" => Ok(Some((element, super::node::children(scope, element).len()))),
        "afterend" => {
            let Some(parent) = super::node::parent(scope, element) else {
                return Ok(None);
            };
            Ok(child_index(scope, parent, element).map(|index| (parent, index + 1)))
        }
        _ => Err(()),
    }
}

pub(crate) fn throw_invalid_adjacent_position(scope: &mut v8::PinScope<'_, '_>) {
    super::node::throw_dom_exception(
        scope,
        "SyntaxError",
        "The value provided is not one of 'beforebegin', 'afterbegin', 'beforeend', or 'afterend'",
    );
}
