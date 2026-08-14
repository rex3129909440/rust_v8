use crate::{EdgeRuntime, Evaluation};

fn text(runtime: &mut EdgeRuntime, source: &str) -> String {
    match runtime.evaluate(source).expect("JavaScript evaluation") {
        Evaluation::Undefined => "undefined".to_owned(),
        Evaluation::Null => "null".to_owned(),
        Evaluation::Boolean(value) => value.to_string(),
        Evaluation::Number(value) | Evaluation::String(value) | Evaluation::Other(value) => value,
    }
}

const IFRAME_REALM_MATRIX: &str = r#"
(() => {
  const frame = document.createElement("iframe");
  frame.srcdoc = "<main>first realm</main>";
  document.body.appendChild(frame);
  const proxy = frame.contentWindow;
  const first = {
    Window: proxy.Window,
    Navigator: proxy.Navigator,
    Screen: proxy.Screen,
    Crypto: proxy.Crypto,
    SubtleCrypto: proxy.SubtleCrypto,
    Event: proxy.Event,
    Node: proxy.Node,
    Element: proxy.Element,
    HTMLElement: proxy.HTMLElement,
    Document: proxy.Document,
    HTMLDocument: proxy.HTMLDocument,
    Location: proxy.Location,
    HTMLBodyElement: proxy.HTMLBodyElement,
    CharacterData: proxy.CharacterData,
    Text: proxy.Text,
    Comment: proxy.Comment,
    DocumentFragment: proxy.DocumentFragment,
    Attr: proxy.Attr,
    CustomEvent: proxy.CustomEvent,
    MessageEvent: proxy.MessageEvent,
    ErrorEvent: proxy.ErrorEvent,
    PromiseRejectionEvent: proxy.PromiseRejectionEvent,
    AbortSignal: proxy.AbortSignal,
    AbortController: proxy.AbortController,
    MessagePort: proxy.MessagePort,
    MessageChannel: proxy.MessageChannel,
    BroadcastChannel: proxy.BroadcastChannel,
    Performance: proxy.Performance,
    History: proxy.History,
    CustomElementRegistry: proxy.CustomElementRegistry,
    CookieStore: proxy.CookieStore,
    Scheduler: proxy.Scheduler,
    TrustedTypePolicyFactory: proxy.TrustedTypePolicyFactory,
    Cache: proxy.Cache,
    CacheStorage: proxy.CacheStorage,
    IDBFactory: proxy.IDBFactory,
    Storage: proxy.Storage,
    URL: proxy.URL,
    URLSearchParams: proxy.URLSearchParams,
    URLPattern: proxy.URLPattern,
    Blob: proxy.Blob,
    File: proxy.File,
    FileReader: proxy.FileReader,
    Headers: proxy.Headers,
    Request: proxy.Request,
    Response: proxy.Response,
    FormData: proxy.FormData,
    OffscreenCanvas: proxy.OffscreenCanvas,
    OffscreenCanvasRenderingContext2D: proxy.OffscreenCanvasRenderingContext2D,
    WebGLRenderingContext: proxy.WebGLRenderingContext,
    WebGL2RenderingContext: proxy.WebGL2RenderingContext
  };
  const firstLocalStorage = proxy.localStorage;
  const firstSessionStorage = proxy.sessionStorage;
  localStorage.setItem("realm-shared-local", "local-value");
  sessionStorage.setItem("realm-shared-session", "session-value");
  const div = proxy.document.createElement("div");
  const text = proxy.document.createTextNode("text");
  const comment = proxy.document.createComment("comment");
  const fragment = proxy.document.createDocumentFragment();
  const attr = proxy.document.createAttribute("data-realm");
  const shape = [
    proxy.Window !== Window,
    proxy.Window.prototype !== Window.prototype,
    Object.getPrototypeOf(proxy) === proxy.Window.prototype,
    proxy instanceof proxy.Window,
    !(proxy instanceof Window),
    proxy.EventTarget !== EventTarget,
    Object.getPrototypeOf(
      Object.getPrototypeOf(proxy.Window.prototype)
    ) === proxy.EventTarget.prototype,
    proxy.Event !== Event,
    proxy.Node !== Node,
    proxy.Element !== Element,
    proxy.HTMLElement !== HTMLElement,
    proxy.Document !== Document,
    proxy.HTMLDocument !== HTMLDocument,
    proxy.Location !== Location,
    Object.getPrototypeOf(proxy.Node) === proxy.EventTarget,
    proxy.Node instanceof proxy.Function,
    Object.getPrototypeOf(proxy.Node.prototype) === proxy.EventTarget.prototype,
    Object.getPrototypeOf(proxy.Element.prototype) === proxy.Node.prototype,
    Object.getPrototypeOf(proxy.HTMLElement.prototype) === proxy.Element.prototype,
    proxy.document instanceof proxy.HTMLDocument,
    proxy.document instanceof proxy.Document,
    proxy.document instanceof proxy.Node,
    proxy.document instanceof proxy.EventTarget,
    !(proxy.document instanceof HTMLDocument),
    !(proxy.document instanceof Document),
    !(proxy.document instanceof Node),
    !(proxy.document instanceof EventTarget),
    proxy.document.documentElement instanceof proxy.HTMLHtmlElement,
    proxy.document.head instanceof proxy.HTMLHeadElement,
    proxy.document.body instanceof proxy.HTMLBodyElement,
    proxy.document.body instanceof proxy.HTMLElement,
    proxy.document.body instanceof proxy.Element,
    proxy.document.body instanceof proxy.Node,
    proxy.document.body instanceof proxy.EventTarget,
    !(proxy.document.body instanceof HTMLElement),
    !(proxy.document.body instanceof Element),
    !(proxy.document.body instanceof Node),
    !(proxy.document.body instanceof EventTarget),
    div instanceof proxy.HTMLDivElement,
    div instanceof proxy.HTMLElement,
    div instanceof proxy.Element,
    div instanceof proxy.Node,
    div instanceof proxy.EventTarget,
    !(div instanceof HTMLDivElement),
    !(div instanceof HTMLElement),
    !(div instanceof Element),
    !(div instanceof Node),
    !(div instanceof EventTarget),
    proxy.CharacterData !== CharacterData,
    proxy.Text !== Text,
    proxy.Comment !== Comment,
    proxy.DocumentFragment !== DocumentFragment,
    proxy.Attr !== Attr,
    text instanceof proxy.Text,
    text instanceof proxy.CharacterData,
    text instanceof proxy.Node,
    text instanceof proxy.EventTarget,
    !(text instanceof Text),
    !(text instanceof CharacterData),
    !(text instanceof Node),
    !(text instanceof EventTarget),
    comment instanceof proxy.Comment,
    comment instanceof proxy.CharacterData,
    comment instanceof proxy.Node,
    comment instanceof proxy.EventTarget,
    !(comment instanceof Comment),
    fragment instanceof proxy.DocumentFragment,
    fragment instanceof proxy.Node,
    fragment instanceof proxy.EventTarget,
    !(fragment instanceof DocumentFragment),
    !(fragment instanceof Node),
    attr instanceof proxy.Attr,
    attr instanceof proxy.Node,
    attr instanceof proxy.EventTarget,
    !(attr instanceof Attr),
    !(attr instanceof Node),
    proxy.CustomEvent !== CustomEvent,
    proxy.MessageEvent !== MessageEvent,
    proxy.ErrorEvent !== ErrorEvent,
    proxy.PromiseRejectionEvent !== PromiseRejectionEvent,
    proxy.AbortSignal !== AbortSignal,
    proxy.AbortController !== AbortController,
    proxy.MessagePort !== MessagePort,
    proxy.MessageChannel !== MessageChannel,
    proxy.BroadcastChannel !== BroadcastChannel,
    new proxy.CustomEvent("custom") instanceof proxy.Event,
    new proxy.MessageEvent("message") instanceof proxy.Event,
    new proxy.ErrorEvent("error") instanceof proxy.Event,
    proxy.AbortSignal.abort() instanceof proxy.AbortSignal,
    proxy.AbortSignal.abort() instanceof proxy.EventTarget,
    new proxy.AbortController().signal instanceof proxy.AbortSignal,
    new proxy.AbortController().signal instanceof proxy.EventTarget,
    new proxy.MessageChannel().port1 instanceof proxy.MessagePort,
    new proxy.MessageChannel().port1 instanceof proxy.EventTarget,
    new proxy.BroadcastChannel("realm-matrix") instanceof proxy.BroadcastChannel,
    new proxy.BroadcastChannel("realm-matrix") instanceof proxy.EventTarget,
    proxy.Navigator !== Navigator,
    proxy.navigator !== navigator,
    proxy.navigator === proxy.clientInformation,
    proxy.navigator instanceof proxy.Navigator,
    !(proxy.navigator instanceof Navigator),
    proxy.Screen !== Screen,
    proxy.screen !== screen,
    proxy.screen instanceof proxy.Screen,
    !(proxy.screen instanceof Screen),
    proxy.Crypto !== Crypto,
    proxy.crypto !== crypto,
    proxy.crypto instanceof proxy.Crypto,
    !(proxy.crypto instanceof Crypto),
    proxy.SubtleCrypto !== SubtleCrypto,
    proxy.crypto.subtle instanceof proxy.SubtleCrypto,
    !(proxy.crypto.subtle instanceof SubtleCrypto),
    proxy.console !== console,
    proxy.console.log instanceof proxy.Function,
    proxy.console.createTask instanceof proxy.Function,
    proxy.console.memory !== console.memory,
    proxy.Performance !== Performance,
    proxy.performance !== performance,
    proxy.performance instanceof proxy.Performance,
    proxy.performance instanceof proxy.EventTarget,
    !(proxy.performance instanceof Performance),
    !(proxy.performance instanceof EventTarget),
    proxy.History !== History,
    proxy.history !== history,
    proxy.history instanceof proxy.History,
    !(proxy.history instanceof History),
    proxy.CustomElementRegistry !== CustomElementRegistry,
    proxy.customElements !== customElements,
    proxy.customElements instanceof proxy.CustomElementRegistry,
    !(proxy.customElements instanceof CustomElementRegistry),
    proxy.CookieStore !== CookieStore,
    proxy.cookieStore !== cookieStore,
    proxy.cookieStore instanceof proxy.CookieStore,
    proxy.cookieStore instanceof proxy.EventTarget,
    !(proxy.cookieStore instanceof CookieStore),
    !(proxy.cookieStore instanceof EventTarget),
    proxy.Scheduler !== Scheduler,
    proxy.scheduler !== scheduler,
    proxy.scheduler instanceof proxy.Scheduler,
    !(proxy.scheduler instanceof Scheduler),
    proxy.TrustedTypePolicyFactory !== TrustedTypePolicyFactory,
    proxy.trustedTypes !== trustedTypes,
    proxy.trustedTypes instanceof proxy.TrustedTypePolicyFactory,
    !(proxy.trustedTypes instanceof TrustedTypePolicyFactory),
    proxy.Cache !== Cache,
    proxy.CacheStorage !== CacheStorage,
    proxy.caches !== caches,
    proxy.caches instanceof proxy.CacheStorage,
    !(proxy.caches instanceof CacheStorage),
    proxy.IDBFactory !== IDBFactory,
    proxy.indexedDB !== indexedDB,
    proxy.indexedDB instanceof proxy.IDBFactory,
    !(proxy.indexedDB instanceof IDBFactory),
    proxy.Storage !== Storage,
    proxy.localStorage !== localStorage,
    proxy.sessionStorage !== sessionStorage,
    proxy.localStorage !== proxy.sessionStorage,
    proxy.localStorage instanceof proxy.Storage,
    proxy.sessionStorage instanceof proxy.Storage,
    !(proxy.localStorage instanceof Storage),
    !(proxy.sessionStorage instanceof Storage),
    proxy.localStorage.getItem("realm-shared-local") === "local-value",
    proxy.sessionStorage.getItem("realm-shared-session") === "session-value",
    proxy.URL !== URL,
    proxy.URLSearchParams !== URLSearchParams,
    proxy.URLPattern !== URLPattern,
    new proxy.URL("https://example.com/?a=1") instanceof proxy.URL,
    new proxy.URL("https://example.com/?a=1").searchParams instanceof
      proxy.URLSearchParams,
    proxy.Blob !== Blob,
    proxy.File !== File,
    new proxy.Blob(["blob"]) instanceof proxy.Blob,
    new proxy.File([], "file.txt") instanceof proxy.File,
    new proxy.File([], "file.txt") instanceof proxy.Blob,
    proxy.FileReader !== FileReader,
    new proxy.FileReader() instanceof proxy.FileReader,
    new proxy.FileReader() instanceof proxy.EventTarget,
    proxy.Headers !== Headers,
    proxy.Request !== Request,
    proxy.Response !== Response,
    proxy.FormData !== FormData,
    new proxy.Headers() instanceof proxy.Headers,
    new proxy.Request("https://example.com/").headers instanceof proxy.Headers,
    new proxy.Response("response").headers instanceof proxy.Headers,
    new proxy.FormData() instanceof proxy.FormData,
    proxy.OffscreenCanvas !== OffscreenCanvas,
    proxy.OffscreenCanvasRenderingContext2D !== OffscreenCanvasRenderingContext2D,
    proxy.WebGLRenderingContext !== WebGLRenderingContext,
    proxy.WebGL2RenderingContext !== WebGL2RenderingContext,
    new proxy.OffscreenCanvas(2, 2) instanceof proxy.OffscreenCanvas,
    new proxy.OffscreenCanvas(2, 2) instanceof proxy.EventTarget,
    new proxy.OffscreenCanvas(2, 2).getContext("2d") instanceof
      proxy.OffscreenCanvasRenderingContext2D,
    new proxy.OffscreenCanvas(2, 2).getContext("webgl") instanceof
      proxy.WebGLRenderingContext,
    new proxy.OffscreenCanvas(2, 2).getContext("webgl2") instanceof
      proxy.WebGL2RenderingContext,
    proxy.Object !== Object,
    proxy.Array !== Array,
    Function.prototype.toString.call(proxy.Crypto).includes("[native code]"),
    Function.prototype.toString.call(
      Object.getOwnPropertyDescriptor(proxy.Crypto.prototype, "subtle").get
    ).includes("[native code]")
  ];
  frame.srcdoc = "<main>second realm</main>";
  shape.push(
    proxy === frame.contentWindow,
    first.Window !== proxy.Window,
    first.Navigator !== proxy.Navigator,
    first.Screen !== proxy.Screen,
    first.Crypto !== proxy.Crypto,
    first.SubtleCrypto !== proxy.SubtleCrypto,
    first.Event !== proxy.Event,
    first.Node !== proxy.Node,
    first.Element !== proxy.Element,
    first.HTMLElement !== proxy.HTMLElement,
    first.Document !== proxy.Document,
    first.HTMLDocument !== proxy.HTMLDocument,
    first.Location !== proxy.Location,
    first.HTMLBodyElement !== proxy.HTMLBodyElement,
    first.CharacterData !== proxy.CharacterData,
    first.Text !== proxy.Text,
    first.Comment !== proxy.Comment,
    first.DocumentFragment !== proxy.DocumentFragment,
    first.Attr !== proxy.Attr,
    first.CustomEvent !== proxy.CustomEvent,
    first.MessageEvent !== proxy.MessageEvent,
    first.ErrorEvent !== proxy.ErrorEvent,
    first.PromiseRejectionEvent !== proxy.PromiseRejectionEvent,
    first.AbortSignal !== proxy.AbortSignal,
    first.AbortController !== proxy.AbortController,
    first.MessagePort !== proxy.MessagePort,
    first.MessageChannel !== proxy.MessageChannel,
    first.BroadcastChannel !== proxy.BroadcastChannel,
    first.Performance !== proxy.Performance,
    first.History !== proxy.History,
    first.CustomElementRegistry !== proxy.CustomElementRegistry,
    first.CookieStore !== proxy.CookieStore,
    first.Scheduler !== proxy.Scheduler,
    first.TrustedTypePolicyFactory !== proxy.TrustedTypePolicyFactory,
    first.Cache !== proxy.Cache,
    first.CacheStorage !== proxy.CacheStorage,
    first.IDBFactory !== proxy.IDBFactory,
    first.Storage !== proxy.Storage,
    first.URL !== proxy.URL,
    first.URLSearchParams !== proxy.URLSearchParams,
    first.URLPattern !== proxy.URLPattern,
    first.Blob !== proxy.Blob,
    first.File !== proxy.File,
    first.FileReader !== proxy.FileReader,
    first.Headers !== proxy.Headers,
    first.Request !== proxy.Request,
    first.Response !== proxy.Response,
    first.FormData !== proxy.FormData,
    first.OffscreenCanvas !== proxy.OffscreenCanvas,
    first.OffscreenCanvasRenderingContext2D !==
      proxy.OffscreenCanvasRenderingContext2D,
    first.WebGLRenderingContext !== proxy.WebGLRenderingContext,
    first.WebGL2RenderingContext !== proxy.WebGL2RenderingContext,
    firstLocalStorage !== proxy.localStorage,
    firstSessionStorage !== proxy.sessionStorage,
    proxy.localStorage.getItem("realm-shared-local") === "local-value",
    proxy.sessionStorage.getItem("realm-shared-session") === "session-value",
    proxy instanceof proxy.Window,
    proxy.crypto instanceof proxy.Crypto,
    proxy.document instanceof proxy.HTMLDocument,
    proxy.document.body instanceof proxy.HTMLBodyElement
  );
  return shape.join("|");
})()
"#;

const WORKER_REALM_MATRIX_SETUP: &str = r#"
(() => {
  const source = `
    const localRealmFailures = [];
    for (const name of Object.getOwnPropertyNames(self)) {
      const constructor = self[name];
      if (typeof constructor !== "function") continue;
      let functionCursor = constructor;
      let usesLocalFunctionPrototype = false;
      while (functionCursor !== null) {
        if (functionCursor === Function.prototype) {
          usesLocalFunctionPrototype = true;
          break;
        }
        functionCursor = Object.getPrototypeOf(functionCursor);
      }
      if (!usesLocalFunctionPrototype) {
        localRealmFailures.push(name + ":Function.prototype");
      }
      const descriptor = Object.getOwnPropertyDescriptor(
        constructor,
        "prototype"
      );
      if (!descriptor || typeof descriptor.value !== "object") continue;
      let prototypeCursor = descriptor.value;
      let usesLocalObjectPrototype = false;
      while (prototypeCursor !== null) {
        if (prototypeCursor === Object.prototype) {
          usesLocalObjectPrototype = true;
          break;
        }
        prototypeCursor = Object.getPrototypeOf(prototypeCursor);
      }
      if (!usesLocalObjectPrototype) {
        localRealmFailures.push(name + ":Object.prototype");
      }
    }
    postMessage([
      self instanceof DedicatedWorkerGlobalScope,
      self instanceof WorkerGlobalScope,
      self instanceof EventTarget,
      self === globalThis,
      Object.getPrototypeOf(DedicatedWorkerGlobalScope.prototype) ===
        WorkerGlobalScope.prototype,
      Object.getPrototypeOf(WorkerGlobalScope.prototype) ===
        EventTarget.prototype,
      Event instanceof Function,
      Object.getPrototypeOf(Event) === Function.prototype,
      new Event("event") instanceof Event,
      Object.getPrototypeOf(CustomEvent) === Event,
      new CustomEvent("custom") instanceof Event,
      Object.getPrototypeOf(MessageEvent) === Event,
      new MessageEvent("message") instanceof Event,
      Object.getPrototypeOf(ErrorEvent) === Event,
      new ErrorEvent("error") instanceof Event,
      Object.getPrototypeOf(AbortSignal) === EventTarget,
      AbortSignal.abort() instanceof AbortSignal,
      AbortSignal.abort() instanceof EventTarget,
      new AbortController().signal instanceof AbortSignal,
      new AbortController().signal instanceof EventTarget,
      Object.getPrototypeOf(MessagePort) === EventTarget,
      new MessageChannel().port1 instanceof MessagePort,
      new MessageChannel().port1 instanceof EventTarget,
      Object.getPrototypeOf(BroadcastChannel) === EventTarget,
      console.log instanceof Function,
      console.createTask instanceof Function,
      typeof console.memory === "undefined",
      performance instanceof Performance,
      performance instanceof EventTarget,
      performance === self.performance,
      scheduler instanceof Scheduler,
      scheduler === self.scheduler,
      trustedTypes instanceof TrustedTypePolicyFactory,
      trustedTypes === self.trustedTypes,
      caches instanceof CacheStorage,
      caches === self.caches,
      indexedDB instanceof IDBFactory,
      indexedDB === self.indexedDB,
      new URL("https://example.com/?a=1") instanceof URL,
      new URL("https://example.com/?a=1").searchParams instanceof URLSearchParams,
      new URLPattern({ pathname: "/:id" }) instanceof URLPattern,
      new Blob(["blob"]) instanceof Blob,
      new File([], "file.txt") instanceof File,
      new File([], "file.txt") instanceof Blob,
      new FileReader() instanceof FileReader,
      new FileReader() instanceof EventTarget,
      new Headers() instanceof Headers,
      new Request("https://example.com/").headers instanceof Headers,
      new Response("response").headers instanceof Headers,
      new FormData() instanceof FormData,
      new OffscreenCanvas(2, 2) instanceof OffscreenCanvas,
      new OffscreenCanvas(2, 2) instanceof EventTarget,
      new OffscreenCanvas(2, 2).getContext("2d") instanceof
        OffscreenCanvasRenderingContext2D,
      new OffscreenCanvas(2, 2).getContext("webgl") instanceof
        WebGLRenderingContext,
      new OffscreenCanvas(2, 2).getContext("webgl2") instanceof
        WebGL2RenderingContext,
      navigator instanceof WorkerNavigator,
      crypto instanceof Crypto,
      crypto.subtle instanceof SubtleCrypto,
      crypto === self.crypto,
      typeof Window === "undefined",
      typeof document === "undefined",
      Function.prototype.toString.call(Crypto).includes("[native code]"),
      Function.prototype.toString.call(
        Object.getOwnPropertyDescriptor(
          WorkerGlobalScope.prototype,
          "crypto"
        ).get
      ).includes("[native code]"),
      localRealmFailures.length === 0
        ? true
        : "local Realm failures: " + localRealmFailures.join(",")
    ].join("|"));
  `;
  globalThis.workerRealmMatrix = "pending";
  const workerRealmResults = [];
  for (let index = 0; index < 2; index++) {
    const worker = new Worker(
      "data:text/javascript," + encodeURIComponent(source)
    );
    worker.onmessage = event => {
      workerRealmResults.push(event.data);
      worker.terminate();
      if (workerRealmResults.length !== 2) return;
      workerRealmMatrix =
        workerRealmResults.find(result =>
          result.split("|").some(relationship => relationship !== "true")
        ) ?? workerRealmResults[0];
    };
  }
})()
"#;

#[test]
fn iframe_navigation_uses_distinct_realm_constructors_prototypes_and_instances() {
    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    let direct_shape = text(&mut direct, IFRAME_REALM_MATRIX);
    assert!(
        !direct_shape.is_empty()
            && direct_shape
                .split('|')
                .all(|relationship| relationship == "true"),
        "iframe Realm relationship mismatch: {direct_shape}"
    );

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    assert_eq!(text(&mut traced, IFRAME_REALM_MATRIX), direct_shape);
    assert!(!traced.proxy_trace().is_empty());
}

#[test]
fn worker_uses_local_event_target_crypto_and_native_shaped_accessors() {
    let mut direct = EdgeRuntime::new().expect("direct Edge runtime");
    text(&mut direct, WORKER_REALM_MATRIX_SETUP);
    let direct_shape = text(&mut direct, "workerRealmMatrix");
    assert!(
        !direct_shape.is_empty()
            && direct_shape
                .split('|')
                .all(|relationship| relationship == "true"),
        "Worker Realm relationship mismatch: {direct_shape}"
    );

    let mut traced = EdgeRuntime::new().expect("traced Edge runtime");
    traced.enable_proxy_trace().expect("enable Proxy trace");
    text(&mut traced, WORKER_REALM_MATRIX_SETUP);
    assert_eq!(text(&mut traced, "workerRealmMatrix"), direct_shape);
    assert!(!traced.proxy_trace().is_empty());
}
