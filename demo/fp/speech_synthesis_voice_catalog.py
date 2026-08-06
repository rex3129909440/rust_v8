"""
Country-aware speechSynthesis voices catalog.

The runtime environment expects:

    {"speechSynthesis": {"voices": [SpeechSynthesisVoice-like dicts]}}

Each voice object follows the browser-facing Web Speech API shape:
voiceURI, name, lang, localService, default. The catalog is intentionally
desktop Chromium/Windows-oriented because the rest of the current fingerprint
pool is PC-focused.
"""

from __future__ import annotations

import ast
import copy
import json
import random
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent
BROWSER_VOICES_PATH = ROOT / "browser_voices.py"
BROWSER_CAPTURED_FULL_PROFILE_IDS = frozenset((
    "browser_captured_full",
    "browser_voices_full",
    "local_browser_voices_full",
))


# locale, name, weight
#
# The names below model common Microsoft desktop voices exposed through
# Chromium/Edge speechSynthesis on Windows language packs. Not every Windows
# install has every voice; choose_speech_synthesis_voice_profile builds a small
# installed-voices subset from the country's primary and secondary languages.
SPEECH_VOICE_ROWS: tuple[tuple[str, str, int], ...] = (
    ("en-US", "Microsoft David - English (United States)", 38),
    ("en-US", "Microsoft Zira - English (United States)", 42),
    ("en-US", "Microsoft Mark - English (United States)", 24),
    ("en-GB", "Microsoft Hazel - English (United Kingdom)", 28),
    ("en-GB", "Microsoft George - English (United Kingdom)", 18),
    ("en-GB", "Microsoft Susan - English (United Kingdom)", 14),
    ("en-AU", "Microsoft Catherine - English (Australia)", 18),
    ("en-AU", "Microsoft James - English (Australia)", 10),
    ("en-CA", "Microsoft Linda - English (Canada)", 12),
    ("en-CA", "Microsoft Richard - English (Canada)", 8),
    ("en-IN", "Microsoft Heera - English (India)", 10),
    ("en-IN", "Microsoft Ravi - English (India)", 8),
    ("zh-CN", "Microsoft Huihui - Chinese (Simplified, PRC)", 36),
    ("zh-CN", "Microsoft Yaoyao - Chinese (Simplified, PRC)", 20),
    ("zh-CN", "Microsoft Kangkang - Chinese (Simplified, PRC)", 14),
    ("zh-TW", "Microsoft Hanhan - Chinese (Traditional, Taiwan)", 18),
    ("zh-TW", "Microsoft Yating - Chinese (Traditional, Taiwan)", 14),
    ("zh-TW", "Microsoft Zhiwei - Chinese (Traditional, Taiwan)", 10),
    ("zh-HK", "Microsoft Tracy - Chinese (Traditional, Hong Kong SAR)", 12),
    ("zh-HK", "Microsoft Danny - Chinese (Traditional, Hong Kong SAR)", 8),
    ("ja-JP", "Microsoft Haruka - Japanese (Japan)", 24),
    ("ja-JP", "Microsoft Ayumi - Japanese (Japan)", 12),
    ("ja-JP", "Microsoft Ichiro - Japanese (Japan)", 10),
    ("ko-KR", "Microsoft Heami - Korean (Korea)", 20),
    ("fr-FR", "Microsoft Hortense - French (France)", 30),
    ("fr-FR", "Microsoft Julie - French (France)", 12),
    ("fr-CA", "Microsoft Caroline - French (Canada)", 10),
    ("de-DE", "Microsoft Hedda - German (Germany)", 30),
    ("de-DE", "Microsoft Katja - German (Germany)", 18),
    ("de-DE", "Microsoft Stefan - German (Germany)", 10),
    ("es-ES", "Microsoft Helena - Spanish (Spain)", 26),
    ("es-ES", "Microsoft Pablo - Spanish (Spain)", 12),
    ("es-MX", "Microsoft Sabina - Spanish (Mexico)", 24),
    ("es-MX", "Microsoft Raul - Spanish (Mexico)", 8),
    ("it-IT", "Microsoft Elsa - Italian (Italy)", 22),
    ("it-IT", "Microsoft Cosimo - Italian (Italy)", 8),
    ("pt-BR", "Microsoft Maria - Portuguese (Brazil)", 24),
    ("pt-BR", "Microsoft Daniel - Portuguese (Brazil)", 8),
    ("pt-PT", "Microsoft Helia - Portuguese (Portugal)", 14),
    ("ru-RU", "Microsoft Irina - Russian (Russia)", 24),
    ("ru-RU", "Microsoft Pavel - Russian (Russia)", 8),
    ("nl-NL", "Microsoft Frank - Dutch (Netherlands)", 16),
    ("pl-PL", "Microsoft Paulina - Polish (Poland)", 16),
    ("tr-TR", "Microsoft Tolga - Turkish (Turkey)", 16),
    ("ar-EG", "Microsoft Hoda - Arabic (Egypt)", 12),
    ("ar-SA", "Microsoft Naayf - Arabic (Saudi Arabia)", 10),
    ("he-IL", "Microsoft Asaf - Hebrew (Israel)", 10),
    ("hi-IN", "Microsoft Kalpana - Hindi (India)", 14),
    ("hi-IN", "Microsoft Hemant - Hindi (India)", 8),
    ("id-ID", "Microsoft Andika - Indonesian (Indonesia)", 12),
    ("ms-MY", "Microsoft Rizwan - Malay (Malaysia)", 8),
    ("th-TH", "Microsoft Pattara - Thai (Thailand)", 12),
    ("vi-VN", "Microsoft An - Vietnamese (Vietnam)", 12),
    ("sv-SE", "Microsoft Bengt - Swedish (Sweden)", 10),
    ("da-DK", "Microsoft Helle - Danish (Denmark)", 10),
    ("fi-FI", "Microsoft Heidi - Finnish (Finland)", 10),
    ("nb-NO", "Microsoft Jon - Norwegian Bokmal (Norway)", 10),
    ("cs-CZ", "Microsoft Jakub - Czech (Czech Republic)", 10),
    ("sk-SK", "Microsoft Filip - Slovak (Slovakia)", 8),
    ("hu-HU", "Microsoft Szabolcs - Hungarian (Hungary)", 10),
    ("ro-RO", "Microsoft Andrei - Romanian (Romania)", 10),
    ("el-GR", "Microsoft Stefanos - Greek (Greece)", 10),
    ("uk-UA", "Microsoft Ostap - Ukrainian (Ukraine)", 8),
)


# Extra rows are based on Microsoft Speech locale voice names. They are not
# treated as "all installed on every machine"; the selector still exposes only
# a small per-profile subset, which is closer to real browser getVoices().
SPEECH_VOICE_ROWS = SPEECH_VOICE_ROWS + (
    ("af-ZA", "Microsoft Adri - Afrikaans (South Africa)", 10),
    ("af-ZA", "Microsoft Willem - Afrikaans (South Africa)", 8),
    ("am-ET", "Microsoft Mekdes - Amharic (Ethiopia)", 10),
    ("am-ET", "Microsoft Ameha - Amharic (Ethiopia)", 8),
    ("ar-AE", "Microsoft Fatima - Arabic (United Arab Emirates)", 10),
    ("ar-AE", "Microsoft Hamdan - Arabic (United Arab Emirates)", 8),
    ("ar-BH", "Microsoft Laila - Arabic (Bahrain)", 8),
    ("ar-BH", "Microsoft Ali - Arabic (Bahrain)", 6),
    ("ar-DZ", "Microsoft Amina - Arabic (Algeria)", 8),
    ("ar-DZ", "Microsoft Ismael - Arabic (Algeria)", 6),
    ("ar-EG", "Microsoft Salma - Arabic (Egypt)", 12),
    ("ar-EG", "Microsoft Shakir - Arabic (Egypt)", 8),
    ("ar-IQ", "Microsoft Rana - Arabic (Iraq)", 8),
    ("ar-IQ", "Microsoft Bassel - Arabic (Iraq)", 6),
    ("ar-JO", "Microsoft Sana - Arabic (Jordan)", 8),
    ("ar-JO", "Microsoft Taim - Arabic (Jordan)", 6),
    ("ar-KW", "Microsoft Noura - Arabic (Kuwait)", 8),
    ("ar-KW", "Microsoft Fahed - Arabic (Kuwait)", 6),
    ("ar-LB", "Microsoft Layla - Arabic (Lebanon)", 8),
    ("ar-LB", "Microsoft Rami - Arabic (Lebanon)", 6),
    ("ar-LY", "Microsoft Iman - Arabic (Libya)", 8),
    ("ar-LY", "Microsoft Omar - Arabic (Libya)", 6),
    ("ar-MA", "Microsoft Mouna - Arabic (Morocco)", 8),
    ("ar-MA", "Microsoft Jamal - Arabic (Morocco)", 6),
    ("ar-OM", "Microsoft Aysha - Arabic (Oman)", 8),
    ("ar-OM", "Microsoft Abdullah - Arabic (Oman)", 6),
    ("ar-QA", "Microsoft Amal - Arabic (Qatar)", 8),
    ("ar-QA", "Microsoft Moaz - Arabic (Qatar)", 6),
    ("ar-SA", "Microsoft Zariyah - Arabic (Saudi Arabia)", 12),
    ("ar-SA", "Microsoft Hamed - Arabic (Saudi Arabia)", 8),
    ("ar-SY", "Microsoft Amany - Arabic (Syria)", 8),
    ("ar-SY", "Microsoft Laith - Arabic (Syria)", 6),
    ("ar-TN", "Microsoft Reem - Arabic (Tunisia)", 8),
    ("ar-TN", "Microsoft Hedi - Arabic (Tunisia)", 6),
    ("ar-YE", "Microsoft Maryam - Arabic (Yemen)", 8),
    ("ar-YE", "Microsoft Saleh - Arabic (Yemen)", 6),
    ("as-IN", "Microsoft Yashica - Assamese (India)", 8),
    ("as-IN", "Microsoft Priyom - Assamese (India)", 6),
    ("az-AZ", "Microsoft Banu - Azerbaijani (Azerbaijan)", 8),
    ("az-AZ", "Microsoft Babek - Azerbaijani (Azerbaijan)", 6),
    ("bg-BG", "Microsoft Kalina - Bulgarian (Bulgaria)", 10),
    ("bg-BG", "Microsoft Borislav - Bulgarian (Bulgaria)", 8),
    ("bn-BD", "Microsoft Nabanita - Bangla (Bangladesh)", 10),
    ("bn-BD", "Microsoft Pradeep - Bangla (Bangladesh)", 8),
    ("bn-IN", "Microsoft Tanishaa - Bengali (India)", 10),
    ("bn-IN", "Microsoft Bashkar - Bengali (India)", 8),
    ("bs-BA", "Microsoft Vesna - Bosnian (Bosnia and Herzegovina)", 8),
    ("bs-BA", "Microsoft Goran - Bosnian (Bosnia and Herzegovina)", 6),
    ("ca-ES", "Microsoft Joana - Catalan (Spain)", 10),
    ("ca-ES", "Microsoft Enric - Catalan (Spain)", 8),
    ("ca-ES", "Microsoft Alba - Catalan (Spain)", 6),
    ("cs-CZ", "Microsoft Vlasta - Czech (Czechia)", 10),
    ("cs-CZ", "Microsoft Antonin - Czech (Czechia)", 8),
    ("cy-GB", "Microsoft Nia - Welsh (United Kingdom)", 8),
    ("cy-GB", "Microsoft Aled - Welsh (United Kingdom)", 6),
    ("da-DK", "Microsoft Christel - Danish (Denmark)", 10),
    ("da-DK", "Microsoft Jeppe - Danish (Denmark)", 8),
    ("de-AT", "Microsoft Ingrid - German (Austria)", 10),
    ("de-AT", "Microsoft Jonas - German (Austria)", 8),
    ("de-CH", "Microsoft Leni - German (Switzerland)", 10),
    ("de-CH", "Microsoft Jan - German (Switzerland)", 8),
    ("de-DE", "Microsoft Katja Neural - German (Germany)", 14),
    ("de-DE", "Microsoft Conrad - German (Germany)", 10),
    ("de-DE", "Microsoft Amala - German (Germany)", 8),
    ("de-DE", "Microsoft Bernd - German (Germany)", 6),
    ("el-GR", "Microsoft Athina - Greek (Greece)", 10),
    ("el-GR", "Microsoft Nestoras - Greek (Greece)", 8),
    ("en-GB", "Microsoft Sonia - English (United Kingdom)", 16),
    ("en-GB", "Microsoft Ryan - English (United Kingdom)", 14),
    ("en-HK", "Microsoft Yan - English (Hong Kong SAR)", 8),
    ("en-HK", "Microsoft Sam - English (Hong Kong SAR)", 6),
    ("en-IE", "Microsoft Emily - English (Ireland)", 12),
    ("en-IE", "Microsoft Connor - English (Ireland)", 10),
    ("en-IN", "Microsoft Neerja - English (India)", 12),
    ("en-IN", "Microsoft Prabhat - English (India)", 10),
    ("en-KE", "Microsoft Asilia - English (Kenya)", 8),
    ("en-KE", "Microsoft Chilemba - English (Kenya)", 6),
    ("en-NG", "Microsoft Ezinne - English (Nigeria)", 8),
    ("en-NG", "Microsoft Abeo - English (Nigeria)", 6),
    ("en-NZ", "Microsoft Molly - English (New Zealand)", 12),
    ("en-NZ", "Microsoft Mitchell - English (New Zealand)", 10),
    ("en-PH", "Microsoft Rosa - English (Philippines)", 10),
    ("en-PH", "Microsoft James - English (Philippines)", 8),
    ("en-SG", "Microsoft Luna - English (Singapore)", 10),
    ("en-SG", "Microsoft Wayne - English (Singapore)", 8),
    ("en-US", "Microsoft Aria - English (United States)", 16),
    ("en-US", "Microsoft Guy - English (United States)", 14),
    ("en-US", "Microsoft Jenny - English (United States)", 14),
    ("en-ZA", "Microsoft Leah - English (South Africa)", 10),
    ("en-ZA", "Microsoft Luke - English (South Africa)", 8),
    ("es-AR", "Microsoft Elena - Spanish (Argentina)", 10),
    ("es-AR", "Microsoft Tomas - Spanish (Argentina)", 8),
    ("es-BO", "Microsoft Sofia - Spanish (Bolivia)", 8),
    ("es-BO", "Microsoft Marcelo - Spanish (Bolivia)", 6),
    ("es-CL", "Microsoft Catalina - Spanish (Chile)", 10),
    ("es-CL", "Microsoft Lorenzo - Spanish (Chile)", 8),
    ("es-CO", "Microsoft Salome - Spanish (Colombia)", 10),
    ("es-CO", "Microsoft Gonzalo - Spanish (Colombia)", 8),
    ("es-CR", "Microsoft Maria - Spanish (Costa Rica)", 8),
    ("es-CR", "Microsoft Juan - Spanish (Costa Rica)", 6),
    ("es-CU", "Microsoft Belkys - Spanish (Cuba)", 8),
    ("es-CU", "Microsoft Manuel - Spanish (Cuba)", 6),
    ("es-DO", "Microsoft Ramona - Spanish (Dominican Republic)", 8),
    ("es-DO", "Microsoft Emilio - Spanish (Dominican Republic)", 6),
    ("es-EC", "Microsoft Andrea - Spanish (Ecuador)", 8),
    ("es-EC", "Microsoft Luis - Spanish (Ecuador)", 6),
    ("es-ES", "Microsoft Elvira - Spanish (Spain)", 12),
    ("es-ES", "Microsoft Alvaro - Spanish (Spain)", 10),
    ("es-GQ", "Microsoft Teresa - Spanish (Equatorial Guinea)", 6),
    ("es-GQ", "Microsoft Javier - Spanish (Equatorial Guinea)", 5),
    ("es-GT", "Microsoft Marta - Spanish (Guatemala)", 8),
    ("es-GT", "Microsoft Andres - Spanish (Guatemala)", 6),
    ("es-HN", "Microsoft Karla - Spanish (Honduras)", 8),
    ("es-HN", "Microsoft Carlos - Spanish (Honduras)", 6),
    ("es-MX", "Microsoft Dalia - Spanish (Mexico)", 12),
    ("es-MX", "Microsoft Jorge - Spanish (Mexico)", 10),
    ("es-NI", "Microsoft Yolanda - Spanish (Nicaragua)", 8),
    ("es-NI", "Microsoft Federico - Spanish (Nicaragua)", 6),
    ("es-PA", "Microsoft Margarita - Spanish (Panama)", 8),
    ("es-PA", "Microsoft Roberto - Spanish (Panama)", 6),
    ("es-PE", "Microsoft Camila - Spanish (Peru)", 10),
    ("es-PE", "Microsoft Alex - Spanish (Peru)", 8),
    ("es-PR", "Microsoft Karina - Spanish (Puerto Rico)", 8),
    ("es-PR", "Microsoft Victor - Spanish (Puerto Rico)", 6),
    ("es-PY", "Microsoft Tania - Spanish (Paraguay)", 8),
    ("es-PY", "Microsoft Mario - Spanish (Paraguay)", 6),
    ("es-SV", "Microsoft Lorena - Spanish (El Salvador)", 8),
    ("es-SV", "Microsoft Rodrigo - Spanish (El Salvador)", 6),
    ("es-US", "Microsoft Paloma - Spanish (United States)", 10),
    ("es-US", "Microsoft Alonso - Spanish (United States)", 8),
    ("es-UY", "Microsoft Valentina - Spanish (Uruguay)", 8),
    ("es-UY", "Microsoft Mateo - Spanish (Uruguay)", 6),
    ("es-VE", "Microsoft Paola - Spanish (Venezuela)", 10),
    ("es-VE", "Microsoft Sebastian - Spanish (Venezuela)", 8),
    ("et-EE", "Microsoft Anu - Estonian (Estonia)", 8),
    ("et-EE", "Microsoft Kert - Estonian (Estonia)", 6),
    ("eu-ES", "Microsoft Ainhoa - Basque (Spain)", 8),
    ("eu-ES", "Microsoft Ander - Basque (Spain)", 6),
    ("fa-IR", "Microsoft Dilara - Persian (Iran)", 10),
    ("fa-IR", "Microsoft Farid - Persian (Iran)", 8),
    ("fi-FI", "Microsoft Noora - Finnish (Finland)", 10),
    ("fi-FI", "Microsoft Harri - Finnish (Finland)", 8),
    ("fil-PH", "Microsoft Blessica - Filipino (Philippines)", 10),
    ("fil-PH", "Microsoft Angelo - Filipino (Philippines)", 8),
    ("fr-BE", "Microsoft Charline - French (Belgium)", 10),
    ("fr-BE", "Microsoft Gerard - French (Belgium)", 8),
    ("fr-CA", "Microsoft Sylvie - French (Canada)", 12),
    ("fr-CA", "Microsoft Antoine - French (Canada)", 10),
    ("fr-CH", "Microsoft Ariane - French (Switzerland)", 10),
    ("fr-CH", "Microsoft Fabrice - French (Switzerland)", 8),
    ("fr-FR", "Microsoft Denise - French (France)", 12),
    ("fr-FR", "Microsoft Henri - French (France)", 10),
    ("ga-IE", "Microsoft Orla - Irish (Ireland)", 8),
    ("ga-IE", "Microsoft Colm - Irish (Ireland)", 6),
    ("gl-ES", "Microsoft Sabela - Galician (Spain)", 8),
    ("gl-ES", "Microsoft Roi - Galician (Spain)", 6),
    ("gu-IN", "Microsoft Dhwani - Gujarati (India)", 8),
    ("gu-IN", "Microsoft Niranjan - Gujarati (India)", 6),
    ("he-IL", "Microsoft Hila - Hebrew (Israel)", 10),
    ("he-IL", "Microsoft Avri - Hebrew (Israel)", 8),
    ("hi-IN", "Microsoft Swara - Hindi (India)", 10),
    ("hi-IN", "Microsoft Madhur - Hindi (India)", 8),
    ("hr-HR", "Microsoft Gabrijela - Croatian (Croatia)", 8),
    ("hr-HR", "Microsoft Srecko - Croatian (Croatia)", 6),
    ("hu-HU", "Microsoft Noemi - Hungarian (Hungary)", 10),
    ("hu-HU", "Microsoft Tamas - Hungarian (Hungary)", 8),
    ("hy-AM", "Microsoft Anahit - Armenian (Armenia)", 8),
    ("hy-AM", "Microsoft Hayk - Armenian (Armenia)", 6),
    ("id-ID", "Microsoft Gadis - Indonesian (Indonesia)", 10),
    ("id-ID", "Microsoft Ardi - Indonesian (Indonesia)", 8),
    ("is-IS", "Microsoft Gudrun - Icelandic (Iceland)", 8),
    ("is-IS", "Microsoft Gunnar - Icelandic (Iceland)", 6),
    ("it-IT", "Microsoft Isabella - Italian (Italy)", 12),
    ("it-IT", "Microsoft Diego - Italian (Italy)", 10),
    ("iu-Cans-CA", "Microsoft Siqiniq - Inuktitut (Syllabics, Canada)", 6),
    ("iu-Cans-CA", "Microsoft Taqqiq - Inuktitut (Syllabics, Canada)", 5),
    ("iu-Latn-CA", "Microsoft Siqiniq - Inuktitut (Latin, Canada)", 6),
    ("iu-Latn-CA", "Microsoft Taqqiq - Inuktitut (Latin, Canada)", 5),
    ("ja-JP", "Microsoft Nanami - Japanese (Japan)", 12),
    ("ja-JP", "Microsoft Keita - Japanese (Japan)", 10),
    ("jv-ID", "Microsoft Siti - Javanese (Indonesia)", 8),
    ("jv-ID", "Microsoft Dimas - Javanese (Indonesia)", 6),
    ("ka-GE", "Microsoft Eka - Georgian (Georgia)", 8),
    ("ka-GE", "Microsoft Giorgi - Georgian (Georgia)", 6),
    ("kk-KZ", "Microsoft Aigul - Kazakh (Kazakhstan)", 8),
    ("kk-KZ", "Microsoft Daulet - Kazakh (Kazakhstan)", 6),
    ("km-KH", "Microsoft Sreymom - Khmer (Cambodia)", 8),
    ("km-KH", "Microsoft Piseth - Khmer (Cambodia)", 6),
    ("kn-IN", "Microsoft Sapna - Kannada (India)", 8),
    ("kn-IN", "Microsoft Gagan - Kannada (India)", 6),
    ("ko-KR", "Microsoft SunHi - Korean (Korea)", 12),
    ("ko-KR", "Microsoft InJoon - Korean (Korea)", 10),
    ("lo-LA", "Microsoft Keomany - Lao (Laos)", 8),
    ("lo-LA", "Microsoft Chanthavong - Lao (Laos)", 6),
    ("lt-LT", "Microsoft Ona - Lithuanian (Lithuania)", 8),
    ("lt-LT", "Microsoft Leonas - Lithuanian (Lithuania)", 6),
    ("lv-LV", "Microsoft Everita - Latvian (Latvia)", 8),
    ("lv-LV", "Microsoft Nils - Latvian (Latvia)", 6),
    ("mk-MK", "Microsoft Marija - Macedonian (North Macedonia)", 8),
    ("mk-MK", "Microsoft Aleksandar - Macedonian (North Macedonia)", 6),
    ("ml-IN", "Microsoft Sobhana - Malayalam (India)", 8),
    ("ml-IN", "Microsoft Midhun - Malayalam (India)", 6),
    ("mn-MN", "Microsoft Yesui - Mongolian (Mongolia)", 8),
    ("mn-MN", "Microsoft Bataa - Mongolian (Mongolia)", 6),
    ("mr-IN", "Microsoft Aarohi - Marathi (India)", 8),
    ("mr-IN", "Microsoft Manohar - Marathi (India)", 6),
    ("ms-MY", "Microsoft Yasmin - Malay (Malaysia)", 10),
    ("ms-MY", "Microsoft Osman - Malay (Malaysia)", 8),
    ("mt-MT", "Microsoft Grace - Maltese (Malta)", 8),
    ("mt-MT", "Microsoft Joseph - Maltese (Malta)", 6),
    ("my-MM", "Microsoft Nilar - Burmese (Myanmar)", 8),
    ("my-MM", "Microsoft Thiha - Burmese (Myanmar)", 6),
    ("nb-NO", "Microsoft Pernille - Norwegian Bokmal (Norway)", 10),
    ("nb-NO", "Microsoft Finn - Norwegian Bokmal (Norway)", 8),
    ("ne-NP", "Microsoft Hemkala - Nepali (Nepal)", 8),
    ("ne-NP", "Microsoft Sagar - Nepali (Nepal)", 6),
    ("nl-BE", "Microsoft Dena - Dutch (Belgium)", 10),
    ("nl-BE", "Microsoft Arnaud - Dutch (Belgium)", 8),
    ("nl-NL", "Microsoft Fenna - Dutch (Netherlands)", 12),
    ("nl-NL", "Microsoft Maarten - Dutch (Netherlands)", 10),
    ("pa-IN", "Microsoft Vaani - Punjabi (India)", 8),
    ("pa-IN", "Microsoft Ojas - Punjabi (India)", 6),
    ("pl-PL", "Microsoft Zofia - Polish (Poland)", 10),
    ("pl-PL", "Microsoft Marek - Polish (Poland)", 8),
    ("ps-AF", "Microsoft Latifa - Pashto (Afghanistan)", 8),
    ("ps-AF", "Microsoft Gul Nawaz - Pashto (Afghanistan)", 6),
    ("pt-BR", "Microsoft Francisca - Portuguese (Brazil)", 12),
    ("pt-BR", "Microsoft Antonio - Portuguese (Brazil)", 10),
    ("pt-PT", "Microsoft Raquel - Portuguese (Portugal)", 10),
    ("pt-PT", "Microsoft Duarte - Portuguese (Portugal)", 8),
    ("ro-RO", "Microsoft Alina - Romanian (Romania)", 10),
    ("ro-RO", "Microsoft Emil - Romanian (Romania)", 8),
    ("ru-RU", "Microsoft Svetlana - Russian (Russia)", 10),
    ("ru-RU", "Microsoft Dmitry - Russian (Russia)", 8),
    ("si-LK", "Microsoft Thilini - Sinhala (Sri Lanka)", 8),
    ("si-LK", "Microsoft Sameera - Sinhala (Sri Lanka)", 6),
    ("sk-SK", "Microsoft Viktoria - Slovak (Slovakia)", 10),
    ("sk-SK", "Microsoft Lukas - Slovak (Slovakia)", 8),
    ("sl-SI", "Microsoft Petra - Slovenian (Slovenia)", 8),
    ("sl-SI", "Microsoft Rok - Slovenian (Slovenia)", 6),
    ("so-SO", "Microsoft Ubax - Somali (Somalia)", 8),
    ("so-SO", "Microsoft Muuse - Somali (Somalia)", 6),
    ("sq-AL", "Microsoft Anila - Albanian (Albania)", 8),
    ("sq-AL", "Microsoft Ilir - Albanian (Albania)", 6),
    ("sr-Latn-RS", "Microsoft Sophie - Serbian Latin (Serbia)", 8),
    ("sr-Latn-RS", "Microsoft Nicholas - Serbian Latin (Serbia)", 6),
    ("sr-RS", "Microsoft Sophie - Serbian (Serbia)", 8),
    ("sr-RS", "Microsoft Nicholas - Serbian (Serbia)", 6),
    ("su-ID", "Microsoft Tuti - Sundanese (Indonesia)", 8),
    ("su-ID", "Microsoft Jajang - Sundanese (Indonesia)", 6),
    ("sv-SE", "Microsoft Sofie - Swedish (Sweden)", 10),
    ("sv-SE", "Microsoft Mattias - Swedish (Sweden)", 8),
    ("sw-KE", "Microsoft Zuri - Kiswahili (Kenya)", 8),
    ("sw-KE", "Microsoft Rafiki - Kiswahili (Kenya)", 6),
    ("sw-TZ", "Microsoft Rehema - Kiswahili (Tanzania)", 8),
    ("sw-TZ", "Microsoft Daudi - Kiswahili (Tanzania)", 6),
    ("ta-IN", "Microsoft Pallavi - Tamil (India)", 8),
    ("ta-IN", "Microsoft Valluvar - Tamil (India)", 6),
    ("ta-LK", "Microsoft Saranya - Tamil (Sri Lanka)", 8),
    ("ta-LK", "Microsoft Kumar - Tamil (Sri Lanka)", 6),
    ("ta-MY", "Microsoft Kani - Tamil (Malaysia)", 8),
    ("ta-MY", "Microsoft Surya - Tamil (Malaysia)", 6),
    ("ta-SG", "Microsoft Venba - Tamil (Singapore)", 8),
    ("ta-SG", "Microsoft Anbu - Tamil (Singapore)", 6),
    ("te-IN", "Microsoft Shruti - Telugu (India)", 8),
    ("te-IN", "Microsoft Mohan - Telugu (India)", 6),
    ("th-TH", "Microsoft Premwadee - Thai (Thailand)", 10),
    ("th-TH", "Microsoft Niwat - Thai (Thailand)", 8),
    ("tr-TR", "Microsoft Emel - Turkish (Turkey)", 10),
    ("tr-TR", "Microsoft Ahmet - Turkish (Turkey)", 8),
    ("uk-UA", "Microsoft Polina - Ukrainian (Ukraine)", 10),
    ("ur-IN", "Microsoft Gul - Urdu (India)", 8),
    ("ur-IN", "Microsoft Salman - Urdu (India)", 6),
    ("ur-PK", "Microsoft Uzma - Urdu (Pakistan)", 10),
    ("ur-PK", "Microsoft Asad - Urdu (Pakistan)", 8),
    ("uz-UZ", "Microsoft Madina - Uzbek (Uzbekistan)", 8),
    ("uz-UZ", "Microsoft Sardor - Uzbek (Uzbekistan)", 6),
    ("vi-VN", "Microsoft HoaiMy - Vietnamese (Vietnam)", 10),
    ("vi-VN", "Microsoft NamMinh - Vietnamese (Vietnam)", 8),
    ("wuu-CN", "Microsoft Xiaotong - Chinese Wu (China)", 6),
    ("wuu-CN", "Microsoft Yunzhe - Chinese Wu (China)", 5),
    ("yue-CN", "Microsoft XiaoMin - Cantonese (China)", 6),
    ("yue-CN", "Microsoft YunSong - Cantonese (China)", 5),
    ("zh-CN", "Microsoft Xiaoxiao - Chinese (Mandarin, Simplified)", 14),
    ("zh-CN", "Microsoft Yunxi - Chinese (Mandarin, Simplified)", 12),
    ("zh-CN", "Microsoft Yunyang - Chinese (Mandarin, Simplified)", 10),
    ("zh-HK", "Microsoft HiuGaai - Chinese (Cantonese, Traditional)", 10),
    ("zh-HK", "Microsoft WanLung - Chinese (Cantonese, Traditional)", 8),
    ("zh-TW", "Microsoft HsiaoChen - Chinese (Taiwan)", 10),
    ("zh-TW", "Microsoft YunJhe - Chinese (Taiwan)", 8),
    ("zu-ZA", "Microsoft Thando - Zulu (South Africa)", 8),
    ("zu-ZA", "Microsoft Themba - Zulu (South Africa)", 6),
)


COUNTRY_LOCALE_HINTS: dict[str, tuple[str, ...]] = {
    "US": ("en-US", "es-US"),
    "GB": ("en-GB",),
    "AU": ("en-AU",),
    "CA": ("en-CA", "fr-CA"),
    "IE": ("en-IE", "en-GB"),
    "NZ": ("en-NZ", "en-AU"),
    "IN": ("hi-IN", "en-IN"),
    "CN": ("zh-CN",),
    "TW": ("zh-TW",),
    "HK": ("zh-HK", "zh-TW"),
    "MO": ("zh-HK", "zh-TW"),
    "JP": ("ja-JP",),
    "KR": ("ko-KR",),
    "FR": ("fr-FR",),
    "BE": ("fr-BE", "nl-BE", "fr-FR", "nl-NL"),
    "CH": ("de-CH", "fr-CH", "it-CH", "de-DE", "fr-FR", "it-IT"),
    "DE": ("de-DE",),
    "AT": ("de-AT", "de-DE"),
    "ES": ("es-ES",),
    "MX": ("es-MX",),
    "AR": ("es-AR", "es-ES"),
    "CL": ("es-CL", "es-ES"),
    "CO": ("es-CO", "es-ES"),
    "PE": ("es-PE", "es-ES"),
    "VE": ("es-VE", "es-ES"),
    "IT": ("it-IT",),
    "BR": ("pt-BR",),
    "PT": ("pt-PT",),
    "RU": ("ru-RU",),
    "UA": ("uk-UA", "ru-RU"),
    "NL": ("nl-NL",),
    "PL": ("pl-PL",),
    "TR": ("tr-TR",),
    "SA": ("ar-SA",),
    "AE": ("ar-AE", "ar-SA"),
    "EG": ("ar-EG",),
    "IL": ("he-IL", "ar-SA"),
    "ID": ("id-ID",),
    "MY": ("ms-MY", "en-GB"),
    "TH": ("th-TH",),
    "VN": ("vi-VN",),
    "SE": ("sv-SE",),
    "DK": ("da-DK",),
    "FI": ("fi-FI",),
    "NO": ("nb-NO",),
    "CZ": ("cs-CZ",),
    "SK": ("sk-SK",),
    "HU": ("hu-HU",),
    "RO": ("ro-RO",),
    "GR": ("el-GR",),
}


FALLBACK_LOCALE_BY_BASE: dict[str, str] = {
    "af": "af-ZA",
    "am": "am-ET",
    "en": "en-US",
    "zh": "zh-CN",
    "ja": "ja-JP",
    "ko": "ko-KR",
    "fr": "fr-FR",
    "de": "de-DE",
    "es": "es-ES",
    "it": "it-IT",
    "pt": "pt-BR",
    "ru": "ru-RU",
    "nl": "nl-NL",
    "pl": "pl-PL",
    "tr": "tr-TR",
    "ar": "ar-SA",
    "he": "he-IL",
    "hi": "hi-IN",
    "id": "id-ID",
    "ms": "ms-MY",
    "th": "th-TH",
    "vi": "vi-VN",
    "sv": "sv-SE",
    "da": "da-DK",
    "fi": "fi-FI",
    "nb": "nb-NO",
    "no": "nb-NO",
    "cs": "cs-CZ",
    "sk": "sk-SK",
    "hu": "hu-HU",
    "ro": "ro-RO",
    "el": "el-GR",
    "uk": "uk-UA",
    "as": "as-IN",
    "az": "az-AZ",
    "bg": "bg-BG",
    "bn": "bn-BD",
    "bs": "bs-BA",
    "ca": "ca-ES",
    "cy": "cy-GB",
    "et": "et-EE",
    "eu": "eu-ES",
    "fa": "fa-IR",
    "fil": "fil-PH",
    "ga": "ga-IE",
    "gl": "gl-ES",
    "gu": "gu-IN",
    "hr": "hr-HR",
    "hy": "hy-AM",
    "is": "is-IS",
    "iu": "iu-Latn-CA",
    "jv": "jv-ID",
    "ka": "ka-GE",
    "kk": "kk-KZ",
    "km": "km-KH",
    "kn": "kn-IN",
    "lo": "lo-LA",
    "lt": "lt-LT",
    "lv": "lv-LV",
    "mk": "mk-MK",
    "ml": "ml-IN",
    "mn": "mn-MN",
    "mr": "mr-IN",
    "mt": "mt-MT",
    "my": "my-MM",
    "ne": "ne-NP",
    "pa": "pa-IN",
    "ps": "ps-AF",
    "si": "si-LK",
    "sl": "sl-SI",
    "so": "so-SO",
    "sq": "sq-AL",
    "sr": "sr-RS",
    "su": "su-ID",
    "sw": "sw-KE",
    "ta": "ta-IN",
    "te": "te-IN",
    "ur": "ur-PK",
    "uz": "uz-UZ",
    "wuu": "wuu-CN",
    "yue": "yue-CN",
    "zu": "zu-ZA",
}


LANGUAGE_DISPLAY_BY_BASE: dict[str, str] = {
    "af": "Afrikaans",
    "ak": "Akan",
    "am": "Amharic",
    "ar": "Arabic",
    "as": "Assamese",
    "ay": "Aymara",
    "az": "Azerbaijani",
    "be": "Belarusian",
    "bem": "Bemba",
    "ber": "Berber",
    "bg": "Bulgarian",
    "bi": "Bislama",
    "bm": "Bambara",
    "bn": "Bangla",
    "bs": "Bosnian",
    "ca": "Catalan",
    "ceb": "Cebuano",
    "ch": "Chamorro",
    "chk": "Chuukese",
    "cnr": "Montenegrin",
    "crs": "Seychellois Creole",
    "cs": "Czech",
    "cy": "Welsh",
    "da": "Danish",
    "de": "German",
    "dv": "Divehi",
    "dz": "Dzongkha",
    "ee": "Ewe",
    "el": "Greek",
    "en": "English",
    "es": "Spanish",
    "et": "Estonian",
    "eu": "Basque",
    "fa": "Persian",
    "ff": "Fulah",
    "fi": "Finnish",
    "fil": "Filipino",
    "fj": "Fijian",
    "fo": "Faroese",
    "fr": "French",
    "fud": "Futuna-Aniwa",
    "ga": "Irish",
    "gd": "Scottish Gaelic",
    "gil": "Gilbertese",
    "gl": "Galician",
    "gn": "Guarani",
    "gu": "Gujarati",
    "gv": "Manx",
    "ha": "Hausa",
    "he": "Hebrew",
    "hi": "Hindi",
    "ho": "Hiri Motu",
    "hr": "Croatian",
    "ht": "Haitian Creole",
    "hu": "Hungarian",
    "hy": "Armenian",
    "id": "Indonesian",
    "ig": "Igbo",
    "ilo": "Iloko",
    "is": "Icelandic",
    "it": "Italian",
    "iu": "Inuktitut",
    "ja": "Japanese",
    "jv": "Javanese",
    "ka": "Georgian",
    "kea": "Kabuverdianu",
    "kg": "Kongo",
    "kk": "Kazakh",
    "kl": "Kalaallisut",
    "km": "Khmer",
    "ko": "Korean",
    "kos": "Kosraean",
    "kri": "Krio",
    "ku": "Kurdish",
    "ky": "Kyrgyz",
    "la": "Latin",
    "lb": "Luxembourgish",
    "lg": "Ganda",
    "ln": "Lingala",
    "lo": "Lao",
    "lt": "Lithuanian",
    "lua": "Luba-Lulua",
    "lv": "Latvian",
    "mfe": "Morisyen",
    "mg": "Malagasy",
    "mh": "Marshallese",
    "mi": "Maori",
    "mk": "Macedonian",
    "ml": "Malayalam",
    "mn": "Mongolian",
    "mos": "Mossi",
    "mr": "Marathi",
    "ms": "Malay",
    "mt": "Maltese",
    "my": "Burmese",
    "na": "Nauru",
    "nb": "Norwegian Bokmal",
    "nd": "North Ndebele",
    "ne": "Nepali",
    "niu": "Niuean",
    "nl": "Dutch",
    "nn": "Norwegian Nynorsk",
    "no": "Norwegian",
    "ny": "Nyanja",
    "om": "Oromo",
    "or": "Odia",
    "pa": "Punjabi",
    "pap": "Papiamento",
    "pau": "Palauan",
    "pis": "Pijin",
    "pl": "Polish",
    "pon": "Pohnpeian",
    "ps": "Pashto",
    "pt": "Portuguese",
    "qu": "Quechua",
    "rm": "Romansh",
    "rn": "Rundi",
    "ro": "Romanian",
    "ru": "Russian",
    "rw": "Kinyarwanda",
    "sd": "Sindhi",
    "se": "Northern Sami",
    "sg": "Sango",
    "si": "Sinhala",
    "sk": "Slovak",
    "sl": "Slovenian",
    "sm": "Samoan",
    "sn": "Shona",
    "so": "Somali",
    "sq": "Albanian",
    "sr": "Serbian",
    "srn": "Sranan Tongo",
    "ss": "Swati",
    "st": "Southern Sotho",
    "su": "Sundanese",
    "sv": "Swedish",
    "sw": "Kiswahili",
    "swb": "Comorian",
    "ta": "Tamil",
    "te": "Telugu",
    "tet": "Tetum",
    "tg": "Tajik",
    "th": "Thai",
    "ti": "Tigrinya",
    "tk": "Turkmen",
    "tkl": "Tokelauan",
    "tn": "Tswana",
    "to": "Tongan",
    "tpi": "Tok Pisin",
    "tr": "Turkish",
    "tvl": "Tuvaluan",
    "ty": "Tahitian",
    "uk": "Ukrainian",
    "ur": "Urdu",
    "uz": "Uzbek",
    "vi": "Vietnamese",
    "wls": "Wallisian",
    "wo": "Wolof",
    "wuu": "Chinese Wu",
    "xh": "Xhosa",
    "yo": "Yoruba",
    "yue": "Cantonese",
    "zh": "Chinese",
    "zu": "Zulu",
}


REGION_DISPLAY_BY_CODE: dict[str, str] = {
    "AD": "Andorra",
    "AE": "United Arab Emirates",
    "AF": "Afghanistan",
    "AL": "Albania",
    "AM": "Armenia",
    "AO": "Angola",
    "AQ": "Antarctica",
    "AR": "Argentina",
    "AT": "Austria",
    "AU": "Australia",
    "AZ": "Azerbaijan",
    "BA": "Bosnia and Herzegovina",
    "BD": "Bangladesh",
    "BE": "Belgium",
    "BG": "Bulgaria",
    "BH": "Bahrain",
    "BO": "Bolivia",
    "BR": "Brazil",
    "CA": "Canada",
    "CH": "Switzerland",
    "CL": "Chile",
    "CN": "China",
    "CO": "Colombia",
    "CR": "Costa Rica",
    "CU": "Cuba",
    "CZ": "Czechia",
    "DE": "Germany",
    "DK": "Denmark",
    "DO": "Dominican Republic",
    "DZ": "Algeria",
    "EC": "Ecuador",
    "EE": "Estonia",
    "EG": "Egypt",
    "ES": "Spain",
    "ET": "Ethiopia",
    "FI": "Finland",
    "FR": "France",
    "GB": "United Kingdom",
    "GE": "Georgia",
    "GR": "Greece",
    "GT": "Guatemala",
    "HK": "Hong Kong SAR",
    "HN": "Honduras",
    "HR": "Croatia",
    "HU": "Hungary",
    "ID": "Indonesia",
    "IE": "Ireland",
    "IL": "Israel",
    "IN": "India",
    "IQ": "Iraq",
    "IR": "Iran",
    "IS": "Iceland",
    "IT": "Italy",
    "JO": "Jordan",
    "JP": "Japan",
    "KE": "Kenya",
    "KG": "Kyrgyzstan",
    "KH": "Cambodia",
    "KR": "Korea",
    "KW": "Kuwait",
    "KZ": "Kazakhstan",
    "LA": "Laos",
    "LB": "Lebanon",
    "LK": "Sri Lanka",
    "LT": "Lithuania",
    "LU": "Luxembourg",
    "LV": "Latvia",
    "LY": "Libya",
    "MA": "Morocco",
    "MK": "North Macedonia",
    "MM": "Myanmar",
    "MN": "Mongolia",
    "MO": "Macao SAR",
    "MT": "Malta",
    "MX": "Mexico",
    "MY": "Malaysia",
    "NI": "Nicaragua",
    "NL": "Netherlands",
    "NO": "Norway",
    "NP": "Nepal",
    "NZ": "New Zealand",
    "OM": "Oman",
    "PA": "Panama",
    "PE": "Peru",
    "PH": "Philippines",
    "PK": "Pakistan",
    "PL": "Poland",
    "PR": "Puerto Rico",
    "PT": "Portugal",
    "PY": "Paraguay",
    "QA": "Qatar",
    "RO": "Romania",
    "RS": "Serbia",
    "RU": "Russia",
    "SA": "Saudi Arabia",
    "SE": "Sweden",
    "SG": "Singapore",
    "SI": "Slovenia",
    "SK": "Slovakia",
    "SO": "Somalia",
    "SV": "El Salvador",
    "SY": "Syria",
    "TH": "Thailand",
    "TN": "Tunisia",
    "TR": "Turkey",
    "TW": "Taiwan",
    "TZ": "Tanzania",
    "UA": "Ukraine",
    "US": "United States",
    "UY": "Uruguay",
    "UZ": "Uzbekistan",
    "VE": "Venezuela",
    "VN": "Vietnam",
    "YE": "Yemen",
    "ZA": "South Africa",
}


REGION_DISPLAY_BY_CODE.update({
    "AG": "Antigua and Barbuda",
    "AI": "Anguilla",
    "AS": "American Samoa",
    "AW": "Aruba",
    "AX": "Aland Islands",
    "BB": "Barbados",
    "BF": "Burkina Faso",
    "BI": "Burundi",
    "BJ": "Benin",
    "BL": "Saint Barthelemy",
    "BM": "Bermuda",
    "BN": "Brunei",
    "BQ": "Caribbean Netherlands",
    "BS": "Bahamas",
    "BT": "Bhutan",
    "BV": "Bouvet Island",
    "BW": "Botswana",
    "BY": "Belarus",
    "BZ": "Belize",
    "CC": "Cocos Islands",
    "CD": "Democratic Republic of the Congo",
    "CF": "Central African Republic",
    "CG": "Republic of the Congo",
    "CI": "Cote d'Ivoire",
    "CK": "Cook Islands",
    "CM": "Cameroon",
    "CV": "Cabo Verde",
    "CW": "Curacao",
    "CX": "Christmas Island",
    "CY": "Cyprus",
    "DJ": "Djibouti",
    "DM": "Dominica",
    "EH": "Western Sahara",
    "ER": "Eritrea",
    "FJ": "Fiji",
    "FK": "Falkland Islands",
    "FM": "Micronesia",
    "FO": "Faroe Islands",
    "GA": "Gabon",
    "GD": "Grenada",
    "GF": "French Guiana",
    "GG": "Guernsey",
    "GH": "Ghana",
    "GI": "Gibraltar",
    "GL": "Greenland",
    "GM": "Gambia",
    "GN": "Guinea",
    "GP": "Guadeloupe",
    "GQ": "Equatorial Guinea",
    "GS": "South Georgia and the South Sandwich Islands",
    "GU": "Guam",
    "GW": "Guinea-Bissau",
    "GY": "Guyana",
    "HM": "Heard Island and McDonald Islands",
    "HT": "Haiti",
    "IM": "Isle of Man",
    "IO": "British Indian Ocean Territory",
    "JE": "Jersey",
    "JM": "Jamaica",
    "KI": "Kiribati",
    "KM": "Comoros",
    "KN": "Saint Kitts and Nevis",
    "KP": "North Korea",
    "KY": "Cayman Islands",
    "LC": "Saint Lucia",
    "LI": "Liechtenstein",
    "LR": "Liberia",
    "LS": "Lesotho",
    "MC": "Monaco",
    "MD": "Moldova",
    "ME": "Montenegro",
    "MF": "Saint Martin",
    "MG": "Madagascar",
    "MH": "Marshall Islands",
    "ML": "Mali",
    "MP": "Northern Mariana Islands",
    "MQ": "Martinique",
    "MR": "Mauritania",
    "MS": "Montserrat",
    "MU": "Mauritius",
    "MV": "Maldives",
    "MW": "Malawi",
    "MZ": "Mozambique",
    "NA": "Namibia",
    "NC": "New Caledonia",
    "NE": "Niger",
    "NF": "Norfolk Island",
    "NG": "Nigeria",
    "NR": "Nauru",
    "NU": "Niue",
    "PF": "French Polynesia",
    "PG": "Papua New Guinea",
    "PM": "Saint Pierre and Miquelon",
    "PN": "Pitcairn Islands",
    "PS": "Palestine",
    "PW": "Palau",
    "RE": "Reunion",
    "RW": "Rwanda",
    "SB": "Solomon Islands",
    "SC": "Seychelles",
    "SD": "Sudan",
    "SH": "Saint Helena",
    "SJ": "Svalbard and Jan Mayen",
    "SL": "Sierra Leone",
    "SM": "San Marino",
    "SN": "Senegal",
    "SR": "Suriname",
    "SS": "South Sudan",
    "ST": "Sao Tome and Principe",
    "SX": "Sint Maarten",
    "SZ": "Eswatini",
    "TC": "Turks and Caicos Islands",
    "TD": "Chad",
    "TF": "French Southern Territories",
    "TG": "Togo",
    "TJ": "Tajikistan",
    "TK": "Tokelau",
    "TL": "Timor-Leste",
    "TM": "Turkmenistan",
    "TO": "Tonga",
    "TT": "Trinidad and Tobago",
    "TV": "Tuvalu",
    "UG": "Uganda",
    "UM": "United States Minor Outlying Islands",
    "VA": "Vatican City",
    "VC": "Saint Vincent and the Grenadines",
    "VG": "British Virgin Islands",
    "VI": "U.S. Virgin Islands",
    "VU": "Vanuatu",
    "WF": "Wallis and Futuna",
    "WS": "Samoa",
    "YT": "Mayotte",
    "ZM": "Zambia",
    "ZW": "Zimbabwe",
})


def normalize_locale(locale: str) -> str:
    parts = str(locale or "").replace("_", "-").split("-")
    if not parts or not parts[0]:
        return ""
    base = parts[0].lower()
    if len(parts) == 1:
        return base
    normalized = [base]
    for part in parts[1:]:
        if len(part) == 4 and part.isalpha():
            normalized.append(part.title())
        elif (len(part) == 2 and part.isalpha()) or (len(part) == 3 and part.isdigit()):
            normalized.append(part.upper())
        else:
            normalized.append(part)
    return "-".join(normalized)


def base_language(locale: str) -> str:
    return normalize_locale(locale).split("-", 1)[0]


def region_from_locale(locale: str) -> str:
    for part in normalize_locale(locale).split("-")[1:]:
        if (len(part) == 2 and part.isalpha()) or (len(part) == 3 and part.isdigit()):
            return part.upper()
    return ""


def _voice_obj(
    name: str,
    lang: str,
    default: bool = False,
    local_service: bool = True,
    voice_uri: str | None = None,
) -> dict[str, object]:
    return {
        "voiceURI": voice_uri or name,
        "name": name,
        "lang": normalize_locale(lang),
        "localService": bool(local_service),
        "default": bool(default),
    }


def _load_browser_captured_voices(path: Path = BROWSER_VOICES_PATH) -> tuple[dict[str, object], ...]:
    # browser_voices.py is a Python string literal containing JSON captured from
    # a real Chromium/Edge speechSynthesis.getVoices() run. Parse it as data;
    # do not import/execute it.
    if not path.exists():
        return ()
    try:
        value = ast.literal_eval(path.read_text(encoding="utf-8").strip())
        rows = json.loads(value) if isinstance(value, str) else value
    except Exception:
        return ()
    if not isinstance(rows, list):
        return ()

    output: list[dict[str, object]] = []
    seen: set[tuple[str, str]] = set()
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            continue
        name = str(row.get("name") or row.get("voiceURI") or "").strip()
        lang = normalize_locale(str(row.get("lang") or ""))
        if not name or not lang:
            continue
        voice = _voice_obj(
            name=name,
            lang=lang,
            default=bool(row.get("default", index == 0)),
            local_service=bool(row.get("localService", False)),
            voice_uri=str(row.get("voiceURI") or name),
        )
        key = (str(voice["name"]), str(voice["lang"]))
        if key in seen:
            continue
        seen.add(key)
        output.append(voice)
    if output and not any(voice.get("default") for voice in output):
        output[0]["default"] = True
    return tuple(output)


VOICE_TEMPLATES_BY_LOCALE: dict[str, tuple[dict[str, object], ...]] = {}
for locale, name, weight in SPEECH_VOICE_ROWS:
    voice = _voice_obj(name, locale)
    voice["weight"] = weight
    VOICE_TEMPLATES_BY_LOCALE.setdefault(normalize_locale(locale), tuple())
    VOICE_TEMPLATES_BY_LOCALE[normalize_locale(locale)] = (
        *VOICE_TEMPLATES_BY_LOCALE[normalize_locale(locale)],
        voice,
    )


BROWSER_CAPTURED_VOICES: tuple[dict[str, object], ...] = _load_browser_captured_voices()
BROWSER_CAPTURED_VOICE_TEMPLATES_BY_LOCALE: dict[str, tuple[dict[str, object], ...]] = {}
for index, voice in enumerate(BROWSER_CAPTURED_VOICES):
    item = copy.deepcopy(voice)
    # Real captured voices should beat the hand-written catalog for exact
    # locale matches, but still get randomized within that locale.
    item["weight"] = 40 if item.get("default") else (28 if item.get("localService") else 16)
    locale = normalize_locale(str(item.get("lang") or ""))
    if not locale:
        continue
    BROWSER_CAPTURED_VOICE_TEMPLATES_BY_LOCALE.setdefault(locale, tuple())
    BROWSER_CAPTURED_VOICE_TEMPLATES_BY_LOCALE[locale] = (
        *BROWSER_CAPTURED_VOICE_TEMPLATES_BY_LOCALE[locale],
        item,
    )


def _generic_voice_for_locale(locale: str) -> dict[str, object] | None:
    # Some country-language pairs have no widely shipped Windows voice. For
    # those cases we still keep the Web Speech shape and requested BCP-47 lang
    # instead of leaking an unrelated en-US/es-ES voice into the profile.
    normalized = normalize_locale(locale)
    if not normalized:
        return None
    base = base_language(normalized)
    language = LANGUAGE_DISPLAY_BY_BASE.get(base, base.upper())
    region = region_from_locale(normalized)
    region_name = REGION_DISPLAY_BY_CODE.get(region, region)
    if region_name:
        name = f"Microsoft {language} - {language} ({region_name})"
    else:
        name = f"Microsoft {language} - {language}"
    return _voice_obj(name, normalized)


def _templates_for_locale(locale: str) -> tuple[dict[str, object], ...]:
    locale = normalize_locale(locale)
    captured = BROWSER_CAPTURED_VOICE_TEMPLATES_BY_LOCALE.get(locale)
    if captured:
        return captured
    exact = VOICE_TEMPLATES_BY_LOCALE.get(locale)
    if exact:
        return exact
    fallback_locale = FALLBACK_LOCALE_BY_BASE.get(base_language(locale))
    if fallback_locale and locale == base_language(locale):
        return (
            BROWSER_CAPTURED_VOICE_TEMPLATES_BY_LOCALE.get(fallback_locale)
            or VOICE_TEMPLATES_BY_LOCALE.get(fallback_locale, ())
        )
    return ()


def _choose_voice(rng: random.Random, locale: str) -> dict[str, object] | None:
    requested_locale = normalize_locale(locale)
    candidates = _templates_for_locale(requested_locale)
    if candidates:
        weights = [int(item.get("weight", 1)) for item in candidates]
        item = rng.choices(candidates, weights=weights, k=1)[0]
        output = copy.deepcopy(item)
        output.pop("weight", None)
        if requested_locale and requested_locale in VOICE_TEMPLATES_BY_LOCALE:
            output["lang"] = requested_locale
        return output
    return _generic_voice_for_locale(requested_locale)


def _dedupe_voices(voices: list[dict[str, object]]) -> list[dict[str, object]]:
    seen: set[tuple[str, str]] = set()
    output: list[dict[str, object]] = []
    for voice in voices:
        key = (str(voice.get("name", "")), str(voice.get("lang", "")))
        if key in seen:
            continue
        seen.add(key)
        output.append(voice)
    return output


def build_speech_synthesis_profile_from_voices(
    voices: list[dict[str, Any]],
    profile_id: str = "custom_speech_synthesis_voices",
) -> dict[str, object]:
    normalized = []
    for index, voice in enumerate(voices or []):
        name = str(voice.get("name") or voice.get("voiceURI") or "").strip()
        lang = normalize_locale(str(voice.get("lang") or ""))
        if not name or not lang:
            continue
        normalized.append({
            "voiceURI": str(voice.get("voiceURI") or name),
            "name": name,
            "lang": lang,
            "localService": bool(voice.get("localService", True)),
            "default": bool(voice.get("default", index == 0)),
        })
    if normalized and not any(voice.get("default") for voice in normalized):
        normalized[0]["default"] = True
    return {
        "id": profile_id,
        "voices": normalized,
        "speechSynthesis": {"voices": normalized},
    }


def build_browser_captured_full_voice_profile(
    profile_id: str = "browser_captured_full",
) -> dict[str, object]:
    voices = [copy.deepcopy(voice) for voice in BROWSER_CAPTURED_VOICES]
    if voices and not any(voice.get("default") for voice in voices):
        voices[0]["default"] = True
    return {
        "id": profile_id,
        "country": "",
        "primaryLocale": normalize_locale(str(voices[0].get("lang") or "")) if voices else "",
        "locales": tuple(dict.fromkeys(str(voice.get("lang") or "") for voice in voices if voice.get("lang"))),
        "voices": voices,
        "speechSynthesis": {"voices": voices},
    }


def _choose_country_languages(
    rng: random.Random,
    country_code: str,
) -> tuple[str, ...]:
    # Keep this import lazy so this catalog can also be imported in isolation.
    # The runtime composer owns the full 249-country language table.
    try:
        from fingerprint_runtime_composer import choose_language_list
    except Exception:
        return ()
    try:
        return tuple(choose_language_list(rng, country_code, include_secondary=True))
    except Exception:
        return ()


def choose_speech_synthesis_voice_profile(
    rng: random.Random,
    country_code: str,
    languages: list[str] | tuple[str, ...] | None = None,
    profile_id: str | None = None,
) -> dict[str, object]:
    if profile_id in BROWSER_CAPTURED_FULL_PROFILE_IDS and BROWSER_CAPTURED_VOICES:
        return build_browser_captured_full_voice_profile(profile_id=profile_id or "browser_captured_full")

    country = str(country_code or "US").upper()
    locale_candidates: list[str] = []
    language_candidates = tuple(languages or ()) or _choose_country_languages(rng, country)
    locale_candidates.extend(normalize_locale(item) for item in language_candidates)
    locale_candidates.extend(COUNTRY_LOCALE_HINTS.get(country, ()))
    locale_candidates = [item for item in locale_candidates if item]
    if not locale_candidates:
        locale_candidates = ["en-US"]
    locale_candidates = list(dict.fromkeys(locale_candidates))

    voices: list[dict[str, object]] = []
    primary = _choose_voice(rng, locale_candidates[0])
    if primary:
        primary["default"] = True
        voices.append(primary)

    # Add a second voice for the same locale when available, which is common on
    # Windows English/Chinese/German/French installs.
    same_locale = [copy.deepcopy(item) for item in _templates_for_locale(locale_candidates[0])]
    rng.shuffle(same_locale)
    for item in same_locale:
        item.pop("weight", None)
        if voices and item.get("name") == voices[0].get("name") and item.get("lang") == voices[0].get("lang"):
            continue
        item["default"] = False
        voices.append(item)
        if len(voices) >= 2:
            break

    # Add one voice per secondary locale, capped to avoid unrealistic huge lists.
    for locale in locale_candidates[1:]:
        if len(voices) >= 5:
            break
        voice = _choose_voice(rng, locale)
        if voice:
            voice["default"] = False
            voices.append(voice)

    # Many desktop installs expose at least one US English voice as a fallback.
    if len(voices) < 2 and base_language(locale_candidates[0]) != "en":
        fallback = _choose_voice(rng, "en-US")
        if fallback:
            fallback["default"] = False
            voices.append(fallback)

    voices = _dedupe_voices(voices)
    if voices and not any(voice.get("default") for voice in voices):
        voices[0]["default"] = True

    profile_key = profile_id or f"speech_{country}_{normalize_locale(locale_candidates[0])}_{len(voices)}v"
    return {
        "id": profile_key,
        "country": country,
        "primaryLocale": normalize_locale(locale_candidates[0]),
        "locales": tuple(locale_candidates),
        "voices": voices,
        "speechSynthesis": {"voices": voices},
    }


def build_speech_synthesis_patch(profile: dict[str, object]) -> dict[str, object]:
    voices = copy.deepcopy(profile.get("voices", []))
    return {
        "speechSynthesis": {
            "voices": voices,
        },
        "speechSynthesisVoiceProfileId": profile.get("id", ""),
    }
