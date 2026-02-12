#!/usr/bin/env python3
"""
xcstrings-convert.py

Bridge Apple .xcstrings files and Android-style strings.xml files (per-locale).

This is used because Weblate does not support .xcstrings and .xcstrings maps
better to strings.xml than to .strings and .stringsdict.

Commands:
  export-strings  Export .xcstrings localizations into Android-style XML files.
  import-strings  Import Android-style XML translations back into .xcstrings.

Examples:
  Export into ./res:
    scripts/xcstrings-convert.py export-strings

  Export into a custom directory:
    scripts/xcstrings-convert.py export-strings --output-dir /tmp/res

  Import non-English locale XML files from ./res and overwrite input:
    scripts/xcstrings-convert.py import-strings --res-dir res

  Import and write to a separate output file:
    scripts/xcstrings-convert.py import-strings --res-dir res --output /tmp/Localizable.xcstrings
"""

import argparse
import contextlib
import io
import json
import os
import re
import sys
import tempfile
import unittest
import xml.etree.ElementTree as ET
from typing import Dict, List, Optional, Tuple, Union


DEFAULT_XCSTRINGS_PATH = "VaultCommon/Resources/Localizable.xcstrings"


# Common iOS format tokens (plus positional forms) seen in .xcstrings.
IOS_PLACEHOLDER_RE = re.compile(
    r"(?<!%)%(?:\d+\$)?(?:@|(?:ll|l|hh|h|z|t|j|q)?[diuoxXfFeEgGaAcCsS])"
)
ANDROID_ORDERED_PLACEHOLDER_RE = re.compile(r"(?<!%)%(\d+)\$([sdf])")
IMPORT_PLACEHOLDER_RE = re.compile(
    r"(?<!%)%(?:([0-9]+)\$([sdf])|(@|(?:ll|l|hh|h|z|t|j|q)?[diuoxXfFeEgGaAcCsS]))"
)


def warn(message: str) -> None:
    print(f"warning: {message}", file=sys.stderr)


def error(message: str) -> None:
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_xcstrings(path: str) -> Tuple[Dict, bool]:
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    has_trailing_newline = text.endswith("\n")
    return json.loads(text), has_trailing_newline


def dump_xcstrings(data: Dict, output_path: str, trailing_newline: bool) -> None:
    payload = json.dumps(
        data,
        ensure_ascii=False,
        indent=2,
        separators=(",", " : "),
    )
    if trailing_newline:
        payload += "\n"
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(payload)


def get_locales(data: Dict) -> List[str]:
    locales = set()
    for entry in data.get("strings", {}).values():
        for locale in entry.get("localizations", {}).keys():
            locales.add(locale)
    return sorted(locales)


def xml_path_for_locale(res_dir: str, locale: str) -> str:
    if locale == "en":
        return os.path.join(res_dir, "values", "strings.xml")
    return os.path.join(res_dir, f"values-{locale}", "strings.xml")


def classify_ios_token(token: str) -> str:
    core = token
    if core.startswith("%"):
        core = core[1:]

    if "$" in core:
        core = core.split("$", 1)[1]

    if core == "@" or core.endswith("s") or core.endswith("S"):
        return "s"

    conv = core[-1]
    if conv in "diuoxXcC":
        return "d"
    if conv in "fFeEgGaA":
        return "f"

    # Safe default for unknown/object-like types.
    return "s"


def extract_ios_placeholders(value: str) -> List[str]:
    assert isinstance(value, str)
    return IOS_PLACEHOLDER_RE.findall(value)


def build_placeholder_mapping(english_value: str) -> List[Dict[str, str]]:
    placeholders = extract_ios_placeholders(english_value)
    mapping: List[Dict[str, str]] = []
    for i, source in enumerate(placeholders, start=1):
        mapping.append({"source": source, "dest": f"%{i}${classify_ios_token(source)}"})
    return mapping


def replace_ordered(
    value: str, mapping: List[Dict[str, str]], reverse: bool = False
) -> str:
    def to_positional_ios_token(token: str, index: int) -> str:
        if "$" in token:
            return token
        if not token.startswith("%"):
            return token
        return f"%{index}${token[1:]}"

    assert isinstance(value, str)
    result = value
    if reverse:
        # Import path: resolve Android placeholders by explicit index so
        # translators can reorder arguments safely (e.g. %2$s ... %1$s).
        #
        # Xcode rule: use positional iOS tokens when a string has multiple
        # placeholders; keep non-positional tokens only for single-placeholder
        # strings.
        requires_positional = len(mapping) > 1

        index_to_source: Dict[int, str] = {}
        for idx, item in enumerate(mapping, start=1):
            source = item["source"]
            if requires_positional:
                source = to_positional_ios_token(source, idx)
            index_to_source[idx] = source

        def repl(match: re.Match[str]) -> str:
            idx = int(match.group(1))
            source = index_to_source.get(idx)
            if source is None:
                return match.group(0)
            return source

        return ANDROID_ORDERED_PLACEHOLDER_RE.sub(repl, result)

    # Export path: convert iOS placeholders to Android ordered placeholders.
    # Supports both non-positional (%@) and positional (%2$@) iOS placeholders.
    next_index = 1

    def repl(match: re.Match[str]) -> str:
        nonlocal next_index
        token = match.group(0)
        core = token[1:]
        if "$" in core:
            index_part, _ = core.split("$", 1)
            index = int(index_part)
        else:
            index = next_index
            next_index += 1
        return f"%{index}${classify_ios_token(token)}"

    return IOS_PLACEHOLDER_RE.sub(repl, result)


def encode_android_newlines(value: str) -> str:
    assert isinstance(value, str)
    # Escape backslashes first so literal "\n" remains literal on roundtrip.
    value = value.replace("\\", "\\\\")
    return value.replace("\n", "\\n")


def decode_android_newlines(value: str) -> str:
    assert isinstance(value, str)
    # Preserve literal backslash sequences (e.g. "\\n") while decoding "\n".
    parts = value.split("\\\\")
    decoded = [part.replace("\\n", "\n") for part in parts]
    return "\\\\".join(decoded)


def encode_android_quotes(value: str) -> str:
    # Android string resources require apostrophes to be escaped.
    assert isinstance(value, str)
    return value.replace("'", "\\'")


def decode_android_quotes(value: str) -> str:
    assert isinstance(value, str)
    return value.replace("\\'", "'")


def is_empty_translation(value: Optional[str]) -> bool:
    return value is None or value.strip() == ""


def validate_expected_placeholders(
    value: str, mapping: List[Dict[str, str]], context: str
) -> None:
    if not value:
        # ignore untranslated strings
        return
    expected_tokens = [item["dest"] for item in mapping]
    expected_set = set(expected_tokens)
    found_expected = set()
    allowed_by_index = {
        i: item["dest"].split("$", 1)[1] for i, item in enumerate(mapping, start=1)
    }
    for match in IMPORT_PLACEHOLDER_RE.finditer(value):
        idx_part = match.group(1)
        conv = match.group(2)
        legacy = match.group(3)

        if idx_part is None:
            error(
                f"placeholder mismatch in {context}: unsupported placeholder '%{legacy}', expected Android ordered placeholders"
            )

        idx = int(idx_part)
        token = f"%{idx}${conv}"
        expected_conv = allowed_by_index.get(idx)
        if expected_conv is None:
            error(f"placeholder mismatch in {context}: unexpected index '{token}'")
        if expected_conv != conv:
            error(
                f"placeholder mismatch in {context}: unexpected type '{token}', expected '%{idx}${expected_conv}'"
            )
        if token in expected_set:
            found_expected.add(token)

    for expected in expected_tokens:
        if expected not in found_expected:
            error(f"placeholder mismatch in {context}: missing '{expected}'")


def ensure_dir(path: str) -> None:
    os.makedirs(path, exist_ok=True)


def write_xml(resources: ET.Element, path: str) -> None:
    tree = ET.ElementTree(resources)
    ET.indent(tree, space="    ")
    body = ET.tostring(resources, encoding="unicode")
    payload = '<?xml version="1.0" encoding="utf-8"?>\n' + body + "\n"
    with open(path, "w", encoding="utf-8") as f:
        f.write(payload)


def get_locale_simple_value(entry: Dict, locale: str) -> Optional[str]:
    loc = entry.get("localizations", {}).get(locale)
    if not loc:
        return None
    unit = loc.get("stringUnit")
    if not unit:
        return None
    return unit.get("value", "")


def get_locale_plural_map(entry: Dict, locale: str) -> Optional[Dict]:
    loc = entry.get("localizations", {}).get(locale)
    if not loc:
        return None
    return loc.get("variations", {}).get("plural")


def get_english_plural_other(en_plural: Dict, key: str) -> str:
    """Return English 'other' plural text used as placeholder mapping source."""
    other = en_plural.get("other")
    if not isinstance(other, dict):
        error(f"missing English plural quantity 'other' for key '{key}'")
    return other.get("stringUnit", {}).get("value", "")


def find_locale_plural_quantities(data: Dict, locale: str) -> Optional[List[str]]:
    """Return plural quantity order from the first locale plural entry."""
    for entry in data.get("strings", {}).values():
        plural = get_locale_plural_map(entry, locale)
        if isinstance(plural, dict) and plural:
            return list(plural.keys())
    return None


def export_strings(xcstrings_path: str, res_dir: str) -> None:
    data, _ = load_xcstrings(xcstrings_path)
    strings = data.get("strings", {})
    warned_plural_fallback_locales = set()

    # Generate one Android-style strings.xml per locale.
    for locale in get_locales(data):
        out_path = xml_path_for_locale(res_dir, locale)
        ensure_dir(os.path.dirname(out_path))
        # Some locales may not define all plural categories on every key. Reuse
        # the first seen category order as a template when needed.
        locale_plural_quantities = find_locale_plural_quantities(data, locale)

        root = ET.Element("resources")

        for key, entry in strings.items():
            source_simple = get_locale_simple_value(entry, "en")
            target_simple = get_locale_simple_value(entry, locale)

            source_plural = get_locale_plural_map(entry, "en")
            target_plural = get_locale_plural_map(entry, locale)

            node = None
            if source_simple is not None:
                # For non-English locales, export only real translations.
                if locale != "en" and is_empty_translation(target_simple):
                    continue
                if target_simple is not None:
                    # Normalize placeholders to Android ordered form (%@/%2$@,
                    # %d/%1$d -> %1$s, %2$d, ...).
                    mapping = build_placeholder_mapping(source_simple)
                    value = replace_ordered(target_simple, mapping, reverse=False)
                    value = encode_android_newlines(value)
                    value = encode_android_quotes(value)
                else:
                    # Keep English source strings even when empty/missing to
                    # preserve keys.
                    value = ""
                node = ET.Element("string", {"name": key})
                node.text = value
            elif source_plural is not None:
                # Use English "other" as placeholder schema source for all
                # quantities.
                source_val = get_english_plural_other(source_plural, key)
                if locale == "en":
                    node = ET.Element("plurals", {"name": key})
                    quantities = list(source_plural.keys())
                elif target_plural is not None:
                    node = ET.Element("plurals", {"name": key})
                    # Export only quantities that actually have a translation
                    # value.
                    quantities = [
                        quantity
                        for quantity, qnode in target_plural.items()
                        if not is_empty_translation(
                            qnode.get("stringUnit", {}).get("value")
                        )
                    ]
                elif locale_plural_quantities:
                    # Missing translation in this locale: skip export.
                    continue
                else:
                    if locale != "en" and locale not in warned_plural_fallback_locales:
                        warn(
                            f"no plural template found for locale '{locale}', falling back to English plural quantities"
                        )
                        warned_plural_fallback_locales.add(locale)
                    # Missing translation in this locale: skip export.
                    continue

                for quantity in quantities:
                    qnode = target_plural.get(quantity, {}) if target_plural else {}
                    target_val = qnode.get("stringUnit", {}).get("value", "")
                    if target_val:
                        # Normalize placeholders to Android ordered form
                        # (%@/%2$@, %d/%1$d -> %1$s, %2$d, ...).
                        mapping = build_placeholder_mapping(source_val)
                        value = replace_ordered(target_val, mapping, reverse=False)
                        value = encode_android_newlines(value)
                        value = encode_android_quotes(value)
                    else:
                        value = ""
                    item_el = ET.SubElement(node, "item", {"quantity": quantity})
                    item_el.text = value
                # If every quantity was empty for this locale, do not emit the
                # <plurals> key.
                if locale != "en" and len(node) == 0:
                    continue
            else:
                continue

            if locale == "en":
                # Comments are only added for English strings.
                comment = entry.get("comment")
                if comment:
                    root.append(ET.Comment(f" {comment} "))
            root.append(node)

        write_xml(root, out_path)


def parse_xml(path: str) -> ET.Element:
    tree = ET.parse(path)
    root = tree.getroot()
    if root.tag != "resources":
        error(f"invalid XML root in '{path}': expected <resources>")
    return root


def get_non_en_locales_from_res_dir(res_dir: str) -> List[str]:
    locales = set()
    if not os.path.isdir(res_dir):
        return []
    for name in os.listdir(res_dir):
        # Only process directories that start with "values-". English (base)
        # values is just "values", not "values-en" and is therefore skipped.
        if not name.startswith("values-"):
            continue
        locale = name[len("values-") :]
        if not locale:
            continue
        if locale == "en":
            continue
        xml_path = os.path.join(res_dir, name, "strings.xml")
        if os.path.isfile(xml_path):
            locales.add(locale)
    return sorted(locales)


def import_strings(
    xcstrings_path: str, res_dir: str, output_path: Optional[str]
) -> None:
    data, trailing_newline = load_xcstrings(xcstrings_path)
    strings = data.get("strings", {})

    # Import all non-English locales known either by xcstrings or by existing
    # XML folders. We do not import English, as it is the source of truth.
    # Values are merged into existing Localizations.xcstrings file, the file is
    # never regenerated.
    locales = sorted(
        set([locale for locale in get_locales(data) if locale != "en"])
        | set(get_non_en_locales_from_res_dir(res_dir))
    )
    if not locales:
        out = output_path or xcstrings_path
        dump_xcstrings(data, out, trailing_newline)
        return

    for locale in locales:
        xml_path = xml_path_for_locale(res_dir, locale)
        if not os.path.exists(xml_path):
            warn(f"missing locale XML for '{locale}': {xml_path}; skipping locale")
            continue
        root = parse_xml(xml_path)

        # Go through all <string> and <plurals> in the XML and update the
        # xcstrings file.
        for child in list(root):
            if child.tag == "string":
                key = child.attrib.get("name")
                if not key:
                    warn(f"skipping <string> without name in {xml_path}")
                    continue

                entry = strings.get(key)
                if entry is None:
                    warn(f"key '{key}' in {xml_path} not found in xcstrings; skipping")
                    continue

                en_simple = get_locale_simple_value(entry, "en")
                if en_simple is None:
                    warn(
                        f"key '{key}' in {xml_path} is not a simple string in xcstrings; skipping"
                    )
                    continue

                xml_text = child.text if child.text is not None else ""
                xml_value = decode_android_newlines(xml_text)
                xml_value = decode_android_quotes(xml_value)
                # Do not import empty values; keep existing xcstrings
                # state/value unchanged.
                if is_empty_translation(xml_value):
                    continue
                # Validate and convert placeholders back to xcstrings style
                # (%1$s -> %@, %2$@ or %1$d).
                mapping = build_placeholder_mapping(en_simple)
                validate_expected_placeholders(
                    xml_value, mapping, f"key '{key}' locale '{locale}'"
                )
                ios_value = replace_ordered(xml_value, mapping, reverse=True)

                # Ensure locale/stringUnit nodes exist only for non-empty imports.
                localizations = entry.setdefault("localizations", {})
                loc = localizations.get(locale)
                if not isinstance(loc, dict):
                    loc = {}
                    localizations[locale] = loc
                node = loc.get("stringUnit")
                if not isinstance(node, dict):
                    node = {}
                    loc["stringUnit"] = node

                # "state" must come before "value" in the JSON.
                node["state"] = "translated"
                node["value"] = ios_value

            elif child.tag == "plurals":
                key = child.attrib.get("name")
                if not key:
                    warn(f"skipping <plurals> without name in {xml_path}")
                    continue

                entry = strings.get(key)
                if entry is None:
                    warn(f"key '{key}' in {xml_path} not found in xcstrings; skipping")
                    continue

                en_plural = get_locale_plural_map(entry, "en")
                if en_plural is None:
                    warn(
                        f"key '{key}' in {xml_path} is not plural in xcstrings; skipping"
                    )
                    continue
                en_value = get_english_plural_other(en_plural, key)

                for item in child.findall("item"):
                    quantity = item.attrib.get("quantity")
                    if not quantity:
                        warn(
                            f"skipping <item> without quantity in key '{key}' ({xml_path})"
                        )
                        continue

                    xml_text = item.text if item.text is not None else ""
                    xml_value = decode_android_newlines(xml_text)
                    xml_value = decode_android_quotes(xml_value)
                    # Ignore empty plural quantities so we do not overwrite with
                    # blank translations.
                    if is_empty_translation(xml_value):
                        continue
                    # Validate and convert placeholders back to xcstrings style
                    # (%1$s -> %@, %2$@ or %1$d).
                    mapping = build_placeholder_mapping(en_value)
                    validate_expected_placeholders(
                        xml_value,
                        mapping,
                        f"key '{key}' locale '{locale}' quantity '{quantity}'",
                    )
                    ios_value = replace_ordered(xml_value, mapping, reverse=True)

                    # Ensure locale/variations/plural nodes exist only when a
                    # non-empty plural item is imported.
                    localizations = entry.setdefault("localizations", {})
                    loc = localizations.get(locale)
                    if not isinstance(loc, dict):
                        loc = {}
                        localizations[locale] = loc
                    variations = loc.get("variations")
                    if not isinstance(variations, dict):
                        variations = {}
                        loc["variations"] = variations
                    plural = variations.get("plural")
                    if not isinstance(plural, dict):
                        plural = {}
                        variations["plural"] = plural

                    quantity_node = plural.get(quantity)
                    if not isinstance(quantity_node, dict):
                        quantity_node = {}
                        plural[quantity] = quantity_node
                    node = quantity_node.get("stringUnit")
                    if not isinstance(node, dict):
                        node = {}
                        quantity_node["stringUnit"] = node

                    # "state" must come before "value" in the JSON.
                    node["state"] = "translated"
                    node["value"] = ios_value

            elif child.tag is ET.Comment:
                continue
            else:
                warn(f"unsupported XML node <{child.tag}> in {xml_path}; skipping")

    out = output_path or xcstrings_path
    dump_xcstrings(data, out, trailing_newline)


LOCALIZABLE_XCSTRINGS_FIXTURE = """{
  "sourceLanguage" : "en",
  "strings" : {
    "example.simple" : {
      "comment" : "Simple comment.",
      "extractionState" : "extracted_with_value",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "new",
            "value" : "Get started"
          }
        },
        "sl" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Želim pričeti"
          }
        }
      }
    },
    "example.simple.multiple_args" : {
      "comment" : "Simple example with multiple arguments.",
      "extractionState" : "extracted_with_value",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "new",
            "value" : "%1$u / %2$u done"
          }
        },
        "sl" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "%1$u / %2$u končano"
          }
        }
      }
    },
    "example.plurals" : {
      "comment" : "Plurals example.",
      "localizations" : {
        "en" : {
          "variations" : {
            "plural" : {
              "one" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u item"
                }
              },
              "other" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u items"
                }
              }
            }
          }
        },
        "sl" : {
          "variations" : {
            "plural" : {
              "few" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u elementi"
                }
              },
              "one" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u element"
                }
              },
              "other" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u elementov"
                }
              },
              "two" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u elementa"
                }
              }
            }
          }
        }
      }
    }
  },
  "version" : "1.1"
}
"""
EXPECTED_EXPORT_VALUES_XML = """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <!-- Simple comment. -->
    <string name="example.simple">Get started</string>
    <!-- Simple example with multiple arguments. -->
    <string name="example.simple.multiple_args">%1$d / %2$d done</string>
    <!-- Plurals example. -->
    <plurals name="example.plurals">
        <item quantity="one">%1$d item</item>
        <item quantity="other">%1$d items</item>
    </plurals>
</resources>
"""
EXPECTED_EXPORT_VALUES_SL_XML = """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple">Želim pričeti</string>
    <string name="example.simple.multiple_args">%1$d / %2$d končano</string>
    <plurals name="example.plurals">
        <item quantity="few">%1$d elementi</item>
        <item quantity="one">%1$d element</item>
        <item quantity="other">%1$d elementov</item>
        <item quantity="two">%1$d elementa</item>
    </plurals>
</resources>
"""
IMPORT_VALUES_SL_XML_FIXTURE = """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple"></string>
    <string name="example.simple.multiple_args">%1$d / %2$d prevedeno</string>
    <plurals name="example.plurals">
        <item quantity="one"></item>
        <item quantity="other">%1$d predmeti</item>
    </plurals>
</resources>
"""
IMPORT_VALUES_SL_FULL_XML_FIXTURE = """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple">Želim pričeti</string>
    <string name="example.simple.multiple_args">%1$d / %2$d končano</string>
    <plurals name="example.plurals">
        <item quantity="few">%1$d elementi</item>
        <item quantity="one">%1$d element</item>
        <item quantity="other">%1$d elementov</item>
        <item quantity="two">%1$d elementa</item>
    </plurals>
</resources>
"""
EXPECTED_IMPORT_XCSTRINGS_JSON = """{
  "sourceLanguage" : "en",
  "strings" : {
    "example.simple" : {
      "comment" : "Simple comment.",
      "extractionState" : "extracted_with_value",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "new",
            "value" : "Get started"
          }
        },
        "sl" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "Želim pričeti"
          }
        }
      }
    },
    "example.simple.multiple_args" : {
      "comment" : "Simple example with multiple arguments.",
      "extractionState" : "extracted_with_value",
      "localizations" : {
        "en" : {
          "stringUnit" : {
            "state" : "new",
            "value" : "%1$u / %2$u done"
          }
        },
        "sl" : {
          "stringUnit" : {
            "state" : "translated",
            "value" : "%1$u / %2$u prevedeno"
          }
        }
      }
    },
    "example.plurals" : {
      "comment" : "Plurals example.",
      "localizations" : {
        "en" : {
          "variations" : {
            "plural" : {
              "one" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u item"
                }
              },
              "other" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u items"
                }
              }
            }
          }
        },
        "sl" : {
          "variations" : {
            "plural" : {
              "few" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u elementi"
                }
              },
              "one" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u element"
                }
              },
              "other" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u predmeti"
                }
              },
              "two" : {
                "stringUnit" : {
                  "state" : "translated",
                  "value" : "%u elementa"
                }
              }
            }
          }
        }
      }
    }
  },
  "version" : "1.1"
}
"""


class TestXcstringsConvert(unittest.TestCase):
    def write_fixture(self, path: str, data: Union[Dict, str]) -> None:
        if isinstance(data, str):
            payload = data
        else:
            payload = json.dumps(
                data, ensure_ascii=False, indent=2, separators=(",", " : ")
            )
            payload += "\n"
        with open(path, "w", encoding="utf-8") as f:
            f.write(payload)

    def parse_res_xml(self, path: str) -> ET.Element:
        tree = ET.parse(path)
        return tree.getroot()

    def find_string(self, root: ET.Element, name: str) -> Optional[ET.Element]:
        for child in root.findall("string"):
            if child.attrib.get("name") == name:
                return child
        return None

    def find_plurals(self, root: ET.Element, name: str) -> Optional[ET.Element]:
        for child in root.findall("plurals"):
            if child.attrib.get("name") == name:
                return child
        return None

    def test_export_from_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            self.write_fixture(xcstrings_path, LOCALIZABLE_XCSTRINGS_FIXTURE)

            export_strings(xcstrings_path, res_dir)

            en_path = xml_path_for_locale(res_dir, "en")
            sl_path = xml_path_for_locale(res_dir, "sl")
            with open(en_path, "r", encoding="utf-8") as f:
                self.assertEqual(f.read(), EXPECTED_EXPORT_VALUES_XML)
            with open(sl_path, "r", encoding="utf-8") as f:
                self.assertEqual(f.read(), EXPECTED_EXPORT_VALUES_SL_XML)

    def test_export_skips_missing_non_english_values(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        fixture["strings"]["example.simple"]["localizations"]["hr"] = {
            "stringUnit": {"state": "translated", "value": ""}
        }
        fixture["strings"]["example.plurals"]["localizations"]["hr"] = {
            "variations": {
                "plural": {
                    "one": {"stringUnit": {"state": "translated", "value": ""}},
                    "other": {"stringUnit": {"state": "translated", "value": ""}},
                }
            }
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            self.write_fixture(xcstrings_path, fixture)

            export_strings(xcstrings_path, res_dir)

            hr_root = self.parse_res_xml(xml_path_for_locale(res_dir, "hr"))
            self.assertIsNone(self.find_string(hr_root, "example.simple"))
            self.assertIsNone(self.find_plurals(hr_root, "example.plurals"))

    def test_import_skips_empty_values(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            sl_dir = os.path.join(res_dir, "values-sl")
            os.makedirs(sl_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(sl_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(IMPORT_VALUES_SL_XML_FIXTURE)

            import_strings(xcstrings_path, res_dir, None)
            with open(xcstrings_path, "r", encoding="utf-8") as f:
                self.assertEqual(f.read(), EXPECTED_IMPORT_XCSTRINGS_JSON)

    def test_import_fails_on_unexpected_placeholder_index(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            sl_dir = os.path.join(res_dir, "values-sl")
            os.makedirs(sl_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(sl_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(
                    """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple.multiple_args">%1$d / %2$d / %3$s končano</string>
</resources>
"""
                )

            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                with self.assertRaises(SystemExit):
                    import_strings(xcstrings_path, res_dir, None)
            self.assertIn("unexpected index '%3$s'", stderr.getvalue())

    def test_import_fails_on_unexpected_placeholder_type(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            sl_dir = os.path.join(res_dir, "values-sl")
            os.makedirs(sl_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(sl_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(
                    """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple.multiple_args">%1$s / %2$d končano</string>
</resources>
"""
                )

            stderr = io.StringIO()
            with contextlib.redirect_stderr(stderr):
                with self.assertRaises(SystemExit):
                    import_strings(xcstrings_path, res_dir, None)
            self.assertIn("unexpected type '%1$s', expected '%1$d'", stderr.getvalue())

    def test_import_fails_on_non_ordered_placeholders_when_source_has_none(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        fixture["strings"]["example.no_placeholders"] = {
            "comment": "No placeholders expected.",
            "extractionState": "extracted_with_value",
            "localizations": {
                "en": {"stringUnit": {"state": "new", "value": "No placeholders here"}}
            },
        }

        for bad_value in ["Hi %@", "Hi %s"]:
            with self.subTest(bad_value=bad_value):
                with tempfile.TemporaryDirectory() as tmpdir:
                    xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
                    res_dir = os.path.join(tmpdir, "res")
                    sl_dir = os.path.join(res_dir, "values-sl")
                    os.makedirs(sl_dir, exist_ok=True)
                    self.write_fixture(xcstrings_path, fixture)

                    with open(
                        os.path.join(sl_dir, "strings.xml"), "w", encoding="utf-8"
                    ) as f:
                        f.write(
                            f"""<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.no_placeholders">{bad_value}</string>
</resources>
"""
                        )

                    stderr = io.StringIO()
                    with contextlib.redirect_stderr(stderr):
                        with self.assertRaises(SystemExit):
                            import_strings(xcstrings_path, res_dir, None)
                    self.assertIn("unsupported placeholder", stderr.getvalue())

    def test_import_ignores_english_even_if_values_en_exists(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            values_dir = os.path.join(res_dir, "values")
            values_en_dir = os.path.join(res_dir, "values-en")
            os.makedirs(values_dir, exist_ok=True)
            os.makedirs(values_en_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(values_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(
                    """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple">CHANGED EN BASE</string>
</resources>
"""
                )
            with open(
                os.path.join(values_en_dir, "strings.xml"), "w", encoding="utf-8"
            ) as f:
                f.write(
                    """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple">CHANGED EN ALIAS</string>
</resources>
"""
                )

            import_strings(xcstrings_path, res_dir, None)
            with open(xcstrings_path, "r", encoding="utf-8") as f:
                content = f.read()
            self.assertEqual(content, LOCALIZABLE_XCSTRINGS_FIXTURE)

    def test_reverse_single_placeholder_stays_non_positional(self) -> None:
        mapping = build_placeholder_mapping("Prefix %@")
        self.assertEqual(
            replace_ordered("Prefix %1$s", mapping, reverse=True), "Prefix %@"
        )

    def test_reverse_multiple_placeholders_are_positional(self) -> None:
        mapping = build_placeholder_mapping("%@ / %d")
        self.assertEqual(
            replace_ordered("%1$s / %2$d", mapping, reverse=True), "%1$@ / %2$d"
        )
        self.assertEqual(
            replace_ordered("%2$d / %1$s", mapping, reverse=True), "%2$d / %1$@"
        )

    def test_import_new_language_matches_existing_language_shape(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        for entry in fixture["strings"].values():
            entry.get("localizations", {}).pop("sl", None)

        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            sl_dir = os.path.join(res_dir, "values-sl")
            os.makedirs(sl_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(sl_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(IMPORT_VALUES_SL_FULL_XML_FIXTURE)

            import_strings(xcstrings_path, res_dir, None)

            with open(xcstrings_path, "r", encoding="utf-8") as f:
                self.assertEqual(f.read(), LOCALIZABLE_XCSTRINGS_FIXTURE)

    def test_import_does_not_create_empty_locale_skeleton(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        for entry in fixture["strings"].values():
            entry.get("localizations", {}).pop("hr", None)

        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            hr_dir = os.path.join(res_dir, "values-hr")
            os.makedirs(hr_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(hr_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(
                    """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.simple"></string>
    <plurals name="example.plurals">
        <item quantity="one"></item>
        <item quantity="other"></item>
    </plurals>
</resources>
"""
                )

            import_strings(xcstrings_path, res_dir, None)

            imported, _ = load_xcstrings(xcstrings_path)
            for entry in imported["strings"].values():
                self.assertNotIn("hr", entry.get("localizations", {}))

    def test_export_unicode_newlines_and_quotes(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        fixture["strings"]["example.unicode"] = {
            "comment": "Unicode + newline + quote.",
            "extractionState": "extracted_with_value",
            "localizations": {
                "en": {
                    "stringUnit": {
                        "state": "new",
                        "value": "First line\nDon't lose %@",
                    }
                },
                "sl": {
                    "stringUnit": {
                        "state": "translated",
                        "value": "Prva vrstica\nŽelim, da %@ ne izgine",
                    }
                },
            },
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            self.write_fixture(xcstrings_path, fixture)

            export_strings(xcstrings_path, res_dir)

            sl_root = self.parse_res_xml(xml_path_for_locale(res_dir, "sl"))
            node = self.find_string(sl_root, "example.unicode")
            self.assertIsNotNone(node)
            self.assertEqual(node.text, "Prva vrstica\\nŽelim, da %1$s ne izgine")

            en_root = self.parse_res_xml(xml_path_for_locale(res_dir, "en"))
            node = self.find_string(en_root, "example.unicode")
            self.assertIsNotNone(node)
            self.assertEqual(node.text, "First line\\nDon\\'t lose %1$s")

    def test_import_unicode_newlines_and_quotes(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        fixture["strings"]["example.unicode"] = {
            "comment": "Unicode + newline + quote.",
            "extractionState": "extracted_with_value",
            "localizations": {
                "en": {
                    "stringUnit": {
                        "state": "new",
                        "value": "First line\nDon't lose %@",
                    }
                }
            },
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            sl_dir = os.path.join(res_dir, "values-sl")
            os.makedirs(sl_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(sl_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(
                    """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.unicode">Prva vrstica\\nDon\\'t lose %1$s</string>
</resources>
"""
                )

            import_strings(xcstrings_path, res_dir, None)

            imported, _ = load_xcstrings(xcstrings_path)
            value = imported["strings"]["example.unicode"]["localizations"]["sl"][
                "stringUnit"
            ]["value"]
            self.assertEqual(value, "Prva vrstica\nDon't lose %@")

    def test_import_preserves_literal_backslash_n(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        fixture["strings"]["example.literal_backslash_n"] = {
            "comment": "Literal backslash n should stay literal.",
            "extractionState": "extracted_with_value",
            "localizations": {
                "en": {"stringUnit": {"state": "new", "value": "Path: %@"}},
            },
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            sl_dir = os.path.join(res_dir, "values-sl")
            os.makedirs(sl_dir, exist_ok=True)
            self.write_fixture(xcstrings_path, fixture)

            with open(os.path.join(sl_dir, "strings.xml"), "w", encoding="utf-8") as f:
                f.write(
                    """<?xml version="1.0" encoding="utf-8"?>
<resources>
    <string name="example.literal_backslash_n">Path\\\\n%1$s</string>
</resources>
"""
                )

            import_strings(xcstrings_path, res_dir, None)

            imported, _ = load_xcstrings(xcstrings_path)
            value = imported["strings"]["example.literal_backslash_n"]["localizations"][
                "sl"
            ]["stringUnit"]["value"]
            self.assertEqual(value, "Path\\\\n%@")

    def test_dump_xcstrings_writes_unicode_without_ascii_escaping(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            self.write_fixture(xcstrings_path, fixture)

            with open(xcstrings_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.assertIn("Želim pričeti", content)
            self.assertNotIn("\\u017d", content.lower())

    def test_export_positional_target_with_non_positional_source(self) -> None:
        fixture = json.loads(LOCALIZABLE_XCSTRINGS_FIXTURE)
        fixture["strings"]["example.positional_target"] = {
            "comment": "Target can use positional iOS placeholders.",
            "extractionState": "extracted_with_value",
            "localizations": {
                "en": {"stringUnit": {"state": "new", "value": "%@ %@ done"}},
                "sl": {"stringUnit": {"state": "translated", "value": "%2$@ in %1$@"}},
            },
        }

        with tempfile.TemporaryDirectory() as tmpdir:
            xcstrings_path = os.path.join(tmpdir, "Localizable.xcstrings")
            res_dir = os.path.join(tmpdir, "res")
            self.write_fixture(xcstrings_path, fixture)

            export_strings(xcstrings_path, res_dir)

            sl_root = self.parse_res_xml(xml_path_for_locale(res_dir, "sl"))
            node = self.find_string(sl_root, "example.positional_target")
            self.assertIsNotNone(node)
            self.assertEqual(node.text, "%2$s in %1$s")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Bridge Apple .xcstrings and Android strings.xml translation files.",
        epilog=(
            "Examples:\n"
            "  %(prog)s export-strings\n"
            "  %(prog)s export-strings --output-dir /tmp/res\n"
            "  %(prog)s import-strings --res-dir res\n"
            "  %(prog)s import-strings --res-dir res --output /tmp/Localizable.xcstrings"
        ),
        formatter_class=argparse.RawTextHelpFormatter,
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    export_parser = subparsers.add_parser(
        "export-strings", help="Export .xcstrings into Android strings.xml files"
    )
    export_parser.add_argument(
        "--xcstrings",
        default=DEFAULT_XCSTRINGS_PATH,
        help=f"Path to .xcstrings file (default: {DEFAULT_XCSTRINGS_PATH})",
    )
    export_parser.add_argument(
        "--output-dir", default="res", help="Output res directory (default: res)"
    )

    import_parser = subparsers.add_parser(
        "import-strings", help="Import Android strings.xml into .xcstrings"
    )
    import_parser.add_argument(
        "--xcstrings",
        default=DEFAULT_XCSTRINGS_PATH,
        help=f"Path to .xcstrings file (default: {DEFAULT_XCSTRINGS_PATH})",
    )
    import_parser.add_argument(
        "--res-dir", default="res", help="Input res directory (default: res)"
    )
    import_parser.add_argument(
        "--output", help="Output .xcstrings path (default: overwrite input)"
    )
    subparsers.add_parser("test", help="Run script self-tests")

    args = parser.parse_args()

    if args.command == "export-strings":
        export_strings(args.xcstrings, args.output_dir)
    elif args.command == "import-strings":
        import_strings(args.xcstrings, args.res_dir, args.output)
    elif args.command == "test":
        unittest.main(argv=[sys.argv[0]])
    else:
        error(f"unknown command: {args.command}")


if __name__ == "__main__":
    main()
