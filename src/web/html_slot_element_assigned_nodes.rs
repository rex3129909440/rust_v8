use super::html_slot_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "assignedNodes", 0, assigned_nodes)
}

pub(crate) fn assigned_nodes(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(nodes) = selected_nodes(scope, a.this(), a.get(0)) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let arr = v8::Array::new(scope, nodes.len() as i32);
    for (i, n) in nodes.iter().enumerate() {
        let _ = arr.set_index(scope, i as u32, (*n).into());
    }
    r.set(arr.into())
}
