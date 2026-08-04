use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::String(value) | Evaluation::Other(value) | Evaluation::Number(value) => value,
        value => value.to_string(),
    }
}

#[test]
fn event_propagates_through_window_document_and_nodes() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const parentNode = document.createElement("section");
          const targetNode = document.createElement("button");
          document.body.appendChild(parentNode);
          parentNode.appendChild(targetNode);
          const order = [];
          const label = value =>
            value === window ? "window" :
            value === document ? "document" :
            value === document.body ? "body" :
            value === document.documentElement ? "html" :
            value === parentNode ? "parent" :
            value === targetNode ? "target" : "other";
          const listen = (object, name, capture) => object.addEventListener(
            "edge-propagation",
            event => {
              order.push(
                name + ":" + event.eventPhase + ":" +
                label(event.currentTarget) + ":" +
                label(event.target)
              );
              if (name === "target-capture") {
                order.push(
                  "path:" + event.composedPath().map(label).join(">")
                );
                order.push("window-event:" + (window.event === event));
              }
            },
            capture
          );
          listen(window, "window-capture", true);
          listen(document, "document-capture", true);
          listen(parentNode, "parent-capture", true);
          listen(targetNode, "target-capture", true);
          listen(targetNode, "target-bubble", false);
          listen(parentNode, "parent-bubble", false);
          listen(document, "document-bubble", false);
          listen(window, "window-bubble", false);
          const event = new Event("edge-propagation", {
            bubbles: true,
            cancelable: true
          });
          const allowed = targetNode.dispatchEvent(event);
          return [
            order.join(","),
            allowed,
            event.target === targetNode,
            event.currentTarget === null,
            event.eventPhase,
            event.composedPath().length,
            event.defaultPrevented,
            event.isTrusted,
            event.timeStamp >= 0,
            window.event === undefined
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "window-capture:1:window:target,document-capture:1:document:target,parent-capture:1:parent:target,target-capture:2:target:target,path:target>parent>body>html>document>window,window-event:true,target-bubble:2:target:target,parent-bubble:3:parent:target,document-bubble:3:document:target,window-bubble:3:window:target|true|true|true|0|0|false|false|true|true"
    );
}

#[test]
fn event_listener_options_mutation_and_exception_isolation_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const target = new EventTarget();
          const counts = {
            duplicate: 0, object: 0, once: 0, removed: 0,
            aborted: 0, afterThrow: 0, added: 0
          };
          const duplicate = () => counts.duplicate++;
          target.addEventListener("duplicate", duplicate);
          target.addEventListener("duplicate", duplicate, { once: true, passive: true });
          target.dispatchEvent(new Event("duplicate"));
          target.dispatchEvent(new Event("duplicate"));

          const listenerObject = {
            handleEvent(event) {
              if (this === listenerObject && event.currentTarget === target) {
                counts.object++;
              }
            }
          };
          target.addEventListener("object", listenerObject);
          target.dispatchEvent(new Event("object"));

          const once = () => counts.once++;
          target.addEventListener("once", once, { once: true });
          target.dispatchEvent(new Event("once"));
          target.dispatchEvent(new Event("once"));

          const removable = () => counts.removed++;
          target.addEventListener("remove", removable, true);
          target.removeEventListener("remove", removable, false);
          target.dispatchEvent(new Event("remove"));
          target.removeEventListener("remove", removable, true);
          target.dispatchEvent(new Event("remove"));

          const controller = new AbortController();
          target.addEventListener(
            "abortable",
            () => counts.aborted++,
            { signal: controller.signal }
          );
          controller.abort("done");
          target.dispatchEvent(new Event("abortable"));
          target.addEventListener(
            "already-aborted",
            () => counts.aborted++,
            { signal: controller.signal }
          );
          target.dispatchEvent(new Event("already-aborted"));

          let removedDuringDispatch = 0;
          const later = () => removedDuringDispatch++;
          target.addEventListener("mutation", () => {
            target.removeEventListener("mutation", later);
            target.addEventListener("mutation", () => counts.added++);
          });
          target.addEventListener("mutation", later);
          target.dispatchEvent(new Event("mutation"));
          target.dispatchEvent(new Event("mutation"));

          target.addEventListener("throws", () => { throw new Error("listener"); });
          target.addEventListener("throws", () => counts.afterThrow++);
          target.dispatchEvent(new Event("throws"));

          const passiveEvent = new Event("passive", { cancelable: true });
          target.addEventListener("passive", event => event.preventDefault(), { passive: true });
          const passiveAllowed = target.dispatchEvent(passiveEvent);
          const activeEvent = new Event("active", { cancelable: true });
          target.addEventListener("active", event => event.preventDefault());
          const activeAllowed = target.dispatchEvent(activeEvent);

          let reentrant = "";
          target.addEventListener("reentrant", event => {
            try { target.dispatchEvent(event); }
            catch (error) { reentrant = error.name; }
          });
          target.dispatchEvent(new Event("reentrant"));

          const optionOrder = [];
          const options = {};
          for (const name of ["capture", "once", "passive", "signal"]) {
            Object.defineProperty(options, name, {
              get() {
                optionOrder.push(name);
                return undefined;
              }
            });
          }
          target.addEventListener("option-order", () => {}, options);
          let nullSignal = "";
          try {
            target.addEventListener(
              "null-signal", () => {}, { signal: null }
            );
          } catch (error) {
            nullSignal = error.name;
          }

          return [
            counts.duplicate, counts.object, counts.once,
            counts.removed, counts.aborted, removedDuringDispatch,
            counts.added, counts.afterThrow,
            passiveAllowed, passiveEvent.defaultPrevented,
            activeAllowed, activeEvent.defaultPrevented,
            reentrant,
            optionOrder.join(","),
            nullSignal
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "2|1|1|1|0|0|1|1|true|false|false|true|InvalidStateError|capture,once,passive,signal|TypeError"
    );
}

#[test]
fn event_stopping_legacy_initialization_and_handlers_work() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const parentNode = document.createElement("div");
          const targetNode = document.createElement("span");
          document.body.appendChild(parentNode);
          parentNode.appendChild(targetNode);
          const calls = [];
          targetNode.addEventListener("stop", event => {
            calls.push("first");
            event.stopPropagation();
            event.cancelBubble = false;
          });
          targetNode.addEventListener("stop", () => calls.push("second"));
          parentNode.addEventListener("stop", () => calls.push("parent"));
          targetNode.dispatchEvent(new Event("stop", { bubbles: true }));

          const immediate = [];
          targetNode.addEventListener("immediate", event => {
            immediate.push("first");
            event.stopImmediatePropagation();
          });
          targetNode.addEventListener("immediate", () => immediate.push("second"));
          targetNode.dispatchEvent(new Event("immediate"));

          window.onmessage = event => {
            calls.push(event.currentTarget === window ? "window-handler" : "bad-handler");
            return false;
          };
          const message = new Event("message", { cancelable: true });
          const messageAllowed = window.dispatchEvent(message);
          window.onmessage = null;

          const handlerOrder = [];
          const handlerElement = document.createElement("div");
          handlerElement.addEventListener(
            "click", () => handlerOrder.push("a")
          );
          handlerElement.onclick = () => handlerOrder.push("handler1");
          handlerElement.addEventListener(
            "click", () => handlerOrder.push("b")
          );
          handlerElement.onclick = () => handlerOrder.push("handler2");
          handlerElement.dispatchEvent(new Event("click"));
          handlerElement.onclick = null;
          handlerElement.onclick = () => handlerOrder.push("handler3");
          handlerElement.dispatchEvent(new Event("click"));

          const legacy = document.createEvent("Event");
          let beforeInit = "";
          try { targetNode.dispatchEvent(legacy); }
          catch (error) { beforeInit = error.name; }
          legacy.initEvent("legacy", true, true);
          const afterInit = targetNode.dispatchEvent(legacy);

          const emptyType = targetNode.dispatchEvent(new Event(""));
          const trustedShape = (() => {
            const event = new Event("shape");
            const descriptor =
              Object.getOwnPropertyDescriptor(event, "isTrusted");
            return [
              !Object.prototype.hasOwnProperty.call(
                Event.prototype, "isTrusted"
              ),
              descriptor.enumerable,
              descriptor.configurable,
              descriptor.set === undefined,
              descriptor.get.name,
              descriptor.get.length,
              Function.prototype.toString.call(descriptor.get),
              Event.prototype.initEvent.length,
              Object.getOwnPropertyNames(Event.prototype).join(",")
            ].join("~");
          })();
          const inheritedInit = [
            new AnimationPlaybackEvent("a", {
              bubbles: true, cancelable: true, composed: true
            }),
            new DeviceOrientationEvent("b", {
              bubbles: true, cancelable: true, composed: true
            }),
            new MIDIMessageEvent("c", {
              bubbles: true, cancelable: true, composed: true
            }),
            new BeforeInstallPromptEvent("d", {
              bubbles: true, cancelable: true, composed: true
            })
          ].every(event =>
            event.bubbles && event.cancelable && event.composed
          );
          const returnValueEvent = new Event("return-value", { cancelable: true });
          returnValueEvent.returnValue = false;
          const plainObjectError = (() => {
            try { targetNode.dispatchEvent({ type: "fake" }); }
            catch (error) { return error.name; }
            return "";
          })();
          return [
            calls.join(","), immediate.join(","),
            messageAllowed, message.defaultPrevented,
            beforeInit, afterInit, emptyType,
            returnValueEvent.defaultPrevented,
            returnValueEvent.returnValue,
            plainObjectError,
            trustedShape,
            inheritedInit,
            handlerOrder.join(","),
            Function.prototype.toString.call(Event.prototype.preventDefault),
            Object.getPrototypeOf(Event.prototype) === Object.prototype,
            Object.getPrototypeOf(EventTarget.prototype) === Object.prototype
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "first,second,window-handler|first|false|true|InvalidStateError|true|true|true|false|TypeError|true~true~false~true~get isTrusted~0~function get isTrusted() { [native code] }~1~type,target,currentTarget,eventPhase,bubbles,cancelable,defaultPrevented,composed,timeStamp,srcElement,returnValue,cancelBubble,NONE,CAPTURING_PHASE,AT_TARGET,BUBBLING_PHASE,composedPath,initEvent,preventDefault,stopImmediatePropagation,stopPropagation,constructor|true|a,handler2,b,a,b,handler3|function preventDefault() { [native code] }|true|true"
    );
}

#[test]
fn composed_events_cross_shadow_root_and_non_composed_events_do_not() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const host = document.createElement("div");
          document.body.appendChild(host);
          const shadow = host.attachShadow({ mode: "open" });
          const inner = document.createElement("button");
          shadow.appendChild(inner);
          const pathName = value =>
            value === inner ? "inner" :
            value === shadow ? "shadow" :
            value === host ? "host" :
            value === document.body ? "body" :
            value === document.documentElement ? "html" :
            value === document ? "document" :
            value === window ? "window" : "other";
          let nonComposedPath = "";
          let composedPath = "";
          let hostCalls = 0;
          let hostTarget = "";
          let nonBubblingHost = "";
          inner.addEventListener("shadow-private", event => {
            nonComposedPath = event.composedPath().map(pathName).join(">");
          });
          inner.addEventListener("shadow-public", event => {
            composedPath = event.composedPath().map(pathName).join(">");
          });
          host.addEventListener("shadow-private", () => hostCalls++);
          host.addEventListener("shadow-public", event => {
            hostCalls++;
            hostTarget = event.target === host ?
              "host:" + event.eventPhase : "wrong";
          });
          inner.dispatchEvent(new Event("shadow-private", {
            bubbles: true,
            composed: false
          }));
          const publicEvent = new Event("shadow-public", {
            bubbles: true,
            composed: true
          });
          inner.dispatchEvent(publicEvent);
          host.addEventListener("shadow-at-target", event => {
            nonBubblingHost =
              (event.target === host ? "host:" : "wrong:") +
              event.eventPhase;
          });
          inner.dispatchEvent(new Event("shadow-at-target", {
            bubbles: false,
            composed: true
          }));
          return [
            nonComposedPath,
            composedPath,
            hostCalls,
            hostTarget,
            publicEvent.target === host,
            nonBubblingHost
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "inner>shadow|inner>shadow>host>body>html>document>window|1|host:2|true|host:2"
    );
}

#[test]
fn closed_shadow_paths_and_abort_attribute_order_match_edge() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const host = document.createElement("div");
          document.body.appendChild(host);
          const shadow = host.attachShadow({ mode: "closed" });
          const inner = document.createElement("span");
          shadow.appendChild(inner);
          const name = value =>
            value === inner ? "inner" :
            value === shadow ? "shadow" :
            value === host ? "host" :
            value === document.body ? "body" :
            value === document.documentElement ? "html" :
            value === document ? "document" :
            value === window ? "window" : "other";
          let insidePath = "";
          let outsidePath = "";
          inner.addEventListener("closed", event => {
            insidePath = event.composedPath().map(name).join(">");
          });
          document.addEventListener("closed", event => {
            outsidePath = event.composedPath().map(name).join(">");
          });
          inner.dispatchEvent(new Event("closed", {
            bubbles: true,
            composed: true
          }));

          const controller = new AbortController();
          const abortOrder = [];
          controller.signal.addEventListener(
            "abort", () => abortOrder.push("a")
          );
          controller.signal.onabort = () => abortOrder.push("handler1");
          controller.signal.addEventListener(
            "abort", () => abortOrder.push("b")
          );
          controller.signal.onabort = () => abortOrder.push("handler2");
          controller.abort();
          return [
            insidePath,
            outsidePath,
            abortOrder.join(",")
          ].join("|");
        })()
        "#,
    );
    assert_eq!(
        result,
        "inner>shadow>host>body>html>document>window|host>body>html>document>window|a,handler2,b"
    );
}
