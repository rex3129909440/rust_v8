use super::svg_element::*;
pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<(), String> {
    let data = v8::String::new(s, name).ok_or_else(|| "invalid touch handler".to_owned())?;
    crate::webidl::define_accessor_with_data(s, p, name, get, set, data.into())
}
fn get(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(s, crate::trace::native_callback_data(s, &a));
    let Some(record) = record(s, a.this()) else {
        return;
    };
    if let Some(v) = record.handlers.get(&name) {
        r.set(v8::Local::new(s, v));
    } else {
        r.set(v8::null(s).into());
    }
}
fn set(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, _: v8::ReturnValue<'_>) {
    let name = crate::webidl::value_to_string(s, crate::trace::native_callback_data(s, &a));
    let value = super::window_event_handler_support::handler_value(s, a.get(0));
    let present = value.is_some();
    if let Some(record) = s
        .get_slot_mut::<SvgElementStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        if let Some(v) = value {
            record.handlers.insert(name.clone(), v);
        } else {
            record.handlers.remove(&name);
        }
    } else {
        return;
    }
    super::event_target::set_attribute_handler(
        s,
        a.this(),
        name.strip_prefix("on").unwrap_or(&name),
        present,
    );
}
