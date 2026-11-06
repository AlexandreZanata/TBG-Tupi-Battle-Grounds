"""Render one review clip from the saved auxiliary Blender timeline."""

import argparse
from pathlib import Path
import sys

import bpy


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("camera",choices=("Mineiro","Mercador"))
    parser.add_argument("output",type=Path)
    parser.add_argument("--first",type=int,default=1)
    parser.add_argument("--last",type=int)
    parser.add_argument("--step",type=int,default=2)
    opts = parser.parse_args(sys.argv[sys.argv.index("--")+1:])
    out = opts.output.resolve()
    out.mkdir(parents=True,exist_ok=True)
    scene = bpy.context.scene
    scene.camera = bpy.data.objects[opts.camera]
    scene.frame_start = opts.first
    if opts.last is not None:
        scene.frame_end = opts.last
    scene.frame_step = opts.step
    scene.render.engine = "BLENDER_WORKBENCH"
    scene.display.render_aa = "FXAA"
    shading = scene.display.shading
    shading.light = "STUDIO"
    shading.color_type = "MATERIAL"
    shading.show_shadows = True
    shading.shadow_intensity = .20
    shading.show_cavity = True
    shading.cavity_type = "WORLD"
    scene.view_settings.look = "None"
    scene.view_settings.exposure = .54
    scene.render.resolution_x,scene.render.resolution_y = 640,400
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "PNG"
    scene.render.filepath = str(out/"frame-")
    bpy.ops.render.render(animation=True)


if __name__=="__main__":
    main()
