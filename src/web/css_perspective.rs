use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CssPerspectiveStore {
    constructor: crate::webidl::RealmConstructor,
    lengths: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CssPerspectiveStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CSSPerspective", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CssPerspectiveStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CSSPerspective",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "length", get_length, set_length)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::css_transform_component::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CssPerspectiveStore>()
        .ok_or_else(|| "CSSPerspective state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn length_value(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<v8::Global<v8::Object>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let record = super::css_unit_value::record(scope, object)?;
    if record.value <= 0.0
        || !matches!(
            record.unit.as_str(),
            "px" | "cm" | "mm" | "in" | "pt" | "pc"
        )
    {
        crate::webidl::throw_type_error(scope, "CSSPerspective requires a positive length");
        return None;
    }
    Some(v8::Global::new(scope, object))
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "CSSPerspective requires a length");
        return;
    }
    if arguments.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSPerspective': Must pass length or none to CSSPerspective",
        );
        return;
    }
    let valid_length = v8::Local::<v8::Object>::try_from(arguments.get(0))
        .ok()
        .is_some_and(|object| super::css_numeric_value::is_numeric(scope, object));
    if !valid_length {
        if crate::webidl::dom_string(scope, arguments.get(0)).is_none() {
            return;
        }
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CSSPerspective': Must pass length or none to CSSPerspective",
        );
        return;
    }
    let Some(length) = length_value(scope, arguments.get(0)) else {
        return;
    };
    scope
        .get_slot_mut::<CssPerspectiveStore>()
        .expect("CSSPerspective state")
        .lengths
        .insert(arguments.this().get_identity_hash().get(), length);
    super::css_transform_component::attach(scope, arguments.this(), false);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<v8::Global<v8::Object>> {
    scope
        .get_slot::<CssPerspectiveStore>()?
        .lengths
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(length) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &length).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Some(length) = length_value(scope, arguments.get(0)) else {
        return;
    };
    if let Some(current) = scope
        .get_slot_mut::<CssPerspectiveStore>()
        .and_then(|store| {
            store
                .lengths
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        *current = length;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn serialize(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let length = record(scope, object)?;
    let length = super::css_unit_value::record(scope, v8::Local::new(scope, &length))?;
    Some(format!(
        "perspective({})",
        super::css_unit_value::serialize(&length)
    ))
}

pub(crate) fn matrix(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<[f64; 16]> {
    let length = record(scope, object)?;
    let length = super::css_unit_value::record(scope, v8::Local::new(scope, &length))?;
    let mut matrix = super::dom_matrix::identity();
    matrix[11] = -1.0 / length.value;
    Some(matrix)
}
