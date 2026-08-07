"""Country-aware fingerprint profile generation APIs.

The public names are resolved lazily.  Importing a catalog such as
``country_profiles.fp.mac_chromium150_capture_catalog`` must not eagerly import
the profile composer because that composer depends on ``mac_edge_profile``.
"""

from __future__ import annotations

from importlib import import_module

__all__ = [
    "DEFAULT_MAC_USER_AGENT",
    "DEFAULT_WINDOWS_USER_AGENT",
    "FingerprintVerification",
    "RandomFingerprint",
    "get_random_fp",
    "get_random_fp_details",
    "test_random_fp_combinations",
    "verify_random_fp",
]


def _load_composer():
    profile_composer = import_module(f"{__name__}.get_random_fp")
    # Importing a child module assigns that module to the matching parent
    # attribute.  Restore the documented public function of the same name and
    # cache all other exports once composition is actually requested.
    globals().update(
        (public_name, getattr(profile_composer, public_name))
        for public_name in __all__
    )
    return profile_composer


def get_random_fp(*args, **kwargs):
    """Lazily compose and return an ``EdgeProfile``."""

    return _load_composer().get_random_fp(*args, **kwargs)


def get_random_fp_details(*args, **kwargs):
    """Lazily compose a profile together with its selection details."""

    return _load_composer().get_random_fp_details(*args, **kwargs)


def test_random_fp_combinations(*args, **kwargs):
    """Lazily run the random-profile combination checks."""

    return _load_composer().test_random_fp_combinations(*args, **kwargs)


def verify_random_fp(*args, **kwargs):
    """Lazily verify a composed profile in an isolated sandbox."""

    return _load_composer().verify_random_fp(*args, **kwargs)


def __getattr__(name: str):
    if name in __all__:
        return getattr(_load_composer(), name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
