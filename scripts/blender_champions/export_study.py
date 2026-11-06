"""Build editable Blender scenes and matching Bevy mesh buffers."""

from collections import defaultdict
import json
from pathlib import Path
from struct import pack
import sys
import bpy

SRC=Path(__file__).resolve().parent
sys.path.insert(0,str(SRC))
from characters import archer, brute
from geometry import GLOW_SWATCHES, Model, SWATCH
from rig import NAMES, INDEX, armature, bind_piece, rest_positions
from volume import bind, generate

DEST=SRC.parent/"exports"
FORMS=(("Base",0,0),("A1",1,0),("A2",2,0),("B1",1,1),("B2",2,1))
UNITS={"archer":archer,"bonebrute":brute}
METHODS=("parts","volume")


def collection(name,parent):
    result=bpy.data.collections.new(name)
    parent.children.link(result)
    return result


def blob(meshes,rest):
    payload=bytearray()
    payload.extend(pack("<69f",*(x for joint in NAMES for x in rest[joint])))
    payload.extend(pack("<H",len(meshes)))
    for (layer,node),(verts,indices) in sorted(meshes.items()):
        payload.extend(pack("<BBII",layer,node,len(verts),len(indices)))
        for p,n,rgb,team,weight in verts:
            payload.extend(pack("<6f4B",*p,*n,*rgb,team))
            if layer==2:
                payload.extend(pack("<2Bf",*weight))
        payload.extend(pack("<"+"I"*len(indices),*indices))
    return bytes(payload)


def append_mesh(group, mesh, rgb, team, weights=None, colors=None):
    vertices, indices=group
    known={}
    mesh.calc_loop_triangles()
    normals=mesh.corner_normals
    for triangle in mesh.loop_triangles:
        for vertex,loop in zip(triangle.vertices,triangle.loops,strict=True):
            normal=tuple(normals[loop].vector)
            key=(vertex,*[round(c,5) for c in normal])
            if key not in known:
                p=tuple(mesh.vertices[vertex].co)
                c=colors[vertex] if colors is not None else rgb
                w=weights[vertex] if weights is not None else None
                known[key]=len(vertices)
                vertices.append((p,normal,c,team,w))
            indices.append(known[key])


def groups_for(model, generated=None):
    meshes=defaultdict(lambda:([],[]))
    deps=bpy.context.evaluated_depsgraph_get()
    for part in model.parts:
        obj=part.obj.evaluated_get(deps)
        evaluated=obj.to_mesh()
        key=(int(part.swatch in GLOW_SWATCHES),INDEX[part.joint])
        append_mesh(meshes[key],evaluated,SWATCH[part.swatch][:3],int(part.swatch=="team"))
        obj.to_mesh_clear()
    if generated is not None:
        body,weights,colors=generated
        append_mesh(meshes[2,0],body.data,None,0,weights,colors)
    return meshes


def export_glb(rig, children, path):
    bpy.ops.object.select_all(action="DESELECT")
    for obj in [rig,*children]:
        obj.select_set(True)
    bpy.context.view_layer.objects.active=rig
    bpy.ops.export_scene.gltf(filepath=str(path),export_format="GLB",use_selection=True,
                              export_animations=False,export_vertex_color="MATERIAL")
    bpy.ops.object.select_all(action="DESELECT")


def generate_unit(unit,make):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    rest=rest_positions(unit)
    rig=armature(unit,rest)
    payloads=[]
    summaries={}
    for method in METHODS:
        method_collection=collection(f"{method.upper()} · {unit}",bpy.context.scene.collection)
        summaries[method]={}
        for form,tier,branch in FORMS:
            dest=collection(f"{method.upper()} / {form}",method_collection)
            model=Model(dest)
            make(model,tier,branch)
            body=generate(model,rest) if method=="volume" else None
            groups=groups_for(model,body)
            payloads.append(blob(groups,rest))
            triangles=sum(len(index)//3 for _,index in groups.values())
            summaries[method][form]={"triangles":triangles,"components":len(model.parts),"layers":len(groups)}
            for part in model.parts:
                bind_piece(part.obj,rig,part.joint,rest)
            objects=[part.obj for part in model.parts]
            if body:
                bind(body[0],rig,body[1],body[2])
                objects.append(body[0])
            if form=="Base":
                export_glb(rig,objects,DEST/f"{unit}-{method}.glb")
            dest.hide_viewport=not (form=="Base" and method=="parts")
            dest.hide_render=dest.hide_viewport
            print(unit,method,form,triangles,flush=True)
    for area in bpy.context.screen.areas:
        if area.type=="VIEW_3D":
            area.spaces.active.shading.color_type="MATERIAL"
            area.spaces.active.overlay.show_floor=False
            area.spaces.active.region_3d.view_location=(0,0,.52)
            area.spaces.active.region_3d.view_distance=2.65
    bpy.context.preferences.filepaths.save_version=0
    bpy.ops.wm.save_as_mainfile(filepath=str(DEST/f"{unit}-rig-study.blend"))
    return payloads,summaries


def main():
    DEST.mkdir(exist_ok=True)
    records, report=[],{}
    for unit,make in UNITS.items():
        payloads,summary=generate_unit(unit,make)
        records.extend(payloads)
        report[unit]=summary
    out=DEST/"blender-forms.bin"
    with out.open("wb") as f:
        f.write(b"TBG3")
        for item in records:
            f.write(pack("<I",len(item)))
        for item in records:
            f.write(item)
    (DEST/"mesh-report.json").write_text(json.dumps(report,indent=2)+"\n")

if __name__=="__main__":
    main()
