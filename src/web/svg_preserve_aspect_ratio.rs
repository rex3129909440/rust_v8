use std::collections::HashMap;

const UNKNOWN: i32 = 0;
const NONE: i32 = 1;
const XMINYMIN: i32 = 2;
const XMIDYMIN: i32 = 3;
const XMAXYMIN: i32 = 4;
const XMINYMID: i32 = 5;
const XMIDYMID: i32 = 6;
const XMAXYMID: i32 = 7;
const XMINYMAX: i32 = 8;
const XMIDYMAX: i32 = 9;
const XMAXYMAX: i32 = 10;
const MEET_OR_SLICE_UNKNOWN: i32 = 0;
const MEET: i32 = 1;
const SLICE: i32 = 2;

#[derive(Default)]
pub(crate) struct SvgPreserveAspectRatioStore {
    constructor: crate::webidl::RealmConstructor,
    next_group: u64,
    objects: HashMap<i32, u64>,
    values: HashMap<u64, PreserveAspectRatioValue>,
}

#[derive(Clone, Copy)]
pub(crate) struct PreserveAspectRatioValue {
    pub align: i32,
    pub meet_or_slice: i32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgPreserveAspectRatioStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGPreserveAspectRatio", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgPreserveAspectRatioStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGPreserveAspectRatio",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "align", get_align, set_align)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "meetOrSlice",
        get_meet_or_slice,
        set_meet_or_slice,
    )?;
    define_constants(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgPreserveAspectRatioStore>()
        .ok_or_else(|| "SVGPreserveAspectRatio state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_UNKNOWN", UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_NONE", NONE)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMINYMIN", XMINYMIN)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMIDYMIN", XMIDYMIN)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMAXYMIN", XMAXYMIN)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMINYMID", XMINYMID)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMIDYMID", XMIDYMID)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMAXYMID", XMAXYMID)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMINYMAX", XMINYMAX)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMIDYMAX", XMIDYMAX)?;
    crate::webidl::define_constant(scope, object, "SVG_PRESERVEASPECTRATIO_XMAXYMAX", XMAXYMAX)?;
    crate::webidl::define_constant(
        scope,
        object,
        "SVG_MEETORSLICE_UNKNOWN",
        MEET_OR_SLICE_UNKNOWN,
    )?;
    crate::webidl::define_constant(scope, object, "SVG_MEETORSLICE_MEET", MEET)?;
    crate::webidl::define_constant(scope, object, "SVG_MEETORSLICE_SLICE", SLICE)
}

pub(crate) fn create_pair<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: PreserveAspectRatioValue,
) -> Result<(v8::Local<'s, v8::Object>, v8::Local<'s, v8::Object>), String> {
    let group = {
        let store = scope
            .get_slot_mut::<SvgPreserveAspectRatioStore>()
            .ok_or_else(|| "SVGPreserveAspectRatio state was not prepared".to_owned())?;
        store.next_group += 1;
        let group = store.next_group;
        store.values.insert(group, value);
        group
    };
    let base = create_for_group(scope, group)?;
    let animated = create_for_group(scope, group)?;
    Ok((base, animated))
}

fn create_for_group<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    group: u64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGPreserveAspectRatio".to_owned());
    }
    scope
        .get_slot_mut::<SvgPreserveAspectRatioStore>()
        .ok_or_else(|| "SVGPreserveAspectRatio state was not prepared".to_owned())?
        .objects
        .insert(object.get_identity_hash().get(), group);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SVGPreserveAspectRatio': Illegal constructor",
    );
}

fn value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PreserveAspectRatioValue> {
    let store = scope.get_slot::<SvgPreserveAspectRatioStore>()?;
    let group = store.objects.get(&object.get_identity_hash().get())?;
    store.values.get(group).copied()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut PreserveAspectRatioValue),
) {
    let Some(store) = scope.get_slot_mut::<SvgPreserveAspectRatioStore>() else {
        return;
    };
    let Some(group) = store
        .objects
        .get(&object.get_identity_hash().get())
        .copied()
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = store.values.get_mut(&group) {
        change(value);
    }
}

fn get_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, value.align).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_align(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).int32_value(scope).unwrap_or(UNKNOWN);
    if !(UNKNOWN..=XMAXYMAX).contains(&value) {
        crate::webidl::throw_type_error(scope, "Invalid SVG preserveAspectRatio alignment");
        return;
    }
    update(scope, arguments.this(), |current| current.align = value);
}

fn get_meet_or_slice(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, value.meet_or_slice).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_meet_or_slice(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).int32_value(scope).unwrap_or(MEET);
    if !(MEET_OR_SLICE_UNKNOWN..=SLICE).contains(&value) {
        crate::webidl::throw_type_error(scope, "Invalid SVG meetOrSlice value");
        return;
    }
    update(scope, arguments.this(), |current| {
        current.meet_or_slice = value
    });
}
