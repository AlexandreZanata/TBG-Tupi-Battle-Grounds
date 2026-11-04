"""Check the shared lane layout and animation on the evaluated offshore models."""

from math import degrees

import bpy
from mathutils import Vector

from aux_parts import FPS
from offshore_sites import SITES, pivot
from terrain import HOLD_CENTERS, LANE_LENGTH, LANE_WIDTH, rim

SAMPLES=(0,2.2,3.2,5.2,6.47,8.3,11.95,12.75,14.27,16.8,18.1,20.1,
         22.85,23.1,24.37,26.35,28.2,29.0)


def frame(scene,time):
    scene.frame_set(round(time*FPS)+1)


def bounds(obj,matrix):
    return [matrix@Vector(corner) for corner in obj.bound_box]


def clear_lane(corners,label):
    xs,ys,_=zip(*corners,strict=True)
    edge=LANE_WIDTH/2
    assert max(xs)<=0 or min(xs)>=LANE_LENGTH or max(ys)<=-edge or min(ys)>=edge,label


def contains(point,outline):
    x,y=point
    return sum(
        (ay>y)!=(by>y) and x < ax+(bx-ax)*(y-ay)/(by-ay)
        for (ax,ay),(bx,by) in zip(outline,outline[1:]+outline[:1],strict=True)
    )%2==1


def validate_layout(scene,sites,road_name,hold_name,slab_name,keep_name,scenic_layers):
    road=scene.collection.children[road_name]
    slabs=[o for o in road.objects if o.instance_collection.name.startswith(slab_name)]
    assert len(slabs)==72 and all(abs(o.location.z)<1e-6 for o in slabs)
    assert tuple(sorted(o.location.x for o in scene.collection.children[hold_name].objects
                        if o.instance_collection.name==keep_name)) == tuple(
                            Vector((x,0,0)).x for x in HOLD_CENTERS)
    match=scene.objects["Desktop match framing"].data
    assert match.sensor_fit=="VERTICAL" and abs(degrees(match.angle_y)-35)<.001
    outline=rim()
    for name,_,(x,y,_),(ax,ay,bx,by),seat in SITES:
        root=pivot(sites[name],name+" precinct")
        sign=-1 if seat=="red" else 1
        assert root.scale.x==sign
        assert not contains((x+sign*ax,y+ay),outline),(name,"bridge must start offshore")
        assert contains((x+sign*bx,y+by),outline),(name,"bridge must land on the plateau")
    for role in ("Mineiro","Mercador"):
        left=pivot(sites[role+" A"],role+" A precinct")
        right=pivot(sites[role+" B"],role+" B precinct")
        assert abs(left.location.x+right.location.x-LANE_LENGTH)<.001
        assert abs(left.location.y-right.location.y)<.001
        # Worker materials match even when precinct flags identify each seat.
        palette=lambda o: sorted(slot.material.name for child in o.children_recursive
                                  for slot in child.material_slots if slot.material)
        assert palette(pivot(sites[role+" A"],role+" root")) == palette(pivot(sites[role+" B"],role+" root"))
    frame(scene,24.37)
    graph=bpy.context.evaluated_depsgraph_get()
    count=0
    for item in graph.object_instances:
        if item.object.type!="MESH" or not item.parent:
            continue
        parent=item.parent.original
        if not any(g.name in scenic_layers for g in parent.users_collection):
            continue
        clear_lane(bounds(item.object,item.matrix_world),parent.name)
        count+=1
    assert count>0
    return count


def validate_auxiliary_clearance(scene,sites):
    count=0
    for time in SAMPLES:
        frame(scene,time)
        graph=bpy.context.evaluated_depsgraph_get()
        for group in sites.values():
            for item in group.all_objects:
                if item.type!="MESH":
                    continue
                obj=item.evaluated_get(graph)
                if max(obj.dimensions)<.002 or max(abs(v) for v in obj.matrix_world.to_scale())<.002:
                    continue
                clear_lane(bounds(obj,obj.matrix_world),f"{time}s · {group.name} · {item.name}")
                count+=1
    return count


def revealed(scene,obj,early,late):
    frame(scene,early)
    assert max(obj.scale)<.01,obj.name
    frame(scene,late)
    assert min(obj.scale)>.99,obj.name


def mine_action(scene,group,decoration):
    rock=pivot(group,"Rock outcrop pivot")
    snow=pivot(group,decoration)
    assert snow.parent==rock
    revealed(scene,rock,3.5,7.3)
    revealed(scene,pivot(group,"Level 2 timber winch pivot"),16,18)
    revealed(scene,pivot(group,"Level 3 mine headframe pivot"),21,23)
    pick=pivot(group,"Forged pickaxe pivot")
    seam=pivot(group,"Exposed ore seam pivot")
    root=pivot(group,group.name+" precinct")
    point=Vector((.329,0,.199))
    frame(scene,12+.45*1.35)
    lifted=pick.matrix_world@point
    frame(scene,12+.56*1.35)
    contact=pick.matrix_world@point
    assert (contact-lifted).length>.12,group.name
    local=root.matrix_world.inverted()@contact
    assert (local.xy-seam.location.xy).length<.006,(group.name,local,seam.location)
    assert .03 < local.z < .25,local
    return [round(v,4) for v in local]


def merchant_action(scene,group,roof,bow):
    awning=pivot(group,"Canvas awning pivot")
    ship=pivot(group,"Merchant caravel pivot")
    assert pivot(group,roof).parent==awning
    assert pivot(group,bow).parent==ship
    revealed(scene,awning,1.7,2.7)
    revealed(scene,pivot(group,"Level 2 cargo stacks pivot"),9,11)
    revealed(scene,pivot(group,"Level 3 market pavilion pivot"),18,20)
    parcel=pivot(group,"Delivered parcel pivot")
    rope=pivot(group,"Working crane line pivot")
    lid=pivot(group,"Hinged treasure lid pivot")
    start=23.1
    frame(scene,start)
    approach=ship.location.y
    frame(scene,start+.82)
    berth=ship.location.y
    assert berth-approach>1.65,group.name
    frame(scene,start+1.27)
    endpoint=rope.matrix_world@Vector((0,0,-1))
    hook=parcel.matrix_world@Vector((0,0,.12))
    assert (endpoint-hook).length<.022,(group.name,endpoint,hook)
    assert abs(degrees(lid.rotation_euler.x))>68,group.name
    frame(scene,start+3.2)
    assert abs(ship.location.y-approach)<.10 and abs(lid.rotation_euler.x)<.005,group.name
    return round(berth-approach,3)
