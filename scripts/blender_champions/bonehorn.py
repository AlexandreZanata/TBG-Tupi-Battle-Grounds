"""Build a low bone-mantled charger on four existing articulated supports."""

from bloomkeeper import leaf

STRIKE = (0,.10,.58)


def bonehorn(m,tier,branch,*,production=False):
    mantle = tier > 0 and branch == 0
    basalt = tier > 0 and branch == 1
    hide = "hide" if basalt else "horn_hide"
    plating = "acid_ivory" if mantle else "horn_stone" if basalt else "oldbone"
    body(m,hide,plating,mantle,basalt,tier)
    head(m,hide,plating,mantle,basalt,tier)
    for side, sign in (("L",1),("R",-1)):
        leg(m,(f"UpperArm{side}",f"Forearm{side}",f"Hand{side}"),sign,hide,plating,mantle)
        leg(m,(f"Thigh{side}",f"Shin{side}",f"Foot{side}"),sign,hide,plating,mantle)


def body(m,hide,plate,mantle,basalt,tier):
    for node,center,size in (
        ("Hips",(0,-.007,-.066),(.275,.245,.304)),
        ("Spine",(0,-.003,.057),(.300,.237,.305)),
        ("Chest",(0,.003,.033),(.337,.284,.328)),
    ):
        m.ell("Hunched charger body",node,hide,center,size,segments=15,rings=8)
    m.tube("Short tufted tail","Hips",hide,[(0,.020,-.27),(.037,-.031,-.421),
           (.019,-.108,-.492)],[.063,.047,.020],n=9)
    for s in (-1,1):
        for i in range(3):
            z = -.209+i*.173
            r = .056 + .016*i + .018*mantle
            m.tube("Overlapping back mantle rib","BackSpring",plate,[(s*.017,.273,z-.05),
                   (s*.155,.258,z),(s*.271,.186,z-.012),(s*.327,.059,z-.072)],
                   [r*.76,r,r*.86,r*.30],n=8)
        if basalt:
            m.tube("Carved shoulder collar","Chest","horn_stone",[(s*.117,.239,.137),
                   (s*.262,.201,.188),(s*.320,.036,.231),(s*.247,-.146,.224)],
                   [.043,.055,.055,.026],n=9)
        if mantle and tier==2:
            for i in range(2):
                z = -.193+i*.21
                leaf(m,"Ancestral mantle fin","BackSpring","acid_ivory",(s*.12,.264,z),
                     (s*.24,.450+i*.023,z-.121),.097)


def head(m,hide,plate,mantle,basalt,tier):
    h = "Head"
    m.ell("Wide neck under skull",h,hide,(0,-.017,-.117),(.266,.229,.222),segments=16)
    m.ell("Long boar skull",h,plate,(0,.029,.134),(.258,.199,.336),segments=16,rings=9)
    m.ell("Lower jaw wedge",h,hide,(0,-.147,.242),(.199,.088,.263),segments=14)
    m.ell("Dark split muzzle",h,"ink",(0,-.048,.433),(.193,.091,.095),segments=13)
    for s in (-1,1):
        m.ell("Recessed eye orbit",h,"ink",(s*.206,.070,.239),(.069,.046,.061),segments=10)
        m.ell("Amber charging eye",h,"sap",(s*.231,.075,.267),(.035,.020,.026),segments=10)
        m.tube("Sharp eye brow",h,plate,[(s*.086,.145,.304),(s*.193,.124,.293),
               (s*.241,.090,.219)],[.027,.046,.031],n=8)
        m.tube("Curved outer tusk",h,"horn_stone" if basalt else "bone",[
               (s*.158,-.152,.272),(s*.230,-.139,.374),(s*.267,-.083,.495),
               (s*(.252 if basalt else .227),.021+.050*(basalt and tier==2),.583)],
               [.052,.058,.036,.003],n=9)
        if basalt and tier==2:
            m.tube("Forked stone-tusk hook",h,"horn_stone",[(s*.204,-.105,.321),
                   (s*.307,-.062,.419),(s*.335,.080,.517)],[.051,.047,.004],n=8)
        if mantle:
            m.tube("Ancestral cheek strut",h,"acid_ivory",[(s*.242,.128,.013),
                   (s*.268,.036,.124),(s*.220,-.093,.302)],[.056,.048,.021],n=8)
    m.tube("Central ram horn at strike",h,"acid_ivory" if mantle else "bone",[
           (0,.143,.223),(0,.217+.058*(mantle and tier==2),.355),
           (0,.171,.462),STRIKE],[.099,.082,.044,.004],n=11)


def leg(m,nodes,s,hide,plate,mantle):
    upper, lower, foot = nodes
    m.tube("Heavy articulated upper leg",upper,hide,[(0,.054,0),(s*.017,-.105,.005),
           (0,-.222,.010),(0,-.325,0)],[.115,.133,.110,.077],n=10)
    m.tube("Articulated shin",lower,"ink",[(0,.044,0),(0,-.117,.012),(0,-.320,0)],
           [.091,.098,.063],n=9)
    m.ell("Rounded bone knee",lower,plate,(0,-.031,.062),(.115,.105,.073),segments=11)
    if mantle:
        m.ell("Layered foreleg plate",upper,plate,(s*.025,-.103,.086),
              (.113,.147,.060),segments=10)
    for x in (-.049,.049):
        m.box("Cloven hoof heel",foot,"slag",(x,.014,.067),(.085,.118,.195),.015)
        m.tube("Cloven bone toe",foot,plate,[(x,.027,.109),(x,.017,.173),(x,-.008,.228)],
               [.045,.043,.011],n=8)
