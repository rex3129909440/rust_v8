"""Public two-argument API for country-aware Edge sandbox profiles."""

from __future__ import annotations

try:  # Installed wheel layout.
    from .country_profiles.get_random_fp import (
        RandomFingerprint,
        get_random_fp,
        get_random_fp_details,
    )
    from . import country_profiles as _country_profiles

    # Loading the composer submodule makes Python assign that module to the
    # parent package's ``get_random_fp`` attribute.  Restore the documented
    # function exports after the installed-layout import completes.
    _country_profiles.get_random_fp = get_random_fp
    _country_profiles.get_random_fp_details = get_random_fp_details
except ImportError:  # Source checkout layout.
    from demo.get_random_fp import (
        RandomFingerprint,
        get_random_fp,
        get_random_fp_details,
    )

from .edge_profile import EdgeProfile


def create_country_profile(
    country_code: str,
    ua: str | None = None,
    *,
    seed: int | None = None,
    body_child_element_count: int | None = 5,
    body_client_height: float | None = 23.0,
) -> EdgeProfile:
    """Return a complete profile from a country code and optional desktop UA.

    ``country_code`` must be an ISO 3166-1 alpha-2 code.  A Windows or macOS
    Chromium UA selects the matching platform catalogs.  Omitting ``ua`` uses
    the fixed Windows Chrome 150 UA. The standalone BODY defaults to five
    children and a 23-pixel client height.
    """

    return get_random_fp(
        country_code,
        ua,
        seed=seed,
        body_child_element_count=body_child_element_count,
        body_client_height=body_client_height,
    )


def create_country_profile_details(
    country_code: str,
    ua: str | None = None,
    *,
    seed: int | None = None,
    body_child_element_count: int | None = 5,
    body_client_height: float | None = 23.0,
) -> RandomFingerprint:
    """Return the profile plus selected time-zone and hardware metadata.

    The standalone BODY defaults to five children and clientHeight 23.
    """

    return get_random_fp_details(
        country_code,
        ua,
        seed=seed,
        body_child_element_count=body_child_element_count,
        body_client_height=body_client_height,
    )


__all__ = [
    "create_country_profile",
    "create_country_profile_details",
]
