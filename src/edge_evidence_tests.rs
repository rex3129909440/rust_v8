use crate::{EdgeRuntime, Evaluation};
use std::collections::HashMap;

const EDGE_HTTPS_WINDOW: &str = include_str!("../tests/evidence/edge_https_window.tsv");
const EDGE_HTTPS_INTERFACES: &str = include_str!("../tests/evidence/edge_https_interfaces.tsv");
const EDGE_HTTPS_METADATA: &str = include_str!("../tests/evidence/edge_https_metadata.tsv");
const EDGE_HTTPS_BEHAVIOR: &str = include_str!("../tests/evidence/edge_https_behavior.tsv");
const EDGE_HTTPS_WORKER: &str = include_str!("../tests/evidence/edge_https_worker.tsv");

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

fn edge_window_names() -> Vec<&'static str> {
    EDGE_HTTPS_WINDOW
        .lines()
        .skip(1)
        .map(|line| line.split('\t').nth(1).expect("Edge Window evidence name"))
        .collect()
}

#[test]
fn window_own_property_names_and_order_match_edge_https_evidence() {
    let expected = edge_window_names();
    assert_eq!(expected.len(), 1232, "{EDGE_HTTPS_METADATA}");

    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let actual = text(
        &mut runtime,
        "Object.getOwnPropertyNames(globalThis).join('\\n')",
    );
    let actual = actual.lines().collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn interface_prototype_descriptors_match_edge_https_evidence() {
    let expected = EDGE_HTTPS_INTERFACES.lines().skip(1).collect::<Vec<_>>();
    assert_eq!(expected.len(), 9908, "{EDGE_HTTPS_METADATA}");

    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let actual = text(
        &mut runtime,
        r#"
        (() => {
          const rows = [];
          for (const interfaceName of Object.getOwnPropertyNames(globalThis)) {
            let constructor;
            try {
              constructor = globalThis[interfaceName];
            } catch {
              continue;
            }
            if (typeof constructor !== "function" || !constructor.prototype) {
              continue;
            }
            const constructorParent =
              Object.getPrototypeOf(constructor)?.name ??
              Object.prototype.toString.call(Object.getPrototypeOf(constructor));
            const prototypeParent =
              Object.getPrototypeOf(constructor.prototype)?.constructor?.name ?? "";
            for (const member of Reflect.ownKeys(constructor.prototype)) {
              const descriptor = Object.getOwnPropertyDescriptor(
                constructor.prototype,
                member
              );
              const descriptorKind =
                descriptor && ("get" in descriptor || "set" in descriptor)
                  ? "accessor"
                  : "data";
              const value =
                descriptorKind === "accessor"
                  ? descriptor?.get ?? descriptor?.set
                  : descriptor?.value;
              const isFunction = typeof value === "function";
              rows.push([
                interfaceName,
                constructorParent,
                prototypeParent,
                typeof member === "symbol" ? member.toString() : member,
                descriptorKind,
                String(Boolean(descriptor?.enumerable)),
                String(Boolean(descriptor?.configurable)),
                String(Boolean(descriptor?.writable)),
                typeof value,
                isFunction ? value.name : "",
                isFunction ? value.length : "",
                isFunction
                  ? String(
                      Function.prototype.toString
                        .call(value)
                        .includes("[native code]")
                    )
                  : ""
              ]);
            }
          }
          const escape = value =>
            String(value ?? "")
              .replaceAll("\\", "\\\\")
              .replaceAll("\t", "\\t")
              .replaceAll("\r", "\\r")
              .replaceAll("\n", "\\n");
          return rows
            .map(row => row.map(escape).join("\t"))
            .join("\n");
        })()
        "#,
    );
    let actual = actual.lines().collect::<Vec<_>>();
    let expected_by_member = expected
        .iter()
        .map(|line| (interface_member_key(line), *line))
        .collect::<HashMap<_, _>>();
    let actual_by_member = actual
        .iter()
        .map(|line| (interface_member_key(line), *line))
        .collect::<HashMap<_, _>>();
    let missing = expected_by_member
        .keys()
        .filter(|key| !actual_by_member.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_member
        .keys()
        .filter(|key| !expected_by_member.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        actual.len(),
        expected.len(),
        "interface descriptor row count differs from Edge HTTPS evidence; missing={missing:?}; extra={extra:?}"
    );
    let mismatches = actual
        .iter()
        .zip(&expected)
        .enumerate()
        .filter(|(_, (actual, expected))| actual != expected)
        .map(|(index, (actual, expected))| {
            format!(
                "row {}:\n  actual:   {actual}\n  expected: {expected}",
                index + 2
            )
        })
        .collect::<Vec<_>>();
    assert!(
        mismatches.is_empty(),
        "{} interface descriptor mismatches:\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(100)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn interface_member_key(line: &str) -> String {
    let mut columns = line.split('\t');
    let interface = columns.next().unwrap_or_default();
    let member = columns.nth(2).unwrap_or_default();
    format!("{interface}.{member}")
}

#[test]
fn targeted_edge_150_behavior_differences_are_implemented() {
    let expected = EDGE_HTTPS_BEHAVIOR
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut columns = line.split('\t');
            let api = columns.next().unwrap_or_default();
            let _case_name = columns.next();
            let value = columns.next().unwrap_or_default();
            matches!(
                api,
                "ProcessingInstruction" | "HTMLScriptElement" | "Worker"
            )
            .then_some(value)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let actual = text(
        &mut runtime,
        r#"
        (() => {
          const values = [];
          const capture = operation => {
            try {
              values.push(String(operation()));
            } catch (error) {
              values.push(`throws ${error?.name ?? "Error"}:${error?.message ?? ""}`);
            }
          };
          capture(() => {
            const order = ["before"];
            window.__edgeCurrentScriptValues = order;
            const element = document.createElement("script");
            element.id = "edge-current-script-inline";
            element.text =
              "__edgeCurrentScriptValues.push(" +
              "[document.currentScript===null," +
              "document.currentScript?.id," +
              "document.currentScript?.tagName," +
              "document.currentScript?.isConnected].join(','))";
            document.head.appendChild(element);
            order.push("after");
            element.remove();
            delete window.__edgeCurrentScriptValues;
            return order.join("|");
          });
          const xml = document.implementation.createDocument("", "", null);
          const instruction = xml.createProcessingInstruction(
            "xml-stylesheet",
            'href="theme.css" media="screen" disabled'
          );
          capture(() => instruction.data);
          capture(() => instruction.getAttribute("href"));
          capture(() => instruction.getAttribute("media"));
          capture(() => instruction.getAttribute("disabled"));
          capture(() => instruction.getAttributeNames().join(","));
          capture(() => instruction.hasAttribute("href"));
          capture(() => instruction.hasAttributes());
          capture(() => {
            instruction.setAttribute("title", "hello world");
            return `${instruction.data}|${instruction.getAttribute("title")}`;
          });
          capture(() =>
            `${instruction.toggleAttribute("disabled")}|${instruction.data}`
          );
          capture(() =>
            `${instruction.toggleAttribute("disabled")}|${instruction.data}`
          );
          capture(() => {
            instruction.removeAttribute("media");
            return `${instruction.data}|${instruction.getAttribute("media")}`;
          });
          capture(() => instruction.setAttribute("bad name", "value"));

          const script = document.createElement("script");
          capture(() => [script.text, script.textContent, script.innerText].join("|"));
          capture(() => {
            script.textContent = "one";
            return [script.text, script.textContent, script.innerText].join("|");
          });
          capture(() => {
            script.innerText = "two";
            return [script.text, script.textContent, script.innerText].join("|");
          });
          capture(() => {
            script.text = "three";
            return [script.text, script.textContent, script.innerText].join("|");
          });

          const worker = new Worker(
            "data:text/javascript,self.onmessage%20%3D%20()%20%3D%3E%20%7B%7D"
          );
          capture(() =>
            Object.prototype.hasOwnProperty.call(
              Worker.prototype,
              "onmessageerror"
            )
          );
          capture(() => "onmessageerror" in worker);
          capture(() => typeof worker.onmessageerror);
          worker.terminate();
          return values.join("\n");
        })()
        "#,
    );
    assert_eq!(actual, expected);
}

#[test]
fn dedicated_worker_surface_matches_edge_https_evidence() {
    let expected = EDGE_HTTPS_WORKER.lines().skip(1).collect::<Vec<_>>();
    assert_eq!(expected.len(), 718, "{EDGE_HTTPS_METADATA}");
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const source = `
            const rows = [];
            const own = Object.getOwnPropertyNames(self);
            for (const name of own) {
              let valueType = "throws";
              try { valueType = typeof self[name]; } catch {}
              rows.push(["global", 0, "", name, valueType]);
            }
            let value = self;
            let depth = 0;
            while (value && depth < 16) {
              const tag = Object.prototype.toString.call(value);
              for (const name of Object.getOwnPropertyNames(value)) {
                const descriptor = Object.getOwnPropertyDescriptor(value, name);
                const member = descriptor && ("value" in descriptor)
                  ? descriptor.value
                  : descriptor?.get ?? descriptor?.set;
                rows.push(["prototype", depth, tag, name, typeof member]);
              }
              value = Object.getPrototypeOf(value);
              depth++;
            }
            const escape = value =>
              String(value ?? "")
                .replaceAll("\\\\", "\\\\\\\\")
                .replaceAll("\\t", "\\\\t")
                .replaceAll("\\r", "\\\\r")
                .replaceAll("\\n", "\\\\n");
            postMessage(
              rows.map(row => row.map(escape).join("\\t")).join("\\n")
            );
          `;
          const worker = new Worker(
            "data:text/javascript," + encodeURIComponent(source)
          );
          globalThis.workerEdgeEvidence = "pending";
          worker.onmessage = event => {
            workerEdgeEvidence = event.data;
            worker.terminate();
          };
        })()
        "#,
    );
    let actual_text = text(&mut runtime, "workerEdgeEvidence");
    let actual = actual_text.lines().collect::<Vec<_>>();
    let expected_global = expected
        .iter()
        .copied()
        .filter(|line| line.starts_with("global\t"))
        .collect::<Vec<_>>();
    let actual_global = actual
        .iter()
        .copied()
        .filter(|line| line.starts_with("global\t"))
        .collect::<Vec<_>>();
    assert_eq!(
        actual_global, expected_global,
        "Worker global own-property order differs from Edge evidence"
    );
    let expected_by_key = expected
        .iter()
        .map(|line| (worker_member_key(line), *line))
        .collect::<HashMap<_, _>>();
    let actual_by_key = actual
        .iter()
        .map(|line| (worker_member_key(line), *line))
        .collect::<HashMap<_, _>>();
    let missing = expected_by_key
        .keys()
        .filter(|key| !actual_by_key.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual_by_key
        .keys()
        .filter(|key| !expected_by_key.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        actual.len(),
        expected.len(),
        "Worker evidence row count differs; missing={missing:?}; extra={extra:?}"
    );
    assert_eq!(
        actual_by_key, expected_by_key,
        "Worker member depth, tag, or value type differs from Edge evidence"
    );
}

fn worker_member_key(line: &str) -> String {
    let mut columns = line.split('\t');
    let section = columns.next().unwrap_or_default();
    let depth = columns.next().unwrap_or_default();
    let tag = columns.next().unwrap_or_default();
    let name = columns.next().unwrap_or_default();
    format!("{section}.{depth}.{tag}.{name}")
}
