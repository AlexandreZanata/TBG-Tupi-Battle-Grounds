"""Keep the three foundry wheels in continuous tooth-ratio synchronization."""

from math import radians, tau

from aux_parts import FPS
from clockwork_mechanics import GEAR_MODULE

DRIVE_TEETH = 24
FOLLOWER_TEETH = 16
DRIVE_SPEED = radians(7)
AXLE_DISTANCE = GEAR_MODULE*(DRIVE_TEETH+FOLLOWER_TEETH)*.5
GEAR_LAYER = "09  Synchronized foundry drive"


def place_drive(put,group,origin):
    x,y,z = origin
    for name,teeth,offset,speed,phase in (
        ("Foundry central drive",DRIVE_TEETH,0,DRIVE_SPEED,0),
        ("Foundry left follower",FOLLOWER_TEETH,-AXLE_DISTANCE,-DRIVE_SPEED*DRIVE_TEETH/FOLLOWER_TEETH,
         -.40*tau/FOLLOWER_TEETH),
        ("Foundry right follower",FOLLOWER_TEETH,AXLE_DISTANCE,-DRIVE_SPEED*DRIVE_TEETH/FOLLOWER_TEETH,
         -.40*tau/FOLLOWER_TEETH),
    ):
        asset = f"{teeth}-tooth brass gear"
        obj = put(group,asset,(x+offset,y,z))
        obj.name = name
        obj["Gear teeth"] = teeth
        obj["Pitch radius"] = GEAR_MODULE*teeth*.5
        obj["Angular speed"] = speed
        obj["Initial phase"] = phase
        driver = obj.driver_add("rotation_euler",1).driver
        driver.type = "SCRIPTED"
        driver.expression = f"{phase!r}+({speed!r})*(frame-1)/{FPS}"
