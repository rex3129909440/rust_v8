#[derive(Default)]
pub(crate) struct StaticRangeStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(StaticRangeStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "StaticRange", c.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<StaticRangeStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(e) = existing {
        return Ok(v8::Local::new(scope, &e));
    }
    let c = crate::webidl::create_function(
        scope,
        "StaticRange",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    let parent = super::abstract_range::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, c, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<StaticRangeStore>()
        .ok_or_else(|| "StaticRange state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn prop<'s>(
    scope: &v8::PinScope<'s, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let k = v8::String::new(scope, n)?;
    let v = o.get(scope, k.into())?;
    v8::Local::<v8::Object>::try_from(v).ok()
}
fn uint(scope: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str) -> u32 {
    v8::String::new(scope, n)
        .and_then(|k| o.get(scope, k.into()))
        .and_then(|v| v.uint32_value(scope))
        .unwrap_or(0)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(scope, "StaticRange requires an init object");
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(scope, "StaticRange init must be an object");
        return;
    };
    let Some(start) = prop(scope, init, "startContainer") else {
        crate::webidl::throw_type_error(scope, "startContainer is required");
        return;
    };
    let Some(end) = prop(scope, init, "endContainer") else {
        crate::webidl::throw_type_error(scope, "endContainer is required");
        return;
    };
    let start_offset = uint(scope, init, "startOffset");
    let end_offset = uint(scope, init, "endOffset");
    if super::node::record(scope, start).is_some_and(|record| record.node_type == 10)
        || super::node::record(scope, end).is_some_and(|record| record.node_type == 10)
    {
        super::node::throw_dom_exception(
            scope,
            "InvalidNodeTypeError",
            "DocumentType cannot be a boundary container",
        );
        return;
    }
    if super::range::boundary_length(scope, start).is_none_or(|length| start_offset > length)
        || super::range::boundary_length(scope, end).is_none_or(|length| end_offset > length)
    {
        super::node::throw_dom_exception(
            scope,
            "IndexSizeError",
            "A boundary offset is out of bounds",
        );
        return;
    }
    super::abstract_range::attach(scope, a.this(), start, start_offset, end, end_offset);
    r.set(a.this().into())
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: &super::abstract_range::RangeRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create StaticRange".to_owned());
    }
    let start = v8::Local::new(scope, &record.start_container);
    let end = v8::Local::new(scope, &record.end_container);
    super::abstract_range::attach(
        scope,
        object,
        start,
        record.start_offset,
        end,
        record.end_offset,
    );
    Ok(object)
}
