pub(crate) fn define(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_method(
        scope,
        prototype,
        "getComposedRanges",
        0,
        get_composed_ranges,
    )
}
fn get_composed_ranges(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = super::selection::record(scope, a.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let out = v8::Array::new(scope, v.ranges.len() as i32);
    let allowed_roots = allowed_shadow_roots(scope, a.get(0));
    for (i, range) in v.ranges.iter().enumerate() {
        let range = v8::Local::new(scope, range);
        let Some(mut record) = super::abstract_range::record(scope, range) else {
            continue;
        };
        let (start, start_offset) = retarget_boundary(
            scope,
            record.start_container.clone(),
            record.start_offset,
            false,
            &allowed_roots,
        );
        let (end, end_offset) = retarget_boundary(
            scope,
            record.end_container.clone(),
            record.end_offset,
            true,
            &allowed_roots,
        );
        record.start_container = start;
        record.start_offset = start_offset;
        record.end_container = end;
        record.end_offset = end_offset;
        let Ok(static_range) = super::static_range::create(scope, &record) else {
            continue;
        };
        let _ = out.set_index(scope, i as u32, static_range.into());
    }
    r.set(out.into())
}

fn allowed_shadow_roots(
    scope: &v8::PinScope<'_, '_>,
    options: v8::Local<'_, v8::Value>,
) -> Vec<i32> {
    if options.is_undefined() || options.is_null() {
        return Vec::new();
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(options) else {
        return Vec::new();
    };
    let Some(key) = v8::String::new(scope, "shadowRoots") else {
        return Vec::new();
    };
    let Some(value) = options.get(scope, key.into()) else {
        return Vec::new();
    };
    let Ok(array) = v8::Local::<v8::Array>::try_from(value) else {
        return Vec::new();
    };
    (0..array.length())
        .filter_map(|index| array.get_index(scope, index))
        .filter_map(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .filter(|root| super::shadow_root::record(scope, *root).is_some())
        .map(|root| root.get_identity_hash().get())
        .collect()
}

fn retarget_boundary(
    scope: &v8::PinScope<'_, '_>,
    mut container: v8::Global<v8::Object>,
    mut offset: u32,
    after_host: bool,
    allowed_roots: &[i32],
) -> (v8::Global<v8::Object>, u32) {
    loop {
        let local = v8::Local::new(scope, &container);
        let root = super::node::root_node(scope, local);
        if super::shadow_root::record(scope, root).is_none()
            || allowed_roots.contains(&root.get_identity_hash().get())
        {
            break;
        }
        let Some(host) = super::shadow_root::host(scope, root) else {
            break;
        };
        let Some(parent) = super::node::parent(scope, host) else {
            break;
        };
        let Some(index) = super::node::children(scope, parent)
            .iter()
            .position(|child| child.strict_equals(host.into()))
        else {
            break;
        };
        container = v8::Global::new(scope, parent);
        offset = index as u32 + u32::from(after_host);
    }
    (container, offset)
}
