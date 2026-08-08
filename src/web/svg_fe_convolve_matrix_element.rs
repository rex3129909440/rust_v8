use std::collections::HashMap;
pub(crate) const EDGE_UNKNOWN: i32 = 0;
pub(crate) const EDGE_DUPLICATE: i32 = 1;
pub(crate) const EDGE_WRAP: i32 = 2;
pub(crate) const EDGE_NONE: i32 = 3;
#[derive(Default)]
pub(crate) struct SvgFeConvolveMatrixElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Record>,
}
#[derive(Clone)]
pub(crate) struct Record {
    pub(crate) input: v8::Global<v8::Object>,
    pub(crate) order_x: v8::Global<v8::Object>,
    pub(crate) order_y: v8::Global<v8::Object>,
    pub(crate) kernel_matrix: v8::Global<v8::Object>,
    pub(crate) divisor: v8::Global<v8::Object>,
    pub(crate) bias: v8::Global<v8::Object>,
    pub(crate) target_x: v8::Global<v8::Object>,
    pub(crate) target_y: v8::Global<v8::Object>,
    pub(crate) edge_mode: v8::Global<v8::Object>,
    pub(crate) kernel_x: v8::Global<v8::Object>,
    pub(crate) kernel_y: v8::Global<v8::Object>,
    pub(crate) preserve_alpha: v8::Global<v8::Object>,
    pub(crate) x: v8::Global<v8::Object>,
    pub(crate) y: v8::Global<v8::Object>,
    pub(crate) width: v8::Global<v8::Object>,
    pub(crate) height: v8::Global<v8::Object>,
    pub(crate) result: v8::Global<v8::Object>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SvgFeConvolveMatrixElementStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "SVGFEConvolveMatrixElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let old = s
        .get_slot::<SvgFeConvolveMatrixElementStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned();
    if let Some(old) = old {
        return Ok(v8::Local::new(s, &old));
    }
    let c = crate::webidl::create_function(
        s,
        "SVGFEConvolveMatrixElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::svg_fe_convolve_matrix_element_in1_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_order_x_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_order_y_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_kernel_matrix_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_divisor_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_bias_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_target_x_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_target_y_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_edge_mode_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_kernel_unit_length_x_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_kernel_unit_length_y_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_preserve_alpha_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_x_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_y_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_width_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_height_property::define(s, p)?;
    super::svg_fe_convolve_matrix_element_result_property::define(s, p)?;
    constants(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    constants(s, c.into())?;
    let parent = super::svg_element::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SvgFeConvolveMatrixElementStore>()
        .ok_or_else(|| "SVGFEConvolveMatrixElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn constants(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(s, o, "SVG_EDGEMODE_UNKNOWN", EDGE_UNKNOWN)?;
    crate::webidl::define_constant(s, o, "SVG_EDGEMODE_DUPLICATE", EDGE_DUPLICATE)?;
    crate::webidl::define_constant(s, o, "SVG_EDGEMODE_WRAP", EDGE_WRAP)?;
    crate::webidl::define_constant(s, o, "SVG_EDGEMODE_NONE", EDGE_NONE)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    owner: Option<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(s)?;
    let o = super::svg_element::create_with_constructor(s, c, "feConvolveMatrix", owner)?;
    let input = super::svg_animated_string::create(s, "")?;
    let order_x = super::svg_animated_integer::create(s, 3)?;
    let order_y = super::svg_animated_integer::create(s, 3)?;
    let kernel_matrix = super::svg_animated_number_list::create(s)?;
    let divisor = super::svg_animated_number::create(s, 1.0)?;
    let bias = super::svg_animated_number::create(s, 0.0)?;
    let target_x = super::svg_animated_integer::create(s, 0)?;
    let target_y = super::svg_animated_integer::create(s, 0)?;
    let edge_mode = super::svg_animated_enumeration::create(s, EDGE_DUPLICATE as u32)?;
    let kernel_x = super::svg_animated_number::create(s, 0.0)?;
    let kernel_y = super::svg_animated_number::create(s, 0.0)?;
    let preserve_alpha = super::svg_animated_boolean::create(s, false)?;
    let x = super::svg_animated_length::create_with_unit(s, 2, 0.0)?;
    let y = super::svg_animated_length::create_with_unit(s, 2, 0.0)?;
    let width = super::svg_animated_length::create_with_unit(s, 2, 100.0)?;
    let height = super::svg_animated_length::create_with_unit(s, 2, 100.0)?;
    let result = super::svg_animated_string::create(s, "")?;
    let r = Record {
        input: v8::Global::new(s, input),
        order_x: v8::Global::new(s, order_x),
        order_y: v8::Global::new(s, order_y),
        kernel_matrix: v8::Global::new(s, kernel_matrix),
        divisor: v8::Global::new(s, divisor),
        bias: v8::Global::new(s, bias),
        target_x: v8::Global::new(s, target_x),
        target_y: v8::Global::new(s, target_y),
        edge_mode: v8::Global::new(s, edge_mode),
        kernel_x: v8::Global::new(s, kernel_x),
        kernel_y: v8::Global::new(s, kernel_y),
        preserve_alpha: v8::Global::new(s, preserve_alpha),
        x: v8::Global::new(s, x),
        y: v8::Global::new(s, y),
        width: v8::Global::new(s, width),
        height: v8::Global::new(s, height),
        result: v8::Global::new(s, result),
    };
    s.get_slot_mut::<SvgFeConvolveMatrixElementStore>()
        .ok_or_else(|| "SVGFEConvolveMatrixElement state was not prepared".to_owned())?
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
        "Failed to construct 'SVGFEConvolveMatrixElement': Illegal constructor",
    )
}
pub(crate) fn rec(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<Record> {
    s.get_slot::<SvgFeConvolveMatrixElementStore>()?
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
pub(crate) fn get_order_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.order_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_order_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.order_y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_kernel_matrix(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.kernel_matrix, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_divisor(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.divisor, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_bias(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.bias, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_target_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.target_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_target_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.target_y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_edge_mode(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.edge_mode, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_kernel_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.kernel_x, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_kernel_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.kernel_y, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_preserve_alpha(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = rec(s, a.this()) {
        ret(s, &v.preserve_alpha, r)
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
