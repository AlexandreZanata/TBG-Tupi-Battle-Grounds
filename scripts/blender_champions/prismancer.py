"""Sculpt a geometric caster with a held crystal prism and an open-frame gauntlet."""

from math import cos, sin, tau

from bloomkeeper import leaf
from completion_people import face, limbs

RELEASE=(0,0,.25)


def prismancer(m,tier,branch,*,production=False):
    focus = tier > 0 and branch == 0
    split = tier > 0 and branch == 1
    cloth = "prism_iris" if focus else "prism_ice" if split else "sun_navy"
    robes(m,cloth,focus,split,tier)
    crown(m,cloth,focus,split,tier)
    limbs(m,cloth,"sun_cream" if focus else "brass",width=.053)
    prism(m,focus,split,tier)


def robes(m,cloth,focus,split,tier):
    m.loft("Narrow prism caster vest","Chest",cloth,[
        (-.038,.116,.092,-.003),(.066,.128,.099,-.004),(.164,.148,.110,0),
        (.263,.167,.100,-.013),(.321,.093,.070,-.006),
    ],n=14)
    m.loft("Angular short caster mantle","BackSpring",cloth,[
        (-.305,.199,.067,-.091),(-.237,.184,.061,-.071),(-.101,.144,.054,-.036),
        (.079,.130,.039,.009),
    ],n=11,opening=1.53,fold=.18)
    for s in (-1,1):
        leaf(m,"Shard neckline plate","Chest","sun_cream" if focus else "brass",
             (s*.052,.106,.117),(s*.165,.279,.096),.047)
        if split:
            leaf(m,"Paired prism shoulder plate","Chest","prism_ice",(s*.109,.257,.047),
                 (s*.241,.351,-.056),.061)
    if focus:
        m.box("Cast porcelain focus bracer","ForearmR","sun_cream",(0,-.080,.038),(.130,.142,.091),.016)


def crown(m,cloth,focus,split,tier):
    face(m)
    h="Head"
    m.loft("Soft polygonal wizard cap","HeadGear",cloth,[(.068,.110,.093,-.010),
           (.104,.123,.107,-.020),(.156,.095,.084,-.040),(.213,.042,.050,-.071)],n=12)
    m.tube("Angular brow circlet",h,"brass",[(-.110,.060,.062),(-.063,.093,.106),
           (0,.086,.120),(.063,.093,.106),(.110,.060,.062)],[.014,.017,.017,.017,.014],n=7)
    leaf(m,"Forehead prism diadem",h,"ritual" if focus else "soul" if split else "sun_cream",
         (0,.054,.122),(0,.181+.047*(tier==2),.115),.041)


def crystal(m,node,color,x,y,rear,tip,r):
    points=[(x,y,rear),(x,y,tip)]
    z=rear+(tip-rear)*.61
    points.extend((x+r*cos(tau*i/6),y+r*sin(tau*i/6),z) for i in range(6))
    faces=[(0,2+(i+1)%6,2+i) for i in range(6)]
    faces.extend((1,2+i,2+(i+1)%6) for i in range(6))
    m.mesh("Faceted optical crystal",node,color,points,faces,smooth=False)


def prism(m,focus,split,tier):
    w="WeaponR"
    m.tube("Hand grip beneath the prism","WeaponR","hide",[(0,0,-.061),(0,0,.060)],
           [.029,.032],n=9)
    for s in (-1,1):
        m.tube("Prism cage lateral spar",w,"brass",[(s*.039,0,-.021),(s*.122,0,.067),
               (s*.113,0,.181),(s*.052,0,.230)],[.020,.022,.020,.009],n=8)
    crystal(m,w,"ritual" if focus else "soul",0,0,.015,RELEASE[2],.079 if focus else .072)
    if focus and tier==2:
        for s in (-1,1):
            leaf(m,"Sharpened focusing rail",w,"sun_cream",(s*.081,.021,.078),
                 (s*.070,.061,.276),.033)
    if split:
        m.rim("Open prism shroud",w,"prism_ice",(0,0,.169),(.151,.124),.013)
        if tier==2:
            for s in (-1,1):
                crystal(m,w,"soul",s*.120,.009,.118,.268,.034)
