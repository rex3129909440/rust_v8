use crate::{EdgeRuntime, EdgeRuntimeOptions, Evaluation, NetworkReplayEntry, PageInit};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

#[test]
fn iframe_window_has_the_complete_edge_surface_and_stable_navigation_identity() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const baseline = Object.getOwnPropertyNames(window);
              const frame = document.createElement("iframe");
              frame.srcdoc =
                "<p id='first'>one</p><script>window.oldNavigationValue=17<\/script>";
              document.body.appendChild(frame);
              const proxy = frame.contentWindow;
              const firstDocument = frame.contentDocument;
              const firstSurface = Object.getOwnPropertyNames(proxy);
              frame.srcdoc =
                "<p id='second'>two</p><script>window.newNavigationValue=29<\/script>";
              const secondSurface = Object.getOwnPropertyNames(frame.contentWindow);
              return [
                baseline.length,
                firstSurface.filter(name => !baseline.includes(name)).join(","),
                baseline.filter(name => !firstSurface.includes(name)).join(","),
                proxy === frame.contentWindow,
                firstDocument !== frame.contentDocument,
                typeof frame.contentWindow.oldNavigationValue,
                frame.contentWindow.newNavigationValue,
                frame.contentDocument.getElementById("second").textContent,
                Object.getPrototypeOf(frame.contentWindow) === Window.prototype,
                frame.contentWindow.Array !== Array,
                secondSurface.includes("SharedArrayBuffer")
              ].join("|");
            })()
            "#,
        ),
        "1232|oldNavigationValue||true|true|undefined|29|two|false|true|false"
    );
}

#[test]
fn iframe_srcdoc_and_cross_origin_url_state_match_edge_150() {
    let child_source = br#"
      <script>
        try {
          parent.postMessage([
            location.href,
            location.origin,
            origin,
            document.URL,
            document.documentURI,
            document.baseURI,
            document.referrer,
            frameElement === null,
            parent !== window,
            top === parent
          ].join("~"), "*");
        } catch (error) {
          parent.postMessage(
            ["ERROR", error.name, error.message].join("~"),
            "*"
          );
        }
      </script>
    "#;
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root/index.html".to_owned(),
            html: "<main>root</main>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![
            NetworkReplayEntry {
                url: "https://app.example.test/root/same.html".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
                body: b"<p>same origin</p>".to_vec(),
            },
            NetworkReplayEntry {
                url: "https://other.example.test/frame/child.html".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
                body: child_source.to_vec(),
            },
        ],
        ..Default::default()
    })
    .expect("configured iframe URL runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          globalThis.iframeUrlChildReport = "missing";
          addEventListener(
            "message",
            event => iframeUrlChildReport = event.data
          );
          const srcdocFrame = document.createElement("iframe");
          srcdocFrame.srcdoc =
            "<!doctype html><base href='relative-base/'><a id='relative' href='asset.js'>x</a>";
          document.body.appendChild(srcdocFrame);
          const srcdocWindow = srcdocFrame.contentWindow;
          const srcdocDocument = srcdocFrame.contentDocument;

          const sameFrame = document.createElement("iframe");
          sameFrame.src = "same.html";
          document.body.appendChild(sameFrame);
          const sameDocument = sameFrame.contentDocument;

          const crossFrame = document.createElement("iframe");
          crossFrame.src = "https://other.example.test/frame/child.html";
          document.body.appendChild(crossFrame);
          const childWindow = crossFrame.contentWindow;
          const capture = callback => {
            try {
              const value = callback();
              return value === null ? "null" : String(value);
            } catch (error) {
              return error.name + ":" + error.message;
            }
          };
          return [
            srcdocFrame.src,
            srcdocWindow.location.href,
            srcdocWindow.location.origin,
            srcdocWindow.origin,
            srcdocDocument.URL,
            srcdocDocument.documentURI,
            srcdocDocument.baseURI,
            srcdocDocument.referrer,
            srcdocDocument.getElementById("relative").href,
            srcdocDocument.querySelector("base").href,
            srcdocFrame.contentDocument === srcdocDocument,
            srcdocWindow.parent === window,
            srcdocWindow.top === window,
            srcdocWindow.frameElement === srcdocFrame,
            sameFrame.contentWindow.location.href,
            sameDocument.URL,
            sameDocument.baseURI,
            sameDocument.referrer,
            sameFrame.contentWindow.frameElement === sameFrame,
            crossFrame.contentDocument === null,
            capture(() => childWindow.document),
            capture(() => childWindow.location.href),
            capture(() => typeof childWindow.location),
            capture(() => childWindow.closed),
            capture(() => childWindow.length),
            capture(() => typeof childWindow.postMessage),
            capture(() => childWindow.frameElement)
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "|about:srcdoc|null|https://app.example.test|about:srcdoc|about:srcdoc|",
            "https://app.example.test/root/relative-base/|https://app.example.test/|",
            "https://app.example.test/root/relative-base/asset.js|",
            "https://app.example.test/root/relative-base/|true|true|true|true|",
            "https://app.example.test/root/same.html|https://app.example.test/root/same.html|",
            "https://app.example.test/root/same.html|https://app.example.test/root/index.html|true|true|",
            "SecurityError:Failed to read a named property 'document' from 'Window': ",
            "Blocked a frame with origin \"https://app.example.test\" from accessing a cross-origin frame.|",
            "SecurityError:Failed to read a named property 'href' from 'Location': ",
            "Blocked a frame with origin \"https://app.example.test\" from accessing a cross-origin frame.|",
            "object|false|0|function|",
            "SecurityError:Failed to read a named property 'frameElement' from 'Window': ",
            "Blocked a frame with origin \"https://app.example.test\" from accessing a cross-origin frame."
        )
    );
    assert_eq!(
        text(&mut runtime, "iframeUrlChildReport"),
        concat!(
            "https://other.example.test/frame/child.html~https://other.example.test~",
            "https://other.example.test~https://other.example.test/frame/child.html~",
            "https://other.example.test/frame/child.html~",
            "https://other.example.test/frame/child.html~https://app.example.test/~true~true~true"
        )
    );
}

#[test]
fn iframe_window_interfaces_and_global_functions_are_realm_local() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              const child = frame.contentWindow;
              const leakedFunctions = Object.getOwnPropertyNames(window)
                .filter(name =>
                  typeof window[name] === "function" &&
                  child[name] === window[name]
                );
              const requiredObjects = [
                "console",
                "performance",
                "history",
                "customElements",
                "localStorage",
                "sessionStorage",
                "cookieStore",
                "caches",
                "indexedDB",
                "scheduler",
                "trustedTypes",
                "crypto",
                "navigator",
                "screen",
                "speechSynthesis"
              ];
              const leakedObjects = requiredObjects.filter(
                name => child[name] === window[name]
              );
              const singletonNames = [
                "navigation",
                "locationbar",
                "menubar",
                "personalbar",
                "scrollbars",
                "statusbar",
                "toolbar",
                "external",
                "visualViewport",
                "styleMedia",
                "Temporal",
                "crashReport",
                "documentPictureInPicture",
                "sharedStorage",
                "viewport",
                "launchQueue"
              ];
              const rootSingletons = singletonNames.map(name => window[name]);
              const firstSingletons = singletonNames.map(name => child[name]);
              const secondFrame = document.createElement("iframe");
              document.body.appendChild(secondFrame);
              const second = secondFrame.contentWindow;
              const secondSingletons =
                singletonNames.map(name => second[name]);
              const singletonIdentityFailures = singletonNames.filter(
                (_, index) =>
                  rootSingletons[index] !== window[singletonNames[index]] ||
                  firstSingletons[index] === rootSingletons[index] ||
                  secondSingletons[index] === rootSingletons[index] ||
                  secondSingletons[index] === firstSingletons[index]
              );
              const singletonRealmFailures = [
                child.navigation instanceof child.Navigation,
                child.locationbar instanceof child.BarProp,
                child.menubar instanceof child.BarProp,
                child.personalbar instanceof child.BarProp,
                child.scrollbars instanceof child.BarProp,
                child.statusbar instanceof child.BarProp,
                child.toolbar instanceof child.BarProp,
                child.external instanceof child.External,
                child.visualViewport instanceof child.VisualViewport,
                Object.prototype.toString.call(child.styleMedia) ===
                  "[object StyleMedia]",
                child.Temporal.PlainDate !== Temporal.PlainDate,
                child.Temporal.PlainDate instanceof child.Function,
                child.crashReport instanceof child.CrashReportContext,
                child.documentPictureInPicture instanceof
                  child.DocumentPictureInPicture,
                child.sharedStorage instanceof child.SharedStorage,
                child.viewport instanceof child.Viewport,
                child.launchQueue instanceof child.LaunchQueue
              ].map((valid, index) => valid ? "" : String(index))
                .filter(Boolean);
              frame.srcdoc = "<body id='navigated'></body>";
              const navigatedSingletons =
                singletonNames.map(name => child[name]);
              const navigationIdentityFailures = singletonNames.filter(
                (_, index) =>
                  navigatedSingletons[index] === firstSingletons[index] ||
                  rootSingletons[index] !== window[singletonNames[index]] ||
                  secondSingletons[index] !== second[singletonNames[index]]
              );
              const inheritanceFailures = [
                ["AnimationEvent", "Event"],
                ["AudioContext", "BaseAudioContext"],
                ["MediaRecorder", "EventTarget"],
                ["SVGElement", "Element"],
                ["XPathEvaluator", "Object"]
              ].filter(([name, parentName]) => {
                const Constructor = child[name];
                const Parent = child[parentName];
                return !(
                  Constructor !== window[name] &&
                  Constructor instanceof child.Function &&
                  Object.getPrototypeOf(Constructor) ===
                    (
                      parentName === "Object"
                        ? child.Function.prototype
                        : Parent
                    ) &&
                  (
                    parentName === "Object" ||
                    Object.getPrototypeOf(Constructor.prototype) ===
                      Parent.prototype
                  )
                );
              }).map(([name]) => name);
              const childFontsPrototype =
                Object.getPrototypeOf(child.document.fonts);
              const fontsRealmFailure = !(
                childFontsPrototype !==
                  Object.getPrototypeOf(document.fonts) &&
                Object.getPrototypeOf(childFontsPrototype) ===
                  child.EventTarget.prototype &&
                Object.prototype.toString.call(child.document.fonts) ===
                  "[object FontFaceSet]"
              );
              return [
                leakedFunctions.join(","),
                leakedObjects.join(","),
                singletonIdentityFailures.join(","),
                singletonRealmFailures.join(","),
                navigationIdentityFailures.join(","),
                inheritanceFailures.join(","),
                fontsRealmFailure
              ].join("|");
            })()
            "#,
        ),
        "||||||false"
    );
}

#[test]
fn iframe_dom_factories_use_the_current_realm_constructor_chain_across_navigation() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              document.body.appendChild(frame);
              const proxy = frame.contentWindow;
              const audit = () => proxy.eval(`
                (() => {
                  const cases = [
                    ["a", "HTMLAnchorElement"],
                    ["area", "HTMLAreaElement"],
                    ["audio", "HTMLAudioElement"],
                    ["button", "HTMLButtonElement"],
                    ["canvas", "HTMLCanvasElement"],
                    ["div", "HTMLDivElement"],
                    ["form", "HTMLFormElement"],
                    ["iframe", "HTMLIFrameElement"],
                    ["img", "HTMLImageElement"],
                    ["input", "HTMLInputElement"],
                    ["link", "HTMLLinkElement"],
                    ["meta", "HTMLMetaElement"],
                    ["script", "HTMLScriptElement"],
                    ["select", "HTMLSelectElement"],
                    ["span", "HTMLSpanElement"],
                    ["style", "HTMLStyleElement"],
                    ["table", "HTMLTableElement"],
                    ["textarea", "HTMLTextAreaElement"],
                    ["video", "HTMLVideoElement"],
                    ["base", "HTMLBaseElement"],
                    ["br", "HTMLBRElement"],
                    ["dl", "HTMLDListElement"],
                    ["data", "HTMLDataElement"],
                    ["datalist", "HTMLDataListElement"],
                    ["details", "HTMLDetailsElement"],
                    ["dialog", "HTMLDialogElement"],
                    ["dir", "HTMLDirectoryElement"],
                    ["embed", "HTMLEmbedElement"],
                    ["fencedframe", "HTMLFencedFrameElement"],
                    ["fieldset", "HTMLFieldSetElement"],
                    ["font", "HTMLFontElement"],
                    ["frame", "HTMLFrameElement"],
                    ["frameset", "HTMLFrameSetElement"],
                    ["geolocation", "HTMLGeolocationElement"],
                    ["h1", "HTMLHeadingElement"],
                    ["hr", "HTMLHRElement"],
                    ["label", "HTMLLabelElement"],
                    ["legend", "HTMLLegendElement"],
                    ["li", "HTMLLIElement"],
                    ["map", "HTMLMapElement"],
                    ["marquee", "HTMLMarqueeElement"],
                    ["menu", "HTMLMenuElement"],
                    ["meter", "HTMLMeterElement"],
                    ["ins", "HTMLModElement"],
                    ["ol", "HTMLOListElement"],
                    ["object", "HTMLObjectElement"],
                    ["optgroup", "HTMLOptGroupElement"],
                    ["option", "HTMLOptionElement"],
                    ["output", "HTMLOutputElement"],
                    ["p", "HTMLParagraphElement"],
                    ["param", "HTMLParamElement"],
                    ["picture", "HTMLPictureElement"],
                    ["pre", "HTMLPreElement"],
                    ["progress", "HTMLProgressElement"],
                    ["q", "HTMLQuoteElement"],
                    ["selectedcontent", "HTMLSelectedContentElement"],
                    ["slot", "HTMLSlotElement"],
                    ["source", "HTMLSourceElement"],
                    ["caption", "HTMLTableCaptionElement"],
                    ["td", "HTMLTableCellElement"],
                    ["col", "HTMLTableColElement"],
                    ["tr", "HTMLTableRowElement"],
                    ["thead", "HTMLTableSectionElement"],
                    ["template", "HTMLTemplateElement"],
                    ["time", "HTMLTimeElement"],
                    ["title", "HTMLTitleElement"],
                    ["track", "HTMLTrackElement"],
                    ["ul", "HTMLUListElement"],
                    ["edgeunknown", "HTMLUnknownElement"]
                  ];
                  return cases.filter(([tag, name]) => {
                    const element = document.createElement(tag);
                    const Constructor = globalThis[name];
                    return !(
                      Constructor !== parent[name] &&
                      element instanceof Constructor &&
                      element instanceof HTMLElement &&
                      element instanceof Element &&
                      element instanceof Node &&
                      element instanceof EventTarget &&
                      Object.getPrototypeOf(element) === Constructor.prototype &&
                      HTMLElement.prototype.isPrototypeOf(Constructor.prototype)
                    );
                  }).map(([, name]) => name).join(",");
                })()
              `);
              const firstConstructor = proxy.HTMLImageElement;
              const first = audit();
              frame.srcdoc = "<p>navigated</p>";
              const second = audit();
              return [
                first,
                second,
                firstConstructor !== proxy.HTMLImageElement,
                proxy === frame.contentWindow
              ].join("|");
            })()
            "#,
        ),
        "||true|true"
    );
}

#[test]
fn iframe_trace_records_local_api_construction_properties_and_calls() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    runtime.enable_proxy_trace().expect("enable iframe trace");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              frame.srcdoc =
                "<script>" +
                "const request=new XMLHttpRequest();" +
                "request.open('GET','https://sandbox.test/trace');" +
                "const utterance=new SpeechSynthesisUtterance('edge');" +
                "const meter=document.createElement('meter');" +
                "meter.value=0.5;" +
                "const template=document.createElement('template');" +
                "const unknown=document.createElement('edgeunknown');" +
                "const animation=new AnimationEvent('edge-animation');" +
                "const face=new FontFace('Edge Sans','local(Edge Sans)');" +
                "const decoded=atob('ZWRnZQ==');" +
                "const meterGetter=Function.prototype.toString.call(" +
                  "Object.getOwnPropertyDescriptor(" +
                    "HTMLMeterElement.prototype,'value').get);" +
                "window.traceAnswer=[" +
                  "request.readyState,utterance.text,meter.value," +
                  "template.content instanceof DocumentFragment," +
                  "unknown instanceof HTMLUnknownElement," +
                  "animation.type,face.family,decoded,meterGetter" +
                "].join('|');" +
                "<\/script>";
              document.body.appendChild(frame);
              return frame.contentWindow.traceAnswer;
            })()
            "#,
        ),
        concat!(
            "1|edge|0.5|true|true|edge-animation|Edge Sans|edge|",
            "function get value() { [native code] }"
        )
    );
    let trace = runtime.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "construct"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".XMLHttpRequest")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.contains("iframe[")
            && entry.api.ends_with(".XMLHttpRequest().open")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.contains("iframe[")
            && entry.api.ends_with(".XMLHttpRequest().readyState")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "construct"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".SpeechSynthesisUtterance")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "construct"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".AnimationEvent")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "construct"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".FontFace")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".atob")
            && entry.arguments == "\"ZWRnZQ==\""
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.contains("iframe[")
            && entry.api.ends_with(".SpeechSynthesisUtterance().text")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".document.createElement")
            && entry.arguments == "\"meter\""
            && entry.result.contains("HTMLMeterElement")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "set"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".document.createElement().value")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && entry.api.starts_with("iframe[")
            && entry.api.ends_with(".document.createElement().content")
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "get" && entry.api == "HTMLTemplateElement.prototype.content"
    }));
    assert!(trace.iter().any(|entry| {
        entry.operation == "call"
            && entry.api.ends_with(".document.createElement")
            && entry.arguments == "\"edgeunknown\""
            && entry.result.contains("HTMLUnknownElement")
    }));
}

#[test]
fn frame_indices_names_removal_and_nested_topology_track_the_dom() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const first = document.createElement("iframe");
              first.name = "firstFrame";
              document.body.appendChild(first);
              const firstWindow = first.contentWindow;
              const second = document.createElement("iframe");
              second.id = "secondFrame";
              document.body.appendChild(second);
              const secondWindow = second.contentWindow;
              const before = [
                window.length,
                window[0] === firstWindow,
                window[1] === secondWindow,
                window.firstFrame === firstWindow,
                window.secondFrame === secondWindow
              ];
              first.name = "renamedFrame";
              const renamed = [
                typeof window.firstFrame,
                window.renamedFrame === firstWindow
              ];
              first.remove();
              const after = [
                window.length,
                window[0] === secondWindow,
                typeof window[1],
                typeof window.renamedFrame
              ];
              const outer = document.createElement("iframe");
              outer.srcdoc =
                "<iframe id='innerId' name='innerName' srcdoc='<p>nested</p>'></iframe>";
              document.body.appendChild(outer);
              const inner = outer.contentDocument.querySelector("iframe");
              const nested = [
                outer.contentWindow.length,
                outer.contentWindow[0] === inner.contentWindow,
                outer.contentWindow.innerName === inner.contentWindow,
                outer.contentWindow.innerId === inner.contentWindow,
                Object.prototype.hasOwnProperty.call(
                  outer.contentWindow,
                  "innerName"
                ),
                inner.contentWindow.parent === outer.contentWindow,
                inner.contentWindow.top === window,
                inner.contentWindow.frameElement === inner
              ];
              return [before, renamed, after, nested]
                .flat()
                .join("|");
            })()
            "#,
        ),
        concat!(
            "2|true|true|true|true|",
            "undefined|true|",
            "1|true|undefined|undefined|",
            "1|true|true|true|true|true|true|true"
        )
    );
}

#[test]
fn iframe_src_uses_typed_network_replay_and_enforces_same_origin_document_access() {
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root/index.html".to_owned(),
            html: "<main>root</main>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![
            NetworkReplayEntry {
                url: "https://app.example.test/frame.html".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![(
                    "Content-Type".to_owned(),
                    "text/html; charset=utf-8".to_owned(),
                )],
                body:
                    b"<article id='replayed'>same</article><script>window.replayedValue=41</script>"
                        .to_vec(),
            },
            NetworkReplayEntry {
                url: "https://other.example.test/cross.html".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
                body: b"<p>cross origin</p>".to_vec(),
            },
        ],
        ..Default::default()
    })
    .expect("configured Edge runtime");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              frame.src = "/frame.html";
              document.body.appendChild(frame);
              const proxy = frame.contentWindow;
              const sameOrigin = [
                frame.contentDocument.URL,
                frame.contentDocument.getElementById("replayed").textContent,
                frame.contentWindow.replayedValue,
                frame.contentDocument.defaultView === proxy
              ];
              frame.src = "https://other.example.test/cross.html";
              return [
                ...sameOrigin,
                proxy === frame.contentWindow,
                frame.contentDocument === null
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "https://app.example.test/frame.html|same|41|true|",
            "true|true"
        )
    );
}

#[test]
fn shared_worker_runtime_identity_is_partitioned_by_creator_origin() {
    let frame_html = br#"
        <script>
          const frameOrigin =
            location.href.startsWith("https://first.") ? "first" : "second";
          parent.postMessage("loaded=" + frameOrigin, "*");
          const source = `
            let connections = 0;
            onconnect = event => {
              connections += 1;
              const port = event.ports[0];
              port.onmessage = () => port.postMessage(connections);
            };
          `;
          const worker = new SharedWorker(
            "data:text/javascript," + encodeURIComponent(source),
            { name: "origin-partition" }
          );
          worker.onerror = event =>
            parent.postMessage("error=" + frameOrigin + ":" + event.message, "*");
          worker.port.onmessage = event =>
            parent.postMessage(frameOrigin + "=" + event.data, "*");
          worker.port.start();
          worker.port.postMessage("ping");
        </script>
    "#
    .to_vec();
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root.html".to_owned(),
            html: "<main>root</main>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![
            NetworkReplayEntry {
                url: "https://first.example.test/frame.html".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
                body: frame_html.clone(),
            },
            NetworkReplayEntry {
                url: "https://second.example.test/frame.html".to_owned(),
                method: "GET".to_owned(),
                status: 200,
                status_text: "OK".to_owned(),
                headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
                body: frame_html,
            },
        ],
        ..Default::default()
    })
    .expect("configured Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              globalThis.sharedOriginAnswers = [];
              addEventListener("message", event => {
                sharedOriginAnswers.push(event.data);
              });
              const first = document.createElement("iframe");
              first.src = "https://first.example.test/frame.html";
              document.body.appendChild(first);
              const second = document.createElement("iframe");
              second.src = "https://second.example.test/frame.html";
              document.body.appendChild(second);
              return "scheduled";
            })()
            "#,
        ),
        "scheduled"
    );
    for _ in 0..4 {
        let _ = text(&mut runtime, "sharedOriginAnswers.length");
    }
    assert_eq!(
        text(&mut runtime, "sharedOriginAnswers.slice().sort().join('|')"),
        "first=1|loaded=first|loaded=second|second=1"
    );
}

#[test]
fn iframe_navigation_clears_old_window_listeners_and_stale_nested_frames() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              frame.srcdoc =
                "<iframe id='staleId' name='staleName' srcdoc='<p>old</p>'></iframe>";
              document.body.appendChild(frame);
              const proxy = frame.contentWindow;
              let staleListenerCalls = 0;
              proxy.addEventListener(
                "realm-navigation-check",
                () => staleListenerCalls++
              );
              const before = [
                proxy.length,
                typeof proxy.staleName,
                typeof proxy.staleId
              ];
              frame.srcdoc = "<main id='fresh'>fresh realm</main>";
              proxy.dispatchEvent(
                new proxy.Event("realm-navigation-check")
              );
              return [
                ...before,
                proxy === frame.contentWindow,
                staleListenerCalls,
                proxy.length,
                typeof proxy.staleName,
                typeof proxy.staleId,
                proxy.document.getElementById("fresh").textContent
              ].join("|");
            })()
            "#,
        ),
        "1|object|object|true|0|0|undefined|undefined|fresh realm"
    );
}

#[test]
fn cross_origin_window_proxy_uses_the_edge_accessible_property_whitelist() {
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root/index.html".to_owned(),
            html: "<main>root</main>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![NetworkReplayEntry {
            url: "https://other.example.test/cross.html".to_owned(),
            method: "GET".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
            body: b"<iframe srcdoc='<p>nested</p>'></iframe><p>cross origin</p>".to_vec(),
        }],
        ..Default::default()
    })
    .expect("configured Edge runtime");

    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const frame = document.createElement("iframe");
              frame.src = "https://other.example.test/cross.html";
              document.body.appendChild(frame);
              const proxy = frame.contentWindow;
              const capture = operation => {
                try {
                  const value = operation();
                  if (value === proxy) return "proxy";
                  if (value === window) return "parent";
                  if (value === null) return "null";
                  return String(value);
                } catch (error) {
                  return "throws " + error.name;
                }
              };
              const descriptorShape = name => {
                const descriptor =
                  Object.getOwnPropertyDescriptor(proxy, name);
                return [
                  "value" in descriptor ? "data" : "accessor",
                  typeof descriptor.value,
                  typeof descriptor.get,
                  descriptor.get?.name ?? "",
                  descriptor.get?.length ?? "",
                  typeof descriptor.set,
                  descriptor.set?.name ?? "",
                  descriptor.set?.length ?? "",
                  String(descriptor.writable),
                  descriptor.enumerable,
                  descriptor.configurable
                ].join(",");
              };
              return [
                capture(() => proxy.window),
                capture(() => proxy.self),
                capture(() => proxy.frames),
                capture(() => proxy.parent),
                capture(() => proxy.top),
                capture(() => proxy.length),
                capture(() => proxy.closed),
                capture(() => typeof proxy.close),
                capture(() => typeof proxy.focus),
                capture(() => typeof proxy.blur),
                capture(() => typeof proxy.postMessage),
                capture(() => proxy.opener),
                capture(() => Object.prototype.toString.call(proxy.location)),
                capture(() => proxy.location.href),
                capture(() => typeof proxy.location.replace),
                capture(() => proxy.blur === window.blur),
                capture(() => proxy.close === window.close),
                capture(() => proxy.focus === window.focus),
                capture(() => proxy.postMessage === window.postMessage),
                capture(() => proxy.postMessage === proxy.postMessage),
                capture(() =>
                  Object.getPrototypeOf(proxy.postMessage) ===
                  Function.prototype
                ),
                capture(() =>
                  Object.getOwnPropertyDescriptor(proxy, "window").get ===
                  Object.getOwnPropertyDescriptor(proxy, "window").get
                ),
                capture(() => typeof proxy[0]),
                capture(() => proxy.document),
                capture(() => proxy.name),
                capture(() => proxy.history),
                capture(() => proxy.navigator),
                capture(() => proxy.Array),
                capture(() => proxy.Event),
                capture(() => proxy.addEventListener),
                capture(() => proxy.edgeSandboxProbe),
                capture(() => "postMessage" in proxy),
                capture(() => "document" in proxy),
                capture(() =>
                  Object.getOwnPropertyDescriptor(
                    proxy,
                    "postMessage"
                  )?.configurable
                ),
                capture(() =>
                  Object.getOwnPropertyDescriptor(proxy, "document")
                ),
                capture(() => Object.getPrototypeOf(proxy)),
                capture(() => Object.getOwnPropertyNames(proxy).join(",")),
                capture(() =>
                  Object.getOwnPropertySymbols(proxy)
                    .map(symbol => String(symbol))
                    .join(",")
                ),
                capture(() =>
                  Reflect.ownKeys(proxy)
                    .map(key => String(key))
                    .join(",")
                ),
                capture(() => Object.keys(proxy).join(",")),
                capture(() =>
                  Object.values(proxy).map(value => typeof value).join(",")
                ),
                capture(() =>
                  Object.entries(proxy)
                    .map(([key, value]) => key + ":" + typeof value)
                    .join(",")
                ),
                capture(() =>
                  Object.getOwnPropertyNames(
                    Object.getOwnPropertyDescriptors(proxy)
                  ).join(",")
                ),
                capture(() => Object.isExtensible(proxy)),
                capture(() => Reflect.isExtensible(proxy)),
                capture(() => Object.setPrototypeOf(proxy, null)),
                capture(() => Reflect.setPrototypeOf(proxy, null)),
                capture(() => Object.preventExtensions(proxy)),
                capture(() => Reflect.preventExtensions(proxy)),
                capture(() => descriptorShape("window")),
                capture(() => descriptorShape("location")),
                capture(() => descriptorShape("closed")),
                capture(() => descriptorShape("length")),
                capture(() => descriptorShape("postMessage")),
                capture(() => descriptorShape("then")),
                capture(() => {
                  const descriptor =
                    Object.getOwnPropertyDescriptor(proxy, "0");
                  return [
                    typeof descriptor.value,
                    descriptor.writable,
                    descriptor.enumerable,
                    descriptor.configurable
                  ].join(",");
                }),
                capture(() =>
                  Object.getOwnPropertySymbols(proxy)
                    .map(symbol => {
                      const descriptor =
                        Object.getOwnPropertyDescriptor(proxy, symbol);
                      return [
                        String(symbol),
                        String(descriptor.value),
                        descriptor.writable,
                        descriptor.enumerable,
                        descriptor.configurable
                      ].join(":");
                    })
                    .join(";")
                ),
                capture(() => String(proxy[Symbol.toStringTag])),
                capture(() => Symbol.toStringTag in proxy),
                capture(() => Reflect.deleteProperty(proxy, "window")),
                capture(() => Reflect.deleteProperty(proxy, "document")),
                capture(() => Reflect.deleteProperty(proxy, "0")),
                capture(() =>
                  Reflect.deleteProperty(proxy, Symbol.toStringTag)
                ),
                capture(() =>
                  Reflect.defineProperty(proxy, "window", { value: 1 })
                ),
                capture(() =>
                  Reflect.defineProperty(
                    proxy,
                    "edgeSandboxProbe",
                    { value: 1 }
                  )
                ),
                capture(() =>
                  Reflect.defineProperty(proxy, "0", { value: 1 })
                ),
                capture(() => Object.freeze(proxy)),
                capture(() => Object.seal(proxy))
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "proxy|proxy|proxy|parent|parent|1|false|",
            "function|function|function|function|null|[object Object]|",
            "throws SecurityError|function|",
            "false|false|false|false|true|true|true|object|",
            "throws SecurityError|throws SecurityError|throws SecurityError|",
            "throws SecurityError|throws SecurityError|throws SecurityError|",
            "throws SecurityError|throws SecurityError|true|throws SecurityError|",
            "true|throws SecurityError|null|",
            "0,window,self,location,closed,frames,length,top,opener,parent,",
            "blur,close,focus,postMessage,then|",
            "Symbol(Symbol.toStringTag),Symbol(Symbol.hasInstance),",
            "Symbol(Symbol.isConcatSpreadable)|",
            "0,window,self,location,closed,frames,length,top,opener,parent,",
            "blur,close,focus,postMessage,then,Symbol(Symbol.toStringTag),",
            "Symbol(Symbol.hasInstance),Symbol(Symbol.isConcatSpreadable)|",
            "0|object|0:object|",
            "0,window,self,location,closed,frames,length,top,opener,parent,",
            "blur,close,focus,postMessage,then|true|true|",
            "throws SecurityError|throws SecurityError|",
            "throws SecurityError|throws SecurityError|",
            "accessor,undefined,function,get window,0,undefined,,,",
            "undefined,false,true|",
            "accessor,undefined,function,get location,0,function,",
            "set location,1,undefined,false,true|",
            "accessor,undefined,function,get closed,0,undefined,,,",
            "undefined,false,true|",
            "accessor,undefined,function,get length,0,undefined,,,",
            "undefined,false,true|",
            "data,function,undefined,,,undefined,,,false,false,true|",
            "data,undefined,undefined,,,undefined,,,false,false,true|",
            "object,false,true,true|",
            "Symbol(Symbol.toStringTag):undefined:false:false:true;",
            "Symbol(Symbol.hasInstance):undefined:false:false:true;",
            "Symbol(Symbol.isConcatSpreadable):undefined:false:false:true|",
            "undefined|true|",
            "throws SecurityError|throws SecurityError|",
            "throws SecurityError|throws SecurityError|",
            "throws SecurityError|throws SecurityError|",
            "throws SecurityError|throws SecurityError|throws SecurityError"
        )
    );
}

#[test]
fn proxy_trace_preserves_the_cross_origin_window_proxy_edge_shape() {
    const SNAPSHOT: &str = r#"
      (() => {
        const frame = document.createElement("iframe");
        frame.src = "https://other.example.test/cross.html";
        document.body.appendChild(frame);
        const proxy = frame.contentWindow;
        const windowDescriptor =
          Object.getOwnPropertyDescriptor(proxy, "window");
        const locationDescriptor =
          Object.getOwnPropertyDescriptor(proxy, "location");
        const native = functionValue =>
          Function.prototype.toString
            .call(functionValue)
            .includes("[native code]");
        void proxy.window;
        void proxy.location;
        void proxy[0];
        windowDescriptor.get.call(proxy);
        return [
          Object.getPrototypeOf(proxy) === null,
          Reflect.getPrototypeOf(proxy) === null,
          Object.getOwnPropertyNames(proxy).join(","),
          Reflect.ownKeys(proxy).map(key => String(key)).join(","),
          Object.keys(proxy).join(","),
          Object.values(proxy).map(value => typeof value).join(","),
          typeof windowDescriptor.get,
          windowDescriptor.get.name,
          windowDescriptor.get.length,
          native(windowDescriptor.get),
          typeof locationDescriptor.set,
          locationDescriptor.set.name,
          locationDescriptor.set.length,
          native(locationDescriptor.set),
          native(Object.getPrototypeOf),
          Object.getPrototypeOf.name,
          Object.getPrototypeOf.length,
          native(Reflect.ownKeys),
          Reflect.ownKeys.name,
          Reflect.ownKeys.length
        ].join("|");
      })()
    "#;

    let options = || EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root/index.html".to_owned(),
            html: "<main>root</main>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![NetworkReplayEntry {
            url: "https://other.example.test/cross.html".to_owned(),
            method: "GET".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
            body: b"<iframe srcdoc='<p>nested</p>'></iframe><p>cross origin</p>".to_vec(),
        }],
        ..Default::default()
    };

    let mut direct = EdgeRuntime::with_options(options()).expect("direct cross-origin runtime");
    let expected = text(&mut direct, SNAPSHOT);

    let mut traced = EdgeRuntime::with_options(options()).expect("traced cross-origin runtime");
    traced
        .enable_proxy_trace()
        .expect("enable cross-origin Proxy trace");
    let actual = text(&mut traced, SNAPSHOT);
    assert_eq!(actual, expected);

    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "call" && entry.api.ends_with("Object.getPrototypeOf")
    }));
    assert!(
        trace
            .iter()
            .any(|entry| entry.operation == "call" && entry.api.ends_with("Reflect.ownKeys"))
    );
    assert!(trace.iter().any(|entry| entry.operation == "get"));
    assert!(trace.iter().any(|entry| entry.operation == "call"));
}

#[test]
fn cross_origin_child_access_to_parent_uses_the_same_edge_whitelist() {
    let child_source = r#"
      <iframe srcdoc="<p>nested</p>"></iframe>
      <script>
        const capture = operation => {
          try {
            const value = operation();
            if (value === parent) return "parent";
            if (value === null) return "null";
            return String(value);
          } catch (error) {
            return "throws " + error.name;
          }
        };
        parent.postMessage([
          capture(() => parent.window),
          capture(() => parent.self),
          capture(() => parent.frames),
          capture(() => parent.parent),
          capture(() => parent.top),
          capture(() => parent.length),
          capture(() => parent.closed),
          capture(() => typeof parent.postMessage),
          capture(() => typeof parent[0]),
          capture(() => parent.document),
          capture(() => parent.navigator),
          capture(() => "postMessage" in parent),
          capture(() => "document" in parent),
          capture(() => Object.getPrototypeOf(parent)),
          capture(() => Object.getOwnPropertyNames(parent)),
          capture(() => Object.getOwnPropertySymbols(parent).map(String)),
          capture(() => parent[Symbol.toStringTag]),
          capture(() => Symbol.toStringTag in parent)
        ].join("|"), "*");
      </script>
    "#;
    let options = || EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root/index.html".to_owned(),
            html: "<main>root</main>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![NetworkReplayEntry {
            url: "https://other.example.test/reverse.html".to_owned(),
            method: "GET".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
            body: child_source.as_bytes().to_vec(),
        }],
        ..Default::default()
    };
    let run_audit = |runtime: &mut EdgeRuntime| {
        let _ = runtime
            .evaluate(
                r#"
              globalThis.reverseCrossOriginAudit = "pending";
              addEventListener("message", event => {
                reverseCrossOriginAudit = event.data;
              });
              const frame = document.createElement("iframe");
              frame.src = "https://other.example.test/reverse.html";
              document.body.appendChild(frame);
            "#,
            )
            .expect("run reverse cross-origin audit");
        text(runtime, "reverseCrossOriginAudit")
    };
    let expected = concat!(
        "parent|parent|parent|parent|parent|1|false|function|object|",
        "throws SecurityError|throws SecurityError|true|throws SecurityError|null|",
        "0,window,self,location,closed,frames,length,top,opener,parent,blur,close,focus,",
        "postMessage,then|",
        "Symbol(Symbol.toStringTag),Symbol(Symbol.hasInstance),",
        "Symbol(Symbol.isConcatSpreadable)|undefined|true"
    );

    let mut direct = EdgeRuntime::with_options(options()).expect("reverse cross-origin runtime");
    assert_eq!(run_audit(&mut direct), expected);

    let mut traced =
        EdgeRuntime::with_options(options()).expect("traced reverse cross-origin runtime");
    traced
        .enable_proxy_trace()
        .expect("enable reverse cross-origin Proxy trace");
    assert_eq!(run_audit(&mut traced), expected);
    let trace = traced.proxy_trace();
    assert!(trace.iter().any(|entry| {
        entry.operation == "get"
            && (entry.api.ends_with(".parent") || entry.api.contains("parent."))
    }));
    assert!(
        trace
            .iter()
            .any(|entry| { entry.operation == "call" && entry.api.ends_with("postMessage") })
    );
}
