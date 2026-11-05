"""Assemble the Barrel Keeper's chest-mounted cask and fitted brewing apron."""

from math import cos, sin, tau

from ogre_parts import ogre

BUNG=(0,0,.218)


def gustdrummer(m,tier,branch,*,production=False):
    balm = tier > 0 and branch == 0
    harvest = tier > 0 and branch == 1
    wood = "mire_reed" if harvest else "wood"
    trim = "sun_cream" if balm else "gold" if harvest else "iron"
    ogre(m,"gustdrummer","horn_hide" if not harvest else "pine","leather",jaw=1.10)
    cask(m,wood,trim,tier,balm,harvest)
    m.plate("Close-fitted brewing apron","Hips","linen" if balm else "leather",
            [(-.138,.024),(.138,.024),(.118,-.213),(-.112,-.213)],(.122,.153))
    m.loft("Brewer's padded cap","Head","mire_amber" if not harvest else "mire_reed",[
        (.074,.136,.108,-.006),(.116,.144,.113,-.012),(.166,.091,.085,-.015),
    ],n=14)
    for s in (-1,1):
        m.tube("Brewing apron shoulder strap","Chest","linen" if balm else "wood",[
            (s*.108,.297,.080),(s*.120,.202,.119),(s*.084,.058,.123),
        ],[.022,.025,.021],n=8)
        if balm:
            m.ell("Balm vials on the belt","Hips","sap",(s*.171,-.039,.078),
                  (.039,.064,.040),segments=10)


def cask(m,wood,trim,tier,balm,harvest):
    n="WeaponR"
    profile=[(-.232,.145),(-.174,.188),(-.077,.204),(.077,.204),(.174,.188),(.232,.145)]
    m.loft("Horizontal stave cask shaped around native grip points",n,wood,
           [(x,r,r,0) for x,r in profile],n=18,transform=lambda p:(p[1],-p[0],p[2]))
    for i in range(12):
        angle = i*tau/12
        points = [(x,r*sin(angle),r*cos(angle)) for x,r in profile]
        m.tube("Individual raised stave joint",n,"pine" if harvest else "darkwood",
               points,[.006,.007,.008,.008,.007,.006],n=6)
    for s in (-1,1):
        m.loft("Metal cask hoop",n,trim,[(s*.174-.015,.193,.193,0),
               (s*.174+.015,.193,.193,0)],n=18,transform=lambda p:(p[1],-p[0],p[2]))
        m.tube("Recessed cask cap and palm brace",n,"capy_fur" if not harvest else "capy_reed",[
            (s*.218,0,0),(s*.239,0,0),(s*.250,0,0),
        ],[.145,.143,.063],n=14)
        if balm and tier==2:
            m.box("Balm cask end cap brace",n,trim,(s*.236,.047,0),(.024,.050,.224),bevel=.009)
    m.tube("Brass bung seat at the native front marker",n,"brass",[(0,0,.198),BUNG],
           [.054,.054],n=12)
    m.ell("Visible remedy bung",n,"sap" if balm else "venom" if harvest else "leaf",
          BUNG,(.029,.029,.016),segments=11)
    if tier==2:
        m.ell("Raised remedy vessel",n,"sap" if balm else "venom",(-.060,.208,0),
              (.041,.061,.046),segments=11)
    if harvest:
        m.tube("Harvest tap with spout",n,"brass",[(.080,-.043,.179),(.092,-.067,.229),
               (.094,-.114-.044*(tier==2),.250)],[.028,.026,.021],n=9)
