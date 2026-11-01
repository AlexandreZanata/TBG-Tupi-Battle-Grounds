#!/usr/bin/env python3
"""Regression cases for complete, strict same-input framebuffer comparison."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from PIL import Image
spec=importlib.util.spec_from_file_location('desktop_ui',Path(__file__).with_name('check-desktop-ui.py'))
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)

class Contract(unittest.TestCase):
    def test_missing_matrix_and_any_pixel_or_identity_change_fail(self):
        contract={'screens':['lobby'],'languages':['en'],'scenarios':[{'viewport':[2,2],'effective_ui_scale':1.0}],'motion_screens':[],'motion_frames':[0],'matching_identity_fields':['source_commit','assets_sha256','cargo_lock_sha256','features','graphics','scene_seed']}
        with tempfile.TemporaryDirectory(prefix='tbg-p68-ui-') as temp:
            directories=[Path(temp)/name for name in ['linux','windows']]
            for directory in directories:
                directory.mkdir();Image.new('RGBA',(2,2),(3,5,7,255)).save(directory/'lobby.png')
                index={'schema':1,'platform':directory.name,'source_commit':'a'*40,'assets_sha256':'b'*64,'cargo_lock_sha256':'c'*64,'binary_sha256':'d'*64,'package_sha256':'e'*64,'features':[],'graphics':{'quality':'high'},'scene_seed':68,'environment':{'os':'test','gpu':'synthetic','driver':'test','backend':'test'},'captures':[{'screen':'lobby','language':'en','viewport':[2,2],'effective_ui_scale':1.0,'frame':0,'file':'lobby.png'}]}
                (directory/'index.json').write_text(json.dumps(index))
            self.assertTrue(module.compare(*directories,contract)['passed'])
            image=Image.open(directories[1]/'lobby.png');image.putpixel((0,0),(4,5,7,255));image.save(directories[1]/'lobby.png')
            self.assertFalse(module.compare(*directories,contract)['passed'])
            Image.new('RGBA',(2,2),(3,5,7,255)).save(directories[1]/'lobby.png')
            index=json.loads((directories[1]/'index.json').read_text());index['source_commit']='f'*40;(directories[1]/'index.json').write_text(json.dumps(index))
            self.assertFalse(module.compare(*directories,contract)['passed'])
            index['source_commit']='a'*40;index['captures']=[];(directories[1]/'index.json').write_text(json.dumps(index))
            self.assertFalse(module.compare(*directories,contract)['passed'])
    def test_capture_paths_cannot_escape_the_package(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);(root/'captures').mkdir();(root/'outside.png').write_bytes(b'not a capture')
            with self.assertRaises(ValueError):module.contained(root/'captures','../outside.png')

if __name__=='__main__':unittest.main()
