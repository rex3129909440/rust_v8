"""Observed Windows Chromium input-control CSS geometry catalogs.

The Chromium 150 rows were captured from Microsoft Edge 150.0.4078.65 on
Windows with a fresh headless profile for every UI locale and device scale
factor.  Values are computed CSS pixels, not physical screen pixels.

The Chromium 148 zh-CN compatibility row comes from the supplied successful
browser observation and is intentionally limited to that version/locale/DPR
combination.  It is not extrapolated to unrelated environments.
"""

from __future__ import annotations

from typing import Final


# main border, color border, search width/height, text content width/height,
# empty-button border-box width/height
_COMMON: Final[dict[float, tuple[float, ...]]] = {
    1.00: (2.0, 1.0, 177.0, 21.0, 169.0, 15.0, 16.0, 21.0),
    1.25: (1.6, 0.8, 169.6, 21.2, 162.4, 16.0, 15.2, 21.2),
    1.50: (2.0, 0.666667, 170.0, 21.3333, 162.0, 15.3333, 16.0, 21.3333),
    1.75: (1.71429, 0.571429, 173.143, 20.8571, 165.714, 15.4286, 15.4286, 20.8571),
    2.00: (2.0, 1.0, 170.5, 21.5, 162.5, 15.5, 16.0, 21.5),
    2.25: (1.77778, 0.888889, 169.778, 20.6667, 162.222, 15.1111, 15.5556, 20.6667),
    2.50: (2.0, 0.8, 172.8, 21.2, 164.8, 15.2, 16.0, 21.2),
    3.00: (2.0, 1.0, 170.333, 21.0, 162.333, 15.0, 16.0, 21.0),
}


# file width/height, submit-reset width/height, time content width/height,
# date content width/height, datetime-local content width/height,
# month content width/height, week content width/height.
_LOCALIZED: Final[dict[str, dict[float, tuple[float, ...]]]] = {
    "en": {
        1.00: (253, 21, 57.5, 21, 93, 20, 111.328, 17.3281, 183.328, 17.3281, 139.328, 17.3281, 131.328, 17.3281),
        1.25: (252.8, 21.2, 56.6875, 21.2, 92.8, 20, 111.725, 17.325, 183.725, 17.325, 138.925, 17.325, 131.725, 17.325),
        1.50: (252.667, 21.3333, 57.5, 21.3333, 92.6667, 20, 111.333, 17.3333, 183.333, 17.3333, 138.667, 17.3333, 131.333, 17.3333),
        1.75: (252.571, 20.8571, 56.9286, 20.8571, 93.1339, 19.9911, 111.616, 17.4286, 183.616, 17.4286, 139.045, 17.4286, 131.616, 17.4286),
        2.00: (252.5, 21.5, 57.4922, 21.5, 93, 20, 111.328, 18, 183.328, 18, 138.828, 18, 131.328, 18),
        2.25: (252.444, 20.6667, 57.0556, 20.6667, 92.8889, 20, 111.556, 18, 183.556, 18, 138.667, 18, 131.556, 18),
        2.50: (252.4, 21.2, 57.4938, 21.2, 92.8, 20, 111.331, 17.6, 183.331, 17.6, 138.931, 17.6, 131.331, 17.6),
        3.00: (252.333, 21, 57.5, 21, 92.6667, 20, 111.333, 17.6667, 183.333, 17.6667, 138.667, 17.6667, 131.333, 17.6667),
    },
    "de": {
        1.00: (259, 21, 76.0469, 21, 69, 20, 111.328, 17.3281, 159.328, 17.3281, 139.328, 17.3281, 139.328, 17.3281),
        1.25: (258.4, 21.2, 75.2375, 21.2, 68.8, 20, 111.725, 17.325, 159.725, 17.325, 138.925, 17.325, 138.925, 17.325),
        1.50: (259.333, 21.3333, 76.0521, 21.3333, 68.6667, 20, 111.333, 17.3333, 159.333, 17.3333, 138.667, 17.3333, 138.667, 17.3333),
        1.75: (258.286, 20.8571, 75.4732, 20.8571, 69.1339, 19.9911, 111.616, 17.4286, 159.616, 17.4286, 139.045, 17.4286, 139.045, 17.4286),
        2.00: (259, 21.5, 76.0391, 21.5, 69, 20, 111.328, 18, 159.328, 18, 138.828, 18, 138.828, 18),
        2.25: (258.667, 20.6667, 75.6111, 20.6667, 68.8889, 20, 111.556, 18, 159.556, 18, 138.667, 18, 138.667, 18),
        2.50: (258.8, 21.2, 76.05, 21.2, 68.8, 20, 111.331, 17.6, 159.331, 17.6, 138.931, 17.6, 138.931, 17.6),
        3.00: (259, 21, 76.0521, 21, 68.6667, 20, 111.333, 17.6667, 159.333, 17.6667, 138.667, 17.6667, 138.667, 17.6667),
    },
    "ja": {
        1.00: (300, 27, 42.6719, 27, 67, 20, 107.328, 17.3281, 153.328, 17.3281, 105.328, 17.3281, 154.328, 17.3281),
        1.25: (299.2, 26, 41.8625, 26, 68, 20, 110.125, 17.325, 157.325, 17.325, 107.725, 17.325, 158.125, 17.325),
        1.50: (300, 27.3333, 42.6667, 27.3333, 65.3333, 20, 103.333, 17.3333, 147.333, 17.3333, 101.333, 17.3333, 148, 17.3333),
        1.75: (298.857, 26.5714, 42.0982, 26.5714, 65.7054, 19.9911, 103.616, 17.3304, 147.616, 17.3304, 101.33, 17.3304, 148.188, 17.3304),
        2.00: (299.5, 27, 42.6641, 27, 65.5, 20, 103.328, 17.3281, 147.328, 17.3281, 101.328, 17.3281, 148.328, 17.3281),
        2.25: (299.111, 26.8889, 42.2222, 26.8889, 65.3333, 20, 103.556, 17.3333, 147.556, 17.3333, 101.333, 17.3333, 148, 17.3333),
        2.50: (299.6, 27.2, 42.6688, 27.2, 65.6, 20, 103.331, 17.3312, 147.331, 17.3312, 101.331, 17.3312, 148.131, 17.3312),
        3.00: (299.667, 27.3333, 42.6667, 27.3333, 65.3333, 20, 103.333, 17.3333, 147.333, 17.3333, 101.333, 17.3333, 148, 17.3333),
    },
    "pt": {
        1.00: (278, 21, 53.7969, 21, 69, 20, 111.328, 17.3281, 159.328, 17.3281, 161.328, 17.3281, 146.328, 17.3281),
        1.25: (277.6, 21.2, 52.9875, 21.2, 68.8, 20, 111.725, 17.325, 159.725, 17.325, 161.325, 17.325, 146.125, 17.325),
        1.50: (278, 21.3333, 53.8021, 21.3333, 68.6667, 20, 111.333, 17.3333, 159.333, 17.3333, 160.667, 17.3333, 146, 17.3333),
        1.75: (277.714, 20.8571, 53.2232, 20.8571, 69.1339, 19.9911, 111.616, 17.4286, 159.616, 17.4286, 160.759, 17.4286, 146.473, 17.4286),
        2.00: (278, 21.5, 53.7891, 21.5, 69, 20, 111.328, 18, 159.328, 18, 160.828, 18, 146.328, 18),
        2.25: (277.778, 20.6667, 53.3542, 20.6667, 68.8889, 20, 111.556, 18, 159.556, 18, 160.889, 18, 146.222, 18),
        2.50: (278, 21.2, 53.7938, 21.2, 68.8, 20, 111.331, 17.6, 159.331, 17.6, 160.931, 17.6, 146.131, 17.6),
        3.00: (278, 21, 53.7969, 21, 68.6667, 20, 111.333, 17.6667, 159.333, 17.6667, 160.667, 17.6667, 146, 17.6667),
    },
    "zh": {
        1.00: (253, 25, 42.6719, 25, 67, 20, 107.328, 17.3281, 153.328, 17.3281, 105.328, 17.3281, 133.328, 17.3281),
        1.25: (252.8, 24.4, 41.8625, 24.4, 68, 20, 110.125, 17.325, 157.325, 17.325, 107.725, 17.325, 136.525, 17.325),
        1.50: (252.667, 25.3333, 42.6667, 25.3333, 65.3333, 20, 103.333, 17.3333, 147.333, 17.3333, 101.333, 17.3333, 128, 17.3333),
        1.75: (252.571, 24.8571, 42.0982, 24.8571, 65.7054, 19.9911, 103.616, 17.3304, 147.616, 17.3304, 101.33, 17.3304, 128.188, 17.3304),
        2.00: (252.5, 25.5, 42.6641, 25.5, 65.5, 20, 103.328, 17.5, 147.328, 17.5, 101.328, 17.5, 128.328, 17.5),
        2.25: (252.444, 25.1111, 42.2222, 25.1111, 65.3333, 20, 103.556, 17.3333, 147.556, 17.3333, 101.333, 17.3333, 128, 17.3333),
        2.50: (252.4, 25.6, 42.6688, 25.6, 65.6, 20, 103.331, 17.6, 147.331, 17.6, 101.331, 17.6, 128.131, 17.6),
        3.00: (252.333, 25.3333, 42.6667, 25.3333, 65.3333, 20, 103.333, 17.3333, 147.333, 17.3333, 101.333, 17.3333, 128, 17.3333),
    },
}


def _number(value: float) -> str:
    return f"{value:g}"


def _nearest_dpr(device_pixel_ratio: float) -> float:
    dpr = float(device_pixel_ratio)
    return min(_COMMON, key=lambda candidate: abs(candidate - dpr))


def _style(
    *,
    box_sizing: str,
    width: float,
    height: float,
    padding: str,
    border_width: float | str,
    border_style: str,
    font_family: str = "Arial",
) -> str:
    border = border_width if isinstance(border_width, str) else _number(border_width)
    return (
        f"display:inline-block;box-sizing:{box_sizing};"
        f"width:{_number(width)}px;height:{_number(height)}px;"
        f"padding:{padding};border-width:{border}px;border-style:{border_style};"
        f"font-family:{font_family};font-size:13.3333px;font-weight:400;"
        "line-height:normal;color:rgb(0, 0, 0);appearance:auto"
    )


def chromium150_windows_css_overrides(
    device_pixel_ratio: float,
    locale: str,
) -> dict[str, str]:
    """Return Edge 150 CSS profile fields linked to DPR and UI locale."""

    dpr = _nearest_dpr(device_pixel_ratio)
    language = str(locale).split("-", 1)[0].lower()
    localized = _LOCALIZED.get(language, _LOCALIZED["en"])[dpr]
    (
        file_width,
        file_height,
        submit_width,
        submit_height,
        time_width,
        time_height,
        date_width,
        date_height,
        datetime_width,
        datetime_height,
        month_width,
        month_height,
        week_width,
        week_height,
    ) = localized
    (
        border,
        color_border,
        search_width,
        search_height,
        text_width,
        text_height,
        button_width,
        button_height,
    ) = _COMMON[dpr]

    return {
        "input_common": "",
        "input_hidden": (
            "display:none;box-sizing:content-box;width:auto;height:auto;"
            "padding:0;border-width:0;border-style:none"
        ),
        "input_search": _style(
            box_sizing="border-box", width=search_width, height=search_height,
            padding="1px 2px", border_width=border, border_style="inset"
        ),
        "input_checkbox_radio": _style(
            box_sizing="border-box", width=13, height=13, padding="0",
            border_width=0, border_style="none"
        ),
        "input_range": _style(
            box_sizing="content-box", width=129, height=16, padding="0",
            border_width=0, border_style="none"
        ),
        "input_color": _style(
            box_sizing="border-box", width=50, height=27, padding="1px 2px",
            border_width=color_border, border_style="solid"
        ),
        "input_date": _style(
            box_sizing="content-box", width=date_width, height=date_height,
            padding="0 0 0 1px", border_width=border, border_style="inset",
            font_family="monospace"
        ),
        "input_time": _style(
            box_sizing="content-box", width=time_width, height=time_height,
            padding="0 0 0 1px", border_width=border, border_style="inset",
            font_family="monospace"
        ),
        "input_datetime_local": _style(
            box_sizing="content-box", width=datetime_width, height=datetime_height,
            padding="0 0 0 1px", border_width=border, border_style="inset",
            font_family="monospace"
        ),
        "input_month": _style(
            box_sizing="content-box", width=month_width, height=month_height,
            padding="0 0 0 1px", border_width=border, border_style="inset",
            font_family="monospace"
        ),
        "input_week": _style(
            box_sizing="content-box", width=week_width, height=week_height,
            padding="0 0 0 1px", border_width=border, border_style="inset",
            font_family="monospace"
        ),
        "input_image": _style(
            box_sizing="content-box", width=0, height=0, padding="0",
            border_width=0, border_style="none"
        ),
        "input_button": _style(
            box_sizing="border-box", width=button_width, height=button_height,
            padding="1px 6px", border_width=border, border_style="outset"
        ),
        "input_submit_reset": _style(
            box_sizing="border-box", width=submit_width, height=submit_height,
            padding="1px 6px", border_width=border, border_style="outset"
        ),
        "input_file": _style(
            box_sizing="content-box", width=file_width, height=file_height,
            padding="0", border_width=0, border_style="none"
        ),
        "input_text": _style(
            box_sizing="content-box", width=text_width, height=text_height,
            padding="1px 2px", border_width=border, border_style="inset"
        ),
    }


def chromium148_zh_cn_dpr1_css_overrides() -> dict[str, str]:
    """Return only the supplied successful Chromium 148 zh-CN/DPR=1 row."""

    values = chromium150_windows_css_overrides(1.0, "zh-CN")
    values.update(
        input_range=_style(
            box_sizing="border-box", width=129, height=16, padding="0",
            border_width=2, border_style="inset"
        ),
        input_color=_style(
            box_sizing="border-box", width=50, height=27, padding="1px 2px",
            border_width=2, border_style="solid"
        ),
        input_checkbox_radio=_style(
            box_sizing="border-box", width=13, height=13, padding="0",
            border_width=2, border_style="inset"
        ),
        input_search=_style(
            box_sizing="border-box", width=170.5, height=21.5,
            padding="1px 2px", border_width=2, border_style="inset"
        ),
        input_text=_style(
            box_sizing="content-box", width=162.5, height=15.5,
            padding="1px 2px", border_width=2, border_style="inset"
        ),
        input_time=_style(
            box_sizing="content-box", width=162.5, height=15.5,
            padding="1px 2px", border_width=2, border_style="inset"
        ),
        input_button=_style(
            box_sizing="border-box", width=61, height=22, padding="1px 6px",
            border_width=2, border_style="outset"
        ),
        input_submit_reset=_style(
            box_sizing="border-box", width=61.4625, height=21.5,
            padding="1px 6px", border_width=2, border_style="outset"
        ),
        input_file=_style(
            box_sizing="content-box", width=299, height=24, padding="0",
            border_width="2.2px 2.1", border_style="inset"
        ),
    )
    return values


__all__ = (
    "chromium148_zh_cn_dpr1_css_overrides",
    "chromium150_windows_css_overrides",
)
