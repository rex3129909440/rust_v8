"""Initialize an iframe XHR hook without trace or command-line arguments."""

from pathlib import Path

try:
    from .edge_runtime_options import EdgeRunOptions, IframeHook, NetworkReplayEntry
    from .run_sandbox import EdgeSandbox
except ImportError:
    from edge_runtime_options import EdgeRunOptions, IframeHook, NetworkReplayEntry
    from run_sandbox import EdgeSandbox


IFRAME_XHR_HOOK = IframeHook(
    name="iframe-xhr",
    source=r"""
const originalOpen = XMLHttpRequest.prototype.open;
const originalSend = XMLHttpRequest.prototype.send;
const requestMetadata = new WeakMap();

XMLHttpRequest.prototype.open = __edgev8.proxy(
  function open(method, url) {
    requestMetadata.set(this, {
      method: String(method),
      url: String(url)
    });
    return Reflect.apply(originalOpen, this, arguments);
  },
  "open"
);

XMLHttpRequest.prototype.send = __edgev8.proxy(
  function send() {
    const body = arguments[0];
    const metadata = requestMetadata.get(this) ?? {};
    parent.__iframeHookRequests ??= [];
    parent.__iframeHookRequests.push({
      method: metadata.method,
      url: metadata.url,
      body: body == null ? null : String(body)
    });
    return Reflect.apply(originalSend, this, arguments);
  },
  "send"
);
""",
)


def run_example(
    *,
    library: Path | None = None,
    worker: Path | None = None,
) -> tuple[str, tuple[object, ...]]:
    options = EdgeRunOptions(
        iframe_hooks=(IFRAME_XHR_HOOK,),
        network_replay=(
            NetworkReplayEntry(
                url="https://sandbox.test/tl",
                method="POST",
                body=b"ok",
                headers=(("content-type", "text/plain"),),
            ),
        ),
    )
    javascript = r"""
    (() => {
      const frame = document.createElement("iframe");
      frame.srcdoc = `<script>
        const request = new XMLHttpRequest();
        request.open("POST", "/tl");
        request.send("payload");
      <\/script>`;
      document.body.appendChild(frame);
      const item = __iframeHookRequests[0];
      return [
        item.method,
        item.url,
        item.body,
        Function.prototype.toString.call(
          frame.contentWindow.XMLHttpRequest.prototype.send
        ),
        "__edgev8" in frame.contentWindow
      ].join("|");
    })()
    """
    with EdgeSandbox(library=library, worker=worker, options=options) as sandbox:
        value = sandbox.evaluate(javascript)
        requests = sandbox.network_requests()
        return value, requests
