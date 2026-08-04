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
