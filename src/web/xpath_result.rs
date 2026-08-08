use std::collections::HashMap;

pub(crate) const ANY_TYPE: i32 = 0;
pub(crate) const NUMBER_TYPE: i32 = 1;
pub(crate) const STRING_TYPE: i32 = 2;
pub(crate) const BOOLEAN_TYPE: i32 = 3;
pub(crate) const UNORDERED_NODE_ITERATOR_TYPE: i32 = 4;
pub(crate) const ORDERED_NODE_ITERATOR_TYPE: i32 = 5;
pub(crate) const UNORDERED_NODE_SNAPSHOT_TYPE: i32 = 6;
pub(crate) const ORDERED_NODE_SNAPSHOT_TYPE: i32 = 7;
pub(crate) const ANY_UNORDERED_NODE_TYPE: i32 = 8;
pub(crate) const FIRST_ORDERED_NODE_TYPE: i32 = 9;

#[derive(Default)]
pub(crate) struct XPathResultStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, XPathResultRecord>,
}

#[derive(Clone)]
pub(crate) enum XPathPayload {
    Number(f64),
    String(String),
    Boolean(bool),
    Nodes(Vec<v8::Global<v8::Object>>),
}

#[derive(Clone)]
pub(crate) struct XPathResultRecord {
    pub(crate) result_type: i32,
    pub(crate) payload: XPathPayload,
    pub(crate) iterator_index: usize,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XPathResultStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XPathResult", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<XPathResultStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XPathResult",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::xpath_result_result_type_property::define(scope, prototype)?;
    super::xpath_result_number_value_property::define(scope, prototype)?;
    super::xpath_result_string_value_property::define(scope, prototype)?;
    super::xpath_result_boolean_value_property::define(scope, prototype)?;
    super::xpath_result_single_node_value_property::define(scope, prototype)?;
    super::xpath_result_invalid_iterator_state_property::define(scope, prototype)?;
    super::xpath_result_snapshot_length_property::define(scope, prototype)?;
    define_result_constants(scope, prototype)?;
    super::xpath_result_iterate_next::define(scope, prototype)?;
    super::xpath_result_snapshot_item::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_result_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XPathResultStore>()
        .ok_or_else(|| "XPathResult state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn define_result_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "ANY_TYPE", ANY_TYPE)?;
    crate::webidl::define_constant(scope, object, "NUMBER_TYPE", NUMBER_TYPE)?;
    crate::webidl::define_constant(scope, object, "STRING_TYPE", STRING_TYPE)?;
    crate::webidl::define_constant(scope, object, "BOOLEAN_TYPE", BOOLEAN_TYPE)?;
    crate::webidl::define_constant(
        scope,
        object,
        "UNORDERED_NODE_ITERATOR_TYPE",
        UNORDERED_NODE_ITERATOR_TYPE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "ORDERED_NODE_ITERATOR_TYPE",
        ORDERED_NODE_ITERATOR_TYPE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "UNORDERED_NODE_SNAPSHOT_TYPE",
        UNORDERED_NODE_SNAPSHOT_TYPE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "ORDERED_NODE_SNAPSHOT_TYPE",
        ORDERED_NODE_SNAPSHOT_TYPE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "ANY_UNORDERED_NODE_TYPE",
        ANY_UNORDERED_NODE_TYPE,
    )?;
    crate::webidl::define_constant(
        scope,
        object,
        "FIRST_ORDERED_NODE_TYPE",
        FIRST_ORDERED_NODE_TYPE,
    )
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    requested_type: i32,
    payload: XPathPayload,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let result_type = if requested_type == ANY_TYPE {
        match &payload {
            XPathPayload::Number(_) => NUMBER_TYPE,
            XPathPayload::String(_) => STRING_TYPE,
            XPathPayload::Boolean(_) => BOOLEAN_TYPE,
            XPathPayload::Nodes(_) => UNORDERED_NODE_ITERATOR_TYPE,
        }
    } else {
        requested_type
    };
    if !(ANY_TYPE..=FIRST_ORDERED_NODE_TYPE).contains(&result_type) {
        return Err("The result type provided is not a valid XPathResult type".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XPathResult".to_owned());
    }
    scope
        .get_slot_mut::<XPathResultStore>()
        .ok_or_else(|| "XPathResult state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            XPathResultRecord {
                result_type,
                payload,
                iterator_index: 0,
            },
        );
    Ok(object)
}

pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'XPathResult': Illegal constructor",
    );
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<XPathResultRecord> {
    scope
        .get_slot::<XPathResultStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn require_record(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<XPathResultRecord> {
    let value = record(scope, object);
    if value.is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
    value
}

pub(crate) fn wrong_result_type(scope: &v8::PinScope<'_, '_>, property: &str) {
    crate::webidl::throw_type_error(
        scope,
        &format!("{property} is not available for this XPathResult type"),
    );
}
