use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct SvgFeComponentTransferElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) input: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) result: v8::Global<v8::Object>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SvgFeComponentTransferElementStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "SVGFEComponentTransferElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = s
        .get_slot::<SvgFeComponentTransferElementStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(s, &old));
    }
    let c = crate::webidl::create_function(
        s,
        "SVGFEComponentTransferElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::svg_fe_component_transfer_element_in1_property::define(s, p)?;
    super::svg_fe_component_transfer_element_x_property::define(s, p)?;
    super::svg_fe_component_transfer_element_y_property::define(s, p)?;
    super::svg_fe_component_transfer_element_width_property::define(s, p)?;
    super::svg_fe_component_transfer_element_height_property::define(s, p)?;
    super::svg_fe_component_transfer_element_result_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::svg_element::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SvgFeComponentTransferElementStore>()
        .ok_or_else(|| "SVGFEComponentTransferElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let o = super::svg_element::create_with_constructor(s, c, "feComponentTransfer", owner)?;
    let input = super::svg_animated_string::create(s, "")?;
    let x = super::svg_animated_length::create_with_unit(s, 2, 0.0)?;
    let y = super::svg_animated_length::create_with_unit(s, 2, 0.0)?;
    let width = super::svg_animated_length::create_with_unit(s, 2, 100.0)?;
    let height = super::svg_animated_length::create_with_unit(s, 2, 100.0)?;
    let result = super::svg_animated_string::create(s, "")?;
    let r = Record {
        input: v8::Global::new(s, input),
        x: v8::Global::new(s, x),
        y: v8::Global::new(s, y),
        width: v8::Global::new(s, width),
        height: v8::Global::new(s, height),
        result: v8::Global::new(s, result),
    };
    s.get_slot_mut::<SvgFeComponentTransferElementStore>()
        .ok_or_else(|| "SVGFEComponentTransferElement state was not prepared".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), r);
    Ok(o)
}
pub(crate) fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'SVGFEComponentTransferElement': Illegal constructor",
    )
}
pub(crate) fn rec(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeComponentTransferElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn ret(
    s: &v8::PinScope<'_, '_>,
    v: &v8::Global<v8::Object>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(v8::Local::new(s, v).into())
}
pub(crate) fn get_input(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.input, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.width, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.height, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_result(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.result, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
