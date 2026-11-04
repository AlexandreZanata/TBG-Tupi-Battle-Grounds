"""Compare exported base GLBs with the same Blender lighting and camera."""

from pathlib import Path
import bpy
from mathutils import Vector

ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/"exports"/"renders"
OUT.mkdir(parents=True, exist_ok=True)


def aim(obj,target):
    obj.rotation_euler=(Vector(target)-obj.location).to_track_quat("-Z","Y").to_euler()


def light(name,location,color,power,size):
    data=bpy.data.lights.new(name,"AREA")
    data.color=color
    data.energy=power
    data.shape="DISK"
    data.size=size
    obj=bpy.data.objects.new(name,data)
    bpy.context.scene.collection.objects.link(obj)
    obj.location=location
    aim(obj,(0,0,.64))


def studio(location, target, scale, size, output):
    bpy.ops.mesh.primitive_plane_add(size=200)
    floor=bpy.context.object
    floor.location.z=-.027
    mat=bpy.data.materials.new("Slate studio")
    mat.use_nodes=True
    shade=mat.node_tree.nodes["Principled BSDF"]
    shade.inputs["Base Color"].default_value=(.034,.055,.067,1)
    shade.inputs["Roughness"].default_value=.72
    floor.data.materials.append(mat)
    light("Warm key",(-3,-4,5),(1,.83,.67),390,3.0)
    light("Cool rim",(2.8,2,3.4),(.56,.78,1),500,2.4)
    light("Fill",(1.2,-4,1.8),(.72,.87,1),98,3.5)
    bpy.ops.object.camera_add(location=location)
    camera=bpy.context.object
    aim(camera,target)
    camera.data.type="ORTHO"
    camera.data.ortho_scale=scale
    scene=bpy.context.scene
    scene.camera=camera
    scene.render.engine="CYCLES"
    scene.cycles.samples=12
    scene.cycles.use_denoising=True
    scene.render.threads_mode="FIXED"
    scene.render.threads=2
    scene.render.resolution_x,scene.render.resolution_y=size
    scene.render.resolution_percentage=100
    scene.render.image_settings.file_format="PNG"
    scene.render.filepath=str(output)
    scene.world=bpy.data.worlds.new("Studio")
    scene.world.use_nodes=True
    scene.world.node_tree.nodes["Background"].inputs[0].default_value=(.13,.16,.20,1)
    scene.world.node_tree.nodes["Background"].inputs[1].default_value=.32

def render(unit):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    for method,x in (("parts",-.66),("volume",.66)):
        before=set(bpy.context.scene.objects)
        bpy.ops.import_scene.gltf(filepath=str(ROOT/"exports"/f"{unit}-{method}.glb"))
        roots=[o for o in set(bpy.context.scene.objects)-before if o.parent is None]
        for root in roots:
            root.location.x+=x
    studio((1.18,-6.1,2.05),(.04,0,.67),2.98,(1040,710),OUT/f"blender-native-{unit}.png")
    bpy.ops.render.render(write_still=True)


if __name__=="__main__":
    for unit in ("archer","bonebrute"):
        render(unit)
