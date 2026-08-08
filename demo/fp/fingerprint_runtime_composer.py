"""
Self-contained runtime fingerprint composer.

Context notes for future edits:

- The user explicitly does not want a command-line generator here.
- The user explicitly does not want this file to import constants from
  fingerprint_profile_export, because that makes the runtime path depend on
  too many files.
- This file should stay as the single Python-side composition module. Add later
  screen/window/webgl/navigator composition steps here instead of creating a
  separate profile-output generator.
- Values produced here are meant to be filled directly into the runtime
  environment config or man8vm fingerprint patch in memory.
- The country pool is the global ISO 3166-1 alpha-2 pool, including countries,
  territories, and special areas, so follow-up fingerprint selection can start
  from any supported two-letter code.
- Timezone data is modeled from IANA tz database country zones plus common
  backward/link aliases and practical GMT/UTC fixed-offset spellings. Offset
  aliases such as GMT+8, UTC+08:00, and Etc/GMT-8 should canonicalize to a
  representative IANA timezone, currently Asia/Shanghai for +08:00.
- Language data is intentionally split into primary and secondary pools.
  primary is the usual navigator.language; secondary is only a candidate pool
  for bounded ordered navigator.languages profiles. Do not generate arbitrary
  secondary-language permutations; those are not realistic browser settings.
"""

from __future__ import annotations

import random
import re
from typing import Iterable, Sequence


# ISO 3166-1 alpha-2 current country, territory, and special area codes.
#
# Keep this list self-contained. Do not replace it with an import from
# fingerprint_generator.py or any generated JSON file; the whole point is that
# runtime composition can run from this one module.
WORLD_COUNTRY_CODE_POOL: tuple[str, ...] = (
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT",
    "AU", "AW", "AX", "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI",
    "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS", "BT", "BV", "BW", "BY",
    "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN",
    "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM",
    "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK",
    "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL",
    "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM",
    "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR",
    "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN",
    "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR", "LS",
    "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK",
    "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW",
    "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP",
    "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM",
    "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW",
    "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM",
    "SN", "SO", "SR", "SS", "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF",
    "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR", "TT", "TV", "TW",
    "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
    "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
)

WORLD_COUNTRY_CODE_SET: frozenset[str] = frozenset(WORLD_COUNTRY_CODE_POOL)


# Country -> IANA timezone candidates.
#
# This is self-contained instead of being read from system tzdata. Countries
# with multiple civil timezones keep multiple candidates so a later fingerprint
# step can choose a realistic regional value. Some ISO codes are uninhabited or
# have no direct IANA zone; those are assigned conservative representative zones
# only so the 249-code country pool remains total.
COUNTRY_TIMEZONES: dict[str, tuple[str, ...]] = {
    "AD": ("Europe/Andorra",),
    "AE": ("Asia/Dubai",),
    "AF": ("Asia/Kabul",),
    "AG": ("America/Antigua",),
    "AI": ("America/Anguilla",),
    "AL": ("Europe/Tirane",),
    "AM": ("Asia/Yerevan",),
    "AO": ("Africa/Luanda",),
    "AQ": (
        "Antarctica/McMurdo", "Antarctica/Casey", "Antarctica/Davis",
        "Antarctica/DumontDUrville", "Antarctica/Mawson", "Antarctica/Palmer",
        "Antarctica/Rothera", "Antarctica/Syowa", "Antarctica/Troll",
        "Antarctica/Vostok",
    ),
    "AR": (
        "America/Argentina/Buenos_Aires", "America/Argentina/Cordoba",
        "America/Argentina/Salta", "America/Argentina/Jujuy",
        "America/Argentina/Tucuman", "America/Argentina/Catamarca",
        "America/Argentina/La_Rioja", "America/Argentina/San_Juan",
        "America/Argentina/Mendoza", "America/Argentina/San_Luis",
        "America/Argentina/Rio_Gallegos", "America/Argentina/Ushuaia",
    ),
    "AS": ("Pacific/Pago_Pago",),
    "AT": ("Europe/Vienna",),
    "AU": (
        "Australia/Lord_Howe", "Antarctica/Macquarie", "Australia/Hobart",
        "Australia/Melbourne", "Australia/Sydney", "Australia/Broken_Hill",
        "Australia/Brisbane", "Australia/Lindeman", "Australia/Adelaide",
        "Australia/Darwin", "Australia/Perth", "Australia/Eucla",
    ),
    "AW": ("America/Aruba",),
    "AX": ("Europe/Mariehamn",),
    "AZ": ("Asia/Baku",),
    "BA": ("Europe/Sarajevo",),
    "BB": ("America/Barbados",),
    "BD": ("Asia/Dhaka",),
    "BE": ("Europe/Brussels",),
    "BF": ("Africa/Ouagadougou",),
    "BG": ("Europe/Sofia",),
    "BH": ("Asia/Bahrain",),
    "BI": ("Africa/Bujumbura",),
    "BJ": ("Africa/Porto-Novo",),
    "BL": ("America/St_Barthelemy",),
    "BM": ("Atlantic/Bermuda",),
    "BN": ("Asia/Brunei",),
    "BO": ("America/La_Paz",),
    "BQ": ("America/Kralendijk",),
    "BR": (
        "America/Noronha", "America/Belem", "America/Fortaleza",
        "America/Recife", "America/Araguaina", "America/Maceio",
        "America/Bahia", "America/Sao_Paulo", "America/Campo_Grande",
        "America/Cuiaba", "America/Santarem", "America/Porto_Velho",
        "America/Boa_Vista", "America/Manaus", "America/Eirunepe",
        "America/Rio_Branco",
    ),
    "BS": ("America/Nassau",),
    "BT": ("Asia/Thimphu",),
    "BV": ("Etc/UTC",),
    "BW": ("Africa/Gaborone",),
    "BY": ("Europe/Minsk",),
    "BZ": ("America/Belize",),
    "CA": (
        "America/St_Johns", "America/Halifax", "America/Glace_Bay",
        "America/Moncton", "America/Goose_Bay", "America/Blanc-Sablon",
        "America/Toronto", "America/Iqaluit", "America/Atikokan",
        "America/Winnipeg", "America/Resolute", "America/Rankin_Inlet",
        "America/Regina", "America/Swift_Current", "America/Edmonton",
        "America/Cambridge_Bay", "America/Inuvik", "America/Vancouver",
        "America/Creston", "America/Dawson_Creek", "America/Fort_Nelson",
        "America/Whitehorse", "America/Dawson",
    ),
    "CC": ("Indian/Cocos",),
    "CD": ("Africa/Kinshasa", "Africa/Lubumbashi"),
    "CF": ("Africa/Bangui",),
    "CG": ("Africa/Brazzaville",),
    "CH": ("Europe/Zurich",),
    "CI": ("Africa/Abidjan",),
    "CK": ("Pacific/Rarotonga",),
    "CL": ("America/Santiago", "America/Coyhaique", "America/Punta_Arenas", "Pacific/Easter"),
    "CM": ("Africa/Douala",),
    "CN": ("Asia/Shanghai", "Asia/Urumqi"),
    "CO": ("America/Bogota",),
    "CR": ("America/Costa_Rica",),
    "CU": ("America/Havana",),
    "CV": ("Atlantic/Cape_Verde",),
    "CW": ("America/Curacao",),
    "CX": ("Indian/Christmas",),
    "CY": ("Asia/Nicosia", "Asia/Famagusta"),
    "CZ": ("Europe/Prague",),
    "DE": ("Europe/Berlin", "Europe/Busingen"),
    "DJ": ("Africa/Djibouti",),
    "DK": ("Europe/Copenhagen",),
    "DM": ("America/Dominica",),
    "DO": ("America/Santo_Domingo",),
    "DZ": ("Africa/Algiers",),
    "EC": ("America/Guayaquil", "Pacific/Galapagos"),
    "EE": ("Europe/Tallinn",),
    "EG": ("Africa/Cairo",),
    "EH": ("Africa/El_Aaiun",),
    "ER": ("Africa/Asmara",),
    "ES": ("Europe/Madrid", "Africa/Ceuta", "Atlantic/Canary"),
    "ET": ("Africa/Addis_Ababa",),
    "FI": ("Europe/Helsinki",),
    "FJ": ("Pacific/Fiji",),
    "FK": ("Atlantic/Stanley",),
    "FM": ("Pacific/Chuuk", "Pacific/Pohnpei", "Pacific/Kosrae"),
    "FO": ("Atlantic/Faroe",),
    "FR": ("Europe/Paris",),
    "GA": ("Africa/Libreville",),
    "GB": ("Europe/London",),
    "GD": ("America/Grenada",),
    "GE": ("Asia/Tbilisi",),
    "GF": ("America/Cayenne",),
    "GG": ("Europe/Guernsey",),
    "GH": ("Africa/Accra",),
    "GI": ("Europe/Gibraltar",),
    "GL": ("America/Nuuk", "America/Danmarkshavn", "America/Scoresbysund", "America/Thule"),
    "GM": ("Africa/Banjul",),
    "GN": ("Africa/Conakry",),
    "GP": ("America/Guadeloupe",),
    "GQ": ("Africa/Malabo",),
    "GR": ("Europe/Athens",),
    "GS": ("Atlantic/South_Georgia",),
    "GT": ("America/Guatemala",),
    "GU": ("Pacific/Guam",),
    "GW": ("Africa/Bissau",),
    "GY": ("America/Guyana",),
    "HK": ("Asia/Hong_Kong",),
    "HM": ("Indian/Kerguelen",),
    "HN": ("America/Tegucigalpa",),
    "HR": ("Europe/Zagreb",),
    "HT": ("America/Port-au-Prince",),
    "HU": ("Europe/Budapest",),
    "ID": ("Asia/Jakarta", "Asia/Pontianak", "Asia/Makassar", "Asia/Jayapura"),
    "IE": ("Europe/Dublin",),
    "IL": ("Asia/Jerusalem",),
    "IM": ("Europe/Isle_of_Man",),
    "IN": ("Asia/Kolkata",),
    "IO": ("Indian/Chagos",),
    "IQ": ("Asia/Baghdad",),
    "IR": ("Asia/Tehran",),
    "IS": ("Atlantic/Reykjavik",),
    "IT": ("Europe/Rome",),
    "JE": ("Europe/Jersey",),
    "JM": ("America/Jamaica",),
    "JO": ("Asia/Amman",),
    "JP": ("Asia/Tokyo",),
    "KE": ("Africa/Nairobi",),
    "KG": ("Asia/Bishkek",),
    "KH": ("Asia/Phnom_Penh",),
    "KI": ("Pacific/Tarawa", "Pacific/Kanton", "Pacific/Kiritimati"),
    "KM": ("Indian/Comoro",),
    "KN": ("America/St_Kitts",),
    "KP": ("Asia/Pyongyang",),
    "KR": ("Asia/Seoul",),
    "KW": ("Asia/Kuwait",),
    "KY": ("America/Cayman",),
    "KZ": (
        "Asia/Almaty", "Asia/Qyzylorda", "Asia/Qostanay", "Asia/Aqtobe",
        "Asia/Aqtau", "Asia/Atyrau", "Asia/Oral",
    ),
    "LA": ("Asia/Vientiane",),
    "LB": ("Asia/Beirut",),
    "LC": ("America/St_Lucia",),
    "LI": ("Europe/Vaduz",),
    "LK": ("Asia/Colombo",),
    "LR": ("Africa/Monrovia",),
    "LS": ("Africa/Maseru",),
    "LT": ("Europe/Vilnius",),
    "LU": ("Europe/Luxembourg",),
    "LV": ("Europe/Riga",),
    "LY": ("Africa/Tripoli",),
    "MA": ("Africa/Casablanca",),
    "MC": ("Europe/Monaco",),
    "MD": ("Europe/Chisinau",),
    "ME": ("Europe/Podgorica",),
    "MF": ("America/Marigot",),
    "MG": ("Indian/Antananarivo",),
    "MH": ("Pacific/Majuro", "Pacific/Kwajalein"),
    "MK": ("Europe/Skopje",),
    "ML": ("Africa/Bamako",),
    "MM": ("Asia/Yangon",),
    "MN": ("Asia/Ulaanbaatar", "Asia/Hovd"),
    "MO": ("Asia/Macau",),
    "MP": ("Pacific/Saipan",),
    "MQ": ("America/Martinique",),
    "MR": ("Africa/Nouakchott",),
    "MS": ("America/Montserrat",),
    "MT": ("Europe/Malta",),
    "MU": ("Indian/Mauritius",),
    "MV": ("Indian/Maldives",),
    "MW": ("Africa/Blantyre",),
    "MX": (
        "America/Mexico_City", "America/Cancun", "America/Merida",
        "America/Monterrey", "America/Matamoros", "America/Chihuahua",
        "America/Ciudad_Juarez", "America/Ojinaga", "America/Mazatlan",
        "America/Bahia_Banderas", "America/Hermosillo", "America/Tijuana",
    ),
    "MY": ("Asia/Kuala_Lumpur", "Asia/Kuching"),
    "MZ": ("Africa/Maputo",),
    "NA": ("Africa/Windhoek",),
    "NC": ("Pacific/Noumea",),
    "NE": ("Africa/Niamey",),
    "NF": ("Pacific/Norfolk",),
    "NG": ("Africa/Lagos",),
    "NI": ("America/Managua",),
    "NL": ("Europe/Amsterdam",),
    "NO": ("Europe/Oslo",),
    "NP": ("Asia/Kathmandu",),
    "NR": ("Pacific/Nauru",),
    "NU": ("Pacific/Niue",),
    "NZ": ("Pacific/Auckland", "Pacific/Chatham"),
    "OM": ("Asia/Muscat",),
    "PA": ("America/Panama",),
    "PE": ("America/Lima",),
    "PF": ("Pacific/Tahiti", "Pacific/Marquesas", "Pacific/Gambier"),
    "PG": ("Pacific/Port_Moresby", "Pacific/Bougainville"),
    "PH": ("Asia/Manila",),
    "PK": ("Asia/Karachi",),
    "PL": ("Europe/Warsaw",),
    "PM": ("America/Miquelon",),
    "PN": ("Pacific/Pitcairn",),
    "PR": ("America/Puerto_Rico",),
    "PS": ("Asia/Gaza", "Asia/Hebron"),
    "PT": ("Europe/Lisbon", "Atlantic/Madeira", "Atlantic/Azores"),
    "PW": ("Pacific/Palau",),
    "PY": ("America/Asuncion",),
    "QA": ("Asia/Qatar",),
    "RE": ("Indian/Reunion",),
    "RO": ("Europe/Bucharest",),
    "RS": ("Europe/Belgrade",),
    "RU": (
        "Europe/Kaliningrad", "Europe/Moscow", "Europe/Simferopol",
        "Europe/Kirov", "Europe/Volgograd", "Europe/Astrakhan",
        "Europe/Saratov", "Europe/Ulyanovsk", "Europe/Samara",
        "Asia/Yekaterinburg", "Asia/Omsk", "Asia/Novosibirsk",
        "Asia/Barnaul", "Asia/Tomsk", "Asia/Novokuznetsk",
        "Asia/Krasnoyarsk", "Asia/Irkutsk", "Asia/Chita",
        "Asia/Yakutsk", "Asia/Khandyga", "Asia/Vladivostok",
        "Asia/Ust-Nera", "Asia/Magadan", "Asia/Sakhalin",
        "Asia/Srednekolymsk", "Asia/Kamchatka", "Asia/Anadyr",
    ),
    "RW": ("Africa/Kigali",),
    "SA": ("Asia/Riyadh",),
    "SB": ("Pacific/Guadalcanal",),
    "SC": ("Indian/Mahe",),
    "SD": ("Africa/Khartoum",),
    "SE": ("Europe/Stockholm",),
    "SG": ("Asia/Singapore",),
    "SH": ("Atlantic/St_Helena",),
    "SI": ("Europe/Ljubljana",),
    "SJ": ("Arctic/Longyearbyen",),
    "SK": ("Europe/Bratislava",),
    "SL": ("Africa/Freetown",),
    "SM": ("Europe/San_Marino",),
    "SN": ("Africa/Dakar",),
    "SO": ("Africa/Mogadishu",),
    "SR": ("America/Paramaribo",),
    "SS": ("Africa/Juba",),
    "ST": ("Africa/Sao_Tome",),
    "SV": ("America/El_Salvador",),
    "SX": ("America/Lower_Princes",),
    "SY": ("Asia/Damascus",),
    "SZ": ("Africa/Mbabane",),
    "TC": ("America/Grand_Turk",),
    "TD": ("Africa/Ndjamena",),
    "TF": ("Indian/Kerguelen",),
    "TG": ("Africa/Lome",),
    "TH": ("Asia/Bangkok",),
    "TJ": ("Asia/Dushanbe",),
    "TK": ("Pacific/Fakaofo",),
    "TL": ("Asia/Dili",),
    "TM": ("Asia/Ashgabat",),
    "TN": ("Africa/Tunis",),
    "TO": ("Pacific/Tongatapu",),
    "TR": ("Europe/Istanbul",),
    "TT": ("America/Port_of_Spain",),
    "TV": ("Pacific/Funafuti",),
    "TW": ("Asia/Taipei",),
    "TZ": ("Africa/Dar_es_Salaam",),
    "UA": ("Europe/Kyiv", "Europe/Simferopol"),
    "UG": ("Africa/Kampala",),
    "UM": ("Pacific/Midway", "Pacific/Wake"),
    "US": (
        "America/New_York", "America/Detroit", "America/Kentucky/Louisville",
        "America/Kentucky/Monticello", "America/Indiana/Indianapolis",
        "America/Indiana/Vincennes", "America/Indiana/Winamac",
        "America/Indiana/Marengo", "America/Indiana/Petersburg",
        "America/Indiana/Vevay", "America/Chicago", "America/Indiana/Tell_City",
        "America/Indiana/Knox", "America/Menominee",
        "America/North_Dakota/Center", "America/North_Dakota/New_Salem",
        "America/North_Dakota/Beulah", "America/Denver", "America/Boise",
        "America/Phoenix", "America/Los_Angeles", "America/Anchorage",
        "America/Juneau", "America/Sitka", "America/Metlakatla",
        "America/Yakutat", "America/Nome", "America/Adak", "Pacific/Honolulu",
    ),
    "UY": ("America/Montevideo",),
    "UZ": ("Asia/Samarkand", "Asia/Tashkent"),
    "VA": ("Europe/Vatican",),
    "VC": ("America/St_Vincent",),
    "VE": ("America/Caracas",),
    "VG": ("America/Tortola",),
    "VI": ("America/St_Thomas",),
    "VN": ("Asia/Ho_Chi_Minh",),
    "VU": ("Pacific/Efate",),
    "WF": ("Pacific/Wallis",),
    "WS": ("Pacific/Apia",),
    "YE": ("Asia/Aden",),
    "YT": ("Indian/Mayotte",),
    "ZA": ("Africa/Johannesburg",),
    "ZM": ("Africa/Lusaka",),
    "ZW": ("Africa/Harare",),
}


# IANA backward/link aliases and legacy names.
#
# canonicalize_timezone() also accepts canonical zone names case-insensitively,
# so this table only needs real alternate names, historical links, and legacy
# family names such as US/Eastern or PRC.
IANA_TIMEZONE_LINK_ALIASES: dict[str, str] = {
    "Africa/Asmera": "Africa/Asmara",
    "Africa/Timbuktu": "Africa/Bamako",
    "America/Argentina/ComodRivadavia": "America/Argentina/Catamarca",
    "America/Atka": "America/Adak",
    "America/Buenos_Aires": "America/Argentina/Buenos_Aires",
    "America/Catamarca": "America/Argentina/Catamarca",
    "America/Coral_Harbour": "America/Atikokan",
    "America/Cordoba": "America/Argentina/Cordoba",
    "America/Ensenada": "America/Tijuana",
    "America/Fort_Wayne": "America/Indiana/Indianapolis",
    "America/Godthab": "America/Nuuk",
    "America/Indianapolis": "America/Indiana/Indianapolis",
    "America/Jujuy": "America/Argentina/Jujuy",
    "America/Knox_IN": "America/Indiana/Knox",
    "America/Louisville": "America/Kentucky/Louisville",
    "America/Mendoza": "America/Argentina/Mendoza",
    "America/Montreal": "America/Toronto",
    "America/Porto_Acre": "America/Rio_Branco",
    "America/Rosario": "America/Argentina/Cordoba",
    "America/Shiprock": "America/Denver",
    "America/Virgin": "America/St_Thomas",
    "Antarctica/South_Pole": "Antarctica/McMurdo",
    "Asia/Ashkhabad": "Asia/Ashgabat",
    "Asia/Calcutta": "Asia/Kolkata",
    "Asia/Chongqing": "Asia/Shanghai",
    "Asia/Chungking": "Asia/Shanghai",
    "Asia/Dacca": "Asia/Dhaka",
    "Asia/Harbin": "Asia/Shanghai",
    "Asia/Istanbul": "Europe/Istanbul",
    "Asia/Kashgar": "Asia/Urumqi",
    "Asia/Katmandu": "Asia/Kathmandu",
    "Asia/Macao": "Asia/Macau",
    "Asia/Rangoon": "Asia/Yangon",
    "Asia/Saigon": "Asia/Ho_Chi_Minh",
    "Asia/Tel_Aviv": "Asia/Jerusalem",
    "Asia/Thimbu": "Asia/Thimphu",
    "Asia/Ujung_Pandang": "Asia/Makassar",
    "Asia/Ulan_Bator": "Asia/Ulaanbaatar",
    "Atlantic/Faeroe": "Atlantic/Faroe",
    "Australia/ACT": "Australia/Sydney",
    "Australia/Canberra": "Australia/Sydney",
    "Australia/LHI": "Australia/Lord_Howe",
    "Australia/NSW": "Australia/Sydney",
    "Australia/North": "Australia/Darwin",
    "Australia/Queensland": "Australia/Brisbane",
    "Australia/South": "Australia/Adelaide",
    "Australia/Tasmania": "Australia/Hobart",
    "Australia/Victoria": "Australia/Melbourne",
    "Australia/West": "Australia/Perth",
    "Australia/Yancowinna": "Australia/Broken_Hill",
    "Brazil/Acre": "America/Rio_Branco",
    "Brazil/DeNoronha": "America/Noronha",
    "Brazil/East": "America/Sao_Paulo",
    "Brazil/West": "America/Manaus",
    "Canada/Atlantic": "America/Halifax",
    "Canada/Central": "America/Winnipeg",
    "Canada/Eastern": "America/Toronto",
    "Canada/Mountain": "America/Edmonton",
    "Canada/Newfoundland": "America/St_Johns",
    "Canada/Pacific": "America/Vancouver",
    "Canada/Saskatchewan": "America/Regina",
    "Canada/Yukon": "America/Whitehorse",
    "Chile/Continental": "America/Santiago",
    "Chile/EasterIsland": "Pacific/Easter",
    "Cuba": "America/Havana",
    "Egypt": "Africa/Cairo",
    "Eire": "Europe/Dublin",
    "Europe/Belfast": "Europe/London",
    "Europe/Kiev": "Europe/Kyiv",
    "Europe/Nicosia": "Asia/Nicosia",
    "Europe/Tiraspol": "Europe/Chisinau",
    "GB": "Europe/London",
    "GB-Eire": "Europe/London",
    "Hongkong": "Asia/Hong_Kong",
    "Iceland": "Atlantic/Reykjavik",
    "Iran": "Asia/Tehran",
    "Israel": "Asia/Jerusalem",
    "Jamaica": "America/Jamaica",
    "Japan": "Asia/Tokyo",
    "Kwajalein": "Pacific/Kwajalein",
    "Libya": "Africa/Tripoli",
    "Mexico/BajaNorte": "America/Tijuana",
    "Mexico/BajaSur": "America/Mazatlan",
    "Mexico/General": "America/Mexico_City",
    "NZ": "Pacific/Auckland",
    "NZ-CHAT": "Pacific/Chatham",
    "Navajo": "America/Denver",
    "PRC": "Asia/Shanghai",
    "Pacific/Johnston": "Pacific/Honolulu",
    "Pacific/Ponape": "Pacific/Pohnpei",
    "Pacific/Samoa": "Pacific/Pago_Pago",
    "Pacific/Truk": "Pacific/Chuuk",
    "Pacific/Yap": "Pacific/Chuuk",
    "Poland": "Europe/Warsaw",
    "Portugal": "Europe/Lisbon",
    "ROC": "Asia/Taipei",
    "ROK": "Asia/Seoul",
    "Singapore": "Asia/Singapore",
    "Turkey": "Europe/Istanbul",
    "US/Alaska": "America/Anchorage",
    "US/Aleutian": "America/Adak",
    "US/Arizona": "America/Phoenix",
    "US/Central": "America/Chicago",
    "US/East-Indiana": "America/Indiana/Indianapolis",
    "US/Eastern": "America/New_York",
    "US/Hawaii": "Pacific/Honolulu",
    "US/Indiana-Starke": "America/Indiana/Knox",
    "US/Michigan": "America/Detroit",
    "US/Mountain": "America/Denver",
    "US/Pacific": "America/Los_Angeles",
    "US/Samoa": "Pacific/Pago_Pago",
    "UTC": "Etc/UTC",
    "Universal": "Etc/UTC",
    "Zulu": "Etc/UTC",
}


# Fixed-offset spellings do not always identify a unique civil timezone. This
# table chooses a representative IANA timezone for each common offset so later
# environment code can still store browser-style timezone names. For +08:00 the
# user specifically expects GMT+8/UTC+08:00/Etc/GMT-8 style values to resolve
# consistently with Asia/Shanghai.
OFFSET_TO_REPRESENTATIVE_TIMEZONE: dict[str, str] = {
    "-12:00": "Etc/GMT+12",
    "-11:00": "Pacific/Pago_Pago",
    "-10:00": "Pacific/Honolulu",
    "-09:30": "Pacific/Marquesas",
    "-09:00": "America/Anchorage",
    "-08:00": "America/Los_Angeles",
    "-07:00": "America/Denver",
    "-06:00": "America/Chicago",
    "-05:00": "America/New_York",
    "-04:00": "America/Puerto_Rico",
    "-03:30": "America/St_Johns",
    "-03:00": "America/Sao_Paulo",
    "-02:00": "America/Noronha",
    "-01:00": "Atlantic/Azores",
    "+00:00": "Etc/UTC",
    "+01:00": "Europe/Paris",
    "+02:00": "Europe/Athens",
    "+03:00": "Europe/Moscow",
    "+03:30": "Asia/Tehran",
    "+04:00": "Asia/Dubai",
    "+04:30": "Asia/Kabul",
    "+05:00": "Asia/Karachi",
    "+05:30": "Asia/Kolkata",
    "+05:45": "Asia/Kathmandu",
    "+06:00": "Asia/Dhaka",
    "+06:30": "Asia/Yangon",
    "+07:00": "Asia/Bangkok",
    "+08:00": "Asia/Shanghai",
    "+08:45": "Australia/Eucla",
    "+09:00": "Asia/Tokyo",
    "+09:30": "Australia/Adelaide",
    "+10:00": "Australia/Sydney",
    "+10:30": "Australia/Lord_Howe",
    "+11:00": "Pacific/Noumea",
    "+12:00": "Pacific/Auckland",
    "+12:45": "Pacific/Chatham",
    "+13:00": "Pacific/Apia",
    "+14:00": "Pacific/Kiritimati",
}

_OFFSET_ALIAS_PATTERN = re.compile(r"^(?:GMT|UTC)\s*([+-])\s*(\d{1,2})(?::?(\d{2}))?$", re.I)


# Country -> browser language candidates.
#
# "primary" is the first language that should usually appear in
# navigator.language. "secondary" is an ordered pool for later
# navigator.languages permutation/combination sampling. Do not treat secondary
# order as the final browser order; build_language_lists() deliberately creates
# ordered variants from that pool.
COUNTRY_LANGUAGE_PROFILES: dict[str, dict[str, tuple[str, ...] | str]] = {
    "AD": {"primary": "ca-AD", "secondary": ("es-ES", "fr-FR", "pt-PT", "en-US")},
    "AE": {"primary": "ar-AE", "secondary": ("en-AE",)},
    "AF": {"primary": "fa-AF", "secondary": ("ps-AF", "uz-AF", "en-US")},
    "AG": {"primary": "en-AG", "secondary": ("en-US",)},
    "AI": {"primary": "en-AI", "secondary": ("en-US",)},
    "AL": {"primary": "sq-AL", "secondary": ("en-US", "it-IT", "el-GR")},
    "AM": {"primary": "hy-AM", "secondary": ("ru-RU", "en-US")},
    "AO": {"primary": "pt-AO", "secondary": ("en-US", "fr-FR")},
    "AQ": {"primary": "en-US", "secondary": ("en-NZ", "en-AU", "fr-FR", "ru-RU")},
    "AR": {"primary": "es-AR", "secondary": ("es-419", "en-US")},
    "AS": {"primary": "en-AS", "secondary": ("sm-WS", "en-US")},
    "AT": {"primary": "de-AT", "secondary": ("de-DE", "en-US")},
    "AU": {"primary": "en-AU", "secondary": ("en-GB", "en-US")},
    "AW": {"primary": "nl-AW", "secondary": ("pap-AW", "es-419", "en-US")},
    "AX": {"primary": "sv-AX", "secondary": ("sv-SE", "fi-FI", "en-US")},
    "AZ": {"primary": "az-AZ", "secondary": ("ru-RU", "tr-TR", "en-US")},
    "BA": {"primary": "bs-BA", "secondary": ("hr-BA", "sr-BA", "en-US")},
    "BB": {"primary": "en-BB", "secondary": ("en-US",)},
    "BD": {"primary": "bn-BD", "secondary": ("en-US",)},
    "BE": {"primary": "nl-BE", "secondary": ("fr-BE", "de-BE", "en-US")},
    "BF": {"primary": "fr-BF", "secondary": ("mos-BF", "en-US")},
    "BG": {"primary": "bg-BG", "secondary": ("en-US", "tr-TR")},
    "BH": {"primary": "ar-BH", "secondary": ("en-US", "hi-IN", "ur-PK")},
    "BI": {"primary": "rn-BI", "secondary": ("fr-BI", "en-US")},
    "BJ": {"primary": "fr-BJ", "secondary": ("yo-BJ", "en-US")},
    "BL": {"primary": "fr-BL", "secondary": ("en-US",)},
    "BM": {"primary": "en-BM", "secondary": ("en-US",)},
    "BN": {"primary": "ms-BN", "secondary": ("en-US", "zh-CN")},
    "BO": {"primary": "es-BO", "secondary": ("qu-BO", "ay-BO", "en-US")},
    "BQ": {"primary": "nl-BQ", "secondary": ("pap-BQ", "en-US", "es-419")},
    "BR": {"primary": "pt-BR", "secondary": ("en-US", "es-419")},
    "BS": {"primary": "en-BS", "secondary": ("en-US",)},
    "BT": {"primary": "dz-BT", "secondary": ("en-US", "ne-NP")},
    "BV": {"primary": "no-NO", "secondary": ("nb-NO", "en-US")},
    "BW": {"primary": "en-BW", "secondary": ("tn-BW",)},
    "BY": {"primary": "be-BY", "secondary": ("ru-RU", "en-US")},
    "BZ": {"primary": "en-BZ", "secondary": ("es-BZ", "en-US")},
    "CA": {"primary": "en-CA", "secondary": ("fr-CA", "en-US")},
    "CC": {"primary": "en-CC", "secondary": ("ms-MY",)},
    "CD": {"primary": "fr-CD", "secondary": ("ln-CD", "sw-CD", "kg-CD", "lua-CD", "en-US")},
    "CF": {"primary": "fr-CF", "secondary": ("sg-CF", "en-US")},
    "CG": {"primary": "fr-CG", "secondary": ("ln-CG", "kg-CG", "en-US")},
    "CH": {"primary": "de-CH", "secondary": ("fr-CH", "it-CH", "rm-CH", "en-US")},
    "CI": {"primary": "fr-CI", "secondary": ("en-US",)},
    "CK": {"primary": "en-CK", "secondary": ("mi-NZ",)},
    "CL": {"primary": "es-CL", "secondary": ("es-419", "en-US")},
    "CM": {"primary": "fr-CM", "secondary": ("en-CM", "en-US")},
    "CN": {"primary": "zh-CN", "secondary": ("zh-Hans-CN", "en-US")},
    "CO": {"primary": "es-CO", "secondary": ("es-419", "en-US")},
    "CR": {"primary": "es-CR", "secondary": ("es-419", "en-US")},
    "CU": {"primary": "es-CU", "secondary": ("es-419", "en-US")},
    "CV": {"primary": "pt-CV", "secondary": ("kea-CV", "en-US")},
    "CW": {"primary": "nl-CW", "secondary": ("pap-CW", "en-US", "es-419")},
    "CX": {"primary": "en-CX", "secondary": ("zh-CN", "ms-MY")},
    "CY": {"primary": "el-CY", "secondary": ("tr-CY", "en-US")},
    "CZ": {"primary": "cs-CZ", "secondary": ("en-US", "sk-SK")},
    "DE": {"primary": "de-DE", "secondary": ("en-US",)},
    "DJ": {"primary": "fr-DJ", "secondary": ("ar-DJ", "so-DJ", "en-US")},
    "DK": {"primary": "da-DK", "secondary": ("en-US", "de-DE")},
    "DM": {"primary": "en-DM", "secondary": ("en-US",)},
    "DO": {"primary": "es-DO", "secondary": ("es-419", "en-US")},
    "DZ": {"primary": "ar-DZ", "secondary": ("fr-DZ", "ber-DZ", "en-US")},
    "EC": {"primary": "es-EC", "secondary": ("es-419", "qu-EC", "en-US")},
    "EE": {"primary": "et-EE", "secondary": ("ru-RU", "en-US")},
    "EG": {"primary": "ar-EG", "secondary": ("en-US", "fr-FR")},
    "EH": {"primary": "ar-EH", "secondary": ("es-ES", "fr-FR")},
    "ER": {"primary": "ti-ER", "secondary": ("ar-ER", "en-US")},
    "ES": {"primary": "es-ES", "secondary": ("ca-ES", "gl-ES", "eu-ES", "en-US")},
    "ET": {"primary": "am-ET", "secondary": ("om-ET", "ti-ET", "en-US")},
    "FI": {"primary": "fi-FI", "secondary": ("sv-FI", "en-US")},
    "FJ": {"primary": "en-FJ", "secondary": ("fj-FJ", "hi-FJ")},
    "FK": {"primary": "en-FK", "secondary": ("en-GB",)},
    "FM": {"primary": "en-FM", "secondary": ("chk-FM", "pon-FM", "kos-FM")},
    "FO": {"primary": "fo-FO", "secondary": ("da-DK", "en-US")},
    "FR": {"primary": "fr-FR", "secondary": ("en-US",)},
    "GA": {"primary": "fr-GA", "secondary": ("en-US",)},
    "GB": {"primary": "en-GB", "secondary": ("en-US", "cy-GB", "gd-GB")},
    "GD": {"primary": "en-GD", "secondary": ("en-US",)},
    "GE": {"primary": "ka-GE", "secondary": ("ru-RU", "en-US")},
    "GF": {"primary": "fr-GF", "secondary": ("en-US",)},
    "GG": {"primary": "en-GG", "secondary": ("fr-FR",)},
    "GH": {"primary": "en-GH", "secondary": ("ak-GH", "ee-GH")},
    "GI": {"primary": "en-GI", "secondary": ("es-ES",)},
    "GL": {"primary": "kl-GL", "secondary": ("da-DK", "en-US")},
    "GM": {"primary": "en-GM", "secondary": ("wo-SN", "ff-GM")},
    "GN": {"primary": "fr-GN", "secondary": ("ff-GN", "en-US")},
    "GP": {"primary": "fr-GP", "secondary": ("en-US",)},
    "GQ": {"primary": "es-GQ", "secondary": ("fr-GQ", "pt-GQ", "en-US")},
    "GR": {"primary": "el-GR", "secondary": ("en-US",)},
    "GS": {"primary": "en-GS", "secondary": ("en-GB",)},
    "GT": {"primary": "es-GT", "secondary": ("es-419", "en-US")},
    "GU": {"primary": "en-GU", "secondary": ("ch-GU", "en-US")},
    "GW": {"primary": "pt-GW", "secondary": ("en-US",)},
    "GY": {"primary": "en-GY", "secondary": ("en-US",)},
    "HK": {"primary": "zh-HK", "secondary": ("zh-Hant-HK", "en-HK", "zh-CN")},
    "HM": {"primary": "en-AU", "secondary": ("en-US",)},
    "HN": {"primary": "es-HN", "secondary": ("es-419", "en-US")},
    "HR": {"primary": "hr-HR", "secondary": ("en-US", "sr-RS")},
    "HT": {"primary": "ht-HT", "secondary": ("fr-HT", "en-US")},
    "HU": {"primary": "hu-HU", "secondary": ("en-US", "de-DE")},
    "ID": {"primary": "id-ID", "secondary": ("jv-ID", "su-ID", "en-US")},
    "IE": {"primary": "en-IE", "secondary": ("ga-IE", "en-GB")},
    "IL": {"primary": "he-IL", "secondary": ("ar-IL", "en-US", "ru-RU")},
    "IM": {"primary": "en-IM", "secondary": ("gv-IM",)},
    "IN": {"primary": "hi-IN", "secondary": ("en-IN", "bn-IN", "ta-IN", "te-IN", "mr-IN", "gu-IN")},
    "IO": {"primary": "en-IO", "secondary": ("en-GB",)},
    "IQ": {"primary": "ar-IQ", "secondary": ("ku-IQ", "en-US")},
    "IR": {"primary": "fa-IR", "secondary": ("az-IR", "ku-IR", "en-US")},
    "IS": {"primary": "is-IS", "secondary": ("en-US", "da-DK")},
    "IT": {"primary": "it-IT", "secondary": ("en-US", "de-DE", "fr-FR")},
    "JE": {"primary": "en-JE", "secondary": ("fr-FR",)},
    "JM": {"primary": "en-JM", "secondary": ("en-US",)},
    "JO": {"primary": "ar-JO", "secondary": ("en-US",)},
    "JP": {"primary": "ja-JP", "secondary": ("en-US",)},
    "KE": {"primary": "sw-KE", "secondary": ("en-KE",)},
    "KG": {"primary": "ky-KG", "secondary": ("ru-RU", "en-US")},
    "KH": {"primary": "km-KH", "secondary": ("en-US", "fr-FR")},
    "KI": {"primary": "en-KI", "secondary": ("gil-KI",)},
    "KM": {"primary": "ar-KM", "secondary": ("fr-KM", "sw-KM")},
    "KN": {"primary": "en-KN", "secondary": ("en-US",)},
    "KP": {"primary": "ko-KP", "secondary": ("en-US",)},
    "KR": {"primary": "ko-KR", "secondary": ("en-US",)},
    "KW": {"primary": "ar-KW", "secondary": ("en-US", "hi-IN")},
    "KY": {"primary": "en-KY", "secondary": ("en-US",)},
    "KZ": {"primary": "kk-KZ", "secondary": ("ru-RU", "en-US")},
    "LA": {"primary": "lo-LA", "secondary": ("en-US", "fr-FR")},
    "LB": {"primary": "ar-LB", "secondary": ("fr-LB", "en-US")},
    "LC": {"primary": "en-LC", "secondary": ("en-US",)},
    "LI": {"primary": "de-LI", "secondary": ("de-CH", "en-US")},
    "LK": {"primary": "si-LK", "secondary": ("ta-LK", "en-US")},
    "LR": {"primary": "en-LR", "secondary": ("en-US",)},
    "LS": {"primary": "st-LS", "secondary": ("en-LS",)},
    "LT": {"primary": "lt-LT", "secondary": ("en-US", "ru-RU", "pl-PL")},
    "LU": {"primary": "lb-LU", "secondary": ("fr-LU", "de-LU", "en-US")},
    "LV": {"primary": "lv-LV", "secondary": ("ru-RU", "en-US")},
    "LY": {"primary": "ar-LY", "secondary": ("en-US", "it-IT")},
    "MA": {"primary": "ar-MA", "secondary": ("fr-MA", "ber-MA", "en-US")},
    "MC": {"primary": "fr-MC", "secondary": ("it-IT", "en-US")},
    "MD": {"primary": "ro-MD", "secondary": ("ru-RU", "uk-UA", "en-US")},
    "ME": {"primary": "sr-ME", "secondary": ("cnr-ME", "bs-BA", "sq-AL", "en-US")},
    "MF": {"primary": "fr-MF", "secondary": ("en-US",)},
    "MG": {"primary": "mg-MG", "secondary": ("fr-MG", "en-US")},
    "MH": {"primary": "en-MH", "secondary": ("mh-MH",)},
    "MK": {"primary": "mk-MK", "secondary": ("sq-MK", "en-US")},
    "ML": {"primary": "fr-ML", "secondary": ("bm-ML", "en-US")},
    "MM": {"primary": "my-MM", "secondary": ("en-US",)},
    "MN": {"primary": "mn-MN", "secondary": ("en-US", "ru-RU")},
    "MO": {"primary": "zh-MO", "secondary": ("zh-Hant-MO", "pt-MO", "en-US")},
    "MP": {"primary": "en-MP", "secondary": ("ch-MP", "en-US")},
    "MQ": {"primary": "fr-MQ", "secondary": ("en-US",)},
    "MR": {"primary": "ar-MR", "secondary": ("fr-MR", "en-US")},
    "MS": {"primary": "en-MS", "secondary": ("en-US",)},
    "MT": {"primary": "mt-MT", "secondary": ("en-MT", "it-IT")},
    "MU": {"primary": "en-MU", "secondary": ("fr-MU", "mfe-MU")},
    "MV": {"primary": "dv-MV", "secondary": ("en-US",)},
    "MW": {"primary": "en-MW", "secondary": ("ny-MW",)},
    "MX": {"primary": "es-MX", "secondary": ("es-419", "en-US")},
    "MY": {"primary": "ms-MY", "secondary": ("en-MY", "zh-MY", "ta-MY")},
    "MZ": {"primary": "pt-MZ", "secondary": ("en-US",)},
    "NA": {"primary": "en-NA", "secondary": ("af-NA", "de-NA")},
    "NC": {"primary": "fr-NC", "secondary": ("en-US",)},
    "NE": {"primary": "fr-NE", "secondary": ("ha-NE", "en-US")},
    "NF": {"primary": "en-NF", "secondary": ("en-AU",)},
    "NG": {"primary": "en-NG", "secondary": ("ha-NG", "yo-NG", "ig-NG")},
    "NI": {"primary": "es-NI", "secondary": ("es-419", "en-US")},
    "NL": {"primary": "nl-NL", "secondary": ("en-US", "de-DE")},
    "NO": {"primary": "nb-NO", "secondary": ("nn-NO", "se-NO", "en-US")},
    "NP": {"primary": "ne-NP", "secondary": ("en-US", "hi-IN")},
    "NR": {"primary": "en-NR", "secondary": ("na-NR",)},
    "NU": {"primary": "en-NU", "secondary": ("niu-NU",)},
    "NZ": {"primary": "en-NZ", "secondary": ("mi-NZ", "en-AU")},
    "OM": {"primary": "ar-OM", "secondary": ("en-US", "hi-IN")},
    "PA": {"primary": "es-PA", "secondary": ("es-419", "en-US")},
    "PE": {"primary": "es-PE", "secondary": ("qu-PE", "ay-PE", "en-US")},
    "PF": {"primary": "fr-PF", "secondary": ("ty-PF", "en-US")},
    "PG": {"primary": "en-PG", "secondary": ("tpi-PG", "ho-PG")},
    "PH": {"primary": "fil-PH", "secondary": ("en-PH", "ceb-PH", "ilo-PH")},
    "PK": {"primary": "ur-PK", "secondary": ("en-PK", "pa-PK", "sd-PK")},
    "PL": {"primary": "pl-PL", "secondary": ("en-US", "de-DE")},
    "PM": {"primary": "fr-PM", "secondary": ("en-US",)},
    "PN": {"primary": "en-PN", "secondary": ("en-GB",)},
    "PR": {"primary": "es-PR", "secondary": ("en-US",)},
    "PS": {"primary": "ar-PS", "secondary": ("en-US", "he-IL")},
    "PT": {"primary": "pt-PT", "secondary": ("en-US", "es-ES")},
    "PW": {"primary": "en-PW", "secondary": ("pau-PW", "ja-JP")},
    "PY": {"primary": "es-PY", "secondary": ("gn-PY", "en-US")},
    "QA": {"primary": "ar-QA", "secondary": ("en-US", "hi-IN")},
    "RE": {"primary": "fr-RE", "secondary": ("en-US",)},
    "RO": {"primary": "ro-RO", "secondary": ("hu-RO", "en-US")},
    "RS": {"primary": "sr-RS", "secondary": ("hu-RS", "en-US")},
    "RU": {"primary": "ru-RU", "secondary": ("en-US",)},
    "RW": {"primary": "rw-RW", "secondary": ("en-US", "fr-RW", "sw-RW")},
    "SA": {"primary": "ar-SA", "secondary": ("en-US", "ur-PK")},
    "SB": {"primary": "en-SB", "secondary": ("pis-SB",)},
    "SC": {"primary": "crs-SC", "secondary": ("en-SC", "fr-SC")},
    "SD": {"primary": "ar-SD", "secondary": ("en-US",)},
    "SE": {"primary": "sv-SE", "secondary": ("en-US", "fi-FI")},
    "SG": {"primary": "en-SG", "secondary": ("zh-SG", "ms-SG", "ta-SG")},
    "SH": {"primary": "en-SH", "secondary": ("en-GB",)},
    "SI": {"primary": "sl-SI", "secondary": ("en-US", "it-IT", "hu-HU")},
    "SJ": {"primary": "nb-SJ", "secondary": ("no-NO", "en-US")},
    "SK": {"primary": "sk-SK", "secondary": ("en-US", "hu-HU", "cs-CZ")},
    "SL": {"primary": "en-SL", "secondary": ("kri-SL",)},
    "SM": {"primary": "it-SM", "secondary": ("en-US",)},
    "SN": {"primary": "fr-SN", "secondary": ("wo-SN", "en-US")},
    "SO": {"primary": "so-SO", "secondary": ("ar-SO", "en-US")},
    "SR": {"primary": "nl-SR", "secondary": ("srn-SR", "en-US")},
    "SS": {"primary": "en-SS", "secondary": ("ar-SS",)},
    "ST": {"primary": "pt-ST", "secondary": ("en-US",)},
    "SV": {"primary": "es-SV", "secondary": ("es-419", "en-US")},
    "SX": {"primary": "nl-SX", "secondary": ("en-SX", "es-419")},
    "SY": {"primary": "ar-SY", "secondary": ("en-US", "fr-FR")},
    "SZ": {"primary": "en-SZ", "secondary": ("ss-SZ",)},
    "TC": {"primary": "en-TC", "secondary": ("en-US",)},
    "TD": {"primary": "fr-TD", "secondary": ("ar-TD", "en-US")},
    "TF": {"primary": "fr-TF", "secondary": ("en-US",)},
    "TG": {"primary": "fr-TG", "secondary": ("ee-TG", "en-US")},
    "TH": {"primary": "th-TH", "secondary": ("en-US",)},
    "TJ": {"primary": "tg-TJ", "secondary": ("ru-RU", "en-US")},
    "TK": {"primary": "en-TK", "secondary": ("tkl-TK",)},
    "TL": {"primary": "tet-TL", "secondary": ("pt-TL", "id-ID", "en-US")},
    "TM": {"primary": "tk-TM", "secondary": ("ru-RU", "en-US")},
    "TN": {"primary": "ar-TN", "secondary": ("fr-TN", "en-US")},
    "TO": {"primary": "to-TO", "secondary": ("en-TO",)},
    "TR": {"primary": "tr-TR", "secondary": ("en-US", "ku-TR")},
    "TT": {"primary": "en-TT", "secondary": ("en-US",)},
    "TV": {"primary": "tvl-TV", "secondary": ("en-TV",)},
    "TW": {"primary": "zh-TW", "secondary": ("zh-Hant-TW", "en-US")},
    "TZ": {"primary": "sw-TZ", "secondary": ("en-TZ",)},
    "UA": {"primary": "uk-UA", "secondary": ("ru-RU", "en-US")},
    "UG": {"primary": "en-UG", "secondary": ("sw-UG", "lg-UG")},
    "UM": {"primary": "en-US", "secondary": ("en-UM",)},
    "US": {
        "primary": "en-US",
        "secondary": ("es-US", "en-GB"),
    },
    "UY": {"primary": "es-UY", "secondary": ("es-419", "en-US")},
    "UZ": {"primary": "uz-UZ", "secondary": ("ru-RU", "en-US")},
    "VA": {"primary": "it-VA", "secondary": ("la-VA", "en-US")},
    "VC": {"primary": "en-VC", "secondary": ("en-US",)},
    "VE": {"primary": "es-VE", "secondary": ("es-419", "en-US")},
    "VG": {"primary": "en-VG", "secondary": ("en-US",)},
    "VI": {"primary": "en-VI", "secondary": ("en-US", "es-PR")},
    "VN": {"primary": "vi-VN", "secondary": ("en-US",)},
    "VU": {"primary": "bi-VU", "secondary": ("en-VU", "fr-VU")},
    "WF": {"primary": "fr-WF", "secondary": ("wls-WF", "fud-WF")},
    "WS": {"primary": "sm-WS", "secondary": ("en-WS",)},
    "YE": {"primary": "ar-YE", "secondary": ("en-US",)},
    "YT": {"primary": "fr-YT", "secondary": ("swb-YT", "en-US")},
    "ZA": {"primary": "en-ZA", "secondary": ("af-ZA", "zu-ZA", "xh-ZA", "st-ZA")},
    "ZM": {"primary": "en-ZM", "secondary": ("bem-ZM", "ny-ZM")},
    "ZW": {"primary": "en-ZW", "secondary": ("sn-ZW", "nd-ZW")},
}


# WebGL/WebGPU GPU candidates.
#
# The local D:\TEST_V8\webgl_gpu sample is a full environment-shaped JSON:
# root.gpu.adapter.* plus root.webgl.unmaskedVendor/unmaskedRenderer, followed
# by large WebGL/WebGL2 parameter tables. This constant keeps only the variable
# GPU identity fields in the same nested naming style. Later composition should
# deep-merge one selected entry into a copied webgl_gpu template instead of
# duplicating the huge parameter table for every GPU.
#
# Chrome on Windows usually exposes WebGL through ANGLE D3D11. The renderer
# strings below intentionally follow that shape:
# ANGLE (<driver vendor>, <GPU model> Direct3D11 vs_5_0 ps_5_0, D3D11)
WEBGL_GPU_CANDIDATES: tuple[dict[str, object], ...] = (
    {
        "id": "win_nvidia_rtx_5090_blackwell",
        "vendor": "nvidia",
        "tier": "enthusiast",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "blackwell", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 5090 (0x00002A01) Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_rtx_5080_blackwell",
        "vendor": "nvidia",
        "tier": "enthusiast",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "blackwell", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 5080 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_rtx_4090_ada",
        "vendor": "nvidia",
        "tier": "enthusiast",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "ada", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 4090 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_rtx_4080_super_ada",
        "vendor": "nvidia",
        "tier": "high",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "ada", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 4080 SUPER Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_rtx_4070_super_ada",
        "vendor": "nvidia",
        "tier": "high",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "ada", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 4070 SUPER Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_rtx_4060_ada",
        "vendor": "nvidia",
        "tier": "mainstream",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "ada", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 4060 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_rtx_3080_ampere",
        "vendor": "nvidia",
        "tier": "high",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "ampere", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_rtx_3060_ampere",
        "vendor": "nvidia",
        "tier": "mainstream",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "ampere", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_gtx_1660_super_turing",
        "vendor": "nvidia",
        "tier": "mainstream",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "turing", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce GTX 1660 SUPER Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_gtx_1650_turing",
        "vendor": "nvidia",
        "tier": "entry",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "turing", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce GTX 1650 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_nvidia_gtx_1060_pascal",
        "vendor": "nvidia",
        "tier": "entry",
        "gpu": {"adapter": {"vendor": "nvidia", "architecture": "pascal", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (NVIDIA)",
            "unmaskedRenderer": "ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 6GB Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_amd_rx_7900_xtx_rdna3",
        "vendor": "amd",
        "tier": "enthusiast",
        "gpu": {"adapter": {"vendor": "amd", "architecture": "rdna3", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (AMD)",
            "unmaskedRenderer": "ANGLE (AMD, AMD Radeon RX 7900 XTX Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_amd_rx_7800_xt_rdna3",
        "vendor": "amd",
        "tier": "high",
        "gpu": {"adapter": {"vendor": "amd", "architecture": "rdna3", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (AMD)",
            "unmaskedRenderer": "ANGLE (AMD, AMD Radeon RX 7800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_amd_rx_7600_rdna3",
        "vendor": "amd",
        "tier": "mainstream",
        "gpu": {"adapter": {"vendor": "amd", "architecture": "rdna3", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (AMD)",
            "unmaskedRenderer": "ANGLE (AMD, AMD Radeon RX 7600 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_amd_rx_6800_xt_rdna2",
        "vendor": "amd",
        "tier": "high",
        "gpu": {"adapter": {"vendor": "amd", "architecture": "rdna2", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (AMD)",
            "unmaskedRenderer": "ANGLE (AMD, AMD Radeon RX 6800 XT Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_amd_rx_6600_rdna2",
        "vendor": "amd",
        "tier": "mainstream",
        "gpu": {"adapter": {"vendor": "amd", "architecture": "rdna2", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (AMD)",
            "unmaskedRenderer": "ANGLE (AMD, AMD Radeon RX 6600 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_amd_radeon_780m_rdna3",
        "vendor": "amd",
        "tier": "integrated",
        "gpu": {"adapter": {"vendor": "amd", "architecture": "rdna3", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (AMD)",
            "unmaskedRenderer": "ANGLE (AMD, AMD Radeon 780M Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_intel_iris_xe",
        "vendor": "intel",
        "tier": "integrated",
        "gpu": {"adapter": {"vendor": "intel", "architecture": "gen-12lp", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (Intel)",
            "unmaskedRenderer": "ANGLE (Intel, Intel(R) Iris(R) Xe Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_intel_uhd_770",
        "vendor": "intel",
        "tier": "integrated",
        "gpu": {"adapter": {"vendor": "intel", "architecture": "gen-12", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (Intel)",
            "unmaskedRenderer": "ANGLE (Intel, Intel(R) UHD Graphics 770 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_intel_uhd_620",
        "vendor": "intel",
        "tier": "integrated",
        "gpu": {"adapter": {"vendor": "intel", "architecture": "gen-9.5", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (Intel)",
            "unmaskedRenderer": "ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
    {
        "id": "win_intel_arc_a770",
        "vendor": "intel",
        "tier": "mainstream",
        "gpu": {"adapter": {"vendor": "intel", "architecture": "xe-hpg", "device": "", "description": ""}},
        "webgl": {
            "unmaskedVendor": "Google Inc. (Intel)",
            "unmaskedRenderer": "ANGLE (Intel, Intel(R) Arc(TM) A770 Graphics Direct3D11 vs_5_0 ps_5_0, D3D11)",
        },
    },
)


def normalize_country_code(value: str | None) -> str:
    code = str(value or "").strip().upper()
    return code if len(code) == 2 and code.isalpha() else ""


def is_supported_country_code(value: str | None) -> bool:
    return normalize_country_code(value) in WORLD_COUNTRY_CODE_SET


def build_country_code_pool(
    include: Iterable[str] | None = None,
    exclude: Iterable[str] | None = None,
) -> tuple[str, ...]:
    # Use this when caller wants to restrict the global 249-code pool, for
    # example only business-target countries or excluding uninhabited areas.
    pool = set(WORLD_COUNTRY_CODE_POOL)
    if include is not None:
        pool &= {
            code
            for item in include
            if (code := normalize_country_code(item)) in WORLD_COUNTRY_CODE_SET
        }
    if exclude is not None:
        pool -= {
            code
            for item in exclude
            if (code := normalize_country_code(item)) in WORLD_COUNTRY_CODE_SET
        }
    return tuple(sorted(pool))


def choose_country_code(
    rng: random.Random,
    country_hint: str | None = None,
    pool: Sequence[str] | None = None,
) -> str:
    # Later composition should call this first, then pass the selected country
    # into choose_timezone_for_country() and choose_language_list().
    active_pool = tuple(pool or WORLD_COUNTRY_CODE_POOL)
    if not active_pool:
        raise ValueError("country code pool is empty")

    hinted = normalize_country_code(country_hint)
    if hinted and hinted in active_pool:
        return hinted
    return rng.choice(active_pool)


def get_country_timezones(country_code: str | None) -> tuple[str, ...]:
    # Empty tuple means the country code is unknown to this module, not that the
    # country has no timezone. Valid 249-code entries should always be present.
    code = normalize_country_code(country_code)
    return COUNTRY_TIMEZONES.get(code, ())


def _format_offset(sign: str, hour_text: str, minute_text: str | None) -> str:
    hours = int(hour_text)
    minutes = int(minute_text or "0")
    if hours > 14 or minutes >= 60:
        return ""
    return f"{sign}{hours:02d}:{minutes:02d}"


def _etc_gmt_to_offset(value: str) -> str:
    # IANA's Etc/GMT sign convention is reversed by design:
    # Etc/GMT-8 is UTC+08:00 and Etc/GMT+5 is UTC-05:00.
    raw = value.strip()
    prefix = "etc/gmt"
    if not raw.lower().startswith(prefix):
        return ""
    suffix = raw[len(prefix):]
    if not suffix:
        return "+00:00"
    sign = suffix[0]
    if sign not in "+-" or not suffix[1:].isdigit():
        return ""
    hours = int(suffix[1:])
    if hours > 14:
        return ""
    # IANA Etc/GMT signs are intentionally reversed: Etc/GMT-8 means UTC+08.
    offset_sign = "+" if sign == "-" else "-"
    return f"{offset_sign}{hours:02d}:00"


def _offset_alias_to_timezone(value: str) -> str:
    raw = value.strip()
    etc_offset = _etc_gmt_to_offset(raw)
    if etc_offset:
        return OFFSET_TO_REPRESENTATIVE_TIMEZONE.get(etc_offset, "")

    match = _OFFSET_ALIAS_PATTERN.match(raw)
    if not match:
        return ""
    offset = _format_offset(match.group(1), match.group(2), match.group(3))
    return OFFSET_TO_REPRESENTATIVE_TIMEZONE.get(offset, "") if offset else ""


def _canonical_timezone_lookup() -> dict[str, str]:
    lookup: dict[str, str] = {}
    for zones in COUNTRY_TIMEZONES.values():
        for zone in zones:
            lookup[zone.casefold()] = zone
    for alias, canonical in IANA_TIMEZONE_LINK_ALIASES.items():
        lookup[alias.casefold()] = canonical
        lookup[canonical.casefold()] = canonical
    for canonical in OFFSET_TO_REPRESENTATIVE_TIMEZONE.values():
        lookup[canonical.casefold()] = canonical
    return lookup


def canonicalize_timezone(value: str | None, country_code: str | None = None) -> str:
    # Accepts common human/runtime inputs and returns the canonical IANA value
    # that should be stored in the environment profile. country_code is optional
    # and currently acts as a preference/validation hint, not a hard rejection.
    raw = str(value or "").strip()
    if not raw:
        return ""

    offset_timezone = _offset_alias_to_timezone(raw)
    if offset_timezone:
        country_zones = get_country_timezones(country_code)
        if country_zones and offset_timezone in country_zones:
            return offset_timezone
        return offset_timezone

    lookup = _canonical_timezone_lookup()
    canonical = lookup.get(raw.casefold(), "")
    if not canonical:
        return ""

    country_zones = get_country_timezones(country_code)
    if not country_zones or canonical in country_zones:
        return canonical
    return canonical


def choose_timezone_for_country(
    rng: random.Random,
    country_code: str,
    timezone_hint: str | None = None,
) -> str:
    # Use this after choose_country_code(). If a valid timezone_hint belongs to
    # the selected country, it wins; otherwise pick a realistic country zone.
    zones = get_country_timezones(country_code)
    if not zones:
        raise ValueError(f"no timezone candidates for country code: {country_code!r}")

    hinted = canonicalize_timezone(timezone_hint, country_code=country_code)
    if hinted and hinted in zones:
        return hinted
    return rng.choice(zones)


def get_country_language_profile(country_code: str | None) -> dict[str, tuple[str, ...] | str]:
    # Unknown codes fall back to en-US so downstream navigator construction does
    # not crash. Valid 249-code entries should always be present in the table.
    code = normalize_country_code(country_code)
    return COUNTRY_LANGUAGE_PROFILES.get(code, {"primary": "en-US", "secondary": ()})


def get_primary_language(country_code: str | None) -> str:
    profile = get_country_language_profile(country_code)
    return str(profile["primary"])


def get_secondary_languages(country_code: str | None) -> tuple[str, ...]:
    profile = get_country_language_profile(country_code)
    primary = normalize_language_tag(str(profile["primary"])) or "en-US"
    secondary = profile.get("secondary", ())
    output: list[str] = []
    seen_preferences = {_language_preference_key(primary)}
    for item in secondary:
        language = normalize_language_tag(str(item))
        if not language:
            continue
        preference_key = _language_preference_key(language)
        if preference_key in seen_preferences:
            continue
        if not is_reasonable_country_language(country_code, primary, language):
            continue
        seen_preferences.add(preference_key)
        output.append(language)
    return tuple(output)


def iter_language_inputs(locales: Iterable[str] | str | None) -> tuple[str, ...]:
    if not locales:
        return ()
    if isinstance(locales, str):
        raw_items = locales.split(",")
    else:
        raw_items = []
        for locale in locales:
            if locale is None:
                continue
            text = str(locale)
            raw_items.extend(text.split(",") if "," in text else (text,))
    return tuple(raw_items)


def normalize_language_tag(language: str | None) -> str:
    # Accept callers may pass navigator.languages, Accept-Language header
    # fragments, or replay-panel strings. Strip q weights before composing the
    # runtime navigator.languages list or rebuilding the header.
    text = str(language or "").strip().strip("'\"[] ")
    if not text:
        return ""
    text = text.split(";", 1)[0].strip()
    if "=" in text:
        left = text.split("=", 1)[0].strip()
        if re.fullmatch(r"[A-Za-z]{2,3}(?:[-_][A-Za-z0-9]{2,8})*", left):
            text = left
        else:
            return ""
    text = text.replace("_", "-")
    parts = [part for part in text.split("-") if part]
    if not parts or not re.fullmatch(r"[A-Za-z]{2,3}", parts[0]):
        return ""

    normalized = [parts[0].lower()]
    for part in parts[1:]:
        if len(part) == 4 and part.isalpha():
            normalized.append(part.title())
        elif (len(part) == 2 and part.isalpha()) or (len(part) == 3 and part.isdigit()):
            normalized.append(part.upper())
        elif re.fullmatch(r"[A-Za-z0-9]{4,8}", part):
            normalized.append(part.lower())
        else:
            return ""
    return "-".join(normalized)


def get_language_base_tag(language: str) -> str:
    normalized = normalize_language_tag(language)
    return normalized.split("-", 1)[0] if normalized else ""


def get_language_region_tag(language: str) -> str:
    parts = normalize_language_tag(language).split("-")
    for part in parts[1:]:
        if (len(part) == 2 and part.isalpha()) or (len(part) == 3 and part.isdigit()):
            return part.upper()
    return ""


# Script-explicit and script-inferred Chinese tags below describe the same
# regional preference. Keeping both in one browser preference list creates a
# duplicate-looking profile rather than an additional language choice.
_LANGUAGE_PREFERENCE_ALIASES: dict[str, str] = {
    "zh-Hans-CN": "zh-CN",
    "zh-Hant-HK": "zh-HK",
    "zh-Hant-MO": "zh-MO",
    "zh-Hant-TW": "zh-TW",
}


def _language_preference_key(language: str) -> str:
    normalized = normalize_language_tag(language)
    return _LANGUAGE_PREFERENCE_ALIASES.get(normalized, normalized)


LATIN_AMERICAN_SPANISH_REGIONS: frozenset[str] = frozenset((
    "AR", "BO", "CL", "CO", "CR", "CU", "DO", "EC", "GT", "HN", "MX", "NI",
    "PA", "PE", "PR", "PY", "SV", "US", "UY", "VE", "419",
))


def is_reasonable_country_language(
    country_code: str | None,
    primary_language: str,
    candidate_language: str,
) -> bool:
    code = normalize_country_code(country_code)
    primary = normalize_language_tag(primary_language) or "en-US"
    candidate = normalize_language_tag(candidate_language)
    if not candidate:
        return False

    primary_base = get_language_base_tag(primary)
    candidate_base = get_language_base_tag(candidate)
    candidate_region = get_language_region_tag(candidate)

    if candidate == primary or candidate == primary_base:
        return False

    # COUNTRY_LANGUAGE_PROFILES is the curated per-address pool. Do not reject
    # a configured language merely because its BCP 47 region is the language's
    # home locale instead of the current address country (for example ru-RU in
    # Armenia or es-ES in Gibraltar).
    configured = COUNTRY_LANGUAGE_PROFILES.get(code)
    if configured is not None:
        candidate_key = _language_preference_key(candidate)
        for item in configured.get("secondary", ()):
            if _language_preference_key(str(item)) == candidate_key:
                return True
    if candidate_region == code:
        return True
    if candidate_base == primary_base:
        return True
    if candidate_base == "en":
        return True
    if candidate == "es-419" and code in LATIN_AMERICAN_SPANISH_REGIONS:
        return True
    if primary_base == "zh" and candidate_base == "zh":
        return True
    return False


def build_primary_language_chain(language: str) -> tuple[str, ...]:
    primary = normalize_language_tag(language) or "en-US"
    base = get_language_base_tag(primary)
    region = get_language_region_tag(primary)
    if primary == "es-419":
        return ("es-419", "es")
    if base == "es" and region in LATIN_AMERICAN_SPANISH_REGIONS:
        return tuple(dict.fromkeys((primary, "es-419", "es")))
    if primary == "zh-HK":
        return ("zh-HK", "zh", "zh-TW")
    if primary == "zh-MO":
        return ("zh-MO", "zh-HK", "zh-TW", "zh")
    if primary == "zh-SG":
        return ("zh-SG", "zh-CN", "zh")
    if primary == "zh-Hans-CN":
        return ("zh-Hans-CN", "zh-CN", "zh")
    if primary in ("zh-Hant-TW", "zh-Hant-HK", "zh-Hant-MO"):
        regional = f"zh-{region}" if region else ""
        return tuple(dict.fromkeys(item for item in (primary, regional, "zh-TW", "zh") if item))
    if base and base != primary.lower():
        return (primary, base)
    return (primary,)


def build_language_chain(locales: Iterable[str] | str | None) -> tuple[str, ...]:
    output: list[str] = []
    pending_fallbacks: list[str] = []
    current_base = ""

    def add_once(language: str) -> None:
        if language and language not in output:
            output.append(language)

    def flush_pending() -> None:
        nonlocal pending_fallbacks
        for fallback in pending_fallbacks:
            add_once(fallback)
        pending_fallbacks = []

    for locale in iter_language_inputs(locales):
        chain = build_primary_language_chain(locale)
        if not chain:
            continue
        language = chain[0]
        base = get_language_base_tag(language)
        if current_base and base != current_base:
            flush_pending()
        current_base = base
        add_once(language)
        for fallback in chain[1:]:
            if get_language_base_tag(fallback) == base:
                if fallback not in output and fallback not in pending_fallbacks:
                    pending_fallbacks.append(fallback)
            else:
                add_once(fallback)
    flush_pending()
    return tuple(output)


def build_language_lists(
    country_code: str | None,
    max_secondary_count: int = 3,
    include_primary_only: bool = True,
    include_secondary: bool = True,
) -> tuple[tuple[str, ...], ...]:
    # This prepares realistic ordered navigator.languages profiles. The source
    # table is full-country coverage; this function must not create arbitrary
    # permutations from the secondary pool, because those quickly become
    # unrealistic browser settings. Profiles stay bounded and ordered:
    # primary-only, primary + one secondary, primary + adjacent secondary
    # slices, and same-country secondary locales as an alternate primary.
    primary = get_primary_language(country_code)
    secondary = get_secondary_languages(country_code)
    max_count = max(0, min(max_secondary_count, len(secondary)))
    variants: list[tuple[str, ...]] = []
    code = normalize_country_code(country_code)

    def add_profile(locales: Iterable[str] | str | None) -> None:
        profile = build_language_chain(locales)
        if profile:
            variants.append(profile)

    if include_primary_only:
        add_profile((primary,))

    if include_secondary:
        bounded_secondary = secondary[:max_count] if max_count else ()
        for language in secondary:
            add_profile((primary, language))
            if (
                code
                and get_language_region_tag(language) == code
                and get_language_base_tag(language) != get_language_base_tag(primary)
            ):
                add_profile((language, primary))
        for count in range(2, max_count + 1):
            for start in range(0, len(bounded_secondary) - count + 1):
                add_profile((primary, *bounded_secondary[start:start + count]))

    seen: set[tuple[str, ...]] = set()
    deduped: list[tuple[str, ...]] = []
    for variant in variants:
        clean = tuple(dict.fromkeys(item for item in variant if item))
        if clean not in seen:
            seen.add(clean)
            deduped.append(clean)
    return tuple(deduped)


def choose_language_list(
    rng: random.Random,
    country_code: str | None,
    max_secondary_count: int = 3,
    include_secondary: bool = True,
) -> tuple[str, ...]:
    # The first item is intended to become navigator.language. The full tuple is
    # intended to become navigator.languages.
    if not include_secondary:
        return build_primary_language_chain(get_primary_language(country_code))

    profiles = build_language_lists(
        country_code,
        max_secondary_count=max_secondary_count,
        include_primary_only=True,
        include_secondary=True,
    )
    if not profiles:
        return build_primary_language_chain(get_primary_language(country_code))

    primary = normalize_language_tag(get_primary_language(country_code)) or "en-US"
    secondary = get_secondary_languages(country_code)
    weights: list[float] = []
    for profile in profiles:
        selected_ranks = tuple(
            index
            for index, language in enumerate(secondary)
            if language in profile
        )
        if profile[0] == primary:
            if not selected_ranks:
                weight = 12.0
            elif len(selected_ranks) == 1:
                weight = max(2.0, 8.0 - selected_ranks[0])
            else:
                weight = max(
                    1.0,
                    5.0 - len(selected_ranks) - selected_ranks[0] * 0.25,
                )
        else:
            alternate_rank = next(
                (
                    index
                    for index, language in enumerate(secondary)
                    if language == profile[0]
                ),
                len(secondary),
            )
            weight = max(1.0, 4.0 - alternate_rank * 0.5)
        weights.append(weight)
    return rng.choices(profiles, weights=weights, k=1)[0]


def get_webgl_gpu_candidates(
    vendor: str | None = None,
    tier: str | None = None,
) -> tuple[dict[str, object], ...]:
    vendor_key = str(vendor or "").strip().lower()
    tier_key = str(tier or "").strip().lower()
    output = []
    for item in WEBGL_GPU_CANDIDATES:
        if vendor_key and str(item.get("vendor", "")).lower() != vendor_key:
            continue
        if tier_key and str(item.get("tier", "")).lower() != tier_key:
            continue
        output.append(item)
    return tuple(output)


def choose_webgl_gpu_candidate(
    rng: random.Random,
    vendor: str | None = None,
    tier: str | None = None,
) -> dict[str, object]:
    candidates = get_webgl_gpu_candidates(vendor=vendor, tier=tier)
    if not candidates:
        candidates = WEBGL_GPU_CANDIDATES
    return rng.choice(candidates)


def build_webgl_gpu_patch(candidate: dict[str, object]) -> dict[str, object]:
    # This returns the variable slice that should be deep-merged into a copied
    # webgl_gpu template. It intentionally does not fabricate the huge WebGL
    # parameters table; those values should come from the local template or a
    # browser-captured profile with matching capability tier.
    gpu = candidate.get("gpu")
    webgl = candidate.get("webgl")
    return {
        "gpu": gpu if isinstance(gpu, dict) else {},
        "webgl": webgl if isinstance(webgl, dict) else {},
        "webgl2": webgl if isinstance(webgl, dict) else {},
        "webglGpuId": candidate.get("id", ""),
        "webglGpuVendor": candidate.get("vendor", ""),
        "webglGpuTier": candidate.get("tier", ""),
    }
