use super::html_slot_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "assignedElements", 0, assigned_elements)
}

pub(crate) fn assigned_elements(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(nodes) = selected_nodes(scope, a.this(), a.get(0)) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let elements = nodes
        .into_iter()
        .filter(|n| super::element::record(scope, *n).is_some())
        .collect::<Vec<_>>();
    let arr = v8::Array::new(scope, elements.len() as i32);
    for (i, n) in elements.iter().enumerate() {
        let _ = arr.set_index(scope, i as u32, (*n).into());
    }
    r.set(arr.into())
}
