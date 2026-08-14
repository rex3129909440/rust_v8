use std::collections::HashMap;

#[derive(Clone, Copy, Default)]
pub(crate) enum DomTokenSupport {
    #[default]
    None,
    HyperlinkRel,
    LinkRel,
    Sandbox,
    Blocking,
    MediaControls,
}

#[derive(Default)]
pub(crate) struct DomTokenListStore {
    constructor: crate::webidl::RealmConstructor,
    next_group: u64,
    objects: HashMap<i32, u64>,
    values: HashMap<u64, Vec<String>>,
    raw_values: HashMap<u64, String>,
    bindings: HashMap<i32, (v8::Global<v8::Object>, String)>,
    support: HashMap<i32, DomTokenSupport>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomTokenListStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMTokenList", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<DomTokenListStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMTokenList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::dom_token_list_entries::define(scope, prototype)?;
    super::dom_token_list_keys::define(scope, prototype)?;
    super::dom_token_list_values::define(scope, prototype)?;
    super::dom_token_list_for_each::define(scope, prototype)?;
    super::dom_token_list_length_property::define(scope, prototype)?;
    super::dom_token_list_value_property::define(scope, prototype)?;
    super::dom_token_list_add::define(scope, prototype)?;
    super::dom_token_list_contains::define(scope, prototype)?;
    super::dom_token_list_item::define(scope, prototype)?;
    super::dom_token_list_remove::define(scope, prototype)?;
    super::dom_token_list_replace::define(scope, prototype)?;
    super::dom_token_list_supports::define(scope, prototype)?;
    super::dom_token_list_toggle::define(scope, prototype)?;
    super::dom_token_list_to_string::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_iterator_alias(scope, prototype, "values")?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomTokenListStore>()
        .ok_or_else(|| "DOMTokenList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_with_support(scope, initial, DomTokenSupport::None)
}

pub(crate) fn create_with_support<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: &str,
    support: DomTokenSupport,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = new_exotic_list(scope)?;
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DOMTokenList".to_owned());
    }
    let group = {
        let store = scope
            .get_slot_mut::<DomTokenListStore>()
            .ok_or_else(|| "DOMTokenList state was not prepared".to_owned())?;
        store.next_group += 1;
        let group = store.next_group;
        store.values.insert(group, parse_tokens(initial));
        store.raw_values.insert(group, initial.to_owned());
        store
            .objects
            .insert(object.get_identity_hash().get(), group);
        store
            .support
            .insert(object.get_identity_hash().get(), support);
        group
    };
    let _ = group;
    Ok(object)
}

fn new_exotic_list<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let template = v8::ObjectTemplate::new(scope);
    template.set_indexed_property_handler(
        v8::IndexedPropertyHandlerConfiguration::new()
            .getter(indexed_getter)
            .setter(indexed_setter)
            .query(indexed_query)
            .deleter(indexed_deleter)
            .enumerator(indexed_enumerator),
    );
    template
        .new_instance(scope)
        .ok_or_else(|| "cannot create DOMTokenList exotic object".to_owned())
}

fn indexed_getter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Value>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "get", index, None);
    let Some(value) = list(scope, arguments.holder())
        .and_then(|values| values.get(index as usize).cloned())
        .and_then(|value| v8::String::new(scope, &value))
    else {
        return v8::Intercepted::kNo;
    };
    result.set(value.into());
    v8::Intercepted::kYes
}

fn indexed_setter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    value: v8::Local<'_, v8::Value>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, ()>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "set", index, Some(value));
    if list(scope, arguments.holder()).is_none_or(|values| (index as usize) >= values.len()) {
        return v8::Intercepted::kNo;
    }
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn indexed_query(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Integer>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "has", index, None);
    if list(scope, arguments.holder()).is_some_and(|values| (index as usize) < values.len()) {
        result.set_int32(1);
        v8::Intercepted::kYes
    } else {
        v8::Intercepted::kNo
    }
}

fn indexed_deleter(
    scope: &mut v8::PinScope<'_, '_>,
    index: u32,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Boolean>,
) -> v8::Intercepted {
    crate::trace::record_indexed_native_intercept(scope, &arguments, "delete", index, None);
    if list(scope, arguments.holder()).is_none_or(|values| (index as usize) >= values.len()) {
        return v8::Intercepted::kNo;
    }
    result.set_bool(false);
    v8::Intercepted::kYes
}

fn indexed_enumerator(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::PropertyCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_, v8::Array>,
) {
    crate::trace::record_native_enumeration(scope, &arguments);
    let length = list(scope, arguments.holder()).map_or(0, |values| values.len());
    let indices = (0..length)
        .map(|index| v8::Integer::new_from_unsigned(scope, index as u32).into())
        .collect::<Vec<v8::Local<v8::Value>>>();
    result.set(v8::Array::new_with_elements(scope, &indices));
}

pub(crate) fn create_bound<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: &str,
    element: v8::Local<'_, v8::Object>,
    attribute: &str,
) -> Result<v8::Local<'s, v8::Object>, String> {
    create_bound_with_support(scope, initial, element, attribute, DomTokenSupport::None)
}

pub(crate) fn create_bound_with_support<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    initial: &str,
    element: v8::Local<'_, v8::Object>,
    attribute: &str,
    support: DomTokenSupport,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let list = create_with_support(scope, initial, support)?;
    let binding = (v8::Global::new(scope, element), attribute.to_owned());
    scope
        .get_slot_mut::<DomTokenListStore>()
        .ok_or_else(|| "DOMTokenList state was not prepared".to_owned())?
        .bindings
        .insert(list.get_identity_hash().get(), binding);
    Ok(list)
}

pub(crate) fn support(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DomTokenSupport> {
    scope
        .get_slot::<DomTokenListStore>()?
        .support
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn string_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    let store = scope.get_slot::<DomTokenListStore>()?;
    let group = store.objects.get(&object.get_identity_hash().get())?;
    store.raw_values.get(group).cloned()
}

pub(crate) fn set_string_value(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    value: &str,
) -> bool {
    let Some(store) = scope.get_slot_mut::<DomTokenListStore>() else {
        return false;
    };
    let Some(group) = store
        .objects
        .get(&object.get_identity_hash().get())
        .copied()
    else {
        return false;
    };
    store.values.insert(group, parse_tokens(value));
    store.raw_values.insert(group, value.to_owned());
    true
}

pub(crate) fn sync_binding_for_attribute(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    attribute: &str,
    value: &str,
) {
    let bindings = scope
        .get_slot::<DomTokenListStore>()
        .map(|store| {
            store
                .bindings
                .iter()
                .map(|(list_id, (bound_element, bound_attribute))| {
                    (*list_id, bound_element.clone(), bound_attribute.clone())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let element_id = element.get_identity_hash().get();
    let groups = bindings
        .into_iter()
        .filter_map(|(list_id, bound_element, bound_attribute)| {
            let bound_element = v8::Local::new(scope, &bound_element);
            (bound_element.get_identity_hash().get() == element_id
                && bound_attribute.eq_ignore_ascii_case(attribute))
            .then_some(list_id)
        })
        .filter_map(|list_id| {
            scope
                .get_slot::<DomTokenListStore>()
                .and_then(|store| store.objects.get(&list_id).copied())
        })
        .collect::<Vec<_>>();
    let tokens = parse_tokens(value);
    if let Some(store) = scope.get_slot_mut::<DomTokenListStore>() {
        for group in groups {
            store.values.insert(group, tokens.clone());
            store.raw_values.insert(group, value.to_owned());
        }
    }
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'DOMTokenList': Illegal constructor",
    );
}

fn parse_tokens(value: &str) -> Vec<String> {
    let mut output = Vec::new();
    for token in value.split_ascii_whitespace() {
        if !output.iter().any(|existing| existing == token) {
            output.push(token.to_owned());
        }
    }
    output
}

pub(crate) fn list(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<String>> {
    let store = scope.get_slot::<DomTokenListStore>()?;
    let group = store.objects.get(&object.get_identity_hash().get())?;
    store.values.get(group).cloned()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut Vec<String>),
) -> bool {
    let Some(store) = scope.get_slot_mut::<DomTokenListStore>() else {
        return false;
    };
    let Some(group) = store
        .objects
        .get(&object.get_identity_hash().get())
        .copied()
    else {
        return false;
    };
    let raw_value = {
        let Some(values) = store.values.get_mut(&group) else {
            return false;
        };
        change(values);
        values.join(" ")
    };
    store.raw_values.insert(group, raw_value);
    true
}

pub(crate) fn commit_binding(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let binding = scope
        .get_slot::<DomTokenListStore>()
        .and_then(|store| store.bindings.get(&object.get_identity_hash().get()))
        .cloned();
    let Some((element, attribute)) = binding else {
        return;
    };
    let value = string_value(scope, object).unwrap_or_default();
    super::element::set_attribute_full(
        scope,
        v8::Local::new(scope, &element),
        attribute,
        value,
        None,
    );
}

pub(crate) fn validate_token(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    method: &str,
) -> Option<String> {
    let token = crate::webidl::value_to_string(scope, value);
    if token.is_empty() {
        super::node::throw_dom_exception(
            scope,
            "SyntaxError",
            &format!(
                "Failed to execute '{method}' on 'DOMTokenList': The token provided must not be empty."
            ),
        );
        None
    } else if token
        .chars()
        .any(|character| matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
    {
        super::node::throw_dom_exception(
            scope,
            "InvalidCharacterError",
            &format!(
                "Failed to execute '{method}' on 'DOMTokenList': The token provided ('{token}') contains HTML space characters, which are not valid in tokens."
            ),
        );
        None
    } else {
        Some(token)
    }
}

pub(crate) fn iterator_from_array(
    scope: &mut v8::PinScope<'_, '_>,
    array: v8::Local<'_, v8::Array>,
    method: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(key) = v8::String::new(scope, method) else {
        return;
    };
    let Some(function) = array
        .get(scope, key.into())
        .and_then(|v| v8::Local::<v8::Function>::try_from(v).ok())
    else {
        return;
    };
    if let Some(iterator) = function.call(scope, array.into(), &[]) {
        result.set(iterator)
    }
}
