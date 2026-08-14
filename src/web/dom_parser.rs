use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct DomParserStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomParserStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMParser", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DomParserStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMParser",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::dom_parser_parse_from_string::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomParserStore>()
        .ok_or_else(|| "DOMParser state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'DOMParser': Please use the 'new' operator",
        );
        return;
    }
    if let Some(store) = scope.get_slot_mut::<DomParserStore>() {
        store
            .instances
            .insert(arguments.this().get_identity_hash().get());
    }
    result.set(arguments.this().into());
}

pub(crate) fn require_instance(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    if scope
        .get_slot::<DomParserStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
    {
        true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        false
    }
}

pub(crate) fn html_source(source: &str) -> String {
    source.to_owned()
}

pub(crate) fn html_title(source: &str) -> Option<String> {
    let lower = source.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let opening_end = lower[start..].find('>')? + start + 1;
    let closing = lower[opening_end..].find("</title>")? + opening_end;
    Some(source[opening_end..closing].to_owned())
}
