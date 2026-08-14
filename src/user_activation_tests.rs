use crate::{
    EdgeRuntime, EdgeRuntimeOptions, Evaluation, HostClickInput, HostKeyboardInput, HostPenInput,
    HostPenPhase, HostTouchInput, NetworkReplayEntry, PageInit,
};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

fn activation_runtime() -> EdgeRuntime {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.permissions.local_fonts = "granted".to_owned();
    EdgeRuntime::with_options(options).expect("activation runtime")
}

fn install_click_probe(runtime: &mut EdgeRuntime) {
    text(
        runtime,
        r#"
        document.body.innerHTML = '<button id="activate" style="width:120px;height:40px">go</button>';
        globalThis.activationLog = [];
        activate.addEventListener('pointerdown', () => activationLog.push([
          'pointerdown', navigator.userActivation.hasBeenActive, navigator.userActivation.isActive
        ]));
        activate.addEventListener('click', () => {
          activationLog.push(['click', navigator.userActivation.hasBeenActive, navigator.userActivation.isActive]);
          queueMicrotask(() => activationLog.push([
            'microtask', navigator.userActivation.hasBeenActive, navigator.userActivation.isActive
          ]));
          setTimeout(() => activationLog.push([
            'timeout0', navigator.userActivation.hasBeenActive, navigator.userActivation.isActive
          ]), 0);
        });
        "#,
    );
}

#[test]
fn trusted_host_input_activates_only_the_current_realm_and_synthetic_click_does_not() {
    let mut runtime = activation_runtime();
    install_click_probe(&mut runtime);
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|')",
        ),
        "false|false"
    );
    text(&mut runtime, "activate.click()");
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|')",
        ),
        "false|false"
    );
    text(&mut runtime, "activationLog.length=0");
    assert!(
        runtime
            .dispatch_host_click(&HostClickInput::primary(60.0, 20.0))
            .expect("trusted host click")
    );
    assert_eq!(
        text(&mut runtime, "JSON.stringify(activationLog)"),
        r#"[["pointerdown",true,true],["click",true,true],["microtask",true,true],["timeout0",true,true]]"#
    );
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|')",
        ),
        "true|true"
    );
}

#[test]
fn granted_local_font_access_uses_sticky_activation_without_consuming_transient_state() {
    let mut runtime = activation_runtime();
    text(
        &mut runtime,
        "document.body.innerHTML='<input id=input style=\"width:120px;height:40px\">';input.focus()",
    );
    assert!(
        runtime
            .dispatch_host_keyboard(&HostKeyboardInput::printable("a", "KeyA"))
            .expect("trusted keyboard")
    );
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|')",
        ),
        "true|true"
    );
    text(
        &mut runtime,
        "globalThis.fontQuery='pending';queryLocalFonts().then(v=>fontQuery='resolved:'+v.length,e=>fontQuery=e.name)",
    );
    assert!(text(&mut runtime, "fontQuery").starts_with("resolved:"));
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|')",
        ),
        "true|true"
    );
    text(
        &mut runtime,
        "globalThis.secondQuery='pending';queryLocalFonts().then(v=>secondQuery='resolved:'+v.length,e=>secondQuery=e.name)",
    );
    assert!(text(&mut runtime, "secondQuery").starts_with("resolved:"));
}

#[test]
fn prompt_local_font_access_consumes_transient_but_keeps_sticky_activation() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.permissions.local_fonts = "prompt".to_owned();
    let mut runtime = EdgeRuntime::with_options(options).expect("prompt activation runtime");
    text(
        &mut runtime,
        "document.body.innerHTML='<input id=input style=\"width:120px;height:40px\">';input.focus()",
    );
    assert!(
        runtime
            .dispatch_host_keyboard(&HostKeyboardInput::printable("a", "KeyA"))
            .expect("trusted keyboard")
    );
    text(
        &mut runtime,
        "globalThis.fontQuery='pending';queryLocalFonts().then(v=>fontQuery='resolved:'+v.length,e=>fontQuery=e.name)",
    );
    assert_eq!(text(&mut runtime, "fontQuery"), "resolved:0");
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|')",
        ),
        "true|false"
    );
    text(
        &mut runtime,
        "globalThis.secondQuery='pending';queryLocalFonts().then(v=>secondQuery='resolved',e=>secondQuery=e.name)",
    );
    assert_eq!(text(&mut runtime, "secondQuery"), "SecurityError");
}

#[test]
fn user_activation_lifecycle_is_unchanged_by_native_trace() {
    let mut runtime = activation_runtime();
    install_click_probe(&mut runtime);
    runtime.enable_proxy_trace().expect("enable native trace");
    assert!(
        runtime
            .dispatch_host_click(&HostClickInput::primary(60.0, 20.0))
            .expect("traced trusted click")
    );
    assert_eq!(
        text(&mut runtime, "JSON.stringify(activationLog)"),
        r#"[["pointerdown",true,true],["click",true,true],["microtask",true,true],["timeout0",true,true]]"#
    );
    assert_eq!(
        text(
            &mut runtime,
            "Function.prototype.toString.call(Object.getOwnPropertyDescriptor(UserActivation.prototype,'isActive').get)",
        ),
        "function get isActive() { [native code] }"
    );
}

#[test]
fn touch_and_pen_activate_on_successful_release_not_contact_start() {
    let mut runtime = activation_runtime();
    text(
        &mut runtime,
        r#"
        document.body.innerHTML='<button id=target style="position:fixed;left:20px;top:20px;width:100px;height:80px">go</button>';
        globalThis.phaseLog=[];
        for (const type of ['touchstart','touchend','pointerdown','pointerup']) {
          target.addEventListener(type, event => phaseLog.push([
            event.pointerType || 'touch', type, navigator.userActivation.isActive
          ]));
        }
        "#,
    );
    assert!(
        runtime
            .dispatch_host_touch(&HostTouchInput::start(1, 60.0, 50.0))
            .expect("touch start")
    );
    assert_eq!(
        text(&mut runtime, "navigator.userActivation.isActive"),
        "false"
    );
    assert!(
        runtime
            .dispatch_host_touch(&HostTouchInput::end(1, 60.0, 50.0))
            .expect("touch end")
    );
    assert_eq!(
        text(&mut runtime, "JSON.stringify(phaseLog.slice(0,4))"),
        r#"[["touch","pointerdown",false],["touch","touchstart",false],["touch","pointerup",true],["touch","touchend",true]]"#
    );

    let mut runtime = activation_runtime();
    text(
        &mut runtime,
        r#"
        document.body.innerHTML='<button id=target style="position:fixed;left:20px;top:20px;width:100px;height:80px">go</button>';
        globalThis.phaseLog=[];
        for (const type of ['pointerdown','pointerup']) {
          target.addEventListener(type, event => phaseLog.push([
            event.pointerType, type, navigator.userActivation.isActive
          ]));
        }
        "#,
    );
    let mut pen = HostPenInput::hover(60.0, 50.0);
    pen.phase = HostPenPhase::Down;
    pen.pressure = 0.5;
    assert!(runtime.dispatch_host_pen(&pen).expect("pen down"));
    assert_eq!(
        text(&mut runtime, "phaseLog[0].join('|')"),
        "pen|pointerdown|false"
    );
    assert_eq!(
        text(&mut runtime, "navigator.userActivation.isActive"),
        "false"
    );
    pen.phase = HostPenPhase::Up;
    pen.pressure = 0.0;
    assert!(runtime.dispatch_host_pen(&pen).expect("pen up"));
    assert_eq!(
        text(&mut runtime, "phaseLog[1].join('|')"),
        "pen|pointerup|true"
    );
    assert_eq!(
        text(&mut runtime, "navigator.userActivation.isActive"),
        "true"
    );
}

#[test]
fn activation_notification_and_consumption_follow_the_same_origin_frame_tree() {
    let mut options = EdgeRuntimeOptions::default();
    options.fingerprint.permissions.local_fonts = "prompt".to_owned();
    let mut runtime = EdgeRuntime::with_options(options).expect("frame activation runtime");
    text(
        &mut runtime,
        r#"
        document.body.innerHTML='<button id=activate style="position:fixed;left:20px;top:20px;width:100px;height:40px">go</button>';
        globalThis.activationFrame=document.createElement('iframe');
        document.body.appendChild(activationFrame);
        activationFrame.srcdoc='<p>child</p>';
        "#,
    );
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.isActive,activationFrame.contentWindow.navigator.userActivation.isActive].join('|')",
        ),
        "false|false"
    );
    assert!(
        runtime
            .dispatch_host_click(&HostClickInput::primary(60.0, 35.0))
            .expect("trusted root click")
    );
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.isActive,activationFrame.contentWindow.navigator.userActivation.isActive].join('|')",
        ),
        "true|true"
    );
    text(
        &mut runtime,
        "globalThis.childQuery='pending';activationFrame.contentWindow.queryLocalFonts().then(v=>childQuery='resolved:'+v.length,e=>childQuery=e.name)",
    );
    assert_eq!(text(&mut runtime, "childQuery"), "resolved:0");
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive,activationFrame.contentWindow.navigator.userActivation.hasBeenActive,activationFrame.contentWindow.navigator.userActivation.isActive].join('|')",
        ),
        "true|false|true|false"
    );
    text(&mut runtime, "activationFrame.srcdoc='<p>navigated</p>'");
    assert_eq!(
        text(
            &mut runtime,
            "[activationFrame.contentWindow.navigator.userActivation.hasBeenActive,activationFrame.contentWindow.navigator.userActivation.isActive].join('|')",
        ),
        "true|false"
    );
}

#[test]
fn transient_activation_expires_after_edge_five_second_window() {
    let mut options = EdgeRuntimeOptions::default();
    options.deterministic.clock_epoch_ms = Some(1_700_000_000_000);
    options.deterministic.clock_step_ms = 0;
    let mut runtime = EdgeRuntime::with_options(options).expect("deterministic activation runtime");
    install_click_probe(&mut runtime);
    assert!(
        runtime
            .dispatch_host_click(&HostClickInput::primary(60.0, 20.0))
            .expect("trusted host click")
    );
    text(
        &mut runtime,
        "setTimeout(()=>globalThis.expiredState=[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|'),5000)",
    );
    assert_eq!(text(&mut runtime, "expiredState"), "true|false");
}

#[test]
fn top_level_notification_excludes_cross_origin_descendants() {
    let child = br#"
      <script>
        addEventListener('message', event => {
          if (event.data === 'activation-state') {
            parent.postMessage('cross:' + [
              navigator.userActivation.hasBeenActive,
              navigator.userActivation.isActive
            ].join('|'), '*');
          }
        });
        parent.postMessage('cross-ready', '*');
      </script>
    "#;
    let mut runtime = EdgeRuntime::with_options(EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root.html".to_owned(),
            html: "<button id='activate' style='position:fixed;left:20px;top:20px;width:100px;height:40px'>go</button>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![NetworkReplayEntry {
            url: "https://other.example.test/activation.html".to_owned(),
            method: "GET".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
            body: child.to_vec(),
        }],
        ..Default::default()
    })
    .expect("cross-origin activation runtime");
    text(
        &mut runtime,
        r#"
        globalThis.activationMessages=[];
        addEventListener('message', event => activationMessages.push(event.data));
        globalThis.sameFrame=document.createElement('iframe');
        sameFrame.srcdoc='<p>same</p>';
        document.body.appendChild(sameFrame);
        globalThis.crossFrame=document.createElement('iframe');
        crossFrame.src='https://other.example.test/activation.html';
        document.body.appendChild(crossFrame);
        "#,
    );
    assert_eq!(
        text(&mut runtime, "activationMessages.join(',')"),
        "cross-ready"
    );
    assert!(
        runtime
            .dispatch_host_click(&HostClickInput::primary(60.0, 35.0))
            .expect("trusted top-level click")
    );
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive,sameFrame.contentWindow.navigator.userActivation.hasBeenActive,sameFrame.contentWindow.navigator.userActivation.isActive].join('|')",
        ),
        "true|true|true|true"
    );
    text(
        &mut runtime,
        "crossFrame.contentWindow.postMessage('activation-state','*')",
    );
    assert_eq!(
        text(&mut runtime, "activationMessages.at(-1)"),
        "cross:false|false"
    );
}

#[test]
fn activation_consumption_clears_cross_origin_descendants_too() {
    let mut options = EdgeRuntimeOptions {
        page: Some(PageInit {
            url: "https://app.example.test/root.html".to_owned(),
            html: "<main>root</main>".to_owned(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }),
        network_replay: vec![NetworkReplayEntry {
            url: "https://other.example.test/activation.html".to_owned(),
            method: "GET".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: vec![("Content-Type".to_owned(), "text/html".to_owned())],
            body: br#"<script>addEventListener('message',event=>{if(event.data==='activation-state')parent.postMessage('cross:'+[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|'),'*')})</script>"#.to_vec(),
        }],
        ..Default::default()
    };
    options.fingerprint.permissions.local_fonts = "prompt".to_owned();
    options
        .fingerprint
        .navigator
        .user_activation_has_been_active = true;
    options.fingerprint.navigator.user_activation_is_active = true;
    let mut runtime = EdgeRuntime::with_options(options).expect("consumption tree runtime");
    text(
        &mut runtime,
        r#"
        globalThis.activationMessages=[];
        addEventListener('message',event=>activationMessages.push(event.data));
        globalThis.crossFrame=document.createElement('iframe');
        crossFrame.src='https://other.example.test/activation.html';
        document.body.appendChild(crossFrame);
        "#,
    );
    text(
        &mut runtime,
        "globalThis.consumeAnswer='pending';queryLocalFonts().then(v=>consumeAnswer='resolved:'+v.length,e=>consumeAnswer=e.name)",
    );
    assert_eq!(text(&mut runtime, "consumeAnswer"), "resolved:0");
    assert_eq!(
        text(
            &mut runtime,
            "[navigator.userActivation.hasBeenActive,navigator.userActivation.isActive].join('|')",
        ),
        "true|false"
    );
    text(
        &mut runtime,
        "crossFrame.contentWindow.postMessage('activation-state','*')",
    );
    assert_eq!(
        text(&mut runtime, "activationMessages.at(-1)"),
        "cross:true|false"
    );
}
