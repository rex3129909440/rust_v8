pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(scope, prototype, "setBaseAndExtent", 4, set_base_and_extent)
}
fn set_base_and_extent(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Ok(base) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "base must be a Node");
        return;
    };
    let Ok(extent) = v8::Local::<v8::Object>::try_from(a.get(2)) else {
        crate::webidl::throw_type_error(scope, "extent must be a Node");
        return;
    };
    let base = v8::Global::new(scope, base);
    let extent = v8::Global::new(scope, extent);
    let bo = a.get(1).uint32_value(scope).unwrap_or(0);
    let eo = a.get(3).uint32_value(scope).unwrap_or(0);
    let base_local = v8::Local::new(scope, &base);
    let extent_local = v8::Local::new(scope, &extent);
    if !super::selection::valid_offset(scope, base_local, bo)
        || !super::selection::valid_offset(scope, extent_local, eo)
    {
        super::node::throw_dom_exception(scope, "IndexSizeError", "The offset is out of bounds");
        return;
    }
    let direction = super::selection::direction_between(scope, base_local, bo, extent_local, eo);
    let range = super::selection::selection_range(scope, base_local, bo, extent_local, eo);
    super::selection::update(scope, a.this(), |v| {
        v.anchor = Some(base);
        v.focus = Some(extent);
        v.anchor_offset = bo;
        v.focus_offset = eo;
        v.ranges = range.into_iter().collect();
        v.direction = direction;
    })
}
