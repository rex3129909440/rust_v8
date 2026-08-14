"""Chromium Android native form-control CSS geometry.

The geometry baseline was captured from the project's connected Pixel 4 over
HTTPS.  It represents Chromium's Android form-control theme, not Pixel GPU or
hardware capability data.  Values are kept separate from Windows controls and
quantized to the selected device DPR in the same way Blink exposes CSS pixels.
"""

from __future__ import annotations


def _number(value: float) -> str:
    return f"{float(value):g}"


def _style(
    *,
    box_sizing: str,
    width: float,
    height: float,
    padding: str,
    border_width: float,
    border_style: str,
    font_family: str = "Arial",
    appearance: str = "auto",
) -> str:
    return (
        f"display:inline-block;box-sizing:{box_sizing};"
        f"width:{_number(width)}px;height:{_number(height)}px;"
        f"padding:{padding};border-width:{_number(border_width)}px;"
        f"border-style:{border_style};font-family:{font_family};"
        "font-size:13.3333px;font-weight:400;line-height:normal;"
        f"color:rgb(0, 0, 0);appearance:{appearance}"
    )


def chromium_android_css_overrides(
    device_pixel_ratio: float,
    locale: str,
    oem: str,
    chromium_major: int,
) -> dict[str, str]:
    """Return a complete Android control profile without desktop leakage."""

    del locale, oem, chromium_major
    # The capture uses DPR 2.75.  Blink snaps native control borders to device
    # pixels, so express the same integer-device-pixel widths at every device
    # DPR instead of copying a Windows DPR table.
    dpr = max(1.0, float(device_pixel_ratio))
    border = 5.0 / dpr
    thin_border = 2.0 / dpr
    text_width = 184.363636
    text_height = 15.636364
    search_width = 192.0
    search_height = 21.272727
    date_width = 133.333333
    date_height = 17.636364
    return {
        "input_common": "",
        "input_hidden": (
            "display:none;box-sizing:content-box;width:auto;height:auto;"
            "padding:0;border-width:0;border-style:none;appearance:none"
        ),
        "input_search": _style(
            box_sizing="border-box", width=search_width,
            height=search_height, padding="1px 2px",
            border_width=border, border_style="inset",
        ),
        "input_checkbox_radio": _style(
            box_sizing="border-box", width=16, height=16, padding="0",
            border_width=0, border_style="none",
        ),
        "input_range": _style(
            box_sizing="content-box", width=129, height=16, padding="0",
            border_width=0, border_style="none",
        ),
        "input_color": _style(
            box_sizing="border-box", width=50, height=27, padding="1px 2px",
            border_width=thin_border, border_style="solid",
        ),
        "input_date": _style(
            box_sizing="content-box", width=date_width, height=date_height,
            padding="1px 2px", border_width=thin_border,
            border_style="solid",
        ),
        "input_time": _style(
            box_sizing="content-box", width=date_width, height=date_height,
            padding="1px 2px", border_width=thin_border,
            border_style="solid",
        ),
        "input_datetime_local": _style(
            box_sizing="content-box", width=199.993513, height=date_height,
            padding="1px 2px", border_width=thin_border,
            border_style="solid",
        ),
        "input_month": _style(
            box_sizing="content-box", width=date_width, height=date_height,
            padding="1px 2px", border_width=thin_border,
            border_style="solid",
        ),
        "input_week": _style(
            box_sizing="content-box", width=date_width, height=date_height,
            padding="1px 2px", border_width=thin_border,
            border_style="solid",
        ),
        "input_image": _style(
            box_sizing="content-box", width=0, height=0, padding="0",
            border_width=0, border_style="none", appearance="none",
        ),
        "input_button": _style(
            box_sizing="border-box", width=15.636364,
            height=search_height, padding="1px 6px",
            border_width=border, border_style="outset",
        ),
        "input_submit_reset": _style(
            box_sizing="border-box", width=57.67614, height=search_height,
            padding="1px 6px", border_width=border,
            border_style="outset",
        ),
        "input_file": _style(
            box_sizing="content-box", width=254.909091, height=21.272727,
            padding="0", border_width=0, border_style="none",
            appearance="none",
        ),
        "input_text": _style(
            box_sizing="content-box", width=text_width, height=text_height,
            padding="1px 2px", border_width=border, border_style="inset",
        ),
    }


__all__ = ["chromium_android_css_overrides"]
