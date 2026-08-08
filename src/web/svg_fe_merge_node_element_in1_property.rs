use super::svg_fe_merge_node_element::*;

pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_readonly_accessor(scope, prototype, "in1", get_input)
}

fn get_input(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let input = s
        .get_slot::<SvgFeMergeNodeElementStore>()
        .and_then(|store| store.inputs.get(&a.this().get_identity_hash().get()))
        .cloned();
    if let Some(input) = input {
        r.set(v8::Local::new(s, &input).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
