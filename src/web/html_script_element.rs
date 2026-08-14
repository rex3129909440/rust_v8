use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub(crate) struct HtmlScriptElementStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, ScriptRecord>,
    pending: VecDeque<PendingScript>,
    module_urls: HashMap<i32, String>,
    modules: HashMap<String, v8::Global<v8::Module>>,
}

struct PendingScript {
    context: v8::Global<v8::Context>,
    element: v8::Global<v8::Object>,
    source: Option<String>,
    url: String,
    module: bool,
}
#[derive(Clone)]
pub(crate) struct ScriptRecord {
    pub(crate) strings: HashMap<String, String>,
    pub(crate) booleans: HashMap<String, bool>,
    pub(crate) cross_origin: Option<String>,
    pub(crate) blocking: v8::Global<v8::Object>,
    pub(crate) parser_inserted: bool,
    pub(crate) already_started: bool,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(HtmlScriptElementStore::default());
    isolate.set_host_initialize_import_meta_object_callback(initialize_import_meta);
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "HTMLScriptElement", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(c) = scope
        .get_slot::<HtmlScriptElementStore>()
        .and_then(|s| s.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &c));
    }
    let parent = super::html_element::ensure_constructor(scope)?;
    let c = crate::webidl::create_function(
        scope,
        "HTMLScriptElement",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, c, parent)?;
    let p = crate::webidl::prototype(scope, c)?;
    crate::webidl::reset_constructor_order(scope, p)?;
    super::html_script_element_src_property::define(scope, p)?;
    super::html_script_element_type_property::define(scope, p)?;
    super::html_script_element_no_module_property::define(scope, p)?;
    super::html_script_element_charset_property::define(scope, p)?;
    super::html_script_element_async_property::define(scope, p)?;
    super::html_script_element_defer_property::define(scope, p)?;
    super::html_script_element_cross_origin_property::define(scope, p)?;
    super::html_script_element_text_property::define(scope, p)?;
    super::html_script_element_referrer_policy_property::define(scope, p)?;
    super::html_script_element_fetch_priority_property::define(scope, p)?;
    super::html_script_element_event_property::define(scope, p)?;
    super::html_script_element_html_for_property::define(scope, p)?;
    super::html_script_element_integrity_property::define(scope, p)?;
    super::html_script_element_blocking_property::define(scope, p)?;
    crate::webidl::define_accessor(scope, p, "cacheHint", get_cache_hint, set_cache_hint)?;
    super::html_script_element_text_content_property::define(scope, p)?;
    super::html_script_element_inner_text_property::define(scope, p)?;
    crate::webidl::finish_constructor(scope, p, c)?;
    crate::webidl::define_method(scope, c.into(), "supports", 1, supports)?;
    super::html_script_element_attribution_src_property::define(scope, p)?;
    let stored = v8::Global::new(scope, c);
    scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .ok_or_else(|| "HTMLScriptElement state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, stored);
    Ok(c)
}

fn get_cache_hint(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let value = record
            .strings
            .get("cacheHint")
            .map(String::as_str)
            .unwrap_or("");
        if let Some(value) = v8::String::new(scope, value) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_cache_hint(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.strings.insert("cacheHint".to_owned(), value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn supports(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'supports' on 'HTMLScriptElement': 1 argument required, but only 0 present.",
        );
        return;
    }
    let script_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let supported = matches!(
        script_type.as_str(),
        "classic" | "module" | "importmap" | "speculationrules"
    );
    result.set(v8::Boolean::new(scope, supported).into());
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure_constructor(scope)?;
    let p = crate::webidl::prototype(scope, c)?;
    let o = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, o, p.into()) != Some(true) {
        return Err("cannot create HTMLScriptElement".to_owned());
    }
    super::html_element::attach(scope, o, "SCRIPT");
    let blocking = super::dom_token_list::create_with_support(
        scope,
        "",
        super::dom_token_list::DomTokenSupport::Blocking,
    )?;
    let blocking = v8::Global::new(scope, blocking);
    scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .ok_or_else(|| "HTMLScriptElement state was not prepared".to_owned())?
        .records
        .insert(
            o.get_identity_hash().get(),
            ScriptRecord {
                strings: HashMap::new(),
                booleans: HashMap::new(),
                cross_origin: None,
                blocking,
                parser_inserted: false,
                already_started: false,
            },
        );
    Ok(o)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<ScriptRecord> {
    scope
        .get_slot::<HtmlScriptElementStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        if let Some(v) = x.cross_origin.and_then(|v| v8::String::new(scope, &v)) {
            r.set(v.into())
        } else {
            r.set(v8::null(scope).into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_cross_origin(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = if a.get(0).is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, a.get(0)))
    };
    if let Some(x) = scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .and_then(|s| s.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.cross_origin = v
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn get_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(scope, a.this()).is_some() {
        if let Some(v) = v8::String::new(scope, &super::node::text_content(scope, a.this())) {
            r.set(v.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
pub(crate) fn set_text(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let v = crate::webidl::value_to_string(scope, a.get(0));
    if record(scope, a.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    for child in super::node::children(scope, a.this()) {
        super::node::detach(scope, child);
    }
    if !v.is_empty()
        && let Ok(text) = super::text::create(scope, v)
    {
        let _ = super::node::insert_node(scope, a.this(), text, 0);
    }
}
pub(crate) fn get_blocking(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(scope, a.this()) {
        r.set(v8::Local::new(scope, &x.blocking).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn set_blocking(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, a.get(0));
    if let Some(record) = record(scope, a.this()) {
        let blocking = v8::Local::new(scope, &record.blocking);
        super::dom_token_list::set_string_value(scope, blocking, &value);
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn mark_parser_inserted(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
) {
    if let Some(record) = scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    {
        record.parser_inserted = true;
    }
}

fn mark_started(scope: &mut v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    let Some(record) = scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .and_then(|store| store.records.get_mut(&element.get_identity_hash().get()))
    else {
        return false;
    };
    if record.already_started {
        return false;
    }
    record.already_started = true;
    true
}

fn is_classic_script(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    let script_type = super::element::attribute_value(scope, element, "type")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    script_type.is_empty()
        || matches!(
            script_type.as_str(),
            "text/javascript"
                | "application/javascript"
                | "application/ecmascript"
                | "text/ecmascript"
                | "text/jscript"
        )
}

fn execute_classic(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    source: &str,
    source_url: Option<&str>,
) {
    let Some(document) = super::node::owner_document(scope, element) else {
        return;
    };
    let prior = super::document::swap_current_script(scope, document, Some(element));
    if let Some(source) = v8::String::new(scope, source) {
        let script = if let Some(source_url) = source_url {
            v8::String::new(scope, source_url).and_then(|resource_name| {
                let origin = v8::ScriptOrigin::new(
                    scope,
                    resource_name.into(),
                    0,
                    0,
                    false,
                    -1,
                    None,
                    false,
                    false,
                    false,
                    None,
                );
                v8::Script::compile(scope, source, Some(&origin))
            })
        } else {
            v8::Script::compile(scope, source, None)
        };
        if let Some(script) = script {
            let _ = script.run(scope);
        }
    }
    let prior = prior.map(|script| v8::Local::new(scope, &script));
    super::document::swap_current_script(scope, document, prior);
}

fn dispatch_completion(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if let Ok(event) = super::event::create(scope, event_type) {
        super::event_target::dispatch(scope, element, event);
    }
}

fn enqueue_external(
    scope: &mut v8::PinScope<'_, '_>,
    element: v8::Local<'_, v8::Object>,
    source: Option<String>,
    url: String,
    module: bool,
) {
    let pending = PendingScript {
        context: v8::Global::new(scope, scope.get_current_context()),
        element: v8::Global::new(scope, element),
        source,
        url,
        module,
    };
    if let Some(store) = scope.get_slot_mut::<HtmlScriptElementStore>() {
        store.pending.push_back(pending);
    }
}

fn is_module_script(scope: &v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) -> bool {
    super::element::attribute_value(scope, element, "type")
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("module"))
}

fn prepare_script(scope: &mut v8::PinScope<'_, '_>, element: v8::Local<'_, v8::Object>) {
    if record(scope, element).is_some_and(|record| record.parser_inserted)
        && super::document_global::value(scope).is_none()
    {
        return;
    }
    if !super::node::is_connected(scope, element) || !mark_started(scope, element) {
        return;
    }
    let module = is_module_script(scope, element);
    if !module && !is_classic_script(scope, element) {
        return;
    }
    if !module && super::element::attribute_value(scope, element, "nomodule").is_some() {
        return;
    }
    let record = record(scope, element);
    let source = if let Some(src) =
        super::element::attribute_value(scope, element, "src").filter(|src| !src.is_empty())
    {
        let loaded =
            super::worker_script_source::load_with_initiator(scope, &src, None, "script").ok();
        let url = loaded
            .as_ref()
            .map(|script| script.url.clone())
            .unwrap_or_else(|| src.clone());
        let source = loaded.map(|script| script.source);
        if module {
            enqueue_external(scope, element, source, url, true);
        } else if record.is_some_and(|record| record.parser_inserted) {
            if let Some(source) = source {
                execute_classic(scope, element, &source, Some(&url));
                dispatch_completion(scope, element, "load");
            } else {
                dispatch_completion(scope, element, "error");
            }
        } else {
            enqueue_external(scope, element, source, url, false);
        }
        return;
    } else {
        super::node::text_content(scope, element)
    };
    if module {
        let url = super::node::owner_document(scope, element)
            .and_then(|document| super::document::stored_value(scope, document, "URL"))
            .map(|value| {
                let value = v8::Local::new(scope, &value);
                crate::webidl::value_to_string(scope, value)
            })
            .unwrap_or_else(|| crate::page_init::url(scope));
        enqueue_external(scope, element, Some(source), url, true);
    } else {
        let source_url = super::node::owner_document(scope, element)
            .and_then(|document| super::document::stored_value(scope, document, "URL"))
            .map(|value| {
                let value = v8::Local::new(scope, &value);
                crate::webidl::value_to_string(scope, value)
            })
            .unwrap_or_else(|| crate::page_init::url(scope));
        execute_classic(scope, element, &source, Some(&source_url));
    }
}

pub(crate) fn notify_connected_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if record(scope, root).is_some_and(|record| !record.parser_inserted) {
        prepare_script(scope, root);
    }
    for child in super::node::children(scope, root) {
        notify_connected_tree(scope, child);
    }
}

pub(crate) fn execute_parser_inserted_tree(
    scope: &mut v8::PinScope<'_, '_>,
    root: v8::Local<'_, v8::Object>,
) {
    if record(scope, root).is_some_and(|record| record.parser_inserted) {
        prepare_script(scope, root);
    }
    for child in super::node::children(scope, root) {
        execute_parser_inserted_tree(scope, child);
    }
}

pub(crate) fn run_pending_tasks(scope: &mut v8::PinScope<'_, '_>) -> bool {
    let pending = scope
        .get_slot_mut::<HtmlScriptElementStore>()
        .and_then(|store| store.pending.pop_front());
    let Some(pending) = pending else {
        return false;
    };
    let context = v8::Local::new(scope, &pending.context);
    let script_scope = &mut v8::ContextScope::new(scope, context);
    super::animation_frame_state::sample_current_task_realm(script_scope);
    let task_start = super::performance_observer::task_start(script_scope);
    let element = v8::Local::new(script_scope, &pending.element);
    if let Some(source) = pending.source {
        let success = if pending.module {
            execute_module(script_scope, &pending.url, &source)
        } else {
            execute_classic(script_scope, element, &source, Some(&pending.url));
            true
        };
        dispatch_completion(
            script_scope,
            element,
            if success { "load" } else { "error" },
        );
    } else {
        dispatch_completion(script_scope, element, "error");
    }
    script_scope.perform_microtask_checkpoint();
    if super::performance_observer::record_completed_task(script_scope, task_start, false) {
        script_scope.perform_microtask_checkpoint();
    }
    true
}

fn module_cache_key(scope: &v8::PinScope<'_, '_>, url: &str) -> String {
    format!("{}|{url}", crate::webidl::realm_id(scope))
}

fn compile_module<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    url: &str,
    source: v8::Local<'s, v8::String>,
) -> Option<v8::Local<'s, v8::Module>> {
    let key = module_cache_key(scope, url);
    if let Some(module) = scope
        .get_slot::<HtmlScriptElementStore>()
        .and_then(|store| store.modules.get(&key))
        .cloned()
    {
        return Some(v8::Local::new(scope, &module));
    }
    let resource_name = v8::String::new(scope, url)?;
    let origin = v8::ScriptOrigin::new(
        scope,
        resource_name.into(),
        0,
        0,
        false,
        -1,
        None,
        false,
        false,
        true,
        None,
    );
    let mut source = v8::script_compiler::Source::new(source, Some(&origin));
    let module = v8::script_compiler::compile_module(scope, &mut source)?;
    let saved = v8::Global::new(scope, module);
    if let Some(store) = scope.get_slot_mut::<HtmlScriptElementStore>() {
        if let Some(script_id) = module.script_id() {
            store.module_urls.insert(script_id, url.to_owned());
        }
        store.modules.insert(key, saved);
    }
    Some(module)
}

fn resolve_module<'s>(
    context: v8::Local<'s, v8::Context>,
    specifier: v8::Local<'s, v8::String>,
    _import_attributes: v8::Local<'s, v8::FixedArray>,
    referrer: v8::Local<'s, v8::Module>,
) -> Option<v8::Local<'s, v8::Module>> {
    v8::callback_scope!(unsafe scope, context);
    let base = referrer.script_id().and_then(|script_id| {
        scope
            .get_slot::<HtmlScriptElementStore>()
            .and_then(|store| store.module_urls.get(&script_id))
            .cloned()
    })?;
    let input = specifier.to_rust_string_lossy(scope);
    let script = match super::worker_script_source::load_with_initiator(
        scope,
        &input,
        Some(&base),
        "script",
    ) {
        Ok(script) => script,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    let source = v8::String::new(scope, &script.source)?;
    compile_module(scope, &script.url, source)
}

fn execute_module(scope: &mut v8::PinScope<'_, '_>, url: &str, source: &str) -> bool {
    v8::tc_scope!(let try_catch, scope);
    let Some(source) = v8::String::new(try_catch, source) else {
        return false;
    };
    let success = compile_module(try_catch, url, source).and_then(|module| {
        module
            .instantiate_module(try_catch, resolve_module)
            .and_then(|instantiated| instantiated.then_some(module))
            .and_then(|module| module.evaluate(try_catch))
    });
    if success.is_some() {
        try_catch.perform_microtask_checkpoint();
        true
    } else {
        false
    }
}

pub(crate) fn dynamic_import<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    resource_name: v8::Local<'s, v8::Value>,
    specifier: v8::Local<'s, v8::String>,
) -> Option<v8::Local<'s, v8::Promise>> {
    let base = resource_name
        .to_string(scope)
        .map(|value| value.to_rust_string_lossy(scope))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| crate::page_init::url(scope));
    let input = specifier.to_rust_string_lossy(scope);
    let script = match super::worker_script_source::load_with_initiator(
        scope,
        &input,
        Some(&base),
        "script",
    ) {
        Ok(script) => script,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return None;
        }
    };
    let source = v8::String::new(scope, &script.source)?;
    let module = compile_module(scope, &script.url, source)?;
    if module.instantiate_module(scope, resolve_module) != Some(true) {
        return None;
    }
    module.evaluate(scope)?;
    let resolver = v8::PromiseResolver::new(scope)?;
    resolver.resolve(scope, module.get_module_namespace())?;
    Some(resolver.get_promise(scope))
}

pub(crate) unsafe extern "C" fn initialize_import_meta(
    context: v8::Local<'_, v8::Context>,
    module: v8::Local<'_, v8::Module>,
    meta: v8::Local<'_, v8::Object>,
) {
    v8::callback_scope!(unsafe scope, context);
    let url = module
        .script_id()
        .and_then(|script_id| {
            scope
                .get_slot::<HtmlScriptElementStore>()
                .and_then(|store| store.module_urls.get(&script_id))
                .cloned()
        })
        .or_else(|| super::worker_global_scope::module_url(scope, module));
    let Some(url) = url else {
        return;
    };
    let Some(key) = v8::String::new(scope, "url") else {
        return;
    };
    let Some(value) = v8::String::new(scope, &url) else {
        return;
    };
    let _ = meta.create_data_property(scope, key.into(), value.into());
}
