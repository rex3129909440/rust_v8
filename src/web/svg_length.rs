use std::collections::HashMap;

const UNKNOWN: i32 = 0;
const NUMBER: i32 = 1;
const PERCENTAGE: i32 = 2;
const EMS: i32 = 3;
const EXS: i32 = 4;
const PX: i32 = 5;
const CM: i32 = 6;
const MM: i32 = 7;
const IN: i32 = 8;
const PT: i32 = 9;
const PC: i32 = 10;

#[derive(Default)]
pub(crate) struct SvgLengthStore {
    constructor: crate::webidl::RealmConstructor,
    next_group: u64,
    objects: HashMap<i32, u64>,
    values: HashMap<u64, LengthValue>,
}

#[derive(Clone, Copy)]
struct LengthValue {
    unit: i32,
    specified: f64,
}

#[derive(Clone, Copy)]
pub(crate) struct LengthSnapshot {
    pub unit: i32,
    pub specified: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SvgLengthStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SVGLength", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<SvgLengthStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SVGLength",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "unitType", get_unit_type)?;
    crate::webidl::define_accessor(scope, prototype, "value", get_value, set_value)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "valueInSpecifiedUnits",
        get_value_in_specified_units,
        set_value_in_specified_units,
    )?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "valueAsString",
        get_value_as_string,
        set_value_as_string,
    )?;
    define_constants(scope, prototype)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "convertToSpecifiedUnits",
        1,
        convert_to_specified_units,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "newValueSpecifiedUnits",
        2,
        new_value_specified_units,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SvgLengthStore>()
        .ok_or_else(|| "SVGLength state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn define_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_UNKNOWN", UNKNOWN)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_NUMBER", NUMBER)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_PERCENTAGE", PERCENTAGE)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_EMS", EMS)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_EXS", EXS)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_PX", PX)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_CM", CM)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_MM", MM)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_IN", IN)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_PT", PT)?;
    crate::webidl::define_constant(scope, object, "SVG_LENGTHTYPE_PC", PC)
}

pub(crate) fn create_pair<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: f64,
) -> Result<(v8::Local<'s, v8::Object>, v8::Local<'s, v8::Object>), String> {
    create_pair_with_unit(scope, NUMBER, initial)
}

pub(crate) fn create_pair_with_unit<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    unit: i32,
    initial: f64,
) -> Result<(v8::Local<'s, v8::Object>, v8::Local<'s, v8::Object>), String> {
    let group = {
        let store = scope
            .get_slot_mut::<SvgLengthStore>()
            .ok_or_else(|| "SVGLength state was not prepared".to_owned())?;
        store.next_group += 1;
        let group = store.next_group;
        store.values.insert(
            group,
            LengthValue {
                unit,
                specified: initial,
            },
        );
        group
    };
    let base = create_for_group(scope, group)?;
    let animated = create_for_group(scope, group)?;
    Ok((base, animated))
}

pub(crate) fn create_single<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    snapshot: LengthSnapshot,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let group = {
        let store = scope
            .get_slot_mut::<SvgLengthStore>()
            .ok_or_else(|| "SVGLength state was not prepared".to_owned())?;
        store.next_group += 1;
        let group = store.next_group;
        store.values.insert(
            group,
            LengthValue {
                unit: snapshot.unit,
                specified: snapshot.specified,
            },
        );
        group
    };
    create_for_group(scope, group)
}

pub(crate) fn snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<LengthSnapshot> {
    record(scope, object).map(|value| LengthSnapshot {
        unit: value.unit,
        specified: value.specified,
    })
}

fn create_for_group<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    group: u64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SVGLength".to_owned());
    }
    scope
        .get_slot_mut::<SvgLengthStore>()
        .ok_or_else(|| "SVGLength state was not prepared".to_owned())?
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
        "Failed to construct 'SVGLength': Illegal constructor",
    );
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<LengthValue> {
    let store = scope.get_slot::<SvgLengthStore>()?;
    let group = store.objects.get(&object.get_identity_hash().get())?;
    store.values.get(group).copied()
}

fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut LengthValue),
) {
    let Some(store) = scope.get_slot_mut::<SvgLengthStore>() else {
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

fn factor(unit: i32) -> f64 {
    match unit {
        CM => 96.0 / 2.54,
        MM => 96.0 / 25.4,
        IN => 96.0,
        PT => 96.0 / 72.0,
        PC => 16.0,
        EMS => 16.0,
        EXS => 8.0,
        _ => 1.0,
    }
}

fn suffix(unit: i32) -> &'static str {
    match unit {
        PERCENTAGE => "%",
        EMS => "em",
        EXS => "ex",
        PX => "px",
        CM => "cm",
        MM => "mm",
        IN => "in",
        PT => "pt",
        PC => "pc",
        _ => "",
    }
}

fn parse_length(source: &str) -> Option<LengthValue> {
    let trimmed = source.trim();
    let choices = [
        ("%", PERCENTAGE),
        ("em", EMS),
        ("ex", EXS),
        ("px", PX),
        ("cm", CM),
        ("mm", MM),
        ("in", IN),
        ("pt", PT),
        ("pc", PC),
    ];
    for (suffix, unit) in choices {
        if let Some(number) = trimmed.strip_suffix(suffix) {
            return Some(LengthValue {
                unit,
                specified: number.trim().parse().ok()?,
            });
        }
    }
    Some(LengthValue {
        unit: NUMBER,
        specified: trimmed.parse().ok()?,
    })
}

fn get_unit_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, value.unit).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, value.specified * factor(value.unit)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_value(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |current| {
        current.specified = value / factor(current.unit)
    });
}

fn get_value_in_specified_units(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, value.specified).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_value_in_specified_units(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = arguments.get(0).number_value(scope).unwrap_or(f64::NAN);
    update(scope, arguments.this(), |current| current.specified = value);
}

fn get_value_as_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(value) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let text = format!("{}{}", value.specified, suffix(value.unit));
    if let Some(text) = v8::String::new(scope, &text) {
        result.set(text.into());
    }
}

fn set_value_as_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let source = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(parsed) = parse_length(&source) else {
        crate::webidl::throw_type_error(scope, "Invalid SVG length");
        return;
    };
    update(scope, arguments.this(), |current| *current = parsed);
}

fn convert_to_specified_units(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let unit = arguments.get(0).int32_value(scope).unwrap_or(UNKNOWN);
    if !(NUMBER..=PC).contains(&unit) {
        crate::webidl::throw_type_error(scope, "Invalid SVG length unit");
        return;
    }
    update(scope, arguments.this(), |current| {
        let absolute = current.specified * factor(current.unit);
        current.unit = unit;
        current.specified = absolute / factor(unit);
    });
}

fn new_value_specified_units(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let unit = arguments.get(0).int32_value(scope).unwrap_or(UNKNOWN);
    let value = arguments.get(1).number_value(scope).unwrap_or(f64::NAN);
    if !(NUMBER..=PC).contains(&unit) {
        crate::webidl::throw_type_error(scope, "Invalid SVG length unit");
        return;
    }
    update(scope, arguments.this(), |current| {
        current.unit = unit;
        current.specified = value;
    });
}
