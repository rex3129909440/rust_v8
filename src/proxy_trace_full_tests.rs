use crate::runtime::EdgeRuntime;

#[test]
fn native_trace_excludes_user_selected_api_paths_and_expands_array_arguments() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    runtime
        .set_native_trace_exclusions(&["window.String".to_owned(), "window.Number".to_owned()])
        .expect("set trace exclusions");
    runtime.enable_native_trace().expect("enable native trace");
    runtime
        .evaluate(
            r#"
            String(1);
            Number("2");
            new Blob(["alpha", 2, [true, null]]);
            navigator.userAgent;
            "#,
        )
        .expect("evaluate filtered trace source");
    let entries = runtime.native_trace();
    assert!(
        entries
            .iter()
            .all(|entry| { entry.api != "window.String" && entry.api != "window.Number" }),
        "excluded APIs were stored in native Trace: {entries:#?}"
    );
    let blob = entries
        .iter()
        .find(|entry| entry.api == "Blob")
        .expect("Blob constructor trace");
    assert!(
        blob.arguments.contains("[\"alpha\",2,[true,null]]"),
        "array arguments were not expanded: {}",
        blob.arguments
    );

    runtime
        .set_native_trace_exclusions(&[])
        .expect("clear trace exclusions");
    runtime.clear_native_trace();
    runtime
        .evaluate("String(3)")
        .expect("evaluate unfiltered API");
    assert!(
        runtime
            .native_trace()
            .iter()
            .any(|entry| entry.api == "window.String"),
        "empty exclusion list did not restore API recording"
    );
}

const GRAPH_SNAPSHOT: &str = r#"
(() => {
  const start = __EDGE_AUDIT_START__;
  const end = __EDGE_AUDIT_END__;
  const names = Object.getOwnPropertyNames(globalThis);
  const roots = names.slice(start, end);
  const seen = new Map();
  const expanded = new Set();
  const queue = [];
  const rows = [];

  const keyText = key => {
    if (typeof key !== "symbol") return String(key);
    const globalKey = Symbol.keyFor(key);
    return globalKey === undefined
      ? "@@" + (key.description === undefined ? "" : key.description)
      : "@@global:" + globalKey;
  };

  const primitiveText = value => {
    if (value === undefined) return "undefined";
    if (value === null) return "null";
    if (typeof value === "string") return "string:" + value;
    if (typeof value === "number") {
      if (Number.isNaN(value)) return "number:NaN";
      if (Object.is(value, -0)) return "number:-0";
    }
    return typeof value + ":" + String(value);
  };

  const objectId = (value, expand) => {
    if ((typeof value !== "object" || value === null) &&
        typeof value !== "function") {
      return primitiveText(value);
    }
    if (value === globalThis) return "window";
    if (!seen.has(value)) seen.set(value, seen.size);
    const id = seen.get(value);
    if (expand && !expanded.has(value)) {
      expanded.add(value);
      queue.push(value);
    }
    return "ref:" + id;
  };

  const safe = action => {
    try {
      return action();
    } catch (error) {
      return "throw:" + error.name + ":" + error.message;
    }
  };

  const functionText = value => [
    safe(() => value.name),
    safe(() => value.length),
    safe(() => Function.prototype.toString.call(value)),
    safe(() => value.toString()),
    safe(() => value.toString.toString()),
    safe(() => String(value)),
    safe(() => Object.prototype.toString.call(value)),
    safe(() => Object.getPrototypeOf(value) === Function.prototype)
  ].join("\u001d");

  const descriptorText = (owner, key) => {
    const descriptor = Object.getOwnPropertyDescriptor(owner, key);
    if (descriptor === undefined) return keyText(key) + ":missing";
    const kind = Object.prototype.hasOwnProperty.call(descriptor, "value")
      ? "data"
      : "accessor";
    const value = kind === "data"
      ? (typeof descriptor.value === "function"
          ? "fn:" + functionText(descriptor.value) + ":" +
            objectId(
              descriptor.value,
              Object.prototype.hasOwnProperty.call(
                descriptor.value,
                "prototype"
              )
            )
          : objectId(descriptor.value, true))
      : "-";
    const getter = kind === "accessor" && typeof descriptor.get === "function"
      ? "fn:" + functionText(descriptor.get) +
        ":" + objectId(descriptor.get, false)
      : primitiveText(descriptor.get);
    const setter = kind === "accessor" && typeof descriptor.set === "function"
      ? "fn:" + functionText(descriptor.set) +
        ":" + objectId(descriptor.set, false)
      : primitiveText(descriptor.set);
    return [
      keyText(key),
      kind,
      Number(descriptor.enumerable),
      Number(descriptor.configurable),
      Number(Boolean(descriptor.writable)),
      value,
      getter,
      setter
    ].join("\u001e");
  };

  for (const name of roots) {
    const value = safe(() => globalThis[name]);
    rows.push("root\u001f" + name + "\u001f" + objectId(value, true));
    rows.push(
      "window-descriptor\u001f" +
      descriptorText(globalThis, name)
    );
  }

  for (let cursor = 0; cursor < queue.length; cursor += 1) {
    if (cursor > 20000) throw new Error("API object graph exceeded limit");
    const value = queue[cursor];
    const type = typeof value;
    const tag = safe(() => Object.prototype.toString.call(value));
    const extensible = safe(() => Object.isExtensible(value));
    const prototype = safe(() => Object.getPrototypeOf(value));
    const keys = safe(() => Reflect.ownKeys(value));
    const header = [
      "node",
      cursor,
      type,
      tag,
      extensible,
      type === "function" ? functionText(value) : "-",
      objectId(prototype, true),
      Array.isArray(keys) ? keys.map(keyText).join("\u001d") : keys
    ].join("\u001f");
    rows.push(header);
    if (!Array.isArray(keys)) continue;
    for (const key of keys) {
      rows.push(
        "descriptor\u001f" + cursor + "\u001f" +
        descriptorText(value, key)
      );
    }
  }

  return rows.join("\n");
})()
"#;

fn source_for_range(start: usize, end: usize) -> String {
    GRAPH_SNAPSHOT
        .replace("__EDGE_AUDIT_START__", &start.to_string())
        .replace("__EDGE_AUDIT_END__", &end.to_string())
}

fn evaluate_direct(runtime: &mut EdgeRuntime, source: &str) -> String {
    runtime
        .evaluate(source)
        .unwrap_or_else(|error| panic!("JavaScript evaluation failed: {error}"))
        .to_string()
}

fn evaluate_traced(runtime: &mut EdgeRuntime, source: &str) -> String {
    runtime
        .evaluate_without_native_trace_entries(source)
        .unwrap_or_else(|error| panic!("native-trace JavaScript evaluation failed: {error}"))
        .to_string()
}

#[test]
fn every_window_api_graph_keeps_its_shape_through_proxy_trace() {
    const WINDOW_PROPERTY_COUNT: usize = 1232;
    const CHUNK_SIZE: usize = 32;

    for start in (0..WINDOW_PROPERTY_COUNT).step_by(CHUNK_SIZE) {
        let end = (start + CHUNK_SIZE).min(WINDOW_PROPERTY_COUNT);
        let source = source_for_range(start, end);

        let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
        let expected = evaluate_direct(&mut direct, &source);

        let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
        traced
            .enable_proxy_trace()
            .expect("enable Proxy trace for full API audit");
        let actual = evaluate_traced(&mut traced, &source);

        assert_eq!(
            actual, expected,
            "Proxy trace changed an API shape in Window range {start}..{end}"
        );
    }
}

#[test]
fn every_window_own_descriptor_keeps_its_edge_shape_after_proxy_warmup() {
    const NATIVE_SHAPE_WARMUP: &str = r#"
      (() => {
        const native = (fn, name) =>
          Function.prototype.toString.call(fn) ===
            `function ${name}() { [native code] }` &&
          fn.toString() === `function ${name}() { [native code] }` &&
          fn.toString.toString() ===
            "function toString() { [native code] }" &&
          String(fn) === `function ${name}() { [native code] }` &&
          Object.prototype.toString.call(fn) === "[object Function]" &&
          Object.getPrototypeOf(fn) === Function.prototype;
        const dataDescriptor = (owner, key, fn) => {
          const descriptor = Object.getOwnPropertyDescriptor(owner, key);
          return "value" in descriptor &&
            descriptor.value === fn &&
            descriptor.enumerable === (owner !== window) &&
            descriptor.configurable === true &&
            descriptor.writable === true &&
            descriptor.value.toString() === fn.toString();
        };
        const userAgent =
          Object.getOwnPropertyDescriptor(Navigator.prototype, "userAgent");
        const href = Object.getOwnPropertyDescriptor(URL.prototype, "href");
        return [
          native(URL, "URL"),
          native(Document.prototype.createElement, "createElement"),
          native(Element.prototype.getAttribute, "getAttribute"),
          native(Element.prototype.setAttribute, "setAttribute"),
          native(Function.prototype.toString, "toString"),
          native(Object.getPrototypeOf, "getPrototypeOf"),
          native(Reflect.ownKeys, "ownKeys"),
          Object.getOwnPropertyNames(URL).join(",") ===
            "length,name,prototype,canParse,parse,createObjectURL,revokeObjectURL",
          dataDescriptor(window, "URL", URL),
          dataDescriptor(
            Document.prototype,
            "createElement",
            Document.prototype.createElement
          ),
          dataDescriptor(
            Element.prototype,
            "getAttribute",
            Element.prototype.getAttribute
          ),
          !("value" in userAgent),
          userAgent.enumerable,
          userAgent.configurable,
          native(userAgent.get, "get userAgent"),
          userAgent.set === undefined,
          !("value" in href),
          href.enumerable,
          href.configurable,
          native(href.get, "get href"),
          native(href.set, "set href"),
          URL.prototype.constructor === URL
        ].every(Boolean);
      })()
    "#;
    const FUNCTION_STRESS_WARMUP: &str = r#"
      (() => {
        function eA(action, cleanup) {
          try {
            throw action(), Error("");
          } catch (error) {
            return (error.name + error.message).length;
          } finally {
            cleanup && cleanup();
          }
        }
        function ZA(api, selected) {
          if (!api) return 0;
          const name = api.name;
          const instance =
            /^Screen|Navigator$/.test(name) &&
            window[name.toLowerCase()];
          const prototype =
            "prototype" in api ? api.prototype : Object.getPrototypeOf(api);
          const contribution = (
            selected && selected.length
              ? selected
              : Object.getOwnPropertyNames(prototype)
          ).reduce((sum, key) => {
            let fn;
            try {
              const descriptor =
                Object.getOwnPropertyDescriptor(prototype, key);
              fn = descriptor && (descriptor.value || descriptor.get);
            } catch (_) {
              fn = null;
            }
            if (!fn) return sum;
            const errors = [
              eA(() => fn().catch(() => {})),
              eA(() => { throw Error(Object.create(fn)); }),
              eA(() => { fn.arguments; fn.caller; }),
              eA(() => {
                fn.toString.arguments;
                fn.toString.caller;
              }),
              eA(() => Object.create(fn).toString())
            ];
            if (fn.name === "toString") {
              const parent = Object.getPrototypeOf(fn);
              errors.push(
                eA(
                  () => Object.setPrototypeOf(fn, Object.create(fn)).toString(),
                  () => Object.setPrototypeOf(fn, parent)
                ),
                eA(
                  () => Reflect.setPrototypeOf(fn, Object.create(fn)),
                  () => Object.setPrototypeOf(fn, parent)
                )
              );
            }
            return sum +
              (instance
                ? typeof Object.getOwnPropertyDescriptor(instance, key).length
                : 0) +
              Object.getOwnPropertyNames(fn).length +
              Number(errors.join("")) +
              (fn.toString() + fn.toString.toString()).length;
          }, 0);
          return (instance ? Object.getOwnPropertyNames(instance).length : 0) +
            contribution;
        }
        return [
          ZA(Function, ["call", "apply", "toString"]),
          ZA(Document, ["createElement", "createComment", "createEvent"]),
          ZA(Element, ["getAttribute", "setAttribute"]),
          ZA(URL, ["href", "toString"])
        ].join("|");
      })()
    "#;
    const WINDOW_DESCRIPTOR_SNAPSHOT: &str = r#"
      (() => {
        const keyName = key =>
          typeof key === "symbol" ? "@@" + (key.description || "") : key;
        const functionValue = value => {
          if (typeof value === "function") {
            return value.name + "/" + value.length + "/" +
              Function.prototype.toString.call(value);
          }
          if (value === null) return "null";
          const kind = typeof value;
          return kind +
            (kind !== "object" && kind !== "function"
              ? "/" + String(value)
              : "");
        };
        const descriptor = (object, key) => {
          const value = Object.getOwnPropertyDescriptor(object, key);
          return [
            keyName(key),
            "value" in value ? "d" : "a",
            Number(value.enumerable),
            Number(value.configurable),
            Number(Boolean(value.writable)),
            functionValue(value.value),
            functionValue(value.get),
            functionValue(value.set)
          ].join("\u001e");
        };
        const names = Object.getOwnPropertyNames(globalThis);
        return names.map(name => descriptor(globalThis, name)).join("\n");
      })()
    "#;
    const WINDOW_DESCRIPTOR_HASH: &str = r#"
      (() => {
        const hash = text => {
          let value = 2166136261;
          for (let index = 0; index < text.length; index += 1) {
            value = Math.imul(value ^ text.charCodeAt(index), 16777619);
          }
          return (value >>> 0).toString(16).padStart(8, "0");
        };
        const keyName = key =>
          typeof key === "symbol" ? "@@" + (key.description || "") : key;
        const functionValue = value => {
          if (typeof value === "function") {
            return value.name + "/" + value.length + "/" +
              Function.prototype.toString.call(value);
          }
          if (value === null) return "null";
          const kind = typeof value;
          return kind +
            (kind !== "object" && kind !== "function"
              ? "/" + String(value)
              : "");
        };
        const descriptor = (object, key) => {
          const value = Object.getOwnPropertyDescriptor(object, key);
          return keyName(key) + ":" +
            ("value" in value ? "d" : "a") + ":" +
            Number(value.enumerable) +
            Number(value.configurable) +
            Number(Boolean(value.writable)) + ":" +
            functionValue(value.value) + ":" +
            functionValue(value.get) + ":" +
            functionValue(value.set);
        };
        const names = Object.getOwnPropertyNames(globalThis);
        return hash(names.map(name => descriptor(globalThis, name)).join("\u001f"));
      })()
    "#;

    let mut direct = EdgeRuntime::new().expect("direct Window descriptor runtime");
    evaluate_direct(
        &mut direct,
        "Object.getOwnPropertyNames(window).length.toString()",
    );
    evaluate_direct(&mut direct, "document.createElement('span').tagName");
    evaluate_direct(&mut direct, NATIVE_SHAPE_WARMUP);
    evaluate_direct(&mut direct, FUNCTION_STRESS_WARMUP);
    let expected = evaluate_direct(&mut direct, WINDOW_DESCRIPTOR_SNAPSHOT);

    let mut traced = EdgeRuntime::new().expect("traced Window descriptor runtime");
    evaluate_direct(
        &mut traced,
        "Object.getOwnPropertyNames(window).length.toString()",
    );
    evaluate_direct(&mut traced, "document.createElement('span').tagName");
    traced
        .enable_proxy_trace()
        .expect("enable Proxy trace for Window descriptor audit");
    evaluate_traced(&mut traced, NATIVE_SHAPE_WARMUP);
    evaluate_traced(&mut traced, FUNCTION_STRESS_WARMUP);
    let actual = evaluate_traced(&mut traced, WINDOW_DESCRIPTOR_SNAPSHOT);

    if actual != expected {
        let mismatch = expected
            .lines()
            .zip(actual.lines())
            .enumerate()
            .find(|(_, (left, right))| left != right);
        panic!("Proxy trace changed a Window descriptor: {mismatch:?}");
    }

    let mut direct_hash = EdgeRuntime::new().expect("direct Window descriptor hash runtime");
    evaluate_direct(
        &mut direct_hash,
        "Object.getOwnPropertyNames(window).length.toString()",
    );
    evaluate_direct(&mut direct_hash, "document.createElement('span').tagName");
    evaluate_direct(&mut direct_hash, NATIVE_SHAPE_WARMUP);
    evaluate_direct(&mut direct_hash, FUNCTION_STRESS_WARMUP);
    let expected_hash = evaluate_direct(&mut direct_hash, WINDOW_DESCRIPTOR_HASH);
    assert_eq!(
        expected_hash, "3a6ffea6",
        "the complete direct Window descriptor hash changed"
    );

    let mut traced_hash = EdgeRuntime::new().expect("traced Window descriptor hash runtime");
    evaluate_direct(
        &mut traced_hash,
        "Object.getOwnPropertyNames(window).length.toString()",
    );
    evaluate_direct(&mut traced_hash, "document.createElement('span').tagName");
    traced_hash
        .enable_proxy_trace()
        .expect("enable Proxy trace for Window descriptor hash audit");
    evaluate_traced(&mut traced_hash, NATIVE_SHAPE_WARMUP);
    evaluate_traced(&mut traced_hash, FUNCTION_STRESS_WARMUP);
    let actual_hash = evaluate_traced(&mut traced_hash, WINDOW_DESCRIPTOR_HASH);
    assert_eq!(
        actual_hash, expected_hash,
        "Proxy trace changed the complete Window descriptor hash"
    );
}

#[test]
fn every_window_alias_keeps_identity_through_proxy_trace() {
    const ALIAS_SNAPSHOT: &str = r#"
      (() => {
        const names = Object.getOwnPropertyNames(globalThis);
        const identities = new Map();
        return names.map(name => {
          let value;
          try {
            value = globalThis[name];
          } catch (error) {
            return name + "\tthrow:" + error.name + ":" + error.message;
          }
          if ((typeof value !== "object" || value === null) &&
              typeof value !== "function") {
            return name + "\t" + typeof value + ":" + String(value);
          }
          if (!identities.has(value)) identities.set(value, name);
          return name + "\tref:" + identities.get(value);
        }).join("\n");
      })()
    "#;

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    let expected = evaluate_direct(&mut direct, ALIAS_SNAPSHOT);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced
        .enable_proxy_trace()
        .expect("enable Proxy trace for alias audit");
    let actual = evaluate_traced(&mut traced, ALIAS_SNAPSHOT);

    assert_eq!(
        actual, expected,
        "Proxy trace changed Window alias identity"
    );
}

#[test]
fn function_carriers_and_derived_construction_keep_edge_semantics() {
    const MUTATION_SNAPSHOT: &str = r#"
      (() => {
        const O = (0, eval)("Object");
        const R = (0, eval)("Reflect");
        class EdgeAuditURL extends URL {}
        const derived = new EdgeAuditURL("/derived", location.href);
        const parent = O.create(null);
        const originalParent = O.getPrototypeOf(URL);
        const changed = R.setPrototypeOf(URL, parent);
        const changedShape = O.getPrototypeOf(URL) === parent;
        const restored = R.setPrototypeOf(URL, originalParent);
        const restoredShape =
          O.getPrototypeOf(URL) === Function.prototype;
        const assignedObject = {};
        URL.edgeAssignedObject = assignedObject;
        const assignedIdentity =
          URL.edgeAssignedObject === assignedObject;
        delete URL.edgeAssignedObject;
        const definedObject = {};
        Object.defineProperty(URL, "edgeDefinedObject", {
          value: definedObject,
          configurable: true
        });
        const definedIdentity =
          URL.edgeDefinedObject === definedObject;
        delete URL.edgeDefinedObject;
        const sourceObject = { edgeAssignedByObject: definedObject };
        Object.assign(URL, sourceObject);
        const objectAssignIdentity =
          URL.edgeAssignedByObject === definedObject;
        delete URL.edgeAssignedByObject;
        const defined = R.defineProperty(URL, "edgeAuditValue", {
          value: URL.prototype,
          enumerable: true,
          configurable: false,
          writable: false
        });
        const descriptor =
          O.getOwnPropertyDescriptor(URL, "edgeAuditValue");
        const keysBefore = R.ownKeys(URL).join(",");
        const extensibleBefore = R.isExtensible(URL);
        const prevented = R.preventExtensions(URL);
        const extensibleAfter = R.isExtensible(URL);
        const keysAfter = R.ownKeys(URL).join(",");
        const deleted = R.deleteProperty(URL, "edgeAuditValue");
        const setAfter = R.set(URL, "edgeAuditAfter", 1);
        return [
          derived instanceof EdgeAuditURL,
          derived instanceof URL,
          O.getPrototypeOf(derived) === EdgeAuditURL.prototype,
          derived.constructor === EdgeAuditURL,
          derived.href,
          changed,
          changedShape,
          restored,
          restoredShape,
          assignedIdentity,
          definedIdentity,
          objectAssignIdentity,
          defined,
          descriptor.value === URL.prototype,
          descriptor.enumerable,
          descriptor.configurable,
          descriptor.writable,
          keysBefore,
          extensibleBefore,
          prevented,
          extensibleAfter,
          keysAfter,
          deleted,
          setAfter,
          Function.prototype.toString.call(URL),
          URL.toString(),
          URL.toString.toString()
        ].join("|");
      })()
    "#;

    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    let expected = evaluate_direct(&mut direct, MUTATION_SNAPSHOT);

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced
        .enable_proxy_trace()
        .expect("enable Proxy trace for mutation audit");
    let actual = evaluate_traced(&mut traced, MUTATION_SNAPSHOT);

    assert_eq!(
        actual, expected,
        "Proxy trace changed function mutation or derived-construction semantics"
    );
}

#[test]
fn every_reachable_window_api_is_recorded_by_the_shared_proxy_handler() {
    const TRACE_AUDIT: &str = r#"
      (() => {
        const names = Object.getOwnPropertyNames(globalThis);
        const seen = new Set();
        const called = new Set();
        const queue = [];
        let propertyReads = 0;
        let functionCalls = 0;

        const enqueue = (value, expand) => {
          if ((typeof value !== "object" || value === null) &&
              typeof value !== "function") return;
          if (value === globalThis) return;
          if (typeof value === "function" && !called.has(value)) {
            called.add(value);
            functionCalls += 1;
            try {
              value.call(undefined);
            } catch (_) {}
          }
          if (!expand || seen.has(value)) return;
          seen.add(value);
          queue.push(value);
        };

        for (const name of names) {
          try {
            enqueue(globalThis[name], true);
          } catch (_) {}
        }

        for (let cursor = 0; cursor < queue.length; cursor += 1) {
          if (cursor > 20000) {
            throw new Error("trace API graph exceeded limit");
          }
          const value = queue[cursor];
          let keys;
          try {
            keys = Reflect.ownKeys(value);
          } catch (_) {
            continue;
          }
          let prototype;
          try {
            prototype = Object.getPrototypeOf(value);
          } catch (_) {
            prototype = null;
          }
          enqueue(prototype, true);
          for (const key of keys) {
            propertyReads += 1;
            try {
              void value[key];
            } catch (_) {}
            let descriptor;
            try {
              descriptor =
                Object.getOwnPropertyDescriptor(value, key);
            } catch (_) {
              continue;
            }
            if (!descriptor) continue;
            if ("value" in descriptor) {
              const child = descriptor.value;
              enqueue(
                child,
                typeof child !== "function" ||
                  Object.prototype.hasOwnProperty.call(
                    child,
                    "prototype"
                  )
              );
            } else {
              enqueue(descriptor.get, false);
              enqueue(descriptor.set, false);
            }
          }
        }
        return [
          names.length,
          propertyReads,
          functionCalls,
          seen.size
        ].join("|");
      })()
    "#;

    let mut names_runtime = EdgeRuntime::new().expect("Window names runtime");
    let names = evaluate_direct(
        &mut names_runtime,
        "Object.getOwnPropertyNames(globalThis).join('\\n')",
    )
    .lines()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    assert_eq!(names.len(), 1232);

    let mut traced = EdgeRuntime::new().expect("full trace audit runtime");
    traced
        .enable_proxy_trace()
        .expect("enable Proxy trace for all-API recording audit");
    let counts = evaluate_direct(&mut traced, TRACE_AUDIT);
    let counts = counts
        .split('|')
        .map(|value| value.parse::<usize>().expect("numeric trace count"))
        .collect::<Vec<_>>();
    assert_eq!(counts.len(), 4);
    assert_eq!(counts[0], 1232);
    assert!(counts[1] > 4000, "too few reachable API properties");
    assert!(counts[2] > 1000, "too few reachable API functions");
    assert_eq!(
        evaluate_direct(&mut traced, "new URL('https://example.com/').href"),
        "https://example.com/"
    );

    let trace = traced.proxy_trace();
    for name in names {
        let expected = format!("window.{name}");
        assert!(
            trace
                .iter()
                .any(|entry| entry.operation == "get" && entry.api == expected),
            "Window API access was not recorded: {expected}"
        );
    }
    let prototype_property_gets = trace
        .iter()
        .filter(|entry| entry.operation == "get" && entry.api.contains(".prototype."))
        .count();
    let calls = trace
        .iter()
        .filter(|entry| entry.operation == "call")
        .count();
    assert!(
        prototype_property_gets > 100,
        "too few native prototype property reads were recorded"
    );
    assert!(calls > 100, "too few native API calls were recorded");
    assert!(
        trace.iter().any(|entry| entry.operation == "construct"),
        "constructor calls were not recorded"
    );
}

#[test]
fn native_trace_records_reflect_calls_without_proxy_meta_traps() {
    const OPERATIONS: &str = r#"
      (() => {
        const R = (0, eval)("Reflect");
        const originalParent = R.getPrototypeOf(URL);
        R.setPrototypeOf(URL, null);
        R.setPrototypeOf(URL, originalParent);
        R.defineProperty(URL, "edgeOperation", {
          value: 1,
          configurable: true,
          enumerable: true,
          writable: true
        });
        R.has(URL, "edgeOperation");
        R.get(URL, "edgeOperation");
        R.set(URL, "edgeOperation", 2);
        R.ownKeys(URL);
        R.getOwnPropertyDescriptor(URL, "edgeOperation");
        R.isExtensible(URL);
        try {
          R.apply(URL, undefined, ["https://example.com/"]);
        } catch {}
        R.deleteProperty(URL, "edgeOperation");
        R.preventExtensions(URL);
        return "recorded";
      })()
    "#;

    let mut runtime = EdgeRuntime::new().expect("Proxy operation runtime");
    runtime
        .enable_proxy_trace()
        .expect("enable Proxy trace for operation audit");
    assert_eq!(evaluate_direct(&mut runtime, OPERATIONS), "recorded");
    let trace = runtime.proxy_trace();
    for api in [
        "Reflect.getPrototypeOf",
        "Reflect.setPrototypeOf",
        "Reflect.ownKeys",
        "Reflect.getOwnPropertyDescriptor",
        "Reflect.preventExtensions",
    ] {
        assert!(
            trace
                .iter()
                .any(|entry| entry.operation == "call" && entry.api == api),
            "native Reflect call was not recorded: {api}"
        );
    }
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "get" && entry.api == "window.URL" })
    );
}

#[test]
fn readonly_invariants_and_user_setter_receivers_do_not_leak_raw_targets() {
    let source = r#"
      (() => {
        let leaked;
        Object.defineProperty(Element.prototype, "__edgeReceiver", {
          configurable: true,
          set() { leaked = this; }
        });
        const element = document.createElement("div");
        element.__edgeReceiver = 1;
        const receiverMatches = leaked === element;
        leaked.id = "still-traced";
        delete Element.prototype.__edgeReceiver;
        return [
          receiverMatches,
          element.id,
          typeof location.assign,
          typeof location.replace,
          typeof location.reload,
          typeof location.toString,
          typeof Element.prototype
        ].join("|");
      })()
    "#;
    let expected = "true|still-traced|function|function|function|function|object";
    let mut direct = EdgeRuntime::new().expect("direct invariant runtime");
    assert_eq!(evaluate_direct(&mut direct, source), expected);
    let mut traced = EdgeRuntime::new().expect("traced invariant runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(evaluate_direct(&mut traced, source), expected);
    assert!(traced.proxy_trace().iter().any(|entry| {
        entry.operation == "set"
            && entry.api.ends_with(".id")
            && entry.arguments == "\"still-traced\""
    }));
}

#[test]
fn native_trace_keeps_strict_mode_errors_and_illegal_receivers_exact() {
    const ERROR_SNAPSHOT: &str = r#"
      (() => {
        const capture = action => {
          try {
            action();
            return "ok";
          } catch (error) {
            return [
              error.name,
              error.message,
              Object.getPrototypeOf(error).constructor.name
            ].join("\u001e");
          }
        };
        const strictSet = Function(
          "object",
          "key",
          "value",
          '"use strict"; object[key] = value;'
        );
        const element = document.createElement("div");
        Object.preventExtensions(URL);
        return [
          capture(() => strictSet(element, "tagName", "SPAN")),
          capture(() => strictSet(URL, "__edgeTraceNewProperty", 1)),
          capture(() =>
            Document.prototype.createElement.call({}, "section")
          ),
          capture(() =>
            EventTarget.prototype.addEventListener.call(
              {},
              "edge",
              () => {}
            )
          ),
          capture(() => URL("https://example.com/"))
        ].join("\u001f");
      })()
    "#;

    let mut direct = EdgeRuntime::new().expect("direct strict-mode runtime");
    let expected = evaluate_direct(&mut direct, ERROR_SNAPSHOT);

    let mut traced = EdgeRuntime::new().expect("native-traced strict-mode runtime");
    traced
        .enable_native_trace()
        .expect("enable native trace for strict-mode audit");
    let actual = evaluate_direct(&mut traced, ERROR_SNAPSHOT);

    assert_eq!(
        actual, expected,
        "native trace changed an exception name, message, or prototype"
    );
}

#[test]
fn native_trace_does_not_add_user_proxy_traps_or_construct_javascript_proxies() {
    const PREPARE: &str = r#"
      (() => {
        const NativeProxy = Proxy;
        const counts = {
          get: 0,
          getPrototypeOf: 0,
          ownKeys: 0,
          construct: 0
        };
        const value = new NativeProxy(
          {
            toString() {
              return "proxy-value";
            }
          },
          {
            get(target, key, receiver) {
              counts.get += 1;
              return Reflect.get(target, key, receiver);
            },
            getPrototypeOf(target) {
              counts.getPrototypeOf += 1;
              return Reflect.getPrototypeOf(target);
            },
            ownKeys(target) {
              counts.ownKeys += 1;
              return Reflect.ownKeys(target);
            }
          }
        );
        const instrumentedProxyConstructor = new NativeProxy(
          NativeProxy,
          {
            construct(target, argumentsList, newTarget) {
              counts.construct += 1;
              return Reflect.construct(
                target,
                argumentsList,
                newTarget
              );
            }
          }
        );
        const exercise = () => {
          const before = [
            counts.get,
            counts.getPrototypeOf,
            counts.ownKeys
          ];
          const element = document.createElement("div");
          element.setAttribute("data-value", value);
          const after = [
            counts.get,
            counts.getPrototypeOf,
            counts.ownKeys
          ];
          return [
            after[0] - before[0],
            after[1] - before[1],
            after[2] - before[2],
            element.getAttribute("data-value")
          ].join("|");
        };
        Object.defineProperties(globalThis, {
          __edgeNativeProxy: {
            value: NativeProxy,
            configurable: true
          },
          __edgeProxyCounts: {
            value: counts,
            configurable: true
          },
          __edgeProxyExercise: {
            value: exercise,
            configurable: true
          }
        });
        globalThis.Proxy = instrumentedProxyConstructor;
        return exercise();
      })()
    "#;
    const EXERCISE: &str = r#"
      [
        __edgeProxyExercise(),
        __edgeProxyCounts.construct
      ].join("\u001f")
    "#;
    const CLEANUP: &str = r#"
      (() => {
        globalThis.Proxy = __edgeNativeProxy;
        delete globalThis.__edgeNativeProxy;
        delete globalThis.__edgeProxyCounts;
        delete globalThis.__edgeProxyExercise;
        return "clean";
      })()
    "#;

    let mut runtime = EdgeRuntime::new().expect("user Proxy trap runtime");
    let expected = evaluate_direct(&mut runtime, PREPARE);
    runtime
        .enable_native_trace()
        .expect("enable native trace with instrumented Proxy global");
    let actual = evaluate_direct(&mut runtime, EXERCISE);
    let mut parts = actual.split('\u{1f}');
    assert_eq!(
        parts.next(),
        Some(expected.as_str()),
        "trace argument rendering invoked an additional user Proxy trap"
    );
    assert_eq!(
        parts.next(),
        Some("0"),
        "native trace constructed a JavaScript Proxy"
    );
    assert_eq!(evaluate_direct(&mut runtime, CLEANUP), "clean");
}

#[test]
fn native_trace_enablement_keeps_descriptors_prototypes_and_native_text_stable() {
    const SHAPE_SNAPSHOT: &str = r#"
      (() => {
        const descriptor = (owner, key) => {
          const value = Object.getOwnPropertyDescriptor(owner, key);
          return [
            "value" in value ? "data" : "accessor",
            Number(value.enumerable),
            Number(value.configurable),
            Number(Boolean(value.writable)),
            typeof value.value === "function"
              ? Function.prototype.toString.call(value.value)
              : typeof value.value,
            typeof value.get === "function"
              ? Function.prototype.toString.call(value.get)
              : typeof value.get,
            typeof value.set === "function"
              ? Function.prototype.toString.call(value.set)
              : typeof value.set
          ].join("\u001e");
        };
        return [
          Object.getOwnPropertyNames(globalThis).length,
          Reflect.ownKeys(Document.prototype).map(String).join(","),
          Reflect.ownKeys(Element.prototype).map(String).join(","),
          descriptor(globalThis, "URL"),
          descriptor(Document.prototype, "createElement"),
          descriptor(Element.prototype, "tagName"),
          descriptor(Navigator.prototype, "userAgent"),
          Object.getPrototypeOf(window) === Window.prototype,
          Object.getPrototypeOf(HTMLDocument.prototype) ===
            Document.prototype,
          Object.getPrototypeOf(Document.prototype) === Node.prototype,
          Object.getPrototypeOf(Node.prototype) ===
            EventTarget.prototype,
          Object.getPrototypeOf(URL) === Function.prototype,
          URL.prototype.constructor === URL,
          Function.prototype.toString.call(URL),
          Function.prototype.toString.call(
            Document.prototype.createElement
          ),
          Function.prototype.toString.call(
            Object.getOwnPropertyDescriptor(
              Navigator.prototype,
              "userAgent"
            ).get
          )
        ].join("\u001f");
      })()
    "#;

    let mut runtime = EdgeRuntime::new().expect("native trace shape runtime");
    let expected = evaluate_direct(&mut runtime, SHAPE_SNAPSHOT);
    assert!(runtime.native_trace().is_empty());

    runtime
        .enable_native_trace()
        .expect("enable native trace for shape audit");
    let actual = evaluate_direct(&mut runtime, SHAPE_SNAPSHOT);
    assert_eq!(
        actual, expected,
        "enabling native trace changed a descriptor, prototype, or function shape"
    );
    assert!(
        !runtime.native_trace().is_empty(),
        "enabled native trace did not record the shape audit"
    );

    runtime.disable_native_trace();
    runtime.clear_native_trace();
    assert_eq!(
        evaluate_direct(
            &mut runtime,
            "document.createElement('div').getAttribute('id')"
        ),
        "null"
    );
    assert!(
        runtime.native_trace().is_empty(),
        "disabled native trace recorded an API operation"
    );
}

#[test]
fn dynamic_webidl_accessors_and_iterators_use_the_native_trace_trampoline() {
    const DYNAMIC_API_SNAPSHOT: &str = r#"
      (() => {
        const capture = action => {
          try {
            action();
            return "ok";
          } catch (error) {
            return error.name + ":" + error.message;
          }
        };
        const matrix = new DOMMatrixReadOnly();
        const input = document.createElement("input");
        input.type = "file";
        const files = input.files;
        const children = document.children;
        const attributes = document.documentElement.attributes;
        const handler = () => {};
        window.onload = handler;
        const elementInternals =
          Object.getOwnPropertyDescriptor(
            ElementInternals.prototype,
            "ariaAtomic"
          );
        const xrHandler =
          Object.getOwnPropertyDescriptor(
            XRSession.prototype,
            "onend"
          );
        const snapshot = [
          matrix.a,
          Array.from(files).length,
          Array.from(children).length,
          Array.from(attributes).length,
          location.href,
          window.onload === handler,
          capture(() => elementInternals.get.call({})),
          capture(() => elementInternals.set.call({}, "true")),
          capture(() => xrHandler.get.call({})),
          capture(() => xrHandler.set.call({}, null)),
          Function.prototype.toString.call(
            Object.getOwnPropertyDescriptor(
              DOMMatrixReadOnly.prototype,
              "a"
            ).get
          ),
          Function.prototype.toString.call(
            FileList.prototype[Symbol.iterator]
          ),
          Function.prototype.toString.call(
            HTMLCollection.prototype[Symbol.iterator]
          ),
          Function.prototype.toString.call(
            NamedNodeMap.prototype[Symbol.iterator]
          ),
          Function.prototype.toString.call(
            Object.getOwnPropertyDescriptor(window, "onload").set
          )
        ].join("\u001f");
        window.onload = null;
        return snapshot;
      })()
    "#;

    let mut direct = EdgeRuntime::new().expect("direct dynamic WebIDL runtime");
    let expected = evaluate_direct(&mut direct, DYNAMIC_API_SNAPSHOT);

    let mut traced = EdgeRuntime::new().expect("native-traced dynamic WebIDL runtime");
    traced
        .enable_native_trace()
        .expect("enable native trace for dynamic WebIDL audit");
    let actual = evaluate_direct(&mut traced, DYNAMIC_API_SNAPSHOT);
    assert_eq!(
        actual, expected,
        "native callback migration changed a dynamic WebIDL API shape or result"
    );

    let trace = traced.native_trace();
    for api_suffix in [
        ".a",
        ".files.values",
        ".children.values",
        ".attributes.values",
        ".location.href",
        ".onload",
        "ElementInternals.prototype.get ariaAtomic",
        "ElementInternals.prototype.set ariaAtomic",
        "XRSession.prototype.get onend",
        "XRSession.prototype.set onend",
    ] {
        assert!(
            trace.iter().any(|entry| entry.api.ends_with(api_suffix)),
            "dynamic WebIDL API bypassed native trace: {api_suffix}"
        );
    }
}

#[test]
fn layout_and_observer_calls_keep_edge_shape_with_proxy_trace_enabled() {
    const SOURCE: &str = r#"
      (async () => {
        const element = document.createElement("div");
        element.style.cssText =
          "position:fixed;left:10px;top:20px;width:100px;height:50px;padding:5px;border:2px solid";
        document.body.appendChild(element);
        const resize = [];
        const resizeObserver = new ResizeObserver(entries => {
          resize.push(entries[0].borderBoxSize[0].inlineSize);
        });
        const intersections = [];
        const intersectionObserver = new IntersectionObserver(entries => {
          intersections.push(entries[0].intersectionRatio);
        });
        resizeObserver.observe(element);
        intersectionObserver.observe(element);
        await Promise.resolve();
        const rect = element.getBoundingClientRect();
        const range = document.createRange();
        range.selectNode(element);
        const rangeRect = range.getBoundingClientRect();
        const rangeRects = range.getClientRects();
        const image = document.createElement("img");
        image.style.cssText =
          "position:fixed;left:400px;top:300px;width:10px;height:20px";
        document.body.appendChild(image);
        const documentHit =
          document.elementFromPoint(11, 21) === element;
        const documentHits = document.elementsFromPoint(11, 21);
        const shadowHost = document.createElement("div");
        shadowHost.style.cssText =
          "position:fixed;left:500px;top:200px;width:100px;height:80px";
        const shadow = shadowHost.attachShadow({ mode: "open" });
        const shadowChild = document.createElement("span");
        shadowChild.style.cssText =
          "position:absolute;left:0;top:0;width:50px;height:40px";
        shadow.appendChild(shadowChild);
        document.body.appendChild(shadowHost);
        const shadowHit =
          shadow.elementFromPoint(510, 210) === shadowChild;
        const shadowHits = shadow.elementsFromPoint(510, 210);
        element.scrollIntoView({ block: "nearest", inline: "nearest" });
        element.scrollIntoViewIfNeeded(false);
        element.scrollLeft = 1000;
        const output = [
          rect.width,
          rect.height,
          element.clientWidth,
          element.offsetWidth,
          element.scrollWidth,
          element.scrollLeft,
          element.getClientRects().length,
          rangeRect.width,
          rangeRect.height,
          Object.prototype.toString.call(rangeRects),
          rangeRects.length,
          image.x,
          image.y,
          image.width,
          image.height,
          documentHit,
          documentHits[0] === element,
          shadowHit,
          shadowHits[0] === shadowChild,
          resize.join(","),
          intersections.join(","),
          Function.prototype.toString.call(
            Element.prototype.getBoundingClientRect
          ),
          Function.prototype.toString.call(
            ResizeObserver.prototype.observe
          ),
          Function.prototype.toString.call(
            IntersectionObserver.prototype.observe
          ),
          Function.prototype.toString.call(
            Range.prototype.getBoundingClientRect
          ),
          Function.prototype.toString.call(
            Range.prototype.getClientRects
          ),
          Object.getOwnPropertyDescriptor(
            Element.prototype,
            "clientWidth"
          ).enumerable,
          Object.getPrototypeOf(resizeObserver) ===
            ResizeObserver.prototype,
          Object.getPrototypeOf(intersectionObserver) ===
            IntersectionObserver.prototype
        ].join("|");
        resizeObserver.disconnect();
        intersectionObserver.disconnect();
        element.remove();
        image.remove();
        shadowHost.remove();
        return output;
      })()
    "#;

    let mut direct = EdgeRuntime::new().expect("direct layout runtime");
    let expected = evaluate_direct(&mut direct, SOURCE);

    let mut traced = EdgeRuntime::new().expect("traced layout runtime");
    traced
        .enable_proxy_trace()
        .expect("enable Proxy trace for layout APIs");
    let actual = evaluate_direct(&mut traced, SOURCE);
    assert_eq!(
        actual, expected,
        "Proxy trace changed geometry, observer delivery, descriptors, or native function text"
    );

    let trace = traced.native_trace();
    for api_name in [
        "getBoundingClientRect",
        "getClientRects",
        "clientWidth",
        "offsetWidth",
        "scrollWidth",
        "scrollLeft",
        "Range",
        "HTMLImageElement.prototype.x",
        "HTMLImageElement.prototype.y",
        "HTMLImageElement.prototype.width",
        "HTMLImageElement.prototype.height",
        "Document.prototype.elementFromPoint",
        "Document.prototype.elementsFromPoint",
        "ShadowRoot.prototype.elementFromPoint",
        "ShadowRoot.prototype.elementsFromPoint",
        "Element.prototype.scrollIntoView",
        "Element.prototype.scrollIntoViewIfNeeded",
        "ResizeObserver",
        "IntersectionObserver",
        "observe",
        "disconnect",
    ] {
        assert!(
            trace.iter().any(|entry| entry.api.contains(api_name)),
            "layout API bypassed native trace: {api_name}"
        );
    }
}
