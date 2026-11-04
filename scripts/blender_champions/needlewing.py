"""Build an agile dragonfly around its four-wing pivot and removable needle."""

from bloomkeeper import leaf
from completion_people import insect_collars, limbs
from insect_legs import auxiliary_legs


def needlewing(m,tier,branch,*,production=False):
    precision = tier > 0 and branch == 0
    volley = tier > 0 and branch == 1
    shell = "sun_teal" if volley else "leaf_ant"
    trim = "sun_cream" if precision else "wasp_amber" if volley else "mire_reed"
    m.loft("Slender dragonfly thorax","Chest",shell,[
        (-.042,.084,.064,0),(.095,.118,.079,.008),
        (.247,.143,.090,0),(.323,.079,.064,.012),
    ],n=12)
    tail=[(0,.017,-.023),(0,.006,-.126),(0,-.028,-.235),(0,-.056,-.344),(0,-.093,-.505)]
    m.tube("Long articulated dragonfly abdomen","Hips",shell,tail,[.070,.065,.054,.038,.011],n=11)
    for _,y,z in tail[1:-1]:
        m.tube("Abdominal segment seam","Hips",trim,[(0,y,z-.014),(0,y+.004,z+.015)],
               [.055 if z>-.30 else .039,.062 if z>-.30 else .042],n=10)
    insect_collars(m,shell)
    limbs(m,shell,"pine",arm=.17*.95,width=.043,skin=shell)
    auxiliary_legs(m,shell,trim)
    face(m,shell,trim,tier,precision)
    wings(m,trim,tier,volley,precision)
    launcher(m,trim,tier,volley,precision)


def face(m,shell,trim,tier,precision):
    h="Head"
    m.ell("Small dragonfly face",h,shell,(0,.010,.020),(.124,.104,.096),segments=13)
    for s in (-1,1):
        m.ell("Compound eye housing",h,"pine",(s*.095,.036,.067),(.066,.073,.068),segments=12)
        m.ell("Compound eye",h,"sap" if precision else "wasp_aqua",(s*.108,.045,.087),
              (.053,.061,.058),segments=12)
        m.ell("Eye catchlight facet",h,"sun_cream",(s*.115,.068,.137),(.019,.021,.012),segments=8)
        m.tube("Short antenna",h,trim,[(s*.035,.084,.058),(s*.068,.149,.067),
               (s*.087,.170,.106)],[.012,.010,.003],n=7)
    if precision and tier==2:
        leaf(m,"Ivory ranging crest",h,trim,(0,.087,.064),(0,.214,-.095),.045,facing=(1,0,0))


def wings(m,trim,tier,volley,precision):
    for s in (-1,1):
        for index,root in enumerate(((s*.077,.249,-.053),(s*.084,.224,-.105))):
            tip=(s*(.500-.056*index+.045*precision),.407-.121*index,-.247-.090*index)
            finish = ("prism_ice" if volley else "wasp_wing") if index==0 else trim
            leaf(m,"Veined dragonfly flight vane","WeaponR",finish,root,tip,.081+.012*volley)
            m.tube("Wing spar seated inside thorax","WeaponR",trim,[root,
                   (s*.240,.313-.075*index,-.143-.061*index),tip],[.025,.015,.004],n=7)
            if volley:
                leaf(m,"Separate volley wing panel","WeaponR","wasp_aqua",
                     (s*.165,.256-.026*index,-.089),tip,.042)
        if volley and tier==2:
            leaf(m,"Forked volley wing tip","WeaponR","wasp_amber",(s*.291,.269,-.209),
                 (s*.490,.223,-.405),.052)


def launcher(m,trim,tier,volley,precision):
    m.tube("Wrist launch grip","HandR","hide",[(0,0,-.054),(0,0,.111)],[.031,.039],n=9)
    for s in (-1,1):
        m.tube("Open launcher rail","HandR",trim,[(s*.028,0,.056),(s*.025,0,.181),
               (s*.019,0,.217 if precision else .196)],[.014,.011,.007],n=7)
        if volley:
            m.box("Amber lateral feed guide","HandR",trim,(s*.041,.014,.105),(.025,.033,.095))
    m.tube("Independent loaded needle","HeadGear","sun_cream",[(0,0,-.049),(0,0,.071),(0,0,.120)],
           [.012,.014,.002],n=7)
    if volley and tier==2:
        for x in (-.041,.041):
            m.tube("Spare volley needle","HeadGear","wasp_aqua",[(x,0,-.027),(x,0,.104)],[.010,.002],n=7)
