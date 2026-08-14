use super::cookie_store::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "getAll", 0, get_all)
}

fn get_all(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::reject_illegal_invocation_promise(scope, "CookieStore", "getAll", result);
        return;
    }
    let name = requested_name(scope, arguments.get(0));
    let entries = super::document_cookie::global_snapshot(scope)
        .into_iter()
        .filter(|entry| name.as_ref().is_none_or(|name| entry.name == *name))
        .map(|entry| entry_from_cookie(&entry))
        .collect::<Vec<_>>();
    let array = v8::Array::new(scope, entries.len() as i32);
    for (index, entry) in entries.iter().enumerate() {
        let value = cookie_object(scope, entry);
        let _ = array.set_index(scope, index as u32, value.into());
    }
    resolved(scope, array.into(), result);
}
