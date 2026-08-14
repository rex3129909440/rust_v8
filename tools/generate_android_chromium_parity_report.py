"""Generate a reproducible PC/Android Chromium 140-151 parity report.

Only previously captured HTTPS evidence is read.  The report intentionally
does not treat ChromeDriver transport globals as browser API evidence; the
collector records the removed names in each Android capture.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PC_DIR = ROOT / "build" / "chromium-version-surfaces"
ANDROID_DIR = ROOT / "build" / "chromium-android-version-surfaces"
REPORT_JSON = ANDROID_DIR / "pc-android-parity-140-151.json"
REPORT_MD = ROOT / "docs" / "ANDROID_CHROMIUM_140_151_PARITY_ZH.md"


def load(major: int, android: bool) -> dict[str, Any]:
    path = (
        ANDROID_DIR / f"chromium-android-{major}-https-surface.json"
        if android
        else PC_DIR / f"chromium-{major}-surface.json"
    )
    return json.loads(path.read_text(encoding="utf-8"))


def ordered_diff(left: list[str], right: list[str]) -> dict[str, Any]:
    left_set = set(left)
    right_set = set(right)
    common_left = [value for value in left if value in right_set]
    common_right = [value for value in right if value in left_set]
    return {
        "leftCount": len(left),
        "rightCount": len(right),
        "pcOnly": [value for value in left if value not in right_set],
        "androidOnly": [value for value in right if value not in left_set],
        "commonOrderEqual": common_left == common_right,
    }


def surface_names(document: dict[str, Any], path: tuple[str, ...]) -> list[str]:
    current: Any = document
    for component in path:
        current = current[component]
    return list(current["names"])


def interface_diffs(
    pc: dict[str, Any], android: dict[str, Any], category: str
) -> dict[str, Any]:
    result: dict[str, Any] = {}
    owners = sorted(set(pc[category]) | set(android[category]))
    for owner in owners:
        left = list(pc[category].get(owner, {}).get("names", []))
        right = list(android[category].get(owner, {}).get("names", []))
        if left != right:
            result[owner] = ordered_diff(left, right)
    return result


def first_by_key(value: Any, target: str) -> Any:
    if isinstance(value, dict):
        if target in value:
            return value[target]
        for child in value.values():
            found = first_by_key(child, target)
            if found is not None:
                return found
    elif isinstance(value, list):
        for child in value:
            found = first_by_key(child, target)
            if found is not None:
                return found
    return None


def unwrap_probe(value: Any) -> Any:
    if isinstance(value, dict) and value.get("ok") is True and "value" in value:
        return value["value"]
    return value


def runtime_summary(document: dict[str, Any]) -> dict[str, Any]:
    runtime = document.get("runtimeEvidence", {})
    performance = runtime.get("performance", {})
    supported = unwrap_probe(first_by_key(performance, "supportedEntryTypes"))
    media = runtime.get("media", {})
    return {
        "secureContext": runtime.get("secureContext"),
        "navigator": runtime.get("navigator"),
        "connection": runtime.get("connection"),
        "screen": runtime.get("screen"),
        "orientation": runtime.get("orientation"),
        "window": runtime.get("window"),
        "visualViewport": runtime.get("visualViewport"),
        "mediaQueries": runtime.get("mediaQueries"),
        "webgl": runtime.get("webgl"),
        "performancePrototype": performance.get("prototype", {}).get("names", []),
        "performanceObserverSupportedEntryTypes": supported,
        "performanceEntries": performance.get("entries"),
        "media": {
            "canPlayType": media.get("canPlayType"),
            "mediaSource": media.get("mediaSource"),
            "mediaRecorder": media.get("mediaRecorder"),
            "supportedConstraints": media.get("supportedConstraints"),
            "decodingInfo": media.get("decodingInfo"),
            "audioContext": media.get("audioContext"),
        },
    }


def render_names(values: list[str]) -> str:
    return "、".join(f"`{value}`" for value in values) if values else "—"


def main() -> None:
    majors = range(140, 152)
    pc_documents = {major: load(major, False) for major in majors}
    android_documents = {major: load(major, True) for major in majors}
    versions: dict[str, Any] = {}
    for major in majors:
        pc = pc_documents[major]
        android = android_documents[major]
        artifact = android.get("webdriverArtifactSanitization", {})
        versions[str(major)] = {
            "pcUserAgent": pc["userAgent"],
            "androidUserAgent": android["userAgent"],
            "httpsEvidence": android.get("httpsEvidence"),
            "webdriverArtifactSanitization": artifact,
            "window": ordered_diff(
                surface_names(pc, ("topWindowBeforeFrame",)),
                surface_names(android, ("topWindowBeforeFrame",)),
            ),
            "navigator": ordered_diff(
                surface_names(pc, ("navigatorPrototype",)),
                surface_names(android, ("navigatorPrototype",)),
            ),
            "workerGlobal": ordered_diff(
                surface_names(pc, ("worker", "global")),
                surface_names(android, ("worker", "global")),
            ),
            "workerNavigator": ordered_diff(
                surface_names(pc, ("worker", "navigatorPrototype")),
                surface_names(android, ("worker", "navigatorPrototype")),
            ),
            "constructorPrototypeDifferences": interface_diffs(
                pc, android, "constructorPrototypes"
            ),
            "constructorStaticDifferences": interface_diffs(
                pc, android, "constructorStatics"
            ),
            "globalObjectDifferences": interface_diffs(pc, android, "globalObjects"),
            "workerConstructorPrototypeDifferences": interface_diffs(
                pc["worker"], android["worker"], "constructorPrototypes"
            ),
            "runtime": {
                "pc": runtime_summary(pc),
                "android": runtime_summary(android),
            },
        }

    manifest = json.loads(
        (ANDROID_DIR / "android-https-manifest.json").read_text(encoding="utf-8")
    )
    report = {
        "scope": "Chromium PC/Android 140-151 on valid HTTPS",
        "evidenceOrigin": manifest["evidenceOrigin"],
        "deviceSerial": manifest["device"],
        "buildEvidence": manifest["builds"],
        "versions": versions,
    }
    REPORT_JSON.write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    lines = [
        "# Chromium 140–151 PC / Android HTTPS 差异与实现基线",
        "",
        "> 本文由 `tools/generate_android_chromium_parity_report.py` 从保存的 HTTPS 证据自动生成。",
        "",
        "## 证据边界",
        "",
        f"- Android 实机：Pixel 4，ADB 序列号 `{manifest['device']}`。",
        f"- 页面来源：`{manifest['evidenceOrigin']}`，所有正式样本均要求 `isSecureContext === true`。",
        "- Android 版本来自 Chromium 官方 Android Arm64 snapshot；每个 APK 的 revision、URL、压缩包元数据和 SHA-256 在机器报告中保存。",
        "- PC 对照来自 Chrome for Testing 官方构建。Chrome 与同版本 Edge 共用 Chromium/Blink/V8 的基础 Web API 表；Edge 自有扩展仍以 Edge 证据单独处理。",
        "- ChromeDriver 注入的 `cdc_*` 和 `ret_nodes` 共 8 个传输属性在统计前剔除，剔除规则和具体名称保存在每份 Android 证据中。",
        "- HTTP、空白页和不安全上下文样本不进入正式基线，因为权限、媒体、设备和若干接口会被安全上下文改变。",
        "",
        "## 顶层表规模",
        "",
        "| 版本 | PC Window | Android Window | PC Navigator | Android Navigator | PC Worker | Android Worker | 原型存在差异的接口数 |",
        "|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for major in majors:
        item = versions[str(major)]
        lines.append(
            f"| {major} | {item['window']['leftCount']} | {item['window']['rightCount']} | "
            f"{item['navigator']['leftCount']} | {item['navigator']['rightCount']} | "
            f"{item['workerGlobal']['leftCount']} | {item['workerGlobal']['rightCount']} | "
            f"{len(item['constructorPrototypeDifferences'])} |"
        )

    lines += [
        "",
        "## 每个版本的顶层平台差异",
        "",
    ]
    for major in majors:
        item = versions[str(major)]["window"]
        lines += [
            f"### {major}",
            "",
            f"- Android 独有：{render_names(item['androidOnly'])}",
            f"- PC 独有：{render_names(item['pcOnly'])}",
            f"- 两端共有成员的相对顺序一致：`{str(item['commonOrderEqual']).lower()}`。",
            "",
        ]

    latest = versions["151"]
    lines += [
        "## 需要单独实现的差异类别",
        "",
        "- 移动输入：触控事件处理器、粗指针/无 hover 媒体查询、`maxTouchPoints`、软键盘与方向变化。",
        "- Android 接口：联系人、Web NFC、条码、Content Index，以及只在部分版本出现的 Cookie/ModelContext/WebMCP 接口。",
        "- 桌面专用接口隐藏：HID、local fonts、EyeDropper、画中画和部分 AI 接口不能泄漏到 Android Window/Navigator。",
        "- 媒体：编解码支持、MediaSource/MediaRecorder、摄像头约束、设备枚举和 AudioContext 参数必须按移动平台选择。",
        "- Performance：原型成员、Observer entry type 和各 Entry 的 `toJSON()` 字段都存在版本增量，不能只改顶层键。",
        "- Realm：顶层 Window、iframe Window、Dedicated Worker 与 WorkerNavigator 分别应用同一平台/版本基线，用户属性不能跨 Realm 泄漏。",
        "- 安全上下文：联系人、NFC、媒体设备等行为以 HTTPS 证据为准；跨源 iframe 仍需执行同源访问限制。",
        "",
        "## Chromium 151 示例：Navigator / Worker 差异",
        "",
        f"- Navigator Android 独有：{render_names(latest['navigator']['androidOnly'])}",
        f"- Navigator PC 独有：{render_names(latest['navigator']['pcOnly'])}",
        f"- WorkerNavigator Android 独有：{render_names(latest['workerNavigator']['androidOnly'])}",
        f"- WorkerNavigator PC 独有：{render_names(latest['workerNavigator']['pcOnly'])}",
        "",
        "## Performance 版本节点",
        "",
        "- 140–143：`Performance.prototype` 尚无 `interactionCount`。",
        "- 144 起：加入 `interactionCount`。",
        "- 143 起：资源/导航条目的 JSON 开始出现 `contentEncoding`。",
        "- 145 起：条目增加 `confidence`；paint/long-animation-frame 增加 paint/presentation 时间字段。",
        "- 148 起：资源/导航条目的 JSON 增加 `contentType`。",
        "- 151：Entry 增加 `navigationId`，PerformanceObserver 新增 `interaction-contentful-paint` 与 `soft-navigation`。",
        "",
        "## 输出与维护规则",
        "",
        f"- 完整机器可读差异：`{REPORT_JSON.relative_to(ROOT).as_posix()}`。",
        "- Rust 运行时表由 `tools/generate_browser_surface_data.py` 分别生成 PC 与 Android 模块；Android 模块包含平台内不变但与 PC 不同的原型，避免桌面成员泄漏。",
        "- 后续每次修复必须同时更新证据、生成表、回归测试和 `docs/SANDBOX_REPAIR_AUDIT_ZH.md`。",
        "- 当前文档是结构基线，不把 Pixel 4 单机可配置值误当成全体 Android 固定值；屏幕、DPR、WebGL、CPU、内存、语言、网络和媒体设备值仍由移动 profile 提供。",
        "",
    ]
    REPORT_MD.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(REPORT_JSON)
    print(REPORT_MD)


if __name__ == "__main__":
    main()
