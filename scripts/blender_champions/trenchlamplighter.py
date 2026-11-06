"""Fit an angler-fish keeper with a tall beacon and a forward signal lens."""

from abyssal_parts import fins, mantle, marine
from bloomkeeper import leaf

RELEASE=(0,.12,.28)


def trenchlamplighter(m,tier,branch,*,production=False):
    amber = tier > 0 and branch == 0
    glass = tier > 0 and branch == 1
    frame = "brass" if amber else "steel_dark" if glass else "bone"
    light = "sap" if amber else "soul" if glass else "forge"
    marine(m,"trenchlamplighter","shark_hide","fish_silver",
           "fortress_sand" if amber else "prism_iris",face=(.103,.093,.119),eyes=.071,snout=.145)
    fins(m,"brew_violet" if glass else frame,light,.168)
    mantle(m,"sun_cream" if amber else "violet_dark",frame,.157,.438)
    m.tube("Curved lure rooted in the angler head","Head","shark_hide",[
        (0,.062,-.034),(0,.173,-.016),(0,.222,.065),(0,.171,.157),
    ],[.027,.017,.014,.010],n=8)
    m.ell("Glowing angler lure bead","Head",light,(0,.170,.159),(.027,.030,.028),segments=10)
    beacon(m,frame,light,tier,amber,glass)


def beacon(m,frame,light,tier,amber,glass):
    n="WeaponR"
    top=.518 if amber and tier==2 else .458 if amber else .410
    m.tube("Curved lighthouse staff through the fist",n,"darkwood",[(0,-.222,0),(0,0,0),
           (0,.135,.060),(0,.189,.205)],[.024,.030,.027,.040],n=9)
    m.ell("Lens at the native signal origin",n,light,RELEASE,(.052,.051,.030),segments=12)
    m.rim("Signal lens brass rim",n,frame,RELEASE,(.054,.053),.012,segments=14)
    m.loft("Cut beacon glass",n,light,[
        (.151,.064,.058,.219),(.197,.088,.078,.210),(.309,.086,.069,.201),
        (top-.030,.041,.035,.188),
    ],n=12)
    for s in (-1,1):
        for z in (.160,.252):
            m.tube("Separate framed window rib",n,frame,[(s*.057,.163,z),
                   (s*.084,.254,z),(s*.036,top-.028,z*.85)],[.012,.013,.011],n=7)
        if glass:
            leaf(m,"Opaque saltglass hood panel",n,"prism_iris",(s*.066,.237,.220),
                 (s*(.127+.026*(tier==2)),.421,.157),.055)
    m.loft("Fitted lighthouse cap",n,frame,[
        (top-.063,.066,.053,.186),(top-.031,.083,.063,.191),
        (top,.060,.050,.187),(top+.042,.018,.020,.182),
    ],n=10)
    if amber:
        for s in (-1,1):
            m.tube("Separate guard rail on the amber beacon",n,frame,[(s*.071,.202,.261),
                   (s*.111,.228,.257),(s*.092,.315,.236)],[.012,.015,.011],n=7)
        if tier==2:
            m.tube("Upper lighthouse spire",n,frame,[(0,top-.013,.188),
                   (0,top+.059,.182),(0,top+.095,.173)],[.024,.016,.004],n=8)
    if glass and tier==2:
        m.rim("Suspended eye held in the cage frame",n,frame,(0,.320,.277),(.071,.058),.010,segments=12)
        m.tube("Joined eye frame stem",n,frame,[(0,.366,.216),(0,.370,.273)],[.017,.015],n=7)
