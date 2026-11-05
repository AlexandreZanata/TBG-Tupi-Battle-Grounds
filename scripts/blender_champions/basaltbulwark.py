"""Carve the Basalt Bulwark's braced shield and massive masonry kit."""

from bloomkeeper import leaf
from ogre_parts import ogre


def basaltbulwark(m,tier,branch,*,production=False):
    wall = tier > 0 and branch == 0
    quake = tier > 0 and branch == 1
    armor = "crag_limestone" if wall else "fortress_basalt"
    trim = "sun_cream" if wall else "crag_fissure" if quake else "fortress_moss"
    ogre(m,"basaltbulwark","hide",armor,jaw=1.12)
    shield(m,armor,trim,tier,wall,quake)
    m.tube("Rock maul grip","WeaponR","darkwood",[(0,0,-.064),(0,0,.225)],[.031,.034],n=9)
    m.box("Squared quarry maul","WeaponR",armor,(0,0,.250),(.218,.144,.151),bevel=.019)
    m.box("Maul striking edge","WeaponR",trim,(0,0,.307),(.165,.103,.041),bevel=.009)
    for s,side in ((-1,"R"),(1,"L")):
        m.ell("Split mantle support",f"UpperArm{side}",armor,(s*.010,.005,.015),
              (.111,.072,.105),segments=10)
        if quake:
            leaf(m,"Jagged fault ridge",f"UpperArm{side}",trim,(0,.027,.029),
                 (s*.111,.108+.046*(tier==2),-.033),.036)
    if wall:
        m.plate("Stone guard headband","Head",armor,[(-.123,.081),(-.104,.126),(.105,.126),
                (.124,.081),(0,.069)],(.060,.106))
    if wall and tier==2:
        m.box("Masonry crest","Head",trim,(0,.153,.006),(.106,.077,.106),bevel=.013)


def shield(m,armor,trim,tier,wall,quake):
    n="OffhandL"
    top=.570 if tier==2 else .483 if wall else .408
    half=.181 if wall else .151
    outline=[(-half,-.088),(-half,.81*top),(-half*.65,top),(.032,top+.018),
             (half,.77*top),(half,-.088),(0,-.156)]
    m.tube("Grip extending through the fist",n,"wood",[(0,-.074,.018),(0,.081,.027)],[.033,.036],n=9)
    m.plate("Continuous deep shield backing",n,"iron",outline,(.025,.088),bevel=.013)
    m.plate("Carved basalt face",n,armor,[(x*.93,y*.96+.004) for x,y in outline],
            (.084,.133),bevel=.018)
    if wall:
        for y in (.003,.145,.287):
            m.box("Horizontal masonry courses",n,trim,(0,y,.139),(.303,.055,.031),bevel=.012)
    elif quake:
        m.tube("Jagged inset fault line",n,trim,[(-.104,.345,.146),(.018,.252,.151),
               (-.030,.169,.152),(.113,.051,.145),(.057,-.058,.148)],[.013,.016,.015,.017,.010],n=7)
        if tier==2:
            leaf(m,"Seismic shield spur",n,trim,(.028,.369,.141),(.177,top+.091,.152),.042)
    else:
        m.ell("Rounded central shield boss",n,trim,(0,.148,.133),(.070,.079,.034),segments=11)
