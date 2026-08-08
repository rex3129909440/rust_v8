pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "deleteFromDocument",
        0,
        delete_from_document,
    )
}
fn delete_from_document(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let (Some(anchor), Some(focus)) = (v.anchor.clone(), v.focus.clone()) {
        let x = v8::Local::new(scope, &anchor);
        let y = v8::Local::new(scope, &focus);
        if x.strict_equals(y.into())
            && let Some(data) = super::character_data::data_if_character(scope, x)
        {
            let units: Vec<u16> = data.encode_utf16().collect();
            let start = v.anchor_offset.min(v.focus_offset) as usize;
            let end = (v.anchor_offset.max(v.focus_offset) as usize).min(units.len());
            let mut out = units;
            out.drain(start..end);
            super::character_data::set_data_if_character(scope, x, String::from_utf16_lossy(&out));
        }
    }
    let anchor = v.anchor.clone();
    let offset = v.anchor_offset;
    super::selection::update(scope, a.this(), |x| {
        x.focus = anchor;
        x.focus_offset = offset;
        x.direction = "none".to_owned();
    })
}
