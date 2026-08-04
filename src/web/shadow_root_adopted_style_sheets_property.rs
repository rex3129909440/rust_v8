pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(
        scope,
        prototype,
        "adoptedStyleSheets",
        get_adopted_style_sheets,
        set_adopted_style_sheets,
    )
}
fn get_adopted_style_sheets(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::shadow_root::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let out = v8::Array::new(scope, v.adopted.len() as i32);
    for (i, s) in v.adopted.iter().enumerate() {
        let s = v8::Local::new(scope, s);
        let _ = out.set_index(scope, i as u32, s.into());
    }
    r.set(out.into())
}

fn set_adopted_style_sheets(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(seq) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "adoptedStyleSheets must be a sequence");
        return;
    };
    let length = v8::String::new(scope, "length")
        .and_then(|k| seq.get(scope, k.into()))
        .and_then(|v| v.uint32_value(scope))
        .unwrap_or(0);
    let mut values = Vec::new();
    for i in 0..length {
        if let Some(v) = seq
            .get_index(scope, i)
            .and_then(|v| v8::Local::<v8::Object>::try_from(v).ok())
        {
            values.push(v8::Global::new(scope, v));
        }
    }
    super::shadow_root::update(scope, a.this(), |v| v.adopted = values)
}
