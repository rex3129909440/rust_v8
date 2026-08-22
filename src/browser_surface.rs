use std::collections::HashMap;

use crate::browser_version::BrowserVersion;

#[derive(Default)]
struct RealmSurfaceStore {
    versions: HashMap<i32, BrowserVersion>,
    staged_window_properties: HashMap<i32, HashMap<String, v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RealmSurfaceStore::default());
}

pub(crate) fn register_current_realm(
    scope: &mut v8::PinScope<'_, '_>,
    version: BrowserVersion,
) -> Result<(), String> {
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<RealmSurfaceStore>()
        .ok_or_else(|| "browser surface state was not prepared".to_owned())?
        .versions
        .insert(realm_id, version);
    Ok(())
}

pub(crate) fn register_current_realm_from_fingerprint(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    register_current_realm(scope, crate::fingerprint::browser_version(scope))
}

pub(crate) fn current_version(scope: &v8::PinScope<'_, '_>) -> BrowserVersion {
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot::<RealmSurfaceStore>()
        .and_then(|store| store.versions.get(&realm_id))
        .copied()
        .unwrap_or_else(|| crate::fingerprint::browser_version(scope))
}

pub(crate) fn window_names(version: BrowserVersion) -> &'static [&'static str] {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::window_names(version.major())
    } else if version.is_android() {
        crate::browser_android_surface_data::window_names(version.major())
    } else {
        crate::browser_surface_data::window_names(version.major())
    }
}

pub(crate) fn current_window_exposes(scope: &v8::PinScope<'_, '_>, name: &str) -> bool {
    window_names(current_version(scope)).contains(&name)
}

pub(crate) fn navigator_names(version: BrowserVersion) -> &'static [&'static str] {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::navigator_names(version.major())
    } else if version.is_android() {
        crate::browser_android_surface_data::navigator_names(version.major())
    } else {
        crate::browser_surface_data::navigator_names(version.major())
    }
}

pub(crate) fn worker_navigator_names(version: BrowserVersion) -> &'static [&'static str] {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::worker_navigator_names(version.major())
    } else if version.is_android() {
        crate::browser_android_surface_data::worker_navigator_names(version.major())
    } else {
        crate::browser_surface_data::worker_navigator_names(version.major())
    }
}

fn prototype_names(version: BrowserVersion, owner: &str) -> Option<&'static [&'static str]> {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::prototype_names(owner, version.major())
    } else if version.is_android() {
        crate::browser_android_surface_data::prototype_names(owner, version.major())
    } else {
        crate::browser_surface_data::prototype_names(owner, version.major())
    }
}

fn constructor_static_names(
    version: BrowserVersion,
    owner: &str,
) -> Option<&'static [&'static str]> {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::constructor_static_names(
            owner,
            version.major(),
        )
    } else if version.is_android() {
        crate::browser_android_surface_data::constructor_static_names(owner, version.major())
    } else {
        crate::browser_surface_data::constructor_static_names(owner, version.major())
    }
}

fn global_object_names(version: BrowserVersion, owner: &str) -> Option<&'static [&'static str]> {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::global_object_names(owner, version.major())
    } else if version.is_android() {
        crate::browser_android_surface_data::global_object_names(owner, version.major())
    } else {
        crate::browser_surface_data::global_object_names(owner, version.major())
    }
}

fn worker_global_names(version: BrowserVersion) -> &'static [&'static str] {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::worker_global_names(version.major())
    } else if version.is_android() {
        crate::browser_android_surface_data::worker_global_names(version.major())
    } else {
        crate::browser_surface_data::worker_global_names(version.major())
    }
}

fn worker_prototype_names(version: BrowserVersion, owner: &str) -> Option<&'static [&'static str]> {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::worker_prototype_names(owner, version.major())
    } else if version.is_android() {
        crate::browser_android_surface_data::worker_prototype_names(owner, version.major())
    } else {
        crate::browser_surface_data::worker_prototype_names(owner, version.major())
    }
}

fn worker_constructor_static_names(
    version: BrowserVersion,
    owner: &str,
) -> Option<&'static [&'static str]> {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::worker_constructor_static_names(
            owner,
            version.major(),
        )
    } else if version.is_android() {
        crate::browser_android_surface_data::worker_constructor_static_names(owner, version.major())
    } else {
        crate::browser_surface_data::worker_constructor_static_names(owner, version.major())
    }
}

fn worker_global_object_names(
    version: BrowserVersion,
    owner: &str,
) -> Option<&'static [&'static str]> {
    if version.is_webview() {
        crate::browser_android_webview_surface_data::worker_global_object_names(
            owner,
            version.major(),
        )
    } else if version.is_android() {
        crate::browser_android_surface_data::worker_global_object_names(owner, version.major())
    } else {
        crate::browser_surface_data::worker_global_object_names(owner, version.major())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RealmSurfaceStore>() {
        store.versions.remove(&realm_id);
        store.staged_window_properties.remove(&realm_id);
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum WindowSurfacePhase {
    Interfaces,
    BeforeChrome,
    AfterChrome,
}

/// Reorders configurable Window properties inside Chromium's native
/// non-configurable anchors. Original V8 descriptors are preserved, without
/// a JavaScript Proxy or wrapper object.
pub(crate) fn apply_window_phase(
    scope: &mut v8::PinScope<'_, '_>,
    phase: WindowSurfacePhase,
) -> Result<(), String> {
    let realm_id = crate::webidl::realm_id(scope);
    let browser = current_version(scope);
    if browser.is_webview() {
        // The supplied WebView evidence orders V8's non-configurable intrinsic
        // globals differently from the embedded V8 build. WebView own-key
        // order is exposed by the native Object/Reflect interception path;
        // physically deleting and rebuilding these anchors is impossible.
        return Ok(());
    }
    let version = browser.major();
    let target = window_names(browser);
    let (anchor, desired, final_phase) = match phase {
        WindowSurfacePhase::Interfaces => (
            "console",
            slice_between(target, "console", "window")?,
            false,
        ),
        WindowSurfacePhase::BeforeChrome => ("top", slice_between(target, "top", "chrome")?, false),
        WindowSurfacePhase::AfterChrome => ("chrome", slice_after(target, "chrome")?, true),
    };
    let global = scope.get_current_context().global(scope);
    let current = own_string_names(scope, global)?;
    let anchor_index = current
        .iter()
        .position(|name| name == anchor)
        .ok_or_else(|| format!("Window surface anchor {anchor} is missing"))?;
    let mut staged = scope
        .get_slot_mut::<RealmSurfaceStore>()
        .and_then(|store| store.staged_window_properties.remove(&realm_id))
        .unwrap_or_default();
    for name in &current[anchor_index + 1..] {
        let key = crate::webidl::string(scope, name)?;
        let descriptor = global
            .get_own_property_descriptor(scope, key.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or_else(|| format!("cannot preserve Window.{name} descriptor"))?;
        if !descriptor_boolean(scope, descriptor, "configurable")? {
            return Err(format!(
                "cannot move non-configurable Window.{name} after {anchor}"
            ));
        }
        if global.delete(scope, key.into()) != Some(true) {
            return Err(format!("cannot stage Window.{name}"));
        }
        staged.insert(name.clone(), v8::Global::new(scope, descriptor));
    }
    for name in desired {
        let Some(descriptor) = staged.remove(*name) else {
            return Err(format!(
                "Chromium {version} Window {phase:?} surface requires unavailable property {name}"
            ));
        };
        let descriptor = v8::Local::new(scope, &descriptor);
        define_from_descriptor_object(scope, global, name, descriptor)?;
    }
    if !final_phase {
        scope
            .get_slot_mut::<RealmSurfaceStore>()
            .ok_or_else(|| "browser surface state was not prepared".to_owned())?
            .staged_window_properties
            .insert(realm_id, staged);
    }
    Ok(())
}

pub(crate) fn virtualize_webview_window_keys<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    original: v8::Local<'s, v8::Array>,
    include_non_enumerable_target: bool,
) -> Option<v8::Local<'s, v8::Array>> {
    let version = current_version(scope);
    if !version.is_webview() {
        return None;
    }
    let global = scope.get_current_context().global(scope);
    if !object.strict_equals(global.into())
        && !crate::web::html_i_frame_element::is_child_window(scope, object)
    {
        return None;
    }

    let mut original_strings = Vec::new();
    let mut original_string_set = std::collections::HashSet::new();
    let mut original_symbols = Vec::new();
    for index in 0..original.length() {
        let value = original.get_index(scope, index)?;
        if value.is_symbol() {
            original_symbols.push(value);
            continue;
        }
        let name = value.to_string(scope)?.to_rust_string_lossy(scope);
        original_string_set.insert(name.clone());
        original_strings.push(name);
    }

    let target = window_names(version);
    let target_set = target
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let modern_set = (crate::browser_version::BrowserVersion::MIN_SUPPORTED
        ..=crate::browser_version::BrowserVersion::MAX_SUPPORTED)
        .flat_map(|major| {
            crate::browser_android_surface_data::window_names(major)
                .iter()
                .chain(crate::browser_surface_data::window_names(major).iter())
        })
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let canonical_index = |name: &str| {
        name.parse::<u32>()
            .is_ok_and(|index| index.to_string() == name)
    };
    let mut output = Vec::<v8::Local<v8::Value>>::new();
    if !include_non_enumerable_target
        && crate::web::html_i_frame_element::is_child_window(scope, object)
        && let Some(enumerable_target) =
            crate::browser_android_webview_surface_data::window_enumerable_names(version.major())
    {
        for name in original_strings.iter().filter(|name| canonical_index(name)) {
            output.push(v8::String::new(scope, name)?.into());
        }
        for name in enumerable_target {
            output.push(v8::String::new(scope, name)?.into());
        }
        let enumerable_set = enumerable_target
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        for name in original_strings.iter().filter(|name| {
            !canonical_index(name)
                && !enumerable_set.contains(name.as_str())
                && !target_set.contains(name.as_str())
                && !modern_set.contains(name.as_str())
        }) {
            output.push(v8::String::new(scope, name)?.into());
        }
        return Some(v8::Array::new_with_elements(scope, &output));
    }
    for name in original_strings.iter().filter(|name| canonical_index(name)) {
        output.push(v8::String::new(scope, name)?.into());
    }
    for name in target {
        if include_non_enumerable_target || original_string_set.contains(*name) {
            output.push(v8::String::new(scope, name)?.into());
        }
    }
    for name in original_strings.iter().filter(|name| {
        !canonical_index(name)
            && !target_set.contains(name.as_str())
            && !modern_set.contains(name.as_str())
    }) {
        output.push(v8::String::new(scope, name)?.into());
    }
    output.extend(original_symbols);
    Some(v8::Array::new_with_elements(scope, &output))
}

pub(crate) fn restore_staged_window_property(
    scope: &mut v8::PinScope<'_, '_>,
    name: &str,
) -> Result<bool, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let descriptor = scope
        .get_slot_mut::<RealmSurfaceStore>()
        .and_then(|store| store.staged_window_properties.get_mut(&realm_id))
        .and_then(|staged| staged.remove(name));
    let Some(descriptor) = descriptor else {
        return Ok(false);
    };
    let descriptor = v8::Local::new(scope, &descriptor);
    let global = scope.get_current_context().global(scope);
    define_from_descriptor_object(scope, global, name, descriptor)?;
    Ok(true)
}

pub(crate) fn reorder_string_properties(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    desired: &[&str],
    label: &str,
) -> Result<(), String> {
    let current = own_string_names(scope, object)?;
    let mut descriptors = HashMap::with_capacity(current.len());
    for name in current {
        let key = crate::webidl::string(scope, &name)?;
        let descriptor = object
            .get_own_property_descriptor(scope, key.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or_else(|| format!("cannot preserve {label}.{name} descriptor"))?;
        if !descriptor_boolean(scope, descriptor, "configurable")? {
            return Err(format!("cannot reorder non-configurable {label}.{name}"));
        }
        if object.delete(scope, key.into()) != Some(true) {
            return Err(format!("cannot reorder {label}.{name}"));
        }
        descriptors.insert(name, v8::Global::new(scope, descriptor));
    }
    for name in desired {
        let Some(descriptor) = descriptors.remove(*name) else {
            return Err(format!(
                "browser surface requires unavailable {label}.{name}"
            ));
        };
        let descriptor = v8::Local::new(scope, &descriptor);
        define_from_descriptor_object(scope, object, name, descriptor)?;
    }
    Ok(())
}

pub(crate) fn finalize_versioned_prototypes(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let version = current_version(scope);
    let global = scope.get_current_context().global(scope);
    let globals = own_string_names(scope, global)?;
    for owner in globals {
        let Some(desired) = prototype_names(version, &owner) else {
            continue;
        };
        if desired.is_empty() {
            continue;
        }
        let owner_key = crate::webidl::string(scope, &owner)?;
        let Some(constructor) = global.get(scope, owner_key.into()) else {
            continue;
        };
        let Ok(constructor) = v8::Local::<v8::Function>::try_from(constructor) else {
            continue;
        };
        let prototype_key = crate::webidl::string(scope, "prototype")?;
        let Some(prototype) = constructor.get(scope, prototype_key.into()) else {
            continue;
        };
        let Ok(prototype) = v8::Local::<v8::Object>::try_from(prototype) else {
            continue;
        };
        reconcile_string_properties(scope, prototype, desired, &format!("{owner}.prototype"))?;
    }
    Ok(())
}

pub(crate) fn finalize_versioned_statics_and_objects(
    scope: &mut v8::PinScope<'_, '_>,
) -> Result<(), String> {
    let version = current_version(scope);
    let global = scope.get_current_context().global(scope);
    let globals = own_string_names(scope, global)?;
    for owner in globals {
        let owner_key = crate::webidl::string(scope, &owner)?;
        let Some(value) = global.get(scope, owner_key.into()) else {
            continue;
        };
        if let Some(desired) = constructor_static_names(version, &owner)
            && let Ok(constructor) = v8::Local::<v8::Function>::try_from(value)
        {
            if desired.is_empty() {
                continue;
            }
            reconcile_string_properties(scope, constructor.into(), desired, &owner)?;
            continue;
        }
        if let Some(desired) = global_object_names(version, &owner)
            && let Ok(object) = v8::Local::<v8::Object>::try_from(value)
        {
            if desired.is_empty() {
                continue;
            }
            reconcile_string_properties(scope, object, desired, &owner)?;
        }
    }
    Ok(())
}

pub(crate) fn finalize_worker_versioned_interfaces(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let version = current_version(scope);
    let owners = own_string_names(scope, object)?;
    for owner in &owners {
        let Some(desired) = worker_prototype_names(version, owner) else {
            continue;
        };
        if desired.is_empty() {
            continue;
        }
        let owner_key = crate::webidl::string(scope, owner)?;
        let Some(constructor) = object.get(scope, owner_key.into()) else {
            continue;
        };
        let Ok(constructor) = v8::Local::<v8::Function>::try_from(constructor) else {
            continue;
        };
        let prototype_key = crate::webidl::string(scope, "prototype")?;
        let Some(prototype) = constructor.get(scope, prototype_key.into()) else {
            continue;
        };
        let Ok(prototype) = v8::Local::<v8::Object>::try_from(prototype) else {
            continue;
        };
        // A few WebIDL constructors are installed internally to manufacture
        // worker-owned objects (for example FontFaceSet) but are not exposed
        // on DedicatedWorkerGlobalScope.  Their generated target is empty.
        if desired.is_empty() {
            continue;
        }
        reconcile_string_properties(
            scope,
            prototype,
            desired,
            &format!("Worker {owner}.prototype"),
        )?;
    }
    for owner in owners {
        let owner_key = crate::webidl::string(scope, &owner)?;
        let Some(value) = object.get(scope, owner_key.into()) else {
            continue;
        };
        if let Some(desired) = worker_constructor_static_names(version, &owner)
            && let Ok(constructor) = v8::Local::<v8::Function>::try_from(value)
        {
            if !desired.is_empty() {
                reconcile_string_properties(
                    scope,
                    constructor.into(),
                    desired,
                    &format!("Worker {owner}"),
                )?;
            }
            continue;
        }
        if let Some(desired) = worker_global_object_names(version, &owner)
            && let Ok(global_object) = v8::Local::<v8::Object>::try_from(value)
            && !desired.is_empty()
        {
            reconcile_string_properties(scope, global_object, desired, &format!("Worker {owner}"))?;
        }
    }
    Ok(())
}

pub(crate) fn finalize_worker_global(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let version = current_version(scope);
    reconcile_suffix_after(
        scope,
        object,
        worker_global_names(version),
        "console",
        "DedicatedWorkerGlobalScope",
    )
}

fn reconcile_suffix_after(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    desired: &[&str],
    anchor: &str,
    label: &str,
) -> Result<(), String> {
    let current = own_string_names(scope, object)?;
    let current_anchor = current
        .iter()
        .position(|name| name == anchor)
        .ok_or_else(|| format!("{label} anchor {anchor} is missing"))?;
    let desired_anchor = desired
        .iter()
        .position(|name| *name == anchor)
        .ok_or_else(|| format!("target {label} anchor {anchor} is missing"))?;
    if !current[..=current_anchor]
        .iter()
        .map(String::as_str)
        .eq(desired[..=desired_anchor].iter().copied())
    {
        return Err(format!("{label} intrinsic prefix differs before {anchor}"));
    }
    let mut descriptors = HashMap::new();
    for name in &current[current_anchor + 1..] {
        let key = crate::webidl::string(scope, name)?;
        let descriptor = object
            .get_own_property_descriptor(scope, key.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or_else(|| format!("cannot preserve {label}.{name}"))?;
        if !descriptor_boolean(scope, descriptor, "configurable")? {
            return Err(format!("cannot move non-configurable {label}.{name}"));
        }
        if object.delete(scope, key.into()) != Some(true) {
            return Err(format!("cannot move {label}.{name}"));
        }
        descriptors.insert(name.clone(), v8::Global::new(scope, descriptor));
    }
    for name in &desired[desired_anchor + 1..] {
        let Some(descriptor) = descriptors.remove(*name) else {
            return Err(format!(
                "browser surface requires unavailable {label}.{name}"
            ));
        };
        let descriptor = v8::Local::new(scope, &descriptor);
        define_from_descriptor_object(scope, object, name, descriptor)?;
    }
    Ok(())
}

fn reconcile_string_properties(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    desired: &[&str],
    label: &str,
) -> Result<(), String> {
    let current = own_string_names(scope, object)?;
    if current
        .iter()
        .map(String::as_str)
        .eq(desired.iter().copied())
    {
        return Ok(());
    }
    let desired_set = desired
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let retained = current
        .iter()
        .filter(|name| desired_set.contains(name.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    if retained == desired {
        for name in current
            .iter()
            .filter(|name| !desired_set.contains(name.as_str()))
        {
            let key = crate::webidl::string(scope, name)?;
            let descriptor = object
                .get_own_property_descriptor(scope, key.into())
                .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
                .ok_or_else(|| format!("cannot inspect {label}.{name} descriptor"))?;
            if !descriptor_boolean(scope, descriptor, "configurable")? {
                return Err(format!(
                    "browser version cannot hide non-configurable {label}.{name}"
                ));
            }
            if object.delete(scope, key.into()) != Some(true) {
                return Err(format!("cannot hide {label}.{name}"));
            }
        }
        return Ok(());
    }
    let mut descriptors = HashMap::with_capacity(current.len());
    for name in current {
        let key = crate::webidl::string(scope, &name)?;
        let descriptor = object
            .get_own_property_descriptor(scope, key.into())
            .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
            .ok_or_else(|| format!("cannot preserve {label}.{name} descriptor"))?;
        let configurable = descriptor_boolean(scope, descriptor, "configurable")?;
        if !configurable {
            if !desired_set.contains(name.as_str()) {
                return Err(format!(
                    "browser version cannot hide non-configurable {label}.{name}"
                ));
            }
            continue;
        }
        if object.delete(scope, key.into()) != Some(true) {
            return Err(format!("cannot reconcile {label}.{name}"));
        }
        descriptors.insert(name, v8::Global::new(scope, descriptor));
    }
    for name in desired {
        let key = crate::webidl::string(scope, name)?;
        if object.has_own_property(scope, key.into()) == Some(true) {
            continue;
        }
        let Some(descriptor) = descriptors.remove(*name) else {
            return Err(format!(
                "browser surface requires unavailable {label}.{name}"
            ));
        };
        let descriptor = v8::Local::new(scope, &descriptor);
        define_from_descriptor_object(scope, object, name, descriptor)?;
    }
    let actual = own_string_names(scope, object)?;
    if !actual
        .iter()
        .map(String::as_str)
        .eq(desired.iter().copied())
    {
        let mismatch = actual
            .iter()
            .map(String::as_str)
            .zip(desired.iter().copied())
            .position(|(actual, desired)| actual != desired)
            .unwrap_or_else(|| actual.len().min(desired.len()));
        return Err(format!(
            "non-configurable property anchors prevent exact {label} order at {mismatch}: actual={:?}, desired={:?}",
            actual.get(mismatch),
            desired.get(mismatch),
        ));
    }
    Ok(())
}

fn slice_between<'a>(
    values: &'a [&'a str],
    start: &str,
    end: &str,
) -> Result<&'a [&'a str], String> {
    let start = values
        .iter()
        .position(|value| *value == start)
        .ok_or_else(|| format!("browser surface start anchor {start} is missing"))?;
    let end = values
        .iter()
        .position(|value| *value == end)
        .ok_or_else(|| format!("browser surface end anchor {end} is missing"))?;
    if end <= start {
        return Err("browser surface anchors are out of order".to_owned());
    }
    Ok(&values[start + 1..end])
}

fn slice_after<'a>(values: &'a [&'a str], anchor: &str) -> Result<&'a [&'a str], String> {
    let index = values
        .iter()
        .position(|value| *value == anchor)
        .ok_or_else(|| format!("browser surface anchor {anchor} is missing"))?;
    Ok(&values[index + 1..])
}

fn own_string_names(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<Vec<String>, String> {
    let arguments = v8::GetPropertyNamesArgs {
        mode: v8::KeyCollectionMode::OwnOnly,
        property_filter: v8::PropertyFilter::SKIP_SYMBOLS,
        index_filter: v8::IndexFilter::IncludeIndices,
        key_conversion: v8::KeyConversionMode::ConvertToString,
    };
    let keys = object
        .get_own_property_names(scope, arguments)
        .ok_or_else(|| "cannot enumerate browser surface".to_owned())?;
    let mut names = Vec::with_capacity(keys.length() as usize);
    for index in 0..keys.length() {
        let value = keys
            .get_index(scope, index)
            .and_then(|value| value.to_string(scope))
            .ok_or_else(|| "cannot read browser surface key".to_owned())?;
        names.push(value.to_rust_string_lossy(scope));
    }
    Ok(names)
}

fn descriptor_value<'s>(
    scope: &v8::PinScope<'s, '_>,
    descriptor: v8::Local<'s, v8::Object>,
    name: &str,
) -> Result<v8::Local<'s, v8::Value>, String> {
    let key = crate::webidl::string(scope, name)?;
    descriptor
        .get(scope, key.into())
        .ok_or_else(|| format!("descriptor field {name} is unavailable"))
}

fn descriptor_has(
    scope: &v8::PinScope<'_, '_>,
    descriptor: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<bool, String> {
    let key = crate::webidl::string(scope, name)?;
    Ok(descriptor.has_own_property(scope, key.into()) == Some(true))
}

fn descriptor_boolean(
    scope: &v8::PinScope<'_, '_>,
    descriptor: v8::Local<'_, v8::Object>,
    name: &str,
) -> Result<bool, String> {
    Ok(descriptor_value(scope, descriptor, name)?.boolean_value(scope))
}

fn define_from_descriptor_object(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    source: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let mut descriptor = if descriptor_has(scope, source, "value")? {
        v8::PropertyDescriptor::new_from_value_writable(
            descriptor_value(scope, source, "value")?,
            descriptor_boolean(scope, source, "writable")?,
        )
    } else {
        v8::PropertyDescriptor::new_from_get_set(
            descriptor_value(scope, source, "get")?,
            descriptor_value(scope, source, "set")?,
        )
    };
    descriptor.set_enumerable(descriptor_boolean(scope, source, "enumerable")?);
    descriptor.set_configurable(descriptor_boolean(scope, source, "configurable")?);
    let key = crate::webidl::string(scope, name)?;
    if object.define_property(scope, key.into(), &descriptor) != Some(true) {
        return Err(format!("cannot restore browser surface property {name}"));
    }
    Ok(())
}
