#!/usr/bin/env python3
"""Static deployment cache contracts: version entry points and transitive module/style imports together."""
import importlib.util
from pathlib import Path
import re
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'web/scripts'))
from asset_versions import rendered
import stage
from sites import SITES


def fixtures(root):
    bodies = {'index.html': b'<script type="module" src="/js/main.js"></script><link href="/css/site.css" rel="stylesheet">',
              'js/main.js': b"import './release.js'; import '../shared/dom.js'; fetch('/downloads.json');",
              'js/release.js': b"export const version = 1;", 'shared/dom.js': b'export const el = 1;',
              'css/site.css': b'body{color:gold}', 'downloads.json': b'{"schema":1}'}
    result = {}
    for name, body in bodies.items():
        path = root/name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(body)
        result[name] = path
    return result


class StaticAssets(unittest.TestCase):
    def test_entry_and_transitive_modules_and_css_share_one_revision(self):
        with tempfile.TemporaryDirectory() as directory:
            mapping = fixtures(Path(directory))
            value = rendered(mapping)
            versions = re.findall(rb'\?v=([0-9a-f]{64})', value['index.html'] + value['js/main.js'])
            self.assertEqual(len(versions), 4)
            self.assertEqual(len(set(versions)), 1)
            self.assertIn(b"fetch('/downloads.json')", value['js/main.js'])
            self.assertEqual(value, rendered(dict(reversed(list(mapping.items())))))
            self.assertNotIn(b'?v=', mapping['index.html'].read_bytes())

    def test_nested_module_change_invalidates_the_whole_graph_for_warm_visitors(self):
        with tempfile.TemporaryDirectory() as directory:
            mapping = fixtures(Path(directory))
            old = rendered(mapping)
            mapping['js/release.js'].write_bytes(b'export const version = 2;')
            new = rendered(mapping)
            self.assertNotEqual(old['index.html'], new['index.html'])
            self.assertNotEqual(old['js/main.js'], new['js/main.js'])
            self.assertEqual(old['downloads.json'], new['downloads.json'])

    def test_external_links_anchors_and_data_stay_intact_existing_query_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            mapping = fixtures(Path(directory))
            mapping['index.html'].write_bytes(b'''<a href="https://example.com/app.js">external</a><a href="#play">go</a><script src="/js/main.js?mode=preview&v=old#tag"></script>''')
            result = rendered(mapping)['index.html']
            self.assertIn(b'href="https://example.com/app.js"', result)
            self.assertIn(b'href="#play"', result)
            self.assertRegex(result, rb'/js/main.js\?mode=preview&v=[0-9a-f]{64}#tag')
            self.assertNotIn(b'v=old', result)

    def test_manual_staging_uses_the_same_rendered_payload_as_attested_packaging(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            mapping = fixtures(root/'source')
            with patch.object(stage, 'files', return_value=mapping):
                stage.stage(SITES['landing'], root/'output')
            self.assertEqual({name: (root/'output'/name).read_bytes() for name in mapping}, rendered(mapping))
            spec = importlib.util.spec_from_file_location('static_packager', ROOT/'scripts/package-static-sites.py')
            package = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(package)
            self.assertIs(package.rendered, rendered)


if __name__ == '__main__': unittest.main()
