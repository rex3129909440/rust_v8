import { createHash } from "node:crypto";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectDirectory = resolve(scriptDirectory, "..");
const evidenceDirectory = join(projectDirectory, "tests", "evidence");
const edgeCandidates = [
  "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
];

async function main() {
const edgeExecutable = await findEdge();
const debuggingPort = await reservePort();
const profileDirectory = await mkdtemp(join(tmpdir(), "edge-sandbox-evidence-"));
const edge = spawn(
  edgeExecutable,
  [
    "--headless=new",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-default-apps",
    "--disable-extensions",
    "--disable-features=Translate",
    "--disable-gpu",
    "--disable-sync",
    "--metrics-recording-only",
    "--no-first-run",
    `--remote-debugging-port=${debuggingPort}`,
    `--user-data-dir=${profileDirectory}`,
    "https://example.com/",
  ],
  {
    stdio: ["ignore", "ignore", "pipe"],
    windowsHide: true,
  },
);

let stderr = "";
edge.stderr.setEncoding("utf8");
edge.stderr.on("data", chunk => {
  stderr = `${stderr}${chunk}`.slice(-16_384);
});

try {
  const target = await waitForPageTarget(debuggingPort, 30_000);
  const cdp = await CdpSession.connect(target.webSocketDebuggerUrl);
  try {
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    const loaded = cdp.waitFor("Page.loadEventFired", 30_000);
    await cdp.send("Page.navigate", { url: "https://example.com/" });
    await loaded;

    const metadata = await evaluate(cdp, metadataProbe);
    const browserVersion = await cdp.send("Browser.getVersion");
    metadata.browserVersion = browserVersion.product;
    const windowRows = await evaluate(cdp, windowProbe);
    const interfaceRows = await evaluate(cdp, interfaceProbe);
    const workerRows = await evaluate(cdp, workerProbe, true);
    const behaviorRows = await evaluate(cdp, targetedBehaviorProbe, true);

    await mkdir(evidenceDirectory, { recursive: true });
    const windowText = toTsv(
      [
        "index",
        "name",
        "descriptor",
        "enumerable",
        "configurable",
        "writable",
        "type",
        "tag",
        "function_name",
        "function_length",
        "native",
        "error",
      ],
      windowRows,
    );
    const interfaceText = toTsv(
      [
        "interface",
        "constructor_parent",
        "prototype_parent",
        "member",
        "descriptor",
        "enumerable",
        "configurable",
        "writable",
        "type",
        "function_name",
        "function_length",
        "native",
      ],
      interfaceRows,
    );
    const workerText = toTsv(
      ["section", "depth", "tag", "name", "value"],
      workerRows,
    );
    const behaviorText = toTsv(["api", "case", "value"], behaviorRows);
    const metadataText = toTsv(
      ["key", "value"],
      [
        ["captured_at_utc", new Date().toISOString()],
        ["browser", metadata.browser],
        ["browser_version", metadata.browserVersion],
        ["user_agent", metadata.userAgent],
        ["user_agent_brands", metadata.userAgentBrands],
        ["url", metadata.url],
        ["secure_context", metadata.secureContext],
        ["cross_origin_isolated", metadata.crossOriginIsolated],
        ["window_own_property_count", windowRows.length],
        ["window_sha256", sha256(windowText)],
        ["interfaces_row_count", interfaceRows.length],
        ["interfaces_sha256", sha256(interfaceText)],
        ["worker_row_count", workerRows.length],
        ["worker_sha256", sha256(workerText)],
        ["targeted_behavior_row_count", behaviorRows.length],
        ["targeted_behavior_sha256", sha256(behaviorText)],
      ],
    );

    await writeFile(
      join(evidenceDirectory, "edge_https_metadata.tsv"),
      metadataText,
      "utf8",
    );
    await writeFile(
      join(evidenceDirectory, "edge_https_window.tsv"),
      windowText,
      "utf8",
    );
    await writeFile(
      join(evidenceDirectory, "edge_https_interfaces.tsv"),
      interfaceText,
      "utf8",
    );
    await writeFile(
      join(evidenceDirectory, "edge_https_worker.tsv"),
      workerText,
      "utf8",
    );
    await writeFile(
      join(evidenceDirectory, "edge_https_behavior.tsv"),
      behaviorText,
      "utf8",
    );

    process.stdout.write(
      [
        `Edge ${metadata.browserVersion}`,
        `${windowRows.length} Window own properties`,
        `${interfaceRows.length} interface descriptor rows`,
        `${workerRows.length} Worker evidence rows`,
        `${behaviorRows.length} targeted behavior rows`,
        evidenceDirectory,
      ].join("\n") + "\n",
    );
  } finally {
    cdp.close();
  }
} catch (error) {
  const detail = stderr.trim();
  throw new Error(
    detail.length === 0
      ? String(error)
      : `${String(error)}\nEdge stderr:\n${detail}`,
  );
} finally {
  await terminateProcessTree(edge);
  await rm(profileDirectory, {
    recursive: true,
    force: true,
    maxRetries: 20,
    retryDelay: 100,
  });
}
}

async function findEdge() {
  const { access } = await import("node:fs/promises");
  for (const candidate of edgeCandidates) {
    try {
      await access(candidate);
      return candidate;
    } catch {
      // Continue to the next fixed installation path.
    }
  }
  throw new Error("Microsoft Edge executable was not found");
}

async function reservePort() {
  return await new Promise((resolvePort, reject) => {
    const server = createServer();
    server.unref();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address ? address.port : 0;
      server.close(error => {
        if (error) {
          reject(error);
        } else {
          resolvePort(port);
        }
      });
    });
  });
}

async function terminateProcessTree(child) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  if (process.platform === "win32") {
    await new Promise(resolveTermination => {
      const killer = spawn(
        "taskkill.exe",
        ["/PID", String(child.pid), "/T", "/F"],
        { stdio: "ignore", windowsHide: true },
      );
      killer.on("error", resolveTermination);
      killer.on("exit", resolveTermination);
    });
  } else {
    child.kill("SIGKILL");
  }
  await new Promise(resolveExit => {
    if (child.exitCode !== null || child.signalCode !== null) {
      resolveExit();
      return;
    }
    const timeout = setTimeout(resolveExit, 5_000);
    child.once("exit", () => {
      clearTimeout(timeout);
      resolveExit();
    });
  });
}

async function waitForPageTarget(port, timeoutMilliseconds) {
  const deadline = Date.now() + timeoutMilliseconds;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`);
      const targets = await response.json();
      const page = targets.find(target => target.type === "page");
      if (page?.webSocketDebuggerUrl) {
        return page;
      }
    } catch (error) {
      lastError = error;
    }
    await new Promise(resolveDelay => setTimeout(resolveDelay, 50));
  }
  throw new Error(`Edge CDP endpoint did not start: ${String(lastError ?? "")}`);
}

class CdpSession {
  static async connect(url) {
    const socket = new WebSocket(url);
    await new Promise((resolveOpen, reject) => {
      socket.addEventListener("open", resolveOpen, { once: true });
      socket.addEventListener("error", reject, { once: true });
    });
    return new CdpSession(socket);
  }

  constructor(socket) {
    this.socket = socket;
    this.nextId = 1;
    this.pending = new Map();
    this.waiters = new Map();
    socket.addEventListener("message", event => this.onMessage(event.data));
  }

  send(method, params = {}) {
    const id = this.nextId++;
    const packet = { id, method, params };
    return new Promise((resolveCommand, reject) => {
      this.pending.set(id, { resolve: resolveCommand, reject });
      this.socket.send(JSON.stringify(packet));
    });
  }

  waitFor(method, timeoutMilliseconds) {
    return new Promise((resolveEvent, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error(`timed out waiting for ${method}`));
      }, timeoutMilliseconds);
      this.waiters.set(method, params => {
        clearTimeout(timeout);
        resolveEvent(params);
      });
    });
  }

  close() {
    this.socket.close();
  }

  onMessage(text) {
    const packet = JSON.parse(text);
    if (packet.id !== undefined) {
      const pending = this.pending.get(packet.id);
      if (!pending) {
        return;
      }
      this.pending.delete(packet.id);
      if (packet.error) {
        pending.reject(new Error(packet.error.message));
      } else {
        pending.resolve(packet.result);
      }
      return;
    }
    if (packet.method) {
      const waiter = this.waiters.get(packet.method);
      if (waiter) {
        this.waiters.delete(packet.method);
        waiter(packet.params);
      }
    }
  }
}

async function evaluate(cdp, probe, awaitPromise = false) {
  const response = await cdp.send("Runtime.evaluate", {
    expression: `(${probe.toString()})()`,
    awaitPromise,
    returnByValue: true,
  });
  if (response.exceptionDetails) {
    throw new Error(response.exceptionDetails.text);
  }
  return response.result.value;
}

function metadataProbe() {
  const brands = navigator.userAgentData?.brands
    ?.map(item => `${item.brand}/${item.version}`)
    .join(",") ?? "";
  const edgeBrand = navigator.userAgentData?.brands?.find(item =>
    item.brand.includes("Microsoft Edge"),
  );
  return {
    browser: edgeBrand ? "Microsoft Edge" : "Chromium",
    browserVersion: edgeBrand?.version ?? "",
    userAgent: navigator.userAgent,
    userAgentBrands: brands,
    url: location.href,
    secureContext: String(isSecureContext),
    crossOriginIsolated: String(crossOriginIsolated),
  };
}

function windowProbe() {
  return Object.getOwnPropertyNames(globalThis).map((name, index) => {
    const descriptor = Object.getOwnPropertyDescriptor(globalThis, name);
    const descriptorKind =
      descriptor && ("get" in descriptor || "set" in descriptor)
        ? "accessor"
        : "data";
    let value;
    let error = "";
    try {
      value = globalThis[name];
    } catch (caught) {
      error = `${caught?.name ?? "Error"}:${caught?.message ?? ""}`;
    }
    const isFunction = typeof value === "function";
    return [
      index,
      name,
      descriptorKind,
      String(Boolean(descriptor?.enumerable)),
      String(Boolean(descriptor?.configurable)),
      String(Boolean(descriptor?.writable)),
      typeof value,
      value == null ? String(value) : Object.prototype.toString.call(value),
      isFunction ? value.name : "",
      isFunction ? value.length : "",
      isFunction
        ? String(Function.prototype.toString.call(value).includes("[native code]"))
        : "",
      error,
    ];
  });
}

function interfaceProbe() {
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
        member,
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
                .includes("[native code]"),
            )
          : "",
      ]);
    }
  }
  return rows;
}

async function workerProbe() {
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
    postMessage(rows);
  `;
  const url = URL.createObjectURL(
    new Blob([source], { type: "text/javascript" }),
  );
  try {
    return await new Promise((resolveWorker, reject) => {
      const worker = new Worker(url);
      worker.onmessage = event => {
        worker.terminate();
        resolveWorker(event.data);
      };
      worker.onerror = event => {
        worker.terminate();
        reject(new Error(event.message));
      };
    });
  } finally {
    URL.revokeObjectURL(url);
  }
}

async function targetedBehaviorProbe() {
  const rows = [];
  const capture = (api, testCase, operation) => {
    try {
      const value = operation();
      rows.push([api, testCase, String(value)]);
    } catch (error) {
      rows.push([
        api,
        testCase,
        `throws ${error?.name ?? "Error"}:${error?.message ?? ""}`,
      ]);
    }
  };

  const canvas = document.createElement("canvas");
  const canvasContext = canvas.getContext("2d");
  const metricValue = metrics =>
    [
      metrics.width,
      metrics.actualBoundingBoxLeft,
      metrics.actualBoundingBoxRight,
      metrics.fontBoundingBoxAscent,
      metrics.fontBoundingBoxDescent,
      metrics.actualBoundingBoxAscent,
      metrics.actualBoundingBoxDescent,
      metrics.hangingBaseline,
      metrics.alphabeticBaseline,
      metrics.ideographicBaseline,
    ].join(",");
  capture("CanvasRenderingContext2D", "measureText-default-abcd", () =>
    metricValue(canvasContext.measureText("abcd")),
  );
  capture("CanvasRenderingContext2D", "measureText-empty", () =>
    metricValue(canvasContext.measureText("")),
  );
  capture("CanvasRenderingContext2D", "measureText-20px-monospace-Wi", () => {
    canvasContext.font = "20px monospace";
    return metricValue(canvasContext.measureText("Wi"));
  });
  capture("CanvasRenderingContext2D", "measureText-20px-sans-serif-Wi", () => {
    canvasContext.font = "20px sans-serif";
    return metricValue(canvasContext.measureText("Wi"));
  });
  capture("CanvasRenderingContext2D", "measureText-spacing", () => {
    canvasContext.font = "10px sans-serif";
    canvasContext.letterSpacing = "1px";
    canvasContext.wordSpacing = "2px";
    return metricValue(canvasContext.measureText("a b"));
  });
  capture("CanvasRenderingContext2D", "measureText-center", () => {
    canvasContext.letterSpacing = "0px";
    canvasContext.wordSpacing = "0px";
    canvasContext.textAlign = "center";
    return metricValue(canvasContext.measureText("ab"));
  });
  capture("CanvasRenderingContext2D", "measureText-right", () => {
    canvasContext.textAlign = "right";
    return metricValue(canvasContext.measureText("ab"));
  });

  const webglParameters = [
    35724, 7936, 7937, 7938, 34921, 36347, 35660, 36348, 36349, 33901,
    33902, 34930, 3379, 35661, 34024, 3386, 34076, 2963, 2968, 36004,
    36005, 3408, 35658, 35371, 37154, 35377, 35659, 35968, 35978, 35979,
    35657, 35373, 37157, 35379, 35077, 34852, 36063, 36183, 32883, 35071,
    34045, 35375, 35376, 35374, 33000, 33001, 36203,
  ];
  const renderParameter = value => {
    if (ArrayBuffer.isView(value)) {
      return `${Object.prototype.toString.call(value)}:${Array.from(value).join(",")}`;
    }
    return `${Object.prototype.toString.call(value)}:${String(value)}`;
  };
  for (const contextName of ["webgl", "webgl2"]) {
    const context = document.createElement("canvas").getContext(contextName);
    const debugRenderer = context.getExtension("WEBGL_debug_renderer_info");
    capture(
      contextName === "webgl"
        ? "WebGLRenderingContext"
        : "WebGL2RenderingContext",
      "getParameter-unmasked-vendor",
      () =>
        renderParameter(
          context.getParameter(debugRenderer.UNMASKED_VENDOR_WEBGL),
        ),
    );
    capture(
      contextName === "webgl"
        ? "WebGLRenderingContext"
        : "WebGL2RenderingContext",
      "getParameter-unmasked-renderer",
      () =>
        renderParameter(
          context.getParameter(debugRenderer.UNMASKED_RENDERER_WEBGL),
        ),
    );
    for (const parameter of webglParameters) {
      capture(
        contextName === "webgl"
          ? "WebGLRenderingContext"
          : "WebGL2RenderingContext",
        `getParameter-${parameter}`,
        () => renderParameter(context.getParameter(parameter)),
      );
      context.getError();
    }
    capture(
      contextName === "webgl"
        ? "WebGLRenderingContext"
        : "WebGL2RenderingContext",
      "getParameter-invalid",
      () => {
        const value = context.getParameter(0xFFFFFFFF);
        return `${renderParameter(value)}|${context.getError()}|${context.getError()}`;
      },
    );
  }

  capture("Document.currentScript", "outside-script", () =>
    document.currentScript,
  );
  capture("HTMLScriptElement", "dynamic-inline-classic", () => {
    const values = ["before"];
    window.__edgeCurrentScriptValues = values;
    const element = document.createElement("script");
    element.id = "edge-current-script-inline";
    element.text =
      "__edgeCurrentScriptValues.push(" +
      "[document.currentScript===null," +
      "document.currentScript?.id," +
      "document.currentScript?.tagName," +
      "document.currentScript?.isConnected].join(','))";
    document.head.appendChild(element);
    values.push("after");
    element.remove();
    delete window.__edgeCurrentScriptValues;
    return values.join("|");
  });
  {
    const values = ["before"];
    window.__edgeExternalScriptValues = values;
    const source = [
      "__edgeExternalScriptValues.push(",
      "[document.currentScript===null,",
      "document.currentScript?.id,",
      "document.currentScript?.src.startsWith('blob:'),",
      "document.currentScript?.isConnected].join(','))",
    ].join("");
    const url = URL.createObjectURL(
      new Blob([source], { type: "text/javascript" }),
    );
    const element = document.createElement("script");
    element.id = "edge-current-script-external";
    element.src = url;
    const completion = new Promise(resolveScript => {
      element.onload = () => {
        values.push(`load:${document.currentScript === null}`);
        resolveScript();
      };
      element.onerror = () => {
        values.push("error");
        resolveScript();
      };
    });
    document.head.appendChild(element);
    values.push("after-append");
    await completion;
    capture("Document.currentScript", "dynamic-external-classic", () =>
      values.join("|"),
    );
    element.remove();
    URL.revokeObjectURL(url);
    delete window.__edgeExternalScriptValues;
  }
  {
    const values = ["before"];
    window.__edgeModuleScriptValues = values;
    const source =
      "__edgeModuleScriptValues.push(" +
      "[document.currentScript===null,typeof import.meta.url].join(','))";
    const url = URL.createObjectURL(
      new Blob([source], { type: "text/javascript" }),
    );
    const element = document.createElement("script");
    element.type = "module";
    element.src = url;
    const completion = new Promise(resolveScript => {
      element.onload = () => {
        values.push(`load:${document.currentScript === null}`);
        resolveScript();
      };
      element.onerror = () => {
        values.push("error");
        resolveScript();
      };
    });
    document.head.appendChild(element);
    values.push("after-append");
    await completion;
    capture("Document.currentScript", "dynamic-external-module", () =>
      values.join("|"),
    );
    element.remove();
    URL.revokeObjectURL(url);
    delete window.__edgeModuleScriptValues;
  }

  const styleSheets = document.styleSheets;
  capture("Document.styleSheets", "stable-identity", () =>
    styleSheets === document.styleSheets,
  );
  capture("Document.styleSheets", "initial-shape", () =>
    [
      Object.prototype.toString.call(styleSheets),
      styleSheets instanceof StyleSheetList,
      styleSheets.length,
      styleSheets.item(0),
      Object.keys(styleSheets).join(","),
    ].join("|"),
  );
  const style = document.createElement("style");
  style.id = "edge-style-sheet-probe";
  style.media = "screen";
  style.textContent =
    "#edge-computed-style-probe { color: rgb(1, 2, 3); margin-left: 7px; }";
  capture("HTMLStyleElement.sheet", "detached", () => style.sheet);
  document.head.appendChild(style);
  const styleSheet = style.sheet;
  capture("HTMLStyleElement.sheet", "connected-shape", () =>
    [
      Object.prototype.toString.call(styleSheet),
      styleSheet instanceof CSSStyleSheet,
      styleSheet.ownerNode === style,
      styleSheet.href,
      styleSheet.parentStyleSheet,
      styleSheet.title,
      styleSheet.media.mediaText,
      styleSheet.disabled,
      styleSheet.cssRules.length,
      styleSheet.cssRules[0]?.cssText,
    ].join("|"),
  );
  capture("CSSStyleSheet.replaceSync", "owner-backed-sheet", () =>
    styleSheet.replaceSync("#edge-computed-style-probe { color: red; }"),
  );
  {
    const replacement = styleSheet.replace(
      "#edge-computed-style-probe { color: red; }",
    );
    capture("CSSStyleSheet.replace", "owner-backed-return", () =>
      Object.prototype.toString.call(replacement),
    );
    try {
      await replacement;
      rows.push(["CSSStyleSheet.replace", "owner-backed-resolution", "fulfilled"]);
    } catch (error) {
      rows.push([
        "CSSStyleSheet.replace",
        "owner-backed-resolution",
        `throws ${error?.name ?? "Error"}:${error?.message ?? ""}`,
      ]);
    }
  }
  capture("HTMLStyleElement.sheet", "disabled-identity", () => {
    style.disabled = true;
    const disabled = [
      style.sheet === styleSheet,
      style.sheet.disabled,
      style.hasAttribute("disabled"),
    ];
    style.disabled = false;
    disabled.push(style.sheet === styleSheet, style.sheet.disabled);
    return disabled.join("|");
  });
  capture("HTMLStyleElement.sheet", "media-update-identity", () => {
    style.media = "print";
    const value = [
      style.sheet === styleSheet,
      style.sheet.media.mediaText,
      style.getAttribute("media"),
    ].join("|");
    style.media = "screen";
    return value;
  });
  capture("Document.styleSheets", "live-after-style-append", () =>
    [
      styleSheets === document.styleSheets,
      styleSheets.length,
      styleSheets[styleSheets.length - 1] === styleSheet,
      styleSheets.item(styleSheets.length - 1) === styleSheet,
      Object.keys(styleSheets).join(","),
    ].join("|"),
  );
  capture("HTMLStyleElement.sheet", "text-update", () => {
    style.textContent =
      "#edge-computed-style-probe { color: rgb(4, 5, 6); padding-top: 9px; }";
    return [
      style.sheet === styleSheet,
      styleSheet.cssRules.length,
      styleSheet.cssRules[0]?.cssText,
    ].join("|");
  });
  const computedElement = document.createElement("div");
  computedElement.id = "edge-computed-style-probe";
  computedElement.style.cssText = "display: inline-block; width: 11px";
  document.body.appendChild(computedElement);
  capture("getComputedStyle", "cascade-and-shape", () => {
    const first = getComputedStyle(computedElement);
    const second = getComputedStyle(computedElement);
    return [
      Object.prototype.toString.call(first),
      first instanceof CSSStyleDeclaration,
      first === second,
      first.color,
      first.paddingTop,
      first.display,
      first.width,
      first.getPropertyValue("color"),
      Object.getOwnPropertyDescriptor(first, "color"),
    ].join("|");
  });
  capture("getComputedStyle", "readonly", () => {
    const computed = getComputedStyle(computedElement);
    const before = computed.color;
    let outcome = "assigned";
    try {
      computed.color = "red";
    } catch (error) {
      outcome = `throws ${error.name}`;
    }
    return `${before}|${outcome}|${computed.color}`;
  });
  {
    const layoutElement = document.createElement("div");
    layoutElement.id = "edge-layout-probe";
    layoutElement.style.cssText = [
      "position: fixed",
      "left: 10px",
      "top: 20px",
      "width: 100px",
      "height: 50px",
      "padding: 5px",
      "border: 2px solid black",
      "box-sizing: content-box",
      "overflow: auto",
    ].join(";");
    document.body.appendChild(layoutElement);
    capture("Element.getBoundingClientRect", "fixed-content-box", () => {
      const rect = layoutElement.getBoundingClientRect();
      return [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        rect.top,
        rect.right,
        rect.bottom,
        rect.left,
        layoutElement.clientWidth,
        layoutElement.clientHeight,
        layoutElement.offsetWidth,
        layoutElement.offsetHeight,
        layoutElement.offsetLeft,
        layoutElement.offsetTop,
        layoutElement.scrollWidth,
        layoutElement.scrollHeight,
        layoutElement.offsetParent,
      ].join("|");
    });
    capture("Element.getClientRects", "fixed-content-box", () => {
      const rects = layoutElement.getClientRects();
      const rect = rects[0];
      return [
        Object.prototype.toString.call(rects),
        rects.length,
        rects.item(0) === rect,
        rect?.x,
        rect?.y,
        rect?.width,
        rect?.height,
      ].join("|");
    });

    const detachedLayout = document.createElement("div");
    detachedLayout.style.cssText =
      "position:fixed;left:10px;top:20px;width:100px;height:50px;padding:5px;border:2px solid";
    capture("Element.getBoundingClientRect", "detached", () => {
      const rect = detachedLayout.getBoundingClientRect();
      return [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        detachedLayout.clientWidth,
        detachedLayout.offsetWidth,
        detachedLayout.getClientRects().length,
      ].join("|");
    });
    capture("Element.getBoundingClientRect", "display-none", () => {
      layoutElement.style.display = "none";
      const rect = layoutElement.getBoundingClientRect();
      const value = [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        layoutElement.clientWidth,
        layoutElement.offsetWidth,
        layoutElement.getClientRects().length,
      ].join("|");
      layoutElement.style.display = "block";
      return value;
    });

    {
      const deliveries = [];
      const observer = new ResizeObserver((entries, deliveredObserver) => {
        const entry = entries[0];
        deliveries.push([
          entries.length,
          deliveredObserver === observer,
          entry?.target === layoutElement,
          entry?.contentRect.width,
          entry?.contentRect.height,
          entry?.contentBoxSize[0]?.inlineSize,
          entry?.contentBoxSize[0]?.blockSize,
          entry?.borderBoxSize[0]?.inlineSize,
          entry?.borderBoxSize[0]?.blockSize,
          entry?.devicePixelContentBoxSize[0]?.inlineSize,
          entry?.devicePixelContentBoxSize[0]?.blockSize,
        ].join(","));
      });
      observer.observe(layoutElement);
      await new Promise(resolveFrame =>
        requestAnimationFrame(() => requestAnimationFrame(resolveFrame)),
      );
      capture("ResizeObserver", "initial-delivery", () =>
        deliveries.join("|"),
      );
      deliveries.length = 0;
      layoutElement.style.width = "120px";
      await new Promise(resolveFrame =>
        requestAnimationFrame(() => requestAnimationFrame(resolveFrame)),
      );
      capture("ResizeObserver", "style-change-delivery", () =>
        deliveries.join("|"),
      );
      observer.unobserve(layoutElement);
      observer.disconnect();
    }

    {
      const deliveries = [];
      const observer = new IntersectionObserver(
        (entries, deliveredObserver) => {
          const entry = entries[0];
          const bounds = entry?.boundingClientRect;
          const intersectionBounds = entry?.intersectionRect;
          const rootBounds = entry?.rootBounds;
          deliveries.push([
            entries.length,
            deliveredObserver === observer,
            entry?.target === layoutElement,
            entry?.isIntersecting,
            entry?.intersectionRatio,
            bounds?.x,
            bounds?.y,
            bounds?.width,
            bounds?.height,
            rootBounds?.x,
            rootBounds?.y,
            rootBounds?.width,
            rootBounds?.height,
            intersectionBounds?.x,
            intersectionBounds?.y,
            intersectionBounds?.width,
            intersectionBounds?.height,
            entry?.isVisible,
            Number.isFinite(entry?.time) && entry.time >= 0,
          ].join(","));
        },
      );
      observer.observe(layoutElement);
      await new Promise(resolveFrame =>
        requestAnimationFrame(() => requestAnimationFrame(resolveFrame)),
      );
      capture("IntersectionObserver", "initial-delivery", () =>
        deliveries.join("|"),
      );
      deliveries.length = 0;
      layoutElement.style.top = "10000px";
      await new Promise(resolveFrame =>
        requestAnimationFrame(() => requestAnimationFrame(resolveFrame)),
      );
      capture("IntersectionObserver", "outside-viewport-delivery", () =>
        deliveries.join("|"),
      );
      observer.unobserve(layoutElement);
      observer.disconnect();
    }

    {
      const rangeRoot = document.createElement("div");
      rangeRoot.style.cssText =
        "position:fixed;left:200px;top:100px;width:200px;height:100px";
      const first = document.createElement("span");
      first.style.cssText =
        "position:absolute;left:0;top:0;width:30px;height:10px;display:block";
      const second = document.createElement("span");
      second.style.cssText =
        "position:absolute;left:50px;top:20px;width:20px;height:15px;display:block";
      rangeRoot.append(first, second);
      document.body.appendChild(rangeRoot);
      const range = document.createRange();
      range.setStartBefore(first);
      range.setEndAfter(second);
      capture("Range.getBoundingClientRect", "positioned-elements", () => {
        const rect = range.getBoundingClientRect();
        return [
          Object.prototype.toString.call(rect),
          rect instanceof DOMRect,
          rect.x,
          rect.y,
          rect.width,
          rect.height,
          rect.top,
          rect.right,
          rect.bottom,
          rect.left,
        ].join("|");
      });
      capture("Range.getClientRects", "positioned-elements", () => {
        const rects = range.getClientRects();
        return [
          Object.prototype.toString.call(rects),
          rects instanceof DOMRectList,
          rects.length,
          rects.item(0) === rects[0],
          Array.from(rects, rect =>
            [rect.x, rect.y, rect.width, rect.height].join(","),
          ).join("/"),
        ].join("|");
      });
      range.collapse(true);
      capture("Range.getBoundingClientRect", "collapsed-element-boundary", () => {
        const rect = range.getBoundingClientRect();
        return [
          rect.x,
          rect.y,
          rect.width,
          rect.height,
          range.getClientRects().length,
        ].join("|");
      });
      rangeRoot.remove();
    }

    {
      const scrolling = document.createElement("div");
      scrolling.style.cssText = [
        "position:fixed",
        "left:300px",
        "top:250px",
        "width:100px",
        "height:50px",
        "padding:5px",
        "border:2px solid",
        "overflow:auto",
      ].join(";");
      const overflowChild = document.createElement("div");
      overflowChild.style.cssText =
        "position:absolute;left:150px;top:80px;width:20px;height:10px";
      scrolling.appendChild(overflowChild);
      document.body.appendChild(scrolling);
      capture("Element.scrollWidth", "positioned-overflow", () => [
        scrolling.clientWidth,
        scrolling.clientHeight,
        scrolling.scrollWidth,
        scrolling.scrollHeight,
        scrolling.scrollLeft,
        scrolling.scrollTop,
      ].join("|"));
      capture("Element.scrollLeft", "clamped-overflow", () => {
        scrolling.scrollLeft = 1000;
        scrolling.scrollTop = 1000;
        const childRect = overflowChild.getBoundingClientRect();
        return [
          scrolling.scrollLeft,
          scrolling.scrollTop,
          childRect.x,
          childRect.y,
        ].join("|");
      });
      scrolling.remove();
    }
    {
      const nonScrolling = document.createElement("div");
      nonScrolling.innerHTML = "<b>two</b>";
      document.body.appendChild(nonScrolling);
      capture("Element.scrollLeft", "non-overflow-container", () => {
        nonScrolling.scroll(4, 5);
        nonScrolling.scrollBy({ left: 3, top: 2 });
        return [
          nonScrolling.scrollLeft,
          nonScrolling.scrollTop,
          nonScrolling.scrollWidth,
          nonScrolling.scrollHeight,
        ].join("|");
      });
      nonScrolling.remove();
    }
    {
      const image = document.createElement("img");
      image.style.cssText =
        "position:fixed;left:400px;top:300px;width:10px;height:20px";
      document.body.appendChild(image);
      capture("HTMLImageElement", "rendered-position-and-size", () => {
        const rect = image.getBoundingClientRect();
        return [
          image.x,
          image.y,
          image.width,
          image.height,
          image.naturalWidth,
          image.naturalHeight,
          rect.x,
          rect.y,
          rect.width,
          rect.height,
        ].join("|");
      });
      image.remove();
    }
    {
      const hitRoot = document.createElement("div");
      hitRoot.id = "hit-root";
      hitRoot.style.cssText = [
        "position:fixed",
        "left:600px",
        "top:100px",
        "width:200px",
        "height:150px",
        "z-index:2147483000",
      ].join(";");
      const hitLow = document.createElement("div");
      hitLow.id = "hit-low";
      hitLow.style.cssText =
        "position:absolute;left:10px;top:10px;width:100px;height:100px;z-index:1";
      const hitHigh = document.createElement("div");
      hitHigh.id = "hit-high";
      hitHigh.style.cssText =
        "position:absolute;left:30px;top:30px;width:100px;height:100px;z-index:2";
      const hitIgnored = document.createElement("div");
      hitIgnored.id = "hit-ignored";
      hitIgnored.style.cssText =
        "position:absolute;left:40px;top:40px;width:100px;height:100px;z-index:3;pointer-events:none";
      hitRoot.append(hitLow, hitHigh, hitIgnored);
      document.body.appendChild(hitRoot);
      capture("Document.elementsFromPoint", "stacking-and-pointer-events", () => {
        const elements = document.elementsFromPoint(650, 150);
        return [
          elements.slice(0, 3).map(element => element.id || element.tagName).join(","),
          document.elementFromPoint(650, 150) === hitHigh,
          elements.includes(hitIgnored),
          Object.prototype.toString.call(elements),
          Array.isArray(elements),
        ].join("|");
      });
      capture("Document.elementFromPoint", "bounds-and-visibility", () => {
        hitHigh.style.visibility = "hidden";
        hitLow.style.opacity = "0";
        const visible = document.elementFromPoint(650, 150);
        const inside = document.elementFromPoint(605, 105);
        const rightEdge = document.elementFromPoint(800, 150);
        return [
          visible === hitLow,
          inside === hitRoot,
          rightEdge === hitRoot,
          document.elementFromPoint(-1, 0),
          document.elementFromPoint(0, -1),
          document.elementFromPoint(innerWidth, 0),
          document.elementFromPoint(0, innerHeight),
        ].join("|");
      });
      hitRoot.remove();
    }
    {
      const host = document.createElement("div");
      host.id = "hit-host";
      host.style.cssText = [
        "position:fixed",
        "left:600px",
        "top:300px",
        "width:100px",
        "height:80px",
        "z-index:2147483000",
      ].join(";");
      const shadow = host.attachShadow({ mode: "open" });
      const shadowLow = document.createElement("div");
      shadowLow.id = "shadow-low";
      shadowLow.style.cssText =
        "position:absolute;left:0;top:0;width:80px;height:60px;z-index:1";
      const shadowHigh = document.createElement("div");
      shadowHigh.id = "shadow-high";
      shadowHigh.style.cssText =
        "position:absolute;left:10px;top:10px;width:50px;height:40px;z-index:2";
      shadow.append(shadowLow, shadowHigh);
      document.body.appendChild(host);
      capture("ShadowRoot.elementFromPoint", "retargeted-hit", () => {
        const documentHits = document.elementsFromPoint(620, 320);
        const shadowHits = shadow.elementsFromPoint(620, 320);
        return [
          document.elementFromPoint(620, 320) === host,
          documentHits[0] === host,
          shadow.elementFromPoint(620, 320) === shadowHigh,
          shadowHits.slice(0, 3).map(element => element.id || element.tagName).join(","),
          Object.prototype.toString.call(shadowHits),
          Array.isArray(shadowHits),
        ].join("|");
      });
      host.remove();
    }
    {
      const scrollContainer = document.createElement("div");
      scrollContainer.style.cssText = [
        "position:fixed",
        "left:900px",
        "top:100px",
        "width:100px",
        "height:50px",
        "overflow:auto",
      ].join(";");
      const scrollTarget = document.createElement("div");
      scrollTarget.style.cssText =
        "position:absolute;left:150px;top:80px;width:20px;height:10px";
      const scrollExtent = document.createElement("div");
      scrollExtent.style.cssText =
        "position:absolute;left:280px;top:180px;width:20px;height:20px";
      scrollContainer.append(scrollTarget, scrollExtent);
      document.body.appendChild(scrollContainer);
      const scrollIntoViewResult = operation => {
        scrollContainer.scrollTo(0, 0);
        operation();
        const rect = scrollTarget.getBoundingClientRect();
        return [
          scrollContainer.scrollLeft,
          scrollContainer.scrollTop,
          rect.x,
          rect.y,
        ].join(",");
      };
      capture("Element.scrollIntoView", "alignment-options", () => [
        scrollContainer.clientWidth,
        scrollContainer.clientHeight,
        scrollContainer.scrollWidth,
        scrollContainer.scrollHeight,
        scrollIntoViewResult(() => scrollTarget.scrollIntoView()),
        scrollIntoViewResult(() => scrollTarget.scrollIntoView(false)),
        scrollIntoViewResult(() =>
          scrollTarget.scrollIntoView({ block: "center", inline: "center" }),
        ),
        scrollIntoViewResult(() =>
          scrollTarget.scrollIntoView({ block: "nearest", inline: "nearest" }),
        ),
      ].join("|"));
      capture("Element.scrollIntoViewIfNeeded", "center-if-needed", () => [
        scrollIntoViewResult(() => scrollTarget.scrollIntoViewIfNeeded()),
        scrollIntoViewResult(() => scrollTarget.scrollIntoViewIfNeeded(false)),
      ].join("|"));
      scrollContainer.remove();
    }
    layoutElement.remove();
  }
  capture("Document.adoptedStyleSheets", "initial-shape", () =>
    [
      Array.isArray(document.adoptedStyleSheets),
      document.adoptedStyleSheets === document.adoptedStyleSheets,
      document.adoptedStyleSheets.length,
    ].join("|"),
  );
  const constructedSheet = new CSSStyleSheet({
    media: "print",
    disabled: true,
    baseURL: "https://ignored.example/",
  });
  capture("CSSStyleSheet", "constructed-shape", () =>
    [
      Object.prototype.toString.call(constructedSheet),
      constructedSheet.ownerNode,
      constructedSheet.ownerRule,
      constructedSheet.href,
      constructedSheet.parentStyleSheet,
      constructedSheet.title,
      constructedSheet.media.mediaText,
      constructedSheet.disabled,
      constructedSheet.cssRules.length,
    ].join("|"),
  );
  capture("CSSStyleSheet.replaceSync", "rules-and-return", () => {
    const value = constructedSheet.replaceSync(
      ".edge-a { color: red; } .edge-b { display: block; }",
    );
    return [
      value,
      constructedSheet.cssRules.length,
      constructedSheet.cssRules[0]?.cssText,
      constructedSheet.cssRules[1]?.cssText,
    ].join("|");
  });
  capture("CSSStyleSheet.insertRule", "default-index", () =>
    [
      constructedSheet.insertRule(".edge-zero { opacity: 0.5; }"),
      constructedSheet.cssRules.length,
      constructedSheet.cssRules[0]?.selectorText,
    ].join("|"),
  );
  capture("CSSStyleSheet.insertRule", "index-error", () =>
    constructedSheet.insertRule(".edge-bad {}", 999),
  );
  {
    const promise = constructedSheet.replace(".edge-replaced { z-index: 2; }");
    capture("CSSStyleSheet.replace", "return-shape", () =>
      [
        Object.prototype.toString.call(promise),
        promise instanceof Promise,
      ].join("|"),
    );
    const resolved = await promise;
    capture("CSSStyleSheet.replace", "resolution", () =>
      [
        resolved === constructedSheet,
        constructedSheet.cssRules.length,
        constructedSheet.cssRules[0]?.cssText,
      ].join("|"),
    );
  }
  capture("Document.adoptedStyleSheets", "assign-constructed", () => {
    const assigned = [constructedSheet];
    document.adoptedStyleSheets = assigned;
    return [
      document.adoptedStyleSheets === assigned,
      document.adoptedStyleSheets.length,
      document.adoptedStyleSheets[0] === constructedSheet,
    ].join("|");
  });
  capture("Document.adoptedStyleSheets", "assign-invalid", () => {
    document.adoptedStyleSheets = [{}];
    return "accepted";
  });
  style.remove();
  capture("HTMLStyleElement.sheet", "after-remove", () =>
    [
      style.sheet,
      styleSheets.length,
      Array.from(styleSheets).includes(styleSheet),
    ].join("|"),
  );
  computedElement.remove();
  document.adoptedStyleSheets = [];

  {
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href =
      "data:text/css,%23edge-link-style-probe%7Bcolor%3Argb(7%2C8%2C9)%7D";
    capture("HTMLLinkElement.sheet", "before-connect", () => link.sheet);
    const completion = new Promise(resolveLink => {
      link.onload = () => resolveLink("load");
      link.onerror = () => resolveLink("error");
    });
    document.head.appendChild(link);
    capture("HTMLLinkElement.sheet", "immediately-after-connect", () =>
      link.sheet,
    );
    const outcome = await completion;
    capture("HTMLLinkElement.sheet", "data-stylesheet-load", () =>
      [
        outcome,
        link.sheet instanceof CSSStyleSheet,
        link.sheet?.ownerNode === link,
        link.sheet?.href,
        link.sheet?.cssRules.length,
        styleSheets[styleSheets.length - 1] === link.sheet,
      ].join("|"),
    );
    link.remove();
  }

  const xml = document.implementation.createDocument("", "", null);
  const instruction = xml.createProcessingInstruction(
    "xml-stylesheet",
    'href="theme.css" media="screen" disabled',
  );
  capture("ProcessingInstruction", "initial-data", () => instruction.data);
  capture("ProcessingInstruction", "getAttribute-href", () =>
    instruction.getAttribute("href"),
  );
  capture("ProcessingInstruction", "getAttribute-media", () =>
    instruction.getAttribute("media"),
  );
  capture("ProcessingInstruction", "getAttribute-disabled", () =>
    instruction.getAttribute("disabled"),
  );
  capture("ProcessingInstruction", "getAttributeNames", () =>
    instruction.getAttributeNames().join(","),
  );
  capture("ProcessingInstruction", "hasAttribute-href", () =>
    instruction.hasAttribute("href"),
  );
  capture("ProcessingInstruction", "hasAttributes", () =>
    instruction.hasAttributes(),
  );
  capture("ProcessingInstruction", "setAttribute", () => {
    instruction.setAttribute("title", "hello world");
    return `${instruction.data}|${instruction.getAttribute("title")}`;
  });
  capture("ProcessingInstruction", "toggleAttribute-remove", () =>
    `${instruction.toggleAttribute("disabled")}|${instruction.data}`,
  );
  capture("ProcessingInstruction", "toggleAttribute-add", () =>
    `${instruction.toggleAttribute("disabled")}|${instruction.data}`,
  );
  capture("ProcessingInstruction", "removeAttribute", () => {
    instruction.removeAttribute("media");
    return `${instruction.data}|${instruction.getAttribute("media")}`;
  });
  capture("ProcessingInstruction", "invalid-name", () =>
    instruction.setAttribute("bad name", "value"),
  );

  const script = document.createElement("script");
  capture("HTMLScriptElement", "empty-reflection", () =>
    [script.text, script.textContent, script.innerText].join("|"),
  );
  capture("HTMLScriptElement", "set-textContent", () => {
    script.textContent = "one";
    return [script.text, script.textContent, script.innerText].join("|");
  });
  capture("HTMLScriptElement", "set-innerText", () => {
    script.innerText = "two";
    return [script.text, script.textContent, script.innerText].join("|");
  });
  capture("HTMLScriptElement", "set-text", () => {
    script.text = "three";
    return [script.text, script.textContent, script.innerText].join("|");
  });

  const workerSource = "self.onmessage = () => {}";
  const workerUrl = URL.createObjectURL(
    new Blob([workerSource], { type: "text/javascript" }),
  );
  const worker = new Worker(workerUrl);
  try {
    capture("Worker", "prototype-own-onmessageerror", () =>
      Object.prototype.hasOwnProperty.call(
        Worker.prototype,
        "onmessageerror",
      ),
    );
    capture("Worker", "instance-has-onmessageerror", () =>
      "onmessageerror" in worker,
    );
    capture("Worker", "instance-read-onmessageerror", () =>
      typeof worker.onmessageerror,
    );
  } finally {
    worker.terminate();
    URL.revokeObjectURL(workerUrl);
  }

  const frame = document.createElement("iframe");
  frame.setAttribute("sandbox", "");
  frame.srcdoc = "<iframe srcdoc='<p>nested</p>'></iframe><p>opaque origin</p>";
  const loaded = new Promise(resolveLoad => {
    frame.addEventListener("load", resolveLoad, { once: true });
  });
  document.body.appendChild(frame);
  await loaded;
  const crossOriginWindow = frame.contentWindow;
  try {
    capture("CrossOriginWindowProxy", "window-identity", () =>
      crossOriginWindow.window === crossOriginWindow,
    );
    capture("CrossOriginWindowProxy", "self-identity", () =>
      crossOriginWindow.self === crossOriginWindow,
    );
    capture("CrossOriginWindowProxy", "frames-identity", () =>
      crossOriginWindow.frames === crossOriginWindow,
    );
    capture("CrossOriginWindowProxy", "parent-identity", () =>
      crossOriginWindow.parent === window,
    );
    capture("CrossOriginWindowProxy", "top-identity", () =>
      crossOriginWindow.top === window,
    );
    capture("CrossOriginWindowProxy", "length", () => crossOriginWindow.length);
    capture("CrossOriginWindowProxy", "closed", () => crossOriginWindow.closed);
    capture("CrossOriginWindowProxy", "close", () => typeof crossOriginWindow.close);
    capture("CrossOriginWindowProxy", "focus", () => typeof crossOriginWindow.focus);
    capture("CrossOriginWindowProxy", "blur", () => typeof crossOriginWindow.blur);
    capture("CrossOriginWindowProxy", "postMessage", () =>
      typeof crossOriginWindow.postMessage,
    );
    capture("CrossOriginWindowProxy", "opener", () => crossOriginWindow.opener);
    capture("CrossOriginWindowProxy", "location-tag", () =>
      Object.prototype.toString.call(crossOriginWindow.location),
    );
    capture("CrossOriginWindowProxy", "location-href", () =>
      crossOriginWindow.location.href,
    );
    capture("CrossOriginWindowProxy", "location-replace", () =>
      typeof crossOriginWindow.location.replace,
    );
    capture("CrossOriginWindowProxy", "blur-parent-identity", () =>
      crossOriginWindow.blur === window.blur,
    );
    capture("CrossOriginWindowProxy", "close-parent-identity", () =>
      crossOriginWindow.close === window.close,
    );
    capture("CrossOriginWindowProxy", "focus-parent-identity", () =>
      crossOriginWindow.focus === window.focus,
    );
    capture("CrossOriginWindowProxy", "postMessage-parent-identity", () =>
      crossOriginWindow.postMessage === window.postMessage,
    );
    capture("CrossOriginWindowProxy", "postMessage-stable-identity", () =>
      crossOriginWindow.postMessage === crossOriginWindow.postMessage,
    );
    capture("CrossOriginWindowProxy", "postMessage-parent-function-prototype", () =>
      Object.getPrototypeOf(crossOriginWindow.postMessage) ===
      Function.prototype,
    );
    capture("CrossOriginWindowProxy", "descriptor-getter-stable-identity", () =>
      Object.getOwnPropertyDescriptor(crossOriginWindow, "window").get ===
      Object.getOwnPropertyDescriptor(crossOriginWindow, "window").get,
    );
    capture("CrossOriginWindowProxy", "indexed-child", () =>
      typeof crossOriginWindow[0],
    );
    capture("CrossOriginWindowProxy", "document", () =>
      crossOriginWindow.document,
    );
    capture("CrossOriginWindowProxy", "name", () => crossOriginWindow.name);
    capture("CrossOriginWindowProxy", "history", () => crossOriginWindow.history);
    capture("CrossOriginWindowProxy", "navigator", () =>
      crossOriginWindow.navigator,
    );
    capture("CrossOriginWindowProxy", "Array", () => crossOriginWindow.Array);
    capture("CrossOriginWindowProxy", "Event", () => crossOriginWindow.Event);
    capture("CrossOriginWindowProxy", "addEventListener", () =>
      crossOriginWindow.addEventListener,
    );
    capture("CrossOriginWindowProxy", "custom-property", () =>
      crossOriginWindow.edgeSandboxProbe,
    );
    capture("CrossOriginWindowProxy", "in-postMessage", () =>
      "postMessage" in crossOriginWindow,
    );
    capture("CrossOriginWindowProxy", "in-document", () =>
      "document" in crossOriginWindow,
    );
    capture("CrossOriginWindowProxy", "descriptor-postMessage", () =>
      Object.getOwnPropertyDescriptor(crossOriginWindow, "postMessage")?.configurable,
    );
    capture("CrossOriginWindowProxy", "descriptor-document", () =>
      Object.getOwnPropertyDescriptor(crossOriginWindow, "document"),
    );
    capture("CrossOriginWindowProxy", "prototype", () =>
      Object.getPrototypeOf(crossOriginWindow),
    );
    capture("CrossOriginWindowProxy", "own-names", () =>
      Object.getOwnPropertyNames(crossOriginWindow).join(","),
    );
    capture("CrossOriginWindowProxy", "own-symbols", () =>
      Object.getOwnPropertySymbols(crossOriginWindow)
        .map(symbol => String(symbol))
        .join(","),
    );
    capture("CrossOriginWindowProxy", "reflect-own-keys", () =>
      Reflect.ownKeys(crossOriginWindow)
        .map(key => String(key))
        .join(","),
    );
    capture("CrossOriginWindowProxy", "object-keys", () =>
      Object.keys(crossOriginWindow).join(","),
    );
    capture("CrossOriginWindowProxy", "object-values", () =>
      Object.values(crossOriginWindow).length,
    );
    capture("CrossOriginWindowProxy", "object-entries", () =>
      Object.entries(crossOriginWindow).length,
    );
    capture("CrossOriginWindowProxy", "own-descriptor-names", () =>
      Object.getOwnPropertyNames(
        Object.getOwnPropertyDescriptors(crossOriginWindow),
      ).join(","),
    );
    capture("CrossOriginWindowProxy", "descriptor-index", () => {
      const descriptor = Object.getOwnPropertyDescriptor(
        crossOriginWindow,
        "0",
      );
      return [
        typeof descriptor?.value,
        descriptor?.writable,
        descriptor?.enumerable,
        descriptor?.configurable,
      ].join(",");
    });
    for (const name of [
      "window",
      "location",
      "closed",
      "length",
      "postMessage",
      "then",
    ]) {
      capture("CrossOriginWindowProxy", `descriptor-shape-${name}`, () => {
        const descriptor = Object.getOwnPropertyDescriptor(
          crossOriginWindow,
          name,
        );
        return [
          "value" in descriptor ? "data" : "accessor",
          typeof descriptor?.value,
          typeof descriptor?.get,
          descriptor?.get?.name ?? "",
          descriptor?.get?.length ?? "",
          typeof descriptor?.set,
          descriptor?.set?.name ?? "",
          descriptor?.set?.length ?? "",
          String(descriptor?.writable),
          descriptor?.enumerable,
          descriptor?.configurable,
        ].join(",");
      });
    }
    capture("CrossOriginWindowProxy", "descriptor-symbols", () =>
      Object.getOwnPropertySymbols(crossOriginWindow)
        .map(symbol => {
          const descriptor = Object.getOwnPropertyDescriptor(
            crossOriginWindow,
            symbol,
          );
          return [
            String(symbol),
            String(descriptor?.value),
            descriptor?.writable,
            descriptor?.enumerable,
            descriptor?.configurable,
          ].join(":");
        })
        .join("|"),
    );
    capture("CrossOriginWindowProxy", "object-values-types", () =>
      Object.values(crossOriginWindow)
        .map(value => typeof value)
        .join(","),
    );
    capture("CrossOriginWindowProxy", "object-entries-values", () =>
      Object.entries(crossOriginWindow)
        .map(([key, value]) => `${key}:${typeof value}`)
        .join(","),
    );
    capture("CrossOriginWindowProxy", "object-is-extensible", () =>
      Object.isExtensible(crossOriginWindow),
    );
    capture("CrossOriginWindowProxy", "reflect-is-extensible", () =>
      Reflect.isExtensible(crossOriginWindow),
    );
    capture("CrossOriginWindowProxy", "object-set-prototype", () =>
      Object.setPrototypeOf(crossOriginWindow, null),
    );
    capture("CrossOriginWindowProxy", "reflect-set-prototype", () =>
      Reflect.setPrototypeOf(crossOriginWindow, null),
    );
    capture("CrossOriginWindowProxy", "object-prevent-extensions", () =>
      Object.preventExtensions(crossOriginWindow),
    );
    capture("CrossOriginWindowProxy", "reflect-prevent-extensions", () =>
      Reflect.preventExtensions(crossOriginWindow),
    );
    capture("CrossOriginWindowProxy", "symbol-read-to-string-tag", () =>
      String(crossOriginWindow[Symbol.toStringTag]),
    );
    capture("CrossOriginWindowProxy", "symbol-in-to-string-tag", () =>
      Symbol.toStringTag in crossOriginWindow,
    );
    capture("CrossOriginWindowProxy", "delete-window", () =>
      Reflect.deleteProperty(crossOriginWindow, "window"),
    );
    capture("CrossOriginWindowProxy", "delete-document", () =>
      Reflect.deleteProperty(crossOriginWindow, "document"),
    );
    capture("CrossOriginWindowProxy", "delete-index", () =>
      Reflect.deleteProperty(crossOriginWindow, "0"),
    );
    capture("CrossOriginWindowProxy", "delete-symbol", () =>
      Reflect.deleteProperty(crossOriginWindow, Symbol.toStringTag),
    );
    capture("CrossOriginWindowProxy", "define-window", () =>
      Reflect.defineProperty(crossOriginWindow, "window", {
        value: 1,
      }),
    );
    capture("CrossOriginWindowProxy", "define-custom", () =>
      Reflect.defineProperty(crossOriginWindow, "edgeSandboxProbe", {
        value: 1,
      }),
    );
    capture("CrossOriginWindowProxy", "define-index", () =>
      Reflect.defineProperty(crossOriginWindow, "0", { value: 1 }),
    );
    capture("CrossOriginWindowProxy", "object-freeze", () =>
      Object.freeze(crossOriginWindow),
    );
    capture("CrossOriginWindowProxy", "object-seal", () =>
      Object.seal(crossOriginWindow),
    );
  } finally {
    frame.remove();
  }
  return rows;
}

function toTsv(headers, rows) {
  const escape = value =>
    String(value ?? "")
      .replaceAll("\\", "\\\\")
      .replaceAll("\t", "\\t")
      .replaceAll("\r", "\\r")
      .replaceAll("\n", "\\n");
  return [headers, ...rows].map(row => row.map(escape).join("\t")).join("\n") + "\n";
}

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

await main();
