use super::html_slot_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "assign", 0, assign)
}

pub(crate) fn assign(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut assigned = Vec::new();
    for i in 0..a.length() {
        let Ok(node) = v8::Local::<v8::Object>::try_from(a.get(i)) else {
            crate::webidl::throw_type_error(scope, "Assigned values must be Nodes");
            return;
        };
        if super::node::record(scope, node).is_none() {
            crate::webidl::throw_type_error(scope, "Assigned values must be Nodes");
            return;
        }
        assigned.push(v8::Global::new(scope, node));
    }
    if let Some(x) = scope
        .get_slot_mut::<HtmlSlotElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.assigned = assigned;
        dispatch_slotchange(scope, a.this());
    }
}
