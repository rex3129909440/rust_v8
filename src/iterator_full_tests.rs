use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => value,
        value => value.to_string(),
    }
}

#[test]
fn edge_150_window_iterable_prototype_matrix_is_complete() {
    let mut runtime = EdgeRuntime::new().expect("runtime");
    let failures = text(
        &mut runtime,
        r#"
        (() => {
          const entriesAliases = [
            "AudioParamMap", "EventCounts", "FormData", "Headers",
            "HighlightRegistry", "KeyboardLayoutMap", "MIDIInputMap",
            "MIDIOutputMap", "Map", "MediaKeyStatusMap", "RTCStatsReport",
            "StylePropertyMapReadOnly", "URLSearchParams", "XRHand"
          ];
          const valuesAliases = [
            "Array", "CSSNumericArray", "CSSTransformValue",
            "CSSUnparsedValue", "CustomStateSet", "DOMTokenList",
            "GPUSupportedFeatures", "Highlight", "NodeList", "RadioNodeList",
            "Set", "TimelineTriggerRangeList", "ViewTransitionTypeSet",
            "WGSLLanguageFeatures", "XRAnchorSet", "XRInputSourceArray",
            "XRPlaneSet"
          ];
          const ownIterators = [
            "CSSKeyframesRule", "CSSRuleList", "CSSStyleDeclaration",
            "DOMRectList", "DOMStringList", "DataTransferItemList", "FileList",
            "HTMLAllCollection", "HTMLCollection", "HTMLFormControlsCollection",
            "HTMLFormElement", "HTMLOptionsCollection", "HTMLSelectElement",
            "ImageTrackList", "Iterator", "MediaList", "MimeTypeArray",
            "NamedNodeMap", "Plugin", "PluginArray", "SVGLengthList",
            "SVGNumberList", "SVGPointList", "SVGStringList", "SVGTransformList",
            "SourceBufferList", "SpeechGrammarList", "String", "StyleSheetList",
            "TextTrackCueList", "TextTrackList", "TouchList",
            "webkitSpeechGrammarList"
          ];
          const failures = [];
          const inspect = (realm, name, alias) => {
            const constructor = realm[name];
            if (typeof constructor !== "function" || !constructor.prototype) {
              failures.push(name + ":constructor");
              return;
            }
            const prototype = constructor.prototype;
            const descriptor = Object.getOwnPropertyDescriptor(
              prototype,
              realm.Symbol.iterator
            );
            if (!descriptor || typeof descriptor.value !== "function") {
              failures.push(name + ":iterator");
              return;
            }
            if (descriptor.enumerable || !descriptor.configurable || !descriptor.writable) {
              failures.push(name + ":descriptor");
            }
            if (descriptor.value.length !== 0) failures.push(name + ":length");
            if (alias && descriptor.value !== prototype[alias]) {
              failures.push(name + ":alias:" + alias);
            }
            if (!Function.prototype.toString.call(descriptor.value).includes("[native code]")) {
              failures.push(name + ":native");
            }
          };
          for (const name of entriesAliases) inspect(window, name, "entries");
          for (const name of valuesAliases) inspect(window, name, "values");
          for (const name of ownIterators) inspect(window, name, null);

          const frame = document.createElement("iframe");
          document.body.appendChild(frame);
          for (const name of entriesAliases) inspect(frame.contentWindow, name, "entries");
          for (const name of valuesAliases) inspect(frame.contentWindow, name, "values");
          for (const name of ownIterators) inspect(frame.contentWindow, name, null);

          const asyncAliases = [
            ["FileSystemDirectoryHandle", "entries"],
            ["ReadableStream", "values"]
          ];
          for (const [name, alias] of asyncAliases) {
            const prototype = window[name].prototype;
            const descriptor = Object.getOwnPropertyDescriptor(
              prototype,
              Symbol.asyncIterator
            );
            if (!descriptor || descriptor.value !== prototype[alias]) {
              failures.push(name + ":async:" + alias);
            } else if (
              descriptor.enumerable ||
              !descriptor.configurable ||
              !descriptor.writable
            ) {
              failures.push(name + ":async-descriptor");
            }
          }
          return failures.join("\n");
        })()
        "#,
    );
    assert!(
        failures.is_empty(),
        "iterable prototype failures:\n{failures}"
    );
}

#[test]
fn hidden_and_live_dom_iterables_have_edge_aliases_and_work() {
    let mut runtime = EdgeRuntime::new().expect("runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          document.body.className = "alpha beta";
          const child = document.createElement("span");
          child.setAttribute("data-value", "one");
          document.body.appendChild(child);
          const style = document.createElement("style");
          style.textContent = "body { color: black; }";
          document.head.appendChild(style);

          const sheet = new CSSStyleSheet();
          sheet.replaceSync(
            "@font-feature-values Test { @styleset { nice: 1; } }"
          );
          const featureMap = sheet.cssRules[0].styleset;
          const featurePrototype = Object.getPrototypeOf(featureMap);
          const featureDescriptor = Object.getOwnPropertyDescriptor(
            featurePrototype,
            Symbol.iterator
          );
          const fontPrototype = Object.getPrototypeOf(document.fonts);

          const form = new FormData();
          form.append("a", "1");
          form.append("b", "2");
          const headers = new Headers({ "x-test": "yes" });
          const params = new URLSearchParams("a=1&b=2");

          return [
            [...document.body.classList].join(",") === "alpha,beta",
            [...document.body.children].includes(child),
            [...child.attributes][0].name === "data-value",
            [...document.styleSheets].length === 1,
            [...style.sheet.cssRules].length === 1,
            [...form].map(pair => pair.join("=")).join("&") === "a=1&b=2",
            [...headers][0].join("=") === "x-test=yes",
            [...params].map(pair => pair.join("=")).join("&") === "a=1&b=2",
            featurePrototype[Symbol.iterator] === featurePrototype.entries,
            featureDescriptor.enumerable === false,
            [...featureMap][0][0] === "nice",
            fontPrototype[Symbol.iterator] === fontPrototype.values,
            [...document.fonts].length === 0
          ].every(Boolean);
        })()
        "#,
    );
    assert_eq!(result, "true");
}

#[test]
fn performance_event_counts_is_the_edge_150_readonly_maplike_instance() {
    let mut runtime = EdgeRuntime::new().expect("runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const counts = performance.eventCounts;
          const prototype = Object.getPrototypeOf(counts);
          const iterator = Object.getOwnPropertyDescriptor(
            prototype,
            Symbol.iterator
          );
          const keys = [...counts.keys()];
          const entries = [...counts];
          let callback = "";
          counts.forEach((value, key, owner) => {
            if (!callback) callback = [value, key, owner === counts].join(",");
          });
          return [
            Object.prototype.toString.call(counts),
            counts instanceof EventCounts,
            Reflect.ownKeys(counts).length,
            counts.size,
            keys.join(","),
            entries.length,
            entries[0].join(","),
            counts.get("pointerdown"),
            counts.has("pointerleave"),
            counts.has("wheel"),
            callback,
            iterator.value === prototype.entries,
            iterator.enumerable,
            iterator.configurable,
            iterator.writable,
            Function.prototype.toString.call(iterator.value)
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "[object EventCounts]|true|0|36|",
            "pointerdown,touchend,input,keydown,mouseleave,mouseenter,drop,beforeinput,",
            "pointerenter,dragend,pointercancel,compositionupdate,mousedown,dragleave,",
            "dragover,mouseup,pointerover,lostpointercapture,mouseover,gotpointercapture,",
            "dblclick,keyup,keypress,pointerup,compositionstart,auxclick,dragstart,",
            "touchstart,compositionend,pointerout,dragenter,touchcancel,click,contextmenu,",
            "mouseout,pointerleave|36|pointerdown,0|0|true|false|0,pointerdown,true|",
            "true|false|true|true|function entries() { [native code] }"
        )
    );
}
