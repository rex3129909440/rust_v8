use crate::{
    EdgeRuntime, EdgeRuntimeOptions, Evaluation, IframeHook, NetworkReplayEntry, PageInit,
};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

fn page_runtime() -> EdgeRuntime {
    EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://www.example.test/app/index.html?edge=1#ready".to_owned(),
            html: r#"<!doctype html>
                <title>Initialized Edge</title>
                <base href="/assets/">
                <main id="app" class="ready root">
                  <p name="target">hello <b>edge</b></p>
                  <svg id="icon"><circle id="dot"></circle></svg>
                  <math id="formula"><mi>x</mi></math>
                </main>"#
                .to_owned(),
            referrer: "https://referrer.test/start".to_owned(),
            content_type: "text/html; charset=utf-8".to_owned(),
        }),
        ..Default::default()
    })
    .expect("configured Edge page")
}

#[test]
fn typed_page_init_materializes_a_connected_html5_document() {
    let mut runtime = page_runtime();
    let values = text(
        &mut runtime,
        r##"
        (() => {
          const app = document.getElementById("app");
          const paragraph = document.getElementsByName("target")[0];
          const icon = document.querySelector("#icon");
          const formula = document.querySelector("#formula");
          return [
            location.href,
            origin,
            document.URL,
            document.documentURI,
            document.referrer,
            document.contentType,
            document.baseURI,
            document.compatMode,
            document.doctype.name,
            document.documentElement.tagName,
            document.head.tagName,
            document.body.tagName,
            document.title,
            app.constructor.name,
            app.isConnected,
            app.ownerDocument === document,
            document.getElementsByClassName("ready").length,
            paragraph.textContent.trim(),
            icon.constructor.name,
            formula.constructor.name
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        values,
        concat!(
            "https://www.example.test/app/index.html?edge=1#ready|",
            "https://www.example.test|",
            "https://www.example.test/app/index.html?edge=1#ready|",
            "https://www.example.test/app/index.html?edge=1#ready|",
            "https://referrer.test/start|text/html|https://www.example.test/assets/|",
            "CSS1Compat|html|HTML|HEAD|BODY|Initialized Edge|HTMLElement|",
            "true|true|1|hello edge|SVGSVGElement|MathMLElement"
        )
    );
}

#[test]
fn parser_inserted_scripts_run_after_window_globals_are_installed() {
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://scripts.example.test/globals".to_owned(),
            html: r#"<!doctype html><script>
              window.pageGlobalEvidence = [
                navigator.toString(),
                navigator.platform,
                typeof MediaRecorder,
                typeof matchMedia,
                document.body.tagName
              ].join("|");
            </script>"#
                .to_owned(),
            ..PageInit::default()
        }),
        ..EdgeRuntimeOptions::default()
    })
    .expect("script page");
    assert_eq!(
        text(&mut runtime, "pageGlobalEvidence"),
        "[object Navigator]|Win32|function|function|BODY"
    );
}

#[test]
fn root_preload_runs_before_page_html_is_materialized() {
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://scripts.example.test/preload-order".to_owned(),
            html: concat!(
                "<!doctype html><html><body><div id='parsed'></div><script>",
                "const frame=document.createElement('iframe');",
                "document.body.appendChild(frame);",
                "window.pagePreloadEvidence=[",
                "window.preloadRuns,",
                "document.getElementById('parsed')!==null,",
                "typeof frame.contentWindow.preloadRuns",
                "].join('|');",
                "</script></body></html>"
            )
            .to_owned(),
            ..PageInit::default()
        }),
        iframe_hooks: vec![IframeHook::new(
            crate::iframe_hook::ROOT_PRELOAD_HOOK_NAME,
            concat!(
                "window.preloadRuns=(window.preloadRuns||0)+1;",
                "window.rootPreloadEvidence=[",
                "document.body.childElementCount,",
                "document.getElementById('parsed')===null,",
                "document.currentScript===null",
                "].join('|');"
            ),
        )],
        ..EdgeRuntimeOptions::default()
    })
    .expect("page with root preload");

    assert_eq!(text(&mut runtime, "rootPreloadEvidence"), "0|true|true");
    assert_eq!(
        text(&mut runtime, "pagePreloadEvidence"),
        "1|true|undefined"
    );
}

#[test]
fn parser_inserted_external_script_uses_network_replay_and_sees_complete_body() {
    let script_url = "https://assets.example.test/runtime.js";
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://www.example.test/app/index.html".to_owned(),
            html: concat!(
                "<!DOCTYPE html><html><head></head><body>",
                "<script>window.KPSDK={};KPSDK.start=performance.now();",
                "window.parserExecution=['inline'];</script>",
                "<script src=\"HTTPS://ASSETS.EXAMPLE.TEST:443/runtime.js#local-fragment\"></script>",
                "</body></html>"
            )
            .to_owned(),
            ..PageInit::default()
        }),
        network_replay: vec![NetworkReplayEntry::get(
            script_url,
            concat!(
                "parserExecution.push('external');",
                "for(let index=0;index<3;index++){",
                "const marker=document.createElement('div');",
                "marker.style.height=index===0?'23px':'0';",
                "document.body.appendChild(marker);",
                "}",
                "window.externalScriptEvidence=[",
                "document.body.childElementCount,",
                "document.body.clientHeight,",
                "document.currentScript.src,",
                "document.currentScript.isConnected,",
                "typeof KPSDK.start",
                "].join('|');",
                "window.externalScriptStack=String(new Error('external marker').stack);"
            )
            .as_bytes()
            .to_vec(),
        )],
        ..EdgeRuntimeOptions::default()
    })
    .expect("HTML page with replayed parser script");

    assert_eq!(
        text(
            &mut runtime,
            "parserExecution.join(',') + '||' + externalScriptEvidence"
        ),
        "inline,external||5|23|https://assets.example.test/runtime.js#local-fragment|true|number"
    );
    assert_eq!(
        text(
            &mut runtime,
            "externalScriptStack.includes('https://assets.example.test/runtime.js')"
        ),
        "true"
    );
}

#[test]
fn layout_uses_unserialized_css_lengths_before_exposing_computed_style() {
    let mut runtime = page_runtime();
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const values = ["1.01in", "1.0078in", "1.0054in"];
              return values.map(value => {
                const element = document.createElement("div");
                element.style.width = value;
                document.body.appendChild(element);
                return [
                  element.getBoundingClientRect().width,
                  getComputedStyle(element).width
                ].join(",");
              }).join("|");
            })()
            "#,
        ),
        "96.953125,96.9531px|96.734375,96.7344px|96.515625,96.5156px"
    );
}

#[test]
fn page_cookie_scope_uses_the_configured_host_and_request_path() {
    let mut runtime = page_runtime();
    assert_eq!(
        text(
            &mut runtime,
            r##"
            document.cookie = "hostOnly=one";
            document.cookie = "parent=two; Domain=example.test; Path=/app";
            document.cookie = "outside=three; Path=/elsewhere";
            [
              document.cookie,
              document.domain,
              (() => {
                document.domain = "example.test";
                return document.domain;
              })()
            ].join("|")
            "##,
        ),
        "hostOnly=one; parent=two|www.example.test|example.test"
    );
}

#[test]
fn invalid_page_init_is_rejected_before_v8_execution() {
    let error = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "http://insecure.example/".to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    })
    .err()
    .expect("insecure page must fail validation");
    assert!(error.contains("HTTPS"));
}

#[test]
fn edge_cssom_style_sheets_are_live_owner_backed_and_feed_computed_style() {
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://styles.example.test/page".to_owned(),
            html: r#"<!doctype html>
              <style id="page-style" media="screen">
                #target { color: rgb(1, 2, 3); padding-top: 9px; }
              </style>
              <div id="target" style="display: inline-block; width: 11px"></div>"#
                .to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    })
    .expect("CSSOM page");
    assert_eq!(
        text(
            &mut runtime,
            r##"
            (() => {
              const list = document.styleSheets;
              const style = document.getElementById("page-style");
              const first = style.sheet;
              const target = document.getElementById("target");
              const computed = getComputedStyle(target);
              let readonly;
              try { computed.color = "red"; readonly = "accepted"; }
              catch (error) { readonly = error.name; }
              let replaceSyncError;
              try { first.replaceSync("#target {}"); replaceSyncError = "accepted"; }
              catch (error) { replaceSyncError = error.name; }
              style.disabled = true;
              const disabledShape = [
                style.sheet === first,
                style.sheet.disabled,
                style.hasAttribute("disabled")
              ].join(",");
              style.disabled = false;
              style.media = "print";
              const mediaShape = [
                style.sheet === first,
                style.sheet.media.mediaText,
                style.getAttribute("media")
              ].join(",");
              style.media = "screen";
              style.textContent = "#target { color: rgb(4, 5, 6); }";
              const second = style.sheet;
              const beforeRemove = [
                list === document.styleSheets,
                list.length,
                first instanceof CSSStyleSheet,
                first.ownerNode === style,
                first.media.mediaText,
                first.cssRules.length,
                computed.color,
                computed.paddingTop,
                computed.display,
                computed.width,
                readonly,
                replaceSyncError,
                disabledShape,
                mediaShape,
                second !== first,
                second.cssRules.length
              ].join("|");
              style.remove();
              return beforeRemove + "|" + [
                style.sheet,
                list.length,
                Array.from(list).includes(second)
              ].join("|");
            })()
            "##,
        ),
        concat!(
            "true|1|true|true|screen|1|rgb(1, 2, 3)|9px|inline-block|11px|",
            "NoModificationAllowedError|NotAllowedError|true,true,false|",
            "true,print,print|true|1||0|false"
        )
    );
}

#[test]
fn current_script_tracks_classic_execution_and_constructed_sheets_are_validated() {
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://scripts.example.test/page".to_owned(),
            html: r#"<!doctype html><script id="parser-script">
              window.parserCurrentScript = [
                document.currentScript === null,
                document.currentScript.id,
                document.currentScript.isConnected
              ].join(",");
            </script>"#
                .to_owned(),
            ..Default::default()
        }),
        ..Default::default()
    })
    .expect("script page");
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              const values = [parserCurrentScript, document.currentScript];
              const script = document.createElement("script");
              script.id = "dynamic-script";
              script.text = "window.dynamicCurrentScript = [" +
                "document.currentScript === null," +
                "document.currentScript.id," +
                "document.currentScript.isConnected].join(',')";
              document.head.appendChild(script);
              values.push(dynamicCurrentScript, document.currentScript);
              const sheet = new CSSStyleSheet({media: "print", disabled: true});
              const replaced = sheet.replaceSync(".a { color: red; }");
              const assigned = [sheet];
              document.adoptedStyleSheets = assigned;
              let invalid;
              try { document.adoptedStyleSheets = [{}]; invalid = "accepted"; }
              catch (error) { invalid = error.name; }
              values.push(
                replaced,
                sheet.media.mediaText,
                sheet.disabled,
                sheet.cssRules.length,
                document.adoptedStyleSheets === assigned,
                document.adoptedStyleSheets[0] === sheet,
                invalid
              );
              return values.join("|");
            })()
            "#,
        ),
        "false,parser-script,true||false,dynamic-script,true|||print|true|1|false|true|TypeError"
    );
}

#[test]
fn replayable_external_scripts_and_stylesheets_complete_on_later_task_turns() {
    let mut runtime = page_runtime();
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (async () => {
              const scriptEvents = ["before"];
              window.externalEvents = scriptEvents;
              const script = document.createElement("script");
              script.id = "external-script";
              script.src = "data:text/javascript,externalEvents.push([document.currentScript.id,document.currentScript.isConnected].join(','))";
              const scriptDone = new Promise(resolve => {
                script.onload = () => {
                  scriptEvents.push("load:" + (document.currentScript === null));
                  resolve();
                };
              });
              document.head.appendChild(script);
              scriptEvents.push("after");
              await scriptDone;

              const target = document.createElement("div");
              target.id = "link-target";
              document.body.appendChild(target);
              const linkEvents = ["before"];
              const link = document.createElement("link");
              link.rel = "stylesheet";
              link.href = "data:text/css,%23link-target%7Bcolor%3Argb(7%2C8%2C9)%7D";
              const linkDone = new Promise(resolve => {
                link.onload = () => {
                  linkEvents.push("load");
                  resolve();
                };
              });
              linkEvents.push(String(link.sheet));
              document.head.appendChild(link);
              linkEvents.push(String(link.sheet instanceof CSSStyleSheet));
              await linkDone;

              const dependencyUrl = URL.createObjectURL(new Blob([
                "export const answer = 42;"
              ], {type: "text/javascript"}));
              const moduleUrl = URL.createObjectURL(new Blob([
                "import { answer } from '" + dependencyUrl + "';",
                "window.moduleValues = [",
                "document.currentScript === null,",
                "answer,",
                "import.meta.url.startsWith('blob:')",
                "].join(',');"
              ], {type: "text/javascript"}));
              const module = document.createElement("script");
              module.type = "module";
              module.src = moduleUrl;
              const moduleDone = new Promise(resolve => {
                module.onload = resolve;
              });
              document.head.appendChild(module);
              const moduleWasDeferred = typeof window.moduleValues;
              await moduleDone;
              URL.revokeObjectURL(moduleUrl);
              URL.revokeObjectURL(dependencyUrl);
              return [
                scriptEvents.join(","),
                linkEvents.join(","),
                link.sheet.ownerNode === link,
                link.sheet.cssRules.length,
                getComputedStyle(target).color,
                document.styleSheets[document.styleSheets.length - 1] === link.sheet,
                moduleWasDeferred,
                moduleValues,
                document.currentScript
              ].join("|");
            })()
            "#,
        ),
        concat!(
            "before,after,external-script,true,load:true|",
            "before,null,true,load|true|1|rgb(7, 8, 9)|true|",
            "undefined|true,42,true|"
        )
    );
}
