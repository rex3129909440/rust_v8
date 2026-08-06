"""Public two-argument API for country-aware Edge sandbox profiles."""

from __future__ import annotations

try:  # Installed wheel layout.
    from .country_profiles.get_random_fp import (
        RandomFingerprint,
        get_random_fp,
        get_random_fp_details,
    )
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
) -> EdgeProfile:
    """Return a complete profile from a country code and optional desktop UA.

    ``country_code`` must be an ISO 3166-1 alpha-2 code.  A Windows or macOS
    Chromium UA selects the matching platform catalogs.  Omitting ``ua`` uses
    the fixed Windows Chrome 150 UA.
    """

    return get_random_fp(country_code, ua, seed=seed)


def create_country_profile_details(
    country_code: str,
    ua: str | None = None,
    *,
    seed: int | None = None,
) -> RandomFingerprint:
    """Return the profile plus selected time-zone and hardware metadata."""

    return get_random_fp_details(country_code, ua, seed=seed)


__all__ = [
    "create_country_profile",
    "create_country_profile_details",
]
