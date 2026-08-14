use std::collections::{HashMap, HashSet};

#[derive(Clone, Default)]
pub(crate) struct LifecycleCallbacks {
    pub connected: Option<v8::Global<v8::Function>>,
    pub disconnected: Option<v8::Global<v8::Function>>,
    pub adopted: Option<v8::Global<v8::Function>>,
    pub attribute_changed: Option<v8::Global<v8::Function>>,
    pub form_associated: Option<v8::Global<v8::Function>>,
    pub form_reset: Option<v8::Global<v8::Function>>,
    pub form_disabled: Option<v8::Global<v8::Function>>,
    pub form_state_restore: Option<v8::Global<v8::Function>>,
}

#[derive(Clone)]
pub(crate) struct Definition {
    pub name: String,
    pub local_name: String,
    pub constructor: v8::Global<v8::Function>,
    pub prototype: v8::Global<v8::Object>,
    pub callbacks: LifecycleCallbacks,
    pub observed_attributes: Vec<String>,
    pub form_associated: bool,
    pub internals_disabled: bool,
}

#[derive(Clone)]
struct CustomElementState {
    definition: Definition,
    form_owner: Option<v8::Global<v8::Object>>,
}

#[derive(Clone)]
struct ConstructionEntry {
    definition: Definition,
    element: v8::Global<v8::Object>,
    consumed: bool,
}

#[derive(Default)]
pub(crate) struct CustomElementRegistryStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    instances: HashSet<i32>,
    definitions: HashMap<i32, HashMap<String, Definition>>,
    waiters: HashMap<(i32, String), Vec<v8::Global<v8::PromiseResolver>>>,
    candidates: HashMap<i32, v8::Global<v8::Object>>,
    candidate_is: HashMap<i32, String>,
    candidate_registry: HashMap<i32, Option<i32>>,
    registry_override: Vec<Option<i32>>,
    custom_elements: HashMap<i32, CustomElementState>,
    failed_elements: HashSet<i32>,
    construction_stack: Vec<ConstructionEntry>,
    suppress_upgrades: usize,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CustomElementRegistryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CustomElementRegistry", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CustomElementRegistry",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "define", 2, define)?;
    crate::webidl::define_method(scope, prototype, "get", 1, get)?;
    crate::webidl::define_method(scope, prototype, "getName", 1, get_name)?;
    crate::webidl::define_method(scope, prototype, "upgrade", 1, upgrade)?;
    crate::webidl::define_method(scope, prototype, "whenDefined", 1, when_defined)?;
    crate::webidl::define_method(scope, prototype, "initialize", 1, initialize)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .ok_or_else(|| "CustomElementRegistry state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CustomElementRegistry".to_owned());
    }
    attach(scope, object);
    Ok(object)
}

fn attach(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) {
    let identity = object.get_identity_hash().get();
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store.instances.insert(identity);
        store.definitions.entry(identity).or_default();
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'CustomElementRegistry': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    attach(scope, arguments.this());
    result.set(arguments.this().into());
}

fn instance_id(scope: &mut v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<i32> {
    let identity = object.get_identity_hash().get();
    if scope
        .get_slot::<CustomElementRegistryStore>()
        .is_some_and(|store| store.instances.contains(&identity))
    {
        Some(identity)
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        None
    }
}

pub(crate) fn is_registry(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<CustomElementRegistryStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}

fn required_arguments(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    method: &str,
    required: i32,
) -> bool {
    if arguments.length() >= required {
        return true;
    }
    let noun = if required == 1 {
        "argument"
    } else {
        "arguments"
    };
    crate::webidl::throw_type_error(
        scope,
        &format!(
            "Failed to execute '{method}' on 'CustomElementRegistry': {required} {noun} required, but only {} present.",
            arguments.length()
        ),
    );
    false
}

fn valid_custom_element_name(name: &str) -> bool {
    const RESERVED: [&str; 8] = [
        "annotation-xml",
        "color-profile",
        "font-face",
        "font-face-src",
        "font-face-uri",
        "font-face-format",
        "font-face-name",
        "missing-glyph",
    ];
    if RESERVED.contains(&name) || !name.contains('-') {
        return false;
    }
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || (first as u32) >= 0x80) {
        return false;
    }
    chars.all(|ch| {
        ch.is_ascii_lowercase()
            || ch.is_ascii_digit()
            || matches!(ch, '.' | '_' | '-')
            || ch == '\u{00B7}'
            || (ch as u32) >= 0xC0
    })
}

fn throw_invalid_name(scope: &mut v8::PinScope<'_, '_>, method: &str, name: &str) {
    super::node::throw_dom_exception(
        scope,
        "SyntaxError",
        &format!(
            "Failed to execute '{method}' on 'CustomElementRegistry': \"{name}\" is not a valid custom element name"
        ),
    );
}

fn callback_property(
    scope: &mut v8::PinScope<'_, '_>,
    prototype: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<Option<v8::Global<v8::Function>>> {
    let key = v8::String::new(scope, name)?;
    let value = prototype.get(scope, key.into())?;
    if value.is_undefined() {
        return Some(None);
    }
    let Ok(function) = v8::Local::<v8::Function>::try_from(value) else {
        crate::webidl::throw_type_error(scope, &format!("The {name} callback must be a function"));
        return None;
    };
    Some(Some(v8::Global::new(scope, function)))
}

fn definition_options_extends(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<Option<String>> {
    if value.is_undefined() {
        return Some(None);
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        return Some(None);
    };
    let key = v8::String::new(scope, "extends")?;
    let raw = options.get(scope, key.into())?;
    if raw.is_undefined() {
        return Some(None);
    }
    crate::webidl::dom_string_with_context(
        scope,
        raw,
        "Failed to execute 'define' on 'CustomElementRegistry': Failed to read the 'extends' property from 'ElementDefinitionOptions'",
    )
    .map(Some)
}

fn is_known_html_element(name: &str) -> bool {
    matches!(
        name,
        "a" | "abbr"
            | "address"
            | "area"
            | "article"
            | "aside"
            | "audio"
            | "b"
            | "base"
            | "bdi"
            | "bdo"
            | "blockquote"
            | "body"
            | "br"
            | "button"
            | "canvas"
            | "caption"
            | "cite"
            | "code"
            | "col"
            | "colgroup"
            | "data"
            | "datalist"
            | "dd"
            | "del"
            | "details"
            | "dfn"
            | "dialog"
            | "div"
            | "dl"
            | "dt"
            | "em"
            | "embed"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "head"
            | "header"
            | "hgroup"
            | "hr"
            | "html"
            | "i"
            | "iframe"
            | "img"
            | "input"
            | "ins"
            | "kbd"
            | "label"
            | "legend"
            | "li"
            | "link"
            | "main"
            | "map"
            | "mark"
            | "menu"
            | "meta"
            | "meter"
            | "nav"
            | "noscript"
            | "object"
            | "ol"
            | "optgroup"
            | "option"
            | "output"
            | "p"
            | "picture"
            | "pre"
            | "progress"
            | "q"
            | "ruby"
            | "s"
            | "samp"
            | "script"
            | "search"
            | "section"
            | "select"
            | "slot"
            | "small"
            | "source"
            | "span"
            | "strong"
            | "style"
            | "sub"
            | "summary"
            | "sup"
            | "table"
            | "tbody"
            | "td"
            | "template"
            | "textarea"
            | "tfoot"
            | "th"
            | "thead"
            | "time"
            | "title"
            | "tr"
            | "track"
            | "u"
            | "ul"
            | "var"
            | "video"
            | "wbr"
    )
}

fn define(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    if !required_arguments(scope, &arguments, "define", 2) {
        return;
    }
    let Some(name) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'define' on 'CustomElementRegistry'",
    ) else {
        return;
    };
    let Ok(constructor) = v8::Local::<v8::Function>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'define' on 'CustomElementRegistry': parameter 2 is not of type 'Function'.",
        );
        return;
    };
    if !valid_custom_element_name(&name) {
        throw_invalid_name(scope, "define", &name);
        return;
    }
    let Some(extends) = definition_options_extends(scope, arguments.get(2)) else {
        return;
    };
    if let Some(extends) = extends.as_deref() {
        if valid_custom_element_name(extends) {
            super::node::throw_dom_exception(
                scope,
                "NotSupportedError",
                &format!(
                    "Failed to execute 'define' on 'CustomElementRegistry': \"{extends}\" is a valid custom element name"
                ),
            );
            return;
        }
        if !is_known_html_element(extends) {
            super::node::throw_dom_exception(
                scope,
                "NotSupportedError",
                &format!(
                    "Failed to execute 'define' on 'CustomElementRegistry': \"{extends}\" is not a valid name for a custom element"
                ),
            );
            return;
        }
    }
    let duplicate_name = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .is_some_and(|definitions| definitions.contains_key(&name));
    if duplicate_name {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            &format!(
                "Failed to execute 'define' on 'CustomElementRegistry': the name \"{name}\" has already been used with this registry"
            ),
        );
        return;
    }
    let duplicate_constructor = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .is_some_and(|definitions| {
            definitions.values().any(|definition| {
                v8::Local::new(scope, &definition.constructor).strict_equals(constructor.into())
            })
        });
    if duplicate_constructor {
        super::node::throw_dom_exception(
            scope,
            "NotSupportedError",
            "Failed to execute 'define' on 'CustomElementRegistry': this constructor has already been used with this registry",
        );
        return;
    }

    let prototype_key = match v8::String::new(scope, "prototype") {
        Some(key) => key,
        None => return,
    };
    let Some(prototype_value) = constructor.get(scope, prototype_key.into()) else {
        return;
    };
    let Ok(prototype) = v8::Local::<v8::Object>::try_from(prototype_value) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'define' on 'CustomElementRegistry': constructor.prototype is not an object",
        );
        return;
    };
    let Some(connected) = callback_property(scope, prototype, "connectedCallback") else {
        return;
    };
    let Some(disconnected) = callback_property(scope, prototype, "disconnectedCallback") else {
        return;
    };
    let Some(adopted) = callback_property(scope, prototype, "adoptedCallback") else {
        return;
    };
    let Some(attribute_changed) = callback_property(scope, prototype, "attributeChangedCallback")
    else {
        return;
    };

    let mut observed_attributes = Vec::new();
    if attribute_changed.is_some() {
        let key = match v8::String::new(scope, "observedAttributes") {
            Some(key) => key,
            None => return,
        };
        let Some(value) = constructor.get(scope, key.into()) else {
            return;
        };
        if !value.is_undefined() {
            let Some(values) = observed_attribute_sequence(scope, value) else {
                return;
            };
            observed_attributes = values;
        }
    }
    // These two static members are intentionally read even when their values
    // are not currently consumed. Their observable getter order is Web IDL.
    let disabled_features = match v8::String::new(scope, "disabledFeatures") {
        Some(key) => key,
        None => return,
    };
    let Some(disabled_features_value) = constructor.get(scope, disabled_features.into()) else {
        return;
    };
    let internals_disabled = if disabled_features_value.is_undefined() {
        false
    } else if let Some(values) = observed_attribute_sequence(scope, disabled_features_value) {
        values.iter().any(|value| value == "internals")
    } else {
        return;
    };
    let form_associated_key = match v8::String::new(scope, "formAssociated") {
        Some(key) => key,
        None => return,
    };
    let Some(form_associated_value) = constructor.get(scope, form_associated_key.into()) else {
        return;
    };
    let form_associated = form_associated_value.boolean_value(scope);
    let (form_associated_callback, form_reset, form_disabled, form_state_restore) =
        if form_associated {
            let Some(associated) = callback_property(scope, prototype, "formAssociatedCallback")
            else {
                return;
            };
            let Some(reset) = callback_property(scope, prototype, "formResetCallback") else {
                return;
            };
            let Some(disabled) = callback_property(scope, prototype, "formDisabledCallback") else {
                return;
            };
            let Some(restore) = callback_property(scope, prototype, "formStateRestoreCallback")
            else {
                return;
            };
            (associated, reset, disabled, restore)
        } else {
            (None, None, None, None)
        };

    let definition = Definition {
        name: name.clone(),
        local_name: extends.unwrap_or_else(|| name.clone()),
        constructor: v8::Global::new(scope, constructor),
        prototype: v8::Global::new(scope, prototype),
        callbacks: LifecycleCallbacks {
            connected,
            disconnected,
            adopted,
            attribute_changed,
            form_associated: form_associated_callback,
            form_reset,
            form_disabled,
            form_state_restore,
        },
        observed_attributes,
        form_associated,
        internals_disabled,
    };
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .expect("CustomElementRegistry state")
        .definitions
        .entry(identity)
        .or_default()
        .insert(name.clone(), definition);
    let waiters = scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .and_then(|store| store.waiters.remove(&(identity, name.clone())))
        .unwrap_or_default();
    for waiter in waiters {
        let waiter = v8::Local::new(scope, &waiter);
        let _ = waiter.resolve(scope, constructor.into());
    }
    upgrade_connected_candidates(scope, identity, &name);
}

fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    if !required_arguments(scope, &arguments, "get", 1) {
        return;
    }
    let Some(name) = crate::webidl::dom_string_with_context(
        scope,
        arguments.get(0),
        "Failed to execute 'get' on 'CustomElementRegistry'",
    ) else {
        return;
    };
    let constructor = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .and_then(|definitions| definitions.get(&name))
        .map(|definition| definition.constructor.clone());
    match constructor {
        Some(constructor) => result.set(v8::Local::new(scope, &constructor).into()),
        None => result.set(v8::undefined(scope).into()),
    }
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    if !required_arguments(scope, &arguments, "getName", 1) {
        return;
    }
    let Ok(wanted) = v8::Local::<v8::Function>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'getName' on 'CustomElementRegistry': parameter 1 is not of type 'Function'.",
        );
        return;
    };
    let found = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .and_then(|definitions| {
            definitions.values().find_map(|definition| {
                v8::Local::new(scope, &definition.constructor)
                    .strict_equals(wanted.into())
                    .then(|| definition.name.clone())
            })
        });
    match found {
        Some(name) => {
            if let Some(name) = v8::String::new(scope, &name) {
                result.set(name.into());
            }
        }
        None => result.set(v8::null(scope).into()),
    }
}

fn upgrade(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(identity) = instance_id(scope, arguments.this()) else {
        return;
    };
    if !required_arguments(scope, &arguments, "upgrade", 1) {
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'upgrade' on 'CustomElementRegistry': parameter 1 is not of type 'Node'.",
        );
        return;
    };
    if super::node::record(scope, node).is_none() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'upgrade' on 'CustomElementRegistry': parameter 1 is not of type 'Node'.",
        );
        return;
    }
    upgrade_tree_for_registry(scope, node, identity, false);
}

fn rejected_dom_exception_promise<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: &str,
    message: &str,
) -> Option<v8::Local<'s, v8::Promise>> {
    let exception =
        super::dom_exception::create(scope, message.to_owned(), name.to_owned()).ok()?;
    super::writable_stream::rejected_promise(scope, exception.into()).ok()
}

fn when_defined(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let identity = arguments.this().get_identity_hash().get();
    let branded = scope
        .get_slot::<CustomElementRegistryStore>()
        .is_some_and(|store| store.instances.contains(&identity));
    if !branded {
        crate::webidl::reject_illegal_invocation_promise(
            scope,
            "CustomElementRegistry",
            "whenDefined",
            result,
        );
        return;
    }
    if arguments.length() < 1 {
        if let Some(promise) = crate::webidl::rejected_type_error_promise(
            scope,
            "Failed to execute 'whenDefined' on 'CustomElementRegistry': 1 argument required, but only 0 present.",
        ) {
            result.set(promise.into());
        }
        return;
    }
    if arguments.get(0).is_symbol() {
        if let Some(promise) = crate::webidl::rejected_type_error_promise(
            scope,
            "Failed to execute 'whenDefined' on 'CustomElementRegistry': Cannot convert a Symbol value to a string",
        ) {
            result.set(promise.into());
        }
        return;
    }
    let Some(name) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    if !valid_custom_element_name(&name) {
        let message = format!(
            "Failed to execute 'whenDefined' on 'CustomElementRegistry': \"{name}\" is not a valid custom element name"
        );
        if let Some(promise) = rejected_dom_exception_promise(scope, "SyntaxError", &message) {
            result.set(promise.into());
        }
        return;
    }
    let existing = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.definitions.get(&identity))
        .and_then(|definitions| definitions.get(&name))
        .map(|definition| definition.constructor.clone());
    if let Some(existing) = existing {
        if let Ok(promise) =
            super::writable_stream::resolved_promise(scope, v8::Local::new(scope, &existing).into())
        {
            result.set(promise.into());
        }
        return;
    }
    let Some(resolver) = v8::PromiseResolver::new(scope) else {
        crate::webidl::throw_type_error(scope, "cannot create promise");
        return;
    };
    let promise = resolver.get_promise(scope);
    let resolver = v8::Global::new(scope, resolver);
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .expect("CustomElementRegistry state")
        .waiters
        .entry((identity, name))
        .or_default()
        .push(resolver);
    result.set(promise.into());
}

fn initialize(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if instance_id(scope, arguments.this()).is_none() {
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'initialize' on 'CustomElementRegistry': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Ok(node) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'initialize' on 'CustomElementRegistry': parameter 1 is not of type 'Node'.",
        );
        return;
    };
    if super::node::record(scope, node).is_none() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'initialize' on 'CustomElementRegistry': parameter 1 is not of type 'Node'.",
        );
    }
}

pub(crate) fn registry_for_document<'s>(
    scope: &v8::PinScope<'s, '_>,
    document: v8::Local<'_, v8::Object>,
) -> Option<v8::Local<'s, v8::Object>> {
    super::document::stored_value(scope, document, "customElementRegistry")
        .and_then(|value| v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &value)).ok())
}

pub(crate) fn definition_for_name(
    scope: &v8::PinScope<'_, '_>,
    registry: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<Definition> {
    scope
        .get_slot::<CustomElementRegistryStore>()?
        .definitions
        .get(&registry.get_identity_hash().get())?
        .get(name)
        .cloned()
}

fn observed_attribute_sequence(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<Vec<String>> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let iterator_method = object.get(scope, v8::Symbol::get_iterator(scope).into())?;
    let iterator_method = v8::Local::<v8::Function>::try_from(iterator_method).ok()?;
    let iterator = iterator_method.call(scope, object.into(), &[])?;
    let iterator = v8::Local::<v8::Object>::try_from(iterator).ok()?;
    let next_key = v8::String::new(scope, "next")?;
    let next = iterator.get(scope, next_key.into())?;
    let next = v8::Local::<v8::Function>::try_from(next).ok()?;
    let done_key = v8::String::new(scope, "done")?;
    let value_key = v8::String::new(scope, "value")?;
    let mut values = Vec::new();
    loop {
        let step = next.call(scope, iterator.into(), &[])?;
        let step = v8::Local::<v8::Object>::try_from(step).ok()?;
        if step
            .get(scope, done_key.into())
            .is_some_and(|done| done.boolean_value(scope))
        {
            break;
        }
        values.push(crate::webidl::dom_string(
            scope,
            step.get(scope, value_key.into())?,
        )?);
        if values.len() >= 65_536 {
            return None;
        }
    }
    Some(values)
}

pub(crate) fn track_candidate(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) {
    let identity = element.get_identity_hash().get();
    let stored = v8::Global::new(scope, element);
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store.candidates.insert(identity, stored);
        if let Some(registry_id) = store.registry_override.last().copied() {
            store
                .candidate_registry
                .entry(identity)
                .or_insert(registry_id);
        }
    }
}

pub(crate) fn set_candidate_is(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: Option<String>,
) {
    let identity = element.get_identity_hash().get();
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        if let Some(name) = name {
            store.candidate_is.insert(identity, name);
        } else {
            store.candidate_is.remove(&identity);
        }
    }
}

pub(crate) fn candidate_is(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<String> {
    scope
        .get_slot::<CustomElementRegistryStore>()?
        .candidate_is
        .get(&element.get_identity_hash().get())
        .cloned()
}

fn definition_for_candidate(
    scope: &v8::PinScope<'_, '_>,
    registry_id: i32,
    element: v8::Local<'_, v8::Object>,
) -> Option<Definition> {
    let local_name = super::element::record(scope, element)?
        .tag_name
        .rsplit(':')
        .next()?
        .to_ascii_lowercase();
    let store = scope.get_slot::<CustomElementRegistryStore>()?;
    let identity = element.get_identity_hash().get();
    if store.failed_elements.contains(&identity) || store.custom_elements.contains_key(&identity) {
        return None;
    }
    let definitions = store.definitions.get(&registry_id)?;
    if let Some(is_name) = store.candidate_is.get(&identity) {
        return definitions
            .get(is_name)
            .filter(|definition| definition.local_name == local_name)
            .cloned();
    }
    definitions
        .get(&local_name)
        .filter(|definition| definition.local_name == local_name)
        .cloned()
}

fn registry_id_for_element(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    if let Some(registry_id) = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| {
            store
                .candidate_registry
                .get(&element.get_identity_hash().get())
        })
    {
        return *registry_id;
    }
    if let Some(registry_id) = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.registry_override.last())
    {
        return *registry_id;
    }
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(root) = super::shadow_root::record(scope, node) {
            if let Some(registry) = root.registry {
                return Some(v8::Local::new(scope, &registry).get_identity_hash().get());
            }
            if root.registry_is_null {
                return None;
            }
        }
        current = super::node::parent(scope, node)
            .or_else(|| super::shadow_root::host(scope, node).and_then(|_| None));
    }
    let document = super::node::owner_document(scope, element)?;
    registry_for_document(scope, document).map(|registry| registry.get_identity_hash().get())
}

pub(crate) fn registry_id_for_context(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    let mut current = Some(context);
    while let Some(node) = current {
        if let Some(root) = super::shadow_root::record(scope, node) {
            if let Some(registry) = root.registry {
                return Some(v8::Local::new(scope, &registry).get_identity_hash().get());
            }
            if root.registry_is_null {
                return None;
            }
        }
        current = super::node::parent(scope, node);
    }
    let document = if super::document::is_document(scope, context) {
        context
    } else {
        super::node::owner_document(scope, context)?
    };
    registry_for_document(scope, document).map(|registry| registry.get_identity_hash().get())
}

pub(crate) fn scoped_registry_id_for_context(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) -> Option<i32> {
    let mut current = Some(context);
    while let Some(node) = current {
        if let Some(root) = super::shadow_root::record(scope, node)
            && let Some(registry) = root.registry
        {
            return Some(v8::Local::new(scope, &registry).get_identity_hash().get());
        }
        current = super::node::parent(scope, node);
    }
    None
}

pub(crate) fn is_shadow_context(
    scope: &v8::PinScope<'_, '_>,
    context: v8::Local<'_, v8::Object>,
) -> bool {
    let mut current = Some(context);
    while let Some(node) = current {
        if super::shadow_root::record(scope, node).is_some() {
            return true;
        }
        current = super::node::parent(scope, node);
    }
    false
}

pub(crate) fn begin_registry_override(scope: &mut v8::PinScope<'_, '_>, registry_id: Option<i32>) {
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store.registry_override.push(registry_id);
    }
}

pub(crate) fn end_registry_override(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store.registry_override.pop();
    }
}

fn remember_registry(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    registry_id: Option<i32>,
) {
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store
            .candidate_registry
            .entry(element.get_identity_hash().get())
            .or_insert(registry_id);
    }
}

fn upgrade_connected_candidates(scope: &mut v8::PinScope<'_, '_>, registry_id: i32, name: &str) {
    let candidates = scope
        .get_slot::<CustomElementRegistryStore>()
        .map(|store| store.candidates.values().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    for candidate in candidates {
        let candidate = v8::Local::new(scope, &candidate);
        if super::node::is_connected(scope, candidate)
            && registry_id_for_element(scope, candidate) == Some(registry_id)
            && definition_for_candidate(scope, registry_id, candidate)
                .is_some_and(|definition| definition.name == name)
        {
            let _ = upgrade_element(scope, candidate, registry_id, true, false);
        }
    }
}

pub(crate) fn upgrade_tree_for_registry(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
    registry_id: i32,
    connected: bool,
) {
    let _ = upgrade_element(scope, root, registry_id, connected, false);
    for child in super::node::children(scope, root) {
        upgrade_tree_for_registry(scope, child, registry_id, connected);
    }
}

pub(crate) fn try_upgrade(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    connected: bool,
) -> bool {
    let Some(registry_id) = registry_id_for_element(scope, element) else {
        return false;
    };
    remember_registry(scope, element, Some(registry_id));
    if scope
        .get_slot::<CustomElementRegistryStore>()
        .is_some_and(|store| store.suppress_upgrades > 0)
    {
        return false;
    }
    upgrade_element(scope, element, registry_id, connected, false)
}

pub(crate) fn try_construct_created(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> bool {
    let Some(registry_id) = registry_id_for_element(scope, element) else {
        return false;
    };
    remember_registry(scope, element, Some(registry_id));
    upgrade_element(scope, element, registry_id, false, true)
}

pub(crate) fn begin_suppress_upgrades(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store.suppress_upgrades += 1;
    }
}

pub(crate) fn end_suppress_upgrades(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
        store.suppress_upgrades = store.suppress_upgrades.saturating_sub(1);
    }
}

fn upgrade_element(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    registry_id: i32,
    connected: bool,
    enforce_creation_invariants: bool,
) -> bool {
    let Some(definition) = definition_for_candidate(scope, registry_id, element) else {
        return false;
    };
    let prototype = v8::Local::new(scope, &definition.prototype);
    if element.set_prototype(scope, prototype.into()) != Some(true) {
        return false;
    }
    let identity = element.get_identity_hash().get();
    let stored_element = v8::Global::new(scope, element);
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .expect("CustomElementRegistry state")
        .construction_stack
        .push(ConstructionEntry {
            definition: definition.clone(),
            element: stored_element,
            consumed: false,
        });
    let constructor = v8::Local::new(scope, &definition.constructor);
    let (constructed, exception) = {
        let try_catch = std::pin::pin!(v8::TryCatch::new(scope));
        let mut try_catch = try_catch.init();
        let constructed = constructor
            .new_instance(&mut try_catch, &[])
            .map(|value| v8::Global::new(&try_catch, value));
        let exception = try_catch
            .exception()
            .map(|value| v8::Global::new(&try_catch, value));
        (constructed, exception)
    };
    let entry = scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .and_then(|store| store.construction_stack.pop());
    let mut succeeded = constructed.as_ref().is_some_and(|constructed| {
        v8::Local::new(scope, constructed).strict_equals(element.into())
            && entry.is_some_and(|entry| entry.consumed)
    });
    let invariant_error = if succeeded && enforce_creation_invariants {
        if !super::element::attributes_snapshot(scope, element)
            .unwrap_or_default()
            .is_empty()
        {
            Some("The result must not have attributes")
        } else if !super::node::children(scope, element).is_empty() {
            Some("The result must not have children")
        } else if super::node::parent(scope, element).is_some() {
            Some("The result must not have a parent")
        } else {
            None
        }
    } else {
        None
    };
    succeeded &= invariant_error.is_none();
    if !succeeded {
        if let Some(store) = scope.get_slot_mut::<CustomElementRegistryStore>() {
            store.failed_elements.insert(identity);
        }
        if let Some(exception) = exception {
            report_upgrade_exception(scope, v8::Local::new(scope, &exception));
        } else if let Some(message) = invariant_error
            && let Ok(exception) = super::dom_exception::create(
                scope,
                message.to_owned(),
                "NotSupportedError".to_owned(),
            )
        {
            report_upgrade_exception(scope, exception.into());
        }
        return false;
    }
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .expect("CustomElementRegistry state")
        .custom_elements
        .insert(
            identity,
            CustomElementState {
                definition: definition.clone(),
                form_owner: None,
            },
        );
    for attribute in super::element::attributes_snapshot(scope, element).unwrap_or_default() {
        if definition
            .observed_attributes
            .iter()
            .any(|name| name == &attribute.name)
        {
            invoke_attribute_changed(
                scope,
                element,
                &definition,
                &attribute.name,
                None,
                Some(&attribute.value),
                attribute.namespace_uri.as_deref(),
            );
        }
    }
    if connected {
        invoke_callback(scope, element, definition.callbacks.connected.as_ref(), &[]);
    }
    true
}

pub(crate) fn is_failed(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<CustomElementRegistryStore>()
        .is_some_and(|store| {
            store
                .failed_elements
                .contains(&element.get_identity_hash().get())
        })
}

fn report_upgrade_exception(scope: &mut v8::PinScope<'_, '_>, exception: v8::Local<'_, v8::Value>) {
    let description = exception
        .to_string(scope)
        .map(|value| value.to_rust_string_lossy(scope))
        .unwrap_or_else(|| "Error".to_owned());
    let Ok(event) =
        super::error_event::create(scope, "error", format!("Uncaught {description}"), exception)
    else {
        return;
    };
    let global = scope.get_current_context().global(scope);
    super::event_target::dispatch(scope, global, event);
}

fn definition_by_constructor(
    scope: &v8::PinScope<'_, '_>,
    constructor: v8::Local<'_, v8::Function>,
) -> Option<Definition> {
    let store = scope.get_slot::<CustomElementRegistryStore>()?;
    for definitions in store.definitions.values() {
        for definition in definitions.values() {
            if v8::Local::new(scope, &definition.constructor).strict_equals(constructor.into()) {
                return Some(definition.clone());
            }
        }
    }
    None
}

pub(crate) fn html_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    expected_local_name: Option<&str>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Illegal constructor");
        return;
    }
    let Ok(new_target) = v8::Local::<v8::Function>::try_from(arguments.new_target()) else {
        crate::webidl::throw_type_error(scope, "Illegal constructor");
        return;
    };
    let stack_match = scope
        .get_slot::<CustomElementRegistryStore>()
        .and_then(|store| store.construction_stack.last())
        .is_some_and(|entry| {
            v8::Local::new(scope, &entry.definition.constructor).strict_equals(new_target.into())
        });
    if stack_match {
        let element = scope
            .get_slot::<CustomElementRegistryStore>()
            .and_then(|store| store.construction_stack.last())
            .map(|entry| entry.element.clone());
        let already_consumed = scope
            .get_slot::<CustomElementRegistryStore>()
            .and_then(|store| store.construction_stack.last())
            .is_some_and(|entry| entry.consumed);
        if already_consumed {
            crate::webidl::throw_type_error(scope, "Illegal constructor");
            return;
        }
        if let Some(entry) = scope
            .get_slot_mut::<CustomElementRegistryStore>()
            .and_then(|store| store.construction_stack.last_mut())
        {
            entry.consumed = true;
        }
        if let Some(element) = element {
            result.set(v8::Local::new(scope, &element).into());
            return;
        }
    }

    let Some(definition) = definition_by_constructor(scope, new_target) else {
        crate::webidl::throw_type_error(scope, "Illegal constructor");
        return;
    };
    if expected_local_name.is_some_and(|expected| definition.local_name != expected) {
        crate::webidl::throw_type_error(scope, "Illegal constructor");
        return;
    }
    let element = if definition.local_name == definition.name {
        super::html_element::create(scope, &definition.local_name)
    } else if definition.local_name == "button" {
        super::html_button_element::create(scope)
    } else {
        Err("unsupported customized built-in element".to_owned())
    };
    let Ok(element) = element else {
        crate::webidl::throw_type_error(scope, "Illegal constructor");
        return;
    };
    if let Some(document) = super::document_global::value(scope) {
        super::node::set_owner_document(scope, element, document);
    }
    let prototype = v8::Local::new(scope, &definition.prototype);
    let _ = element.set_prototype(scope, prototype.into());
    let identity = element.get_identity_hash().get();
    track_candidate(scope, element);
    if definition.local_name != definition.name {
        set_candidate_is(scope, element, Some(definition.name.clone()));
    }
    scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .expect("CustomElementRegistry state")
        .custom_elements
        .insert(
            identity,
            CustomElementState {
                definition,
                form_owner: None,
            },
        );
    result.set(element.into());
}

fn custom_state(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> Option<CustomElementState> {
    scope
        .get_slot::<CustomElementRegistryStore>()?
        .custom_elements
        .get(&element.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_form_associated(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> bool {
    custom_state(scope, element).is_some_and(|state| state.definition.form_associated)
}

pub(crate) fn internals_disabled(
    scope: &v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) -> bool {
    custom_state(scope, element).is_some_and(|state| state.definition.internals_disabled)
}

pub(crate) fn notify_form_associated(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    form: Option<v8::Local<'_, v8::Object>>,
) {
    let Some(state) = custom_state(scope, element) else {
        return;
    };
    if !state.definition.form_associated {
        return;
    }
    let same_owner = match (state.form_owner.as_ref(), form) {
        (None, None) => true,
        (Some(previous), Some(current)) => {
            v8::Local::new(scope, previous).strict_equals(current.into())
        }
        _ => false,
    };
    if same_owner {
        return;
    }
    let stored_form = form.map(|value| v8::Global::new(scope, value));
    if let Some(current) = scope
        .get_slot_mut::<CustomElementRegistryStore>()
        .and_then(|store| {
            store
                .custom_elements
                .get_mut(&element.get_identity_hash().get())
        })
    {
        current.form_owner = stored_form;
    }
    let argument = form
        .map(Into::into)
        .unwrap_or_else(|| v8::null(scope).into());
    super::element_internals::update_form_owner(scope, element, form);
    invoke_callback(
        scope,
        element,
        state.definition.callbacks.form_associated.as_ref(),
        &[argument],
    );
}

pub(crate) fn notify_form_reset(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) {
    let Some(state) = custom_state(scope, element) else {
        return;
    };
    invoke_callback(
        scope,
        element,
        state.definition.callbacks.form_reset.as_ref(),
        &[],
    );
}

pub(crate) fn notify_form_disabled(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    disabled: bool,
) {
    let Some(state) = custom_state(scope, element) else {
        return;
    };
    let value: v8::Local<v8::Value> = v8::Boolean::new(scope, disabled).into();
    invoke_callback(
        scope,
        element,
        state.definition.callbacks.form_disabled.as_ref(),
        &[value],
    );
}

pub(crate) fn notify_form_connected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if is_form_associated(scope, root) {
        let owner = super::html_form_element::ancestor_form(scope, root);
        if owner.is_some() {
            notify_form_associated(scope, root, owner);
        }
    }
    for child in super::node::children(scope, root) {
        notify_form_connected_tree(scope, child);
    }
}

pub(crate) fn refresh_all_form_owners(scope: &mut v8::PinScope<'_, '_>) {
    let elements = scope
        .get_slot::<CustomElementRegistryStore>()
        .map(|store| {
            store
                .custom_elements
                .keys()
                .filter_map(|identity| store.candidates.get(identity).cloned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for element in elements {
        let element = v8::Local::new(scope, &element);
        if is_form_associated(scope, element) {
            let owner = super::html_form_element::ancestor_form(scope, element);
            notify_form_associated(scope, element, owner);
        }
    }
}

pub(crate) fn notify_form_disconnecting_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if is_form_associated(scope, root)
        && super::html_form_element::ancestor_form(scope, root).is_some()
    {
        notify_form_associated(scope, root, None);
    }
    for child in super::node::children(scope, root) {
        notify_form_disconnecting_tree(scope, child);
    }
}

pub(crate) fn form_attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    previous: Option<v8::Local<'_, v8::Object>>,
) {
    if !is_form_associated(scope, element) {
        return;
    }
    let current = super::html_form_element::ancestor_form(scope, element);
    let _ = previous;
    notify_form_associated(scope, element, current);
}

pub(crate) fn notify_disabled_subtree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
    disabled: bool,
) {
    if is_form_associated(scope, root) {
        notify_form_disabled(scope, root, disabled);
    }
    for child in super::node::children(scope, root) {
        notify_disabled_subtree(scope, child, disabled);
    }
}

pub(crate) fn is_custom(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    custom_state(scope, element).is_some()
        || scope
            .get_slot::<CustomElementRegistryStore>()
            .and_then(|store| store.construction_stack.last())
            .is_some_and(|entry| {
                v8::Local::new(scope, &entry.element)
                    .get_identity_hash()
                    .get()
                    == element.get_identity_hash().get()
            })
}

fn invoke_callback(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    callback: Option<&v8::Global<v8::Function>>,
    arguments: &[v8::Local<'_, v8::Value>],
) {
    let Some(callback) = callback else {
        return;
    };
    let callback = v8::Local::new(scope, callback);
    let _ = callback.call(scope, element.into(), arguments);
}

fn nullable_string<'s>(
    scope: &v8::PinScope<'s, '_>,
    value: Option<&str>,
) -> v8::Local<'s, v8::Value> {
    value
        .and_then(|value| v8::String::new(scope, value))
        .map(Into::into)
        .unwrap_or_else(|| v8::null(scope).into())
}

fn invoke_attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    definition: &Definition,
    name: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    namespace: Option<&str>,
) {
    let Some(name_value) = v8::String::new(scope, name) else {
        return;
    };
    let old_value = nullable_string(scope, old_value);
    let new_value = nullable_string(scope, new_value);
    let namespace = nullable_string(scope, namespace);
    invoke_callback(
        scope,
        element,
        definition.callbacks.attribute_changed.as_ref(),
        &[name_value.into(), old_value, new_value, namespace],
    );
}

pub(crate) fn attribute_changed(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    name: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    namespace: Option<&str>,
) {
    let Some(state) = custom_state(scope, element) else {
        return;
    };
    if state
        .definition
        .observed_attributes
        .iter()
        .any(|observed| observed == name)
    {
        invoke_attribute_changed(
            scope,
            element,
            &state.definition,
            name,
            old_value,
            new_value,
            namespace,
        );
    }
}

pub(crate) fn notify_connected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if !try_upgrade(scope, root, true) {
        if let Some(state) = custom_state(scope, root) {
            invoke_callback(
                scope,
                root,
                state.definition.callbacks.connected.as_ref(),
                &[],
            );
        }
    }
    for child in super::node::children(scope, root) {
        notify_connected_tree(scope, child);
    }
    if let Some(shadow_root) = super::element::record(scope, root)
        .and_then(|record| record.shadow_root)
        .map(|shadow_root| v8::Local::new(scope, &shadow_root))
    {
        notify_connected_tree(scope, shadow_root);
    }
}

pub(crate) fn notify_disconnected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if let Some(state) = custom_state(scope, root) {
        invoke_callback(
            scope,
            root,
            state.definition.callbacks.disconnected.as_ref(),
            &[],
        );
    }
    for child in super::node::children(scope, root) {
        notify_disconnected_tree(scope, child);
    }
    if let Some(shadow_root) = super::element::record(scope, root)
        .and_then(|record| record.shadow_root)
        .map(|shadow_root| v8::Local::new(scope, &shadow_root))
    {
        notify_disconnected_tree(scope, shadow_root);
    }
}

pub(crate) fn notify_adopted_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
    old_document: v8::Local<'_, v8::Object>,
    new_document: v8::Local<'_, v8::Object>,
) {
    if let Some(state) = custom_state(scope, root) {
        invoke_callback(
            scope,
            root,
            state.definition.callbacks.adopted.as_ref(),
            &[old_document.into(), new_document.into()],
        );
    }
    for child in super::node::children(scope, root) {
        notify_adopted_tree(scope, child, old_document, new_document);
    }
    if let Some(shadow_root) = super::element::record(scope, root)
        .and_then(|record| record.shadow_root)
        .map(|shadow_root| v8::Local::new(scope, &shadow_root))
    {
        notify_adopted_tree(scope, shadow_root, old_document, new_document);
    }
}
