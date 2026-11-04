"""Sculpt a basalt bruiser with an asymmetrical, wrist-driven mineral claw."""

from bloomkeeper import leaf

TIP = (0,0,.30)


def cragclaw(m,tier,branch,*,production=False):
    edge = tier > 0 and branch == 0
    split = tier > 0 and branch == 1
    stone = "crag_limestone" if edge else "crag_fissure" if split else "fortress_basalt"
    trunk(m,stone,edge,split,tier)
    head(m,stone,split)
    limbs(m,stone,edge,split,tier)
    claw(m,stone,edge,split,tier)


def trunk(m,stone,edge,split,tier):
    m.loft("Basalt torso with faceted boulder planes","Chest",stone,[
        (-.060,.136,.107,0),(.041,.165,.121,-.012),(.184,.206,.143,-.005),
        (.277,.214,.120,-.019),(.330,.113,.085,-.008),
    ],n=9)
    m.ell("Stone pelvic hinge","Hips","ink",(0,-.005,-.005),(.154,.098,.114),segments=10)
    for s in (-1,1):
        m.tube("Deep fault line across chest","Chest","ink",[(s*.026,.259,.118),
               (s*.103,.207,.136),(s*.131,.086,.117)],[.010,.016,.009],n=6)
        if split:
            leaf(m,"Raised shoulder basalt fork","Chest","crag_fissure",(s*.103,.238,-.041),
                 (s*.277,.368,-.083),.071)
        if edge:
            m.box("Angled sandstone torso plate","Chest","crag_limestone",(s*.132,.172,.110),
                  (.119,.199,.061),.016,rotation=s*.24)
    for i in range(3):
        leaf(m,"Dorsal mineral shard","Chest","soul" if split and tier==2 else "crag_fissure" if split else stone,
             (0,.100+i*.067,-.117),(0,.170+i*.076,-.246-.016*i),.046,facing=(1,0,0))


def head(m,stone,split):
    h = "Head"
    m.loft("Faceted broad-jawed basalt head",h,stone,[
        (-.117,.069,.057,.023),(-.061,.122,.096,.017),(.060,.120,.106,.002),
        (.114,.094,.092,-.016),(.148,.062,.061,-.019),
    ],n=8)
    m.ell("Wide dark mouth cleft",h,"ink",(0,-.055,.107),(.074,.019,.012),segments=9)
    for s in (-1,1):
        m.ell("Crystalline eye cavity",h,"ink",(s*.059,.038,.097),(.039,.030,.022),segments=9)
        m.ell("Glow in the stone eye",h,"soul" if split else "sap",(s*.060,.041,.113),
              (.021,.015,.009),segments=8)
        m.tube("Jutting facial stone brow",h,stone,[(s*.018,.064,.112),(s*.077,.075,.107),
               (s*.116,.059,.071)],[.015,.026,.011],n=7)


def limbs(m,stone,edge,split,tier):
    for side,s in (("L",1),("R",-1)):
        upper,fore,hand,thigh,shin,foot = (f"{p}{side}" for p in
                                           ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Stone shoulder and arm hinge",upper,stone,[(0,.030,0),(s*.019,-.079,.012),(0,-.197,0)],
               [.072,.088,.050],n=8)
        m.tube("Dark forearm pivot",fore,"ink",[(0,.025,0),(0,-.176,0)],[.054,.043],n=8)
        if side=="R":
            m.box("Overgrown primary claw knuckle",fore,stone,(0,-.081,.039),
                  (.191+.035*edge,.201,.173),.029,rotation=-.11)
            if tier==2:
                leaf(m,"Chisel ridge on the dominant wrist",fore,"gold" if edge else "soul",
                     (-.054,-.164,.132),(-.104,.011,.133),.048)
        else:
            m.ell("Smaller secondary forearm",fore,stone,(0,-.083,.017),(.069,.087,.058),segments=9)
        m.ell("Rock palm",hand,stone,(0,-.018,.012),(.055,.052,.049),segments=9)
        for node in (thigh,shin):
            m.tube("Compact tapering stone leg",node,stone,[(0,.023,0),(0,-.077,.009),(0,-.192,0)],
                   [.064,.071,.045],n=8)
        m.box("Square basalt foot",foot,"fortress_basalt",(0,-.016,.046),(.134,.091,.176),.020)


def claw(m,stone,edge,split,tier):
    w = "WeaponR"
    m.tube("Stone hand crown before the claw",w,stone,[(0,0,-.012),(0,0,.058),(0,0,.127)],
           [.060,.071,.070],n=9)
    leaf(m,"Central basalt claw at strike",w,"sun_cream" if edge else "crag_fissure" if split else "oldbone",
         (0,0,.077),TIP,.089 if edge else .067,facing=(0,1,0))
    if split:
        for s in (-1,1):
            leaf(m,"Split turquoise side claw",w,"crag_fissure",(s*.035,.011,.091),
                 (s*(.155 if tier==2 else .119),.018,.282),.049,facing=(0,1,0))
            if tier==2:
                leaf(m,"Glowing shard on secondary prong",w,"soul",(s*.069,.035,.143),
                     (s*.151,.030,.281),.019,facing=(0,1,0))
