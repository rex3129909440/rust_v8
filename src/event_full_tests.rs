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

#[test]
fn html_click_uses_pointer_event_propagation_and_activation_defaults() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    let result = text(
        &mut runtime,
        r##"
        (() => {
          const outer = document.createElement("div");
          const button = document.createElement("button");
          outer.appendChild(button);
          document.body.appendChild(outer);
          const order = [];
          let shape = "";
          outer.addEventListener("click", event => order.push("capture:" + event.eventPhase), true);
          button.addEventListener("click", event => {
            order.push("target:" + event.eventPhase);
            shape = [
              event.constructor.name, event.isTrusted, event.bubbles,
              event.cancelable, event.composed, event.detail,
              event.button, event.buttons, event.pointerId, event.pointerType
            ].join(",");
          });
          button.onclick = event => order.push("onclick:" + event.eventPhase);
          outer.addEventListener("click", event => order.push("bubble:" + event.eventPhase));
          button.click();

          const disabled = document.createElement("button");
          disabled.disabled = true;
          let disabledHits = 0;
          disabled.addEventListener("click", () => disabledHits++);
          document.body.appendChild(disabled);
          disabled.click();

          const checkbox = document.createElement("input");
          checkbox.type = "checkbox";
          document.body.appendChild(checkbox);
          const checkboxEvents = [];
          for (const type of ["click", "input", "change"]) {
            checkbox.addEventListener(type, event =>
              checkboxEvents.push(`${type}:${checkbox.checked}:${event.isTrusted}:${event.cancelable}`));
          }
          checkbox.click();
          const cancelled = document.createElement("input");
          cancelled.type = "checkbox";
          document.body.appendChild(cancelled);
          let during = false;
          cancelled.addEventListener("click", event => {
            during = cancelled.checked;
            event.preventDefault();
          });
          cancelled.click();

          const first = document.createElement("input");
          const second = document.createElement("input");
          first.type = second.type = "radio";
          first.name = second.name = "group";
          first.checked = true;
          document.body.append(first, second);
          second.click();

          const label = document.createElement("label");
          const labelled = document.createElement("input");
          labelled.type = "checkbox";
          label.append(labelled);
          document.body.appendChild(label);
          const labelOrder = [];
          label.addEventListener("click", event => labelOrder.push(event.target.tagName));
          labelled.addEventListener("click", () => labelOrder.push("input"));
          label.click();
          const form = document.createElement("form");
          const formInput = document.createElement("input");
          formInput.defaultValue = "default";
          formInput.value = "changed";
          const submit = document.createElement("button");
          const reset = document.createElement("button");
          submit.type = "submit";
          reset.type = "reset";
          form.append(formInput, submit, reset);
          document.body.appendChild(form);
          const formEvents = [];
          form.addEventListener("submit", event => {
            formEvents.push([
              event.constructor.name, event.isTrusted, event.bubbles,
              event.cancelable, event.submitter === submit
            ].join(":"));
            event.preventDefault();
          });
          form.addEventListener("reset", event => formEvents.push([
            event.constructor.name, event.isTrusted, event.bubbles, event.cancelable
          ].join(":")));
          submit.click();
          reset.click();
          const anchor = document.createElement("a");
          anchor.href = "#activation";
          document.body.appendChild(anchor);
          anchor.click();
          return [
            shape, order.join("/"), disabledHits,
            disabled.hasAttribute("disabled"),
            checkbox.checked, checkboxEvents.join("/"),
            during, cancelled.checked,
            first.checked, second.checked,
            labelled.checked, labelOrder.join("/"),
            formEvents.join("/"), formInput.value, location.hash
          ].join("|");
        })()
        "##,
    );
    assert_eq!(
        result,
        concat!(
            "PointerEvent,false,true,true,true,0,0,0,-1,|",
            "capture:1/target:2/onclick:2/bubble:3|0|true|true|",
            "click:true:false:true/input:true:true:false/change:true:true:false|",
            "true|false|false|true|true|LABEL/input/INPUT|",
            "SubmitEvent:true:true:true:true/Event:true:true:true|default|#activation"
        )
    );
}

#[test]
fn host_keyboard_and_wheel_follow_edge_event_and_default_action_shape() {
    let mut runtime = EdgeRuntime::new().expect("Edge runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const records = [];
          const input = document.createElement("input");
          input.id = "keyboard-input";
          input.value = "xy";
          input.style.cssText = "position:absolute;left:20px;top:20px;width:160px;height:30px";
          const box = document.createElement("div");
          box.id = "wheel-box";
          box.style.cssText = "position:absolute;left:20px;top:90px;width:200px;height:100px;overflow:auto";
          const content = document.createElement("div");
          content.style.cssText = "height:600px;width:500px";
          box.appendChild(content);
          document.body.append(input, box);
          input.focus();
          input.setSelectionRange(1, 1);
          for (const type of ["keydown", "keypress", "beforeinput", "input", "keyup", "wheel"]) {
            document.addEventListener(type, event => records.push([
              type, event.constructor.name, event.isTrusted, event.bubbles,
              event.cancelable, event.composed, event.key || "", event.code || "",
              event.charCode || 0, event.keyCode || 0, event.which || 0,
              event.data === undefined ? "undefined" : event.data,
              event.inputType || "", event.deltaY || 0, event.wheelDeltaY || 0,
              input.value, input.selectionStart, box.scrollTop
            ].join(":")));
          }
          globalThis.__hostInputResult = () => [
            records.join("/"), input.value, input.selectionStart,
            input.selectionEnd, box.scrollTop,
            performance.eventCounts.get("keydown"),
            performance.eventCounts.get("keypress"),
            performance.eventCounts.get("beforeinput"),
            performance.eventCounts.get("input"),
            performance.eventCounts.get("keyup"),
            performance.eventCounts.get("wheel") || 0,
            (() => {
              const event = new WheelEvent("wheel", {deltaX: 2, deltaY: 3});
              return [event.wheelDeltaX, event.wheelDeltaY, event.wheelDelta].join(",");
            })()
          ].join("|");
        })()
        "#,
    );
    assert!(
        runtime
            .dispatch_host_keyboard(&crate::HostKeyboardInput::printable("a", "KeyA"))
            .expect("keyboard dispatch")
    );
    assert!(
        runtime
            .dispatch_host_wheel(&crate::HostWheelInput::pixels(100.0, 130.0, 0.0, 53.0))
            .expect("wheel dispatch")
    );
    let result = text(&mut runtime, "__hostInputResult()");
    assert!(result.starts_with(
        "keydown:KeyboardEvent:true:true:true:true:a:KeyA:0:65:65:undefined::0:0:xy:1:0/keypress:KeyboardEvent:true:true:true:true:a:KeyA:97:97:97:undefined::0:0:xy:1:0/beforeinput:InputEvent:true:true:true:true:::0:0:0:a:insertText:0:0:xy:1:0/input:InputEvent:true:true:false:true:::0:0:0:a:insertText:0:0:xay:2:0/keyup:KeyboardEvent:true:true:true:true:a:KeyA:0:65:65:undefined::0:0:xay:2:0/wheel:WheelEvent:true:true:false:true:::0:0:0:undefined::53:-120:xay:2:53|"
    ), "{result}");
    assert!(result.ends_with("xay|2|2|53|1|1|1|1|1|0|2,3,3"), "{result}");
}

#[test]
fn host_click_runs_trusted_checkbox_activation_and_legacy_mouse_which() {
    let mut runtime = EdgeRuntime::new().expect("trusted checkbox click runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          const checkbox = document.createElement("input");
          checkbox.type = "checkbox";
          checkbox.style.cssText =
            "position:fixed;left:30px;top:40px;width:20px;height:20px";
          document.body.appendChild(checkbox);
          globalThis.__trustedCheckboxEvents = [];
          for (const type of ["click", "input", "change"]) {
            checkbox.addEventListener(type, event => __trustedCheckboxEvents.push([
              type, event.isTrusted, event.which ?? "undefined",
              checkbox.checked, checkbox.value
            ].join(":")));
          }
          globalThis.__trustedCheckbox = checkbox;
        })()
        "#,
    );
    assert!(
        runtime
            .dispatch_host_click(&crate::HostClickInput::primary(35.0, 45.0))
            .expect("trusted checkbox click")
    );
    assert_eq!(
        text(
            &mut runtime,
            "[__trustedCheckbox.checked,__trustedCheckbox.value,__trustedCheckboxEvents.join('/')].join('|')"
        ),
        "true|on|click:true:1:true:on/input:true:undefined:true:on/change:true:undefined:true:on"
    );
}

#[test]
fn host_drag_matches_edge_sequence_and_data_transfer_branding() {
    let mut runtime = EdgeRuntime::new().expect("host drag runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          document.body.innerHTML = `
            <div id="source" draggable="true" style="position:fixed;left:20px;top:20px;width:80px;height:50px">source</div>
            <div id="target" style="position:fixed;left:200px;top:20px;width:100px;height:80px">target</div>
          `;
          globalThis.__dragEvents = [];
          globalThis.__dragAccess = [];
          for (const type of [
            "pointerdown", "mousedown", "dragstart", "pointercancel",
            "pointerout", "pointerleave", "drag", "dragenter", "dragleave",
            "dragover", "drop", "dragend", "pointerup", "mouseup"
          ]) document.addEventListener(type, event => {
            if (!event.target.id) return;
            __dragEvents.push([
              type, event.target.id, event.which, event.cancelable,
              event.composed, event.dataTransfer?.types.join(",") ?? "-"
            ].join(":"));
          }, true);
          source.addEventListener("dragstart", event => {
            event.dataTransfer.setData("text/plain", "payload");
          });
          source.addEventListener("drag", event => __dragAccess.push(`drag:${event.dataTransfer.getData("text/plain")}`));
          source.addEventListener("dragend", event => __dragAccess.push(`dragend:${event.dataTransfer.getData("text/plain")}`));
          target.addEventListener("dragover", event => {
            __dragAccess.push(`dragover:${event.dataTransfer.getData("text/plain")}`);
            event.preventDefault();
          });
          target.addEventListener("drop", event => {
            globalThis.__dropData = event.dataTransfer.getData("text/plain");
            __dragAccess.push(`drop:${globalThis.__dropData}`);
          });
        })()
        "#,
    );
    assert!(
        runtime
            .dispatch_host_drag(&crate::HostDragInput::between(50.0, 40.0, 230.0, 40.0))
            .expect("host drag dispatch")
    );
    assert_eq!(
        text(
            &mut runtime,
            "[__dragEvents.join('/'),__dropData].join('|')"
        ),
        concat!(
            "pointerdown:source:1:true:true:-/",
            "mousedown:source:1:true:true:-/",
            "pointerout:source:0:true:true:-/",
            "pointerleave:source:0:false:false:-/",
            "dragstart:source:1:true:true:/",
            "pointercancel:target:1:false:true:-/",
            "pointerout:target:1:true:true:-/",
            "pointerleave:target:1:false:false:-/",
            "drag:source:1:true:true:text/plain/",
            "dragenter:target:1:true:true:text/plain/",
            "dragover:target:1:true:true:text/plain/",
            "drop:target:1:true:true:text/plain/",
            "dragend:source:1:false:true:text/plain|payload"
        )
    );
    assert_eq!(
        text(&mut runtime, "__dragAccess.join('/')"),
        "drag:/dragover:/drop:payload/dragend:"
    );
    assert_eq!(
        text(
            &mut runtime,
            r#"
            (() => {
              let rejected = false;
              try { new DragEvent("x", {dataTransfer: {}}); }
              catch (error) { rejected = error instanceof TypeError; }
              const transfer = new DataTransfer();
              const accepted = new DragEvent("x", {dataTransfer: transfer});
              return [rejected, accepted.dataTransfer === transfer].join("|");
            })()
            "#
        ),
        "true|true"
    );
}

#[test]
fn host_touch_tap_multitouch_and_pen_follow_edge_trusted_sequences() {
    let mut runtime = EdgeRuntime::new().expect("touch and pen runtime");
    text(
        &mut runtime,
        r#"
        (() => {
          document.body.innerHTML = `
            <button id="left" style="position:fixed;left:20px;top:20px;width:100px;height:80px;touch-action:none">left</button>
            <button id="right" style="position:fixed;left:200px;top:20px;width:100px;height:80px;touch-action:none">right</button>
          `;
          globalThis.__pointerEvents = [];
          for (const type of [
            "pointerover", "pointerenter", "pointerdown", "touchstart",
            "pointerup", "pointerout", "pointerleave", "touchend",
            "mouseover", "mouseenter", "mousemove", "mousedown", "mouseup", "click"
          ]) document.addEventListener(type, event => {
            if (!event.target.id) return;
            __pointerEvents.push([
              type, event.target.id, event.constructor.name, event.isTrusted,
              event.which, event.pointerType ?? "-", event.pointerId ?? "-",
              event.isPrimary ?? "-", event.sourceCapabilities?.firesTouchEvents ?? "-",
              event.touches ? Array.from(event.touches, touch => `${touch.identifier}:${touch.target.id}`).join(",") : "-",
              event.targetTouches ? Array.from(event.targetTouches, touch => `${touch.identifier}:${touch.target.id}`).join(",") : "-",
              event.changedTouches ? Array.from(event.changedTouches, touch => `${touch.identifier}:${touch.target.id}`).join(",") : "-"
            ].join(":"));
          }, true);
        })()
        "#,
    );
    assert!(
        runtime
            .dispatch_host_touch(&crate::HostTouchInput::start(0, 60.0, 55.0))
            .expect("touch start")
    );
    assert!(
        runtime
            .dispatch_host_touch(&crate::HostTouchInput::end(0, 60.0, 55.0))
            .expect("touch end")
    );
    assert_eq!(
        text(
            &mut runtime,
            "__pointerEvents.map(v => v.split(':')[0]).join(',')"
        ),
        "pointerover,pointerenter,pointerdown,touchstart,pointerup,pointerout,pointerleave,touchend,mouseover,mouseenter,mousemove,mousedown,mouseup,click"
    );
    assert_eq!(
        text(
            &mut runtime,
            "[document.activeElement.id,__pointerEvents[3],__pointerEvents[7],__pointerEvents[13]].join('|')"
        ),
        concat!(
            "left|touchstart:left:TouchEvent:true:0:-:-:-:true:0:left:0:left:0:left|",
            "touchend:left:TouchEvent:true:0:-:-:-:true:::0:left|",
            "click:left:PointerEvent:true:1:touch:2:false:true:-:-:-"
        )
    );

    text(&mut runtime, "__pointerEvents.length = 0");
    assert!(
        runtime
            .dispatch_host_touch(&crate::HostTouchInput::start(10, 60.0, 55.0))
            .expect("first multitouch start")
    );
    assert!(
        runtime
            .dispatch_host_touch(&crate::HostTouchInput::start(11, 240.0, 55.0))
            .expect("second multitouch start")
    );
    assert!(
        runtime
            .dispatch_host_touch(&crate::HostTouchInput::end(11, 240.0, 55.0))
            .expect("second multitouch end")
    );
    assert!(
        runtime
            .dispatch_host_touch(&crate::HostTouchInput::end(10, 60.0, 55.0))
            .expect("first multitouch end")
    );
    assert_eq!(
        text(
            &mut runtime,
            "__pointerEvents.filter(v => v.startsWith('click:')).length"
        ),
        "0"
    );
    assert!(text(&mut runtime, "__pointerEvents.join('/')").contains(
        "touchstart:right:TouchEvent:true:0:-:-:-:true:10:left,11:right:11:right:11:right"
    ));

    text(&mut runtime, "__pointerEvents.length = 0");
    let mut pen = crate::HostPenInput::hover(60.0, 55.0);
    pen.tilt_x = 10;
    pen.tilt_y = 20;
    pen.twist = 30;
    pen.tangential_pressure = 0.25;
    assert!(runtime.dispatch_host_pen(&pen).expect("pen hover"));
    pen.phase = crate::HostPenPhase::Down;
    pen.pressure = 0.65;
    assert!(runtime.dispatch_host_pen(&pen).expect("pen down"));
    pen.phase = crate::HostPenPhase::Move;
    pen.client_x = 75.0;
    pen.client_y = 65.0;
    pen.pressure = 0.7;
    pen.tilt_x = 15;
    pen.tilt_y = 25;
    pen.twist = 40;
    pen.tangential_pressure = 0.3;
    assert!(runtime.dispatch_host_pen(&pen).expect("pen move"));
    pen.phase = crate::HostPenPhase::Up;
    pen.pressure = 0.0;
    assert!(runtime.dispatch_host_pen(&pen).expect("pen up"));
    assert_eq!(
        text(
            &mut runtime,
            "__pointerEvents.map(v => v.split(':')[0]).join(',')"
        ),
        "pointerover,pointerenter,mouseover,mouseenter,mousemove,pointerdown,mousedown,mousemove,pointerup,mouseup,click"
    );
    assert_eq!(
        text(&mut runtime, "__pointerEvents.at(-1)"),
        "click:left:PointerEvent:true:1:pen:5:false:false:-:-:-"
    );
}

#[test]
fn script_focus_and_blur_follow_trusted_edge_focus_event_transitions() {
    let mut runtime = EdgeRuntime::new().expect("script focus runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          document.body.innerHTML = `
            <input id="first"><input id="second">
            <div id="plain"></div><div id="negative" tabindex="-1"></div>
            <input id="disabled" disabled>
            <input id="hiddenInput" type="hidden">
            <input id="displayNone" style="display:none">
            <input id="visibilityHidden" style="visibility:hidden">
            <div inert><input id="inertInput"></div>
            <fieldset disabled><input id="fieldsetInput"></fieldset>
          `;
          const events = [];
          for (const target of [first, second, negative]) {
            for (const type of ["blur", "focusout", "focus", "focusin"]) {
              target.addEventListener(type, event => events.push([
                target.id, type, event.constructor.name, event.isTrusted,
                event.bubbles, event.cancelable, event.composed,
                event.relatedTarget?.id || "null", event.view === window
              ].join(":")));
            }
          }
          const states = [];
          const step = (name, action) => {
            action();
            states.push(`${name}:${document.activeElement.id || document.activeElement.tagName}`);
          };
          step("first", () => first.focus());
          step("first-again", () => first.focus());
          step("second", () => second.focus());
          step("second-blur", () => second.blur());
          step("plain", () => plain.focus());
          step("negative", () => negative.focus());
          step("negative-blur", () => negative.blur());
          step("disabled", () => disabled.focus());
          step("hidden", () => hiddenInput.focus());
          step("display-none", () => displayNone.focus());
          step("visibility-hidden", () => visibilityHidden.focus());
          step("inert", () => inertInput.focus());
          step("fieldset-disabled", () => fieldsetInput.focus());
          const detached = document.createElement("input");
          step("detached", () => detached.focus());
          return `${events.join("/")}|${states.join("/")}`;
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "first:focus:FocusEvent:true:false:false:true:null:true/",
            "first:focusin:FocusEvent:true:true:false:true:null:true/",
            "first:blur:FocusEvent:true:false:false:true:second:true/",
            "first:focusout:FocusEvent:true:true:false:true:second:true/",
            "second:focus:FocusEvent:true:false:false:true:first:true/",
            "second:focusin:FocusEvent:true:true:false:true:first:true/",
            "second:blur:FocusEvent:true:false:false:true:null:true/",
            "second:focusout:FocusEvent:true:true:false:true:null:true/",
            "negative:focus:FocusEvent:true:false:false:true:null:true/",
            "negative:focusin:FocusEvent:true:true:false:true:null:true/",
            "negative:blur:FocusEvent:true:false:false:true:null:true/",
            "negative:focusout:FocusEvent:true:true:false:true:null:true|",
            "first:first/first-again:first/second:second/second-blur:BODY/",
            "plain:BODY/negative:negative/negative-blur:BODY/disabled:BODY/",
            "hidden:BODY/display-none:BODY/visibility-hidden:BODY/inert:BODY/",
            "fieldset-disabled:BODY/detached:BODY"
        )
    );
}

#[test]
fn shadow_focus_retargets_active_element_event_target_and_related_target() {
    let mut runtime = EdgeRuntime::new().expect("shadow focus runtime");
    let result = text(
        &mut runtime,
        r#"
        (() => {
          const host = document.createElement("div");
          host.id = "host";
          const outer = document.createElement("input");
          outer.id = "outer";
          document.body.append(host, outer);
          const root = host.attachShadow({mode: "open"});
          const inner = document.createElement("input");
          inner.id = "inner";
          root.appendChild(inner);
          const events = [];
          const observe = (target, label, capture = false) =>
            target.addEventListener("focus", event => events.push([
              label, event.target.id, event.relatedTarget?.id || "null",
              event.eventPhase, event.isTrusted
            ].join(":")), capture);
          observe(document, "document", true);
          observe(inner, "inner");
          observe(host, "host");
          observe(outer, "outer");
          inner.focus();
          const innerState = [
            document.activeElement === host,
            root.activeElement === inner,
            document.activeElement.id,
            root.activeElement.id
          ].join(":");
          outer.focus();
          const outerState = [
            document.activeElement === outer,
            root.activeElement === null,
            document.activeElement.id
          ].join(":");
          return `${events.join("/")}|${innerState}|${outerState}`;
        })()
        "#,
    );
    assert_eq!(
        result,
        concat!(
            "document:host:null:1:true/inner:inner:null:2:true/host:host:null:2:true/",
            "document:outer:host:1:true/outer:outer:host:2:true|",
            "true:true:host:inner|true:true:outer"
        )
    );
}
