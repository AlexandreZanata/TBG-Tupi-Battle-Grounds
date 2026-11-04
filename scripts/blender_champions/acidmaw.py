"""Build a corrosive stalker with a spring-mounted reservoir and four jaws."""

from bloomkeeper import leaf

MUZZLE = (0,0,.36)
PETALS = (("WeaponR",1,0),("OffhandL",-1,0),("Back",0,1),("StringTop",0,-1))


def acidmaw(m,tier,branch,*,production=False):
    abyss = tier > 0 and branch == 0
    corrosive = tier > 0 and branch == 1
    skin = "violet_dark" if corrosive else "acid_moss"
    trim = "acid_ivory" if abyss else "sprout" if corrosive else "oldbone"
    stalk(m,skin,trim,abyss,corrosive,tier)
    face(m,skin,trim,abyss,corrosive,tier)
    for node,x,y in PETALS:
        petal(m,node,x,y,skin,trim,abyss,corrosive,tier)
    for side, sign in (("L",1),("R",-1)):
        legs(m,side,sign,skin,trim)


def stalk(m,skin,trim,abyss,corrosive,tier):
    m.ell("Deep hip seed pod","Hips",skin,(0,.012,-.034),(.142,.102,.144),segments=13)
    m.loft("Tapered torso and elongated stalk","Chest",skin,[
        (-.044,.110,.103,-.037),(.078,.157,.129,-.048),(.184,.173,.126,-.036),
        (.283,.141,.094,-.017),(.404,.088,.072,.012),(.535,.068,.058,.050),
        (.655,.081,.070,.060),(.706,.103,.074,.077),
    ],n=15,fold=.07)
    for s in (-1,1):
        m.tube("Spine brace running to the jaw","Chest",trim,[(s*.102,.178,.054),
               (s*.095,.290,.093),(s*.080,.485,.102),(s*.068,.668,.100)],
               [.021,.028+.009*abyss,.019,.012],n=8)
        if abyss:
            m.tube("Segmented ivory shoulder brace","Chest","acid_ivory",[(s*.053,.238,.059),
                   (s*.117,.273,.068),(s*.175,.327,.004),(s*.184,.375,-.060)],
                   [.039,.044,.040,.010],n=8)
    reservoir(m,corrosive,tier)


def reservoir(m,corrosive,tier):
    joint = "BackSpring"
    glow = "venom" if corrosive else "soul"
    width = .104 if corrosive else .083
    m.ell("Thick acid reservoir membrane",joint,"leaf",(0,-.012,.073),
          (width*1.15,.170,.091),segments=13,rings=8)
    m.ell("Luminous exposed throat sac",joint,glow,(0,-.013,.105),
          (width,.148,.074),segments=13,rings=8)
    for s in (-1,1):
        m.tube("Reservoir rib along membrane",joint,"violet" if corrosive else "pine",[
            (s*.039,-.148,.085),(s*width,-.063,.121),(s*width,.036,.126),
            (s*.040,.123,.104),
        ],[.015,.019,.016,.011],n=7)
    if corrosive and tier==2:
        for x,y,z,size in ((-.130,-.108,.013,.057),(.130,-.095,.007,.061),(0,-.219,.043,.068)):
            m.ell("Corrosive reserve pod membrane",joint,"violet",(x,y,z),
                  (size,size*1.27,size*.82),segments=11)
            m.ell("Lit reserve pod window",joint,"venom",(x,y,z+size*.58),
                  (size*.70,size*.88,size*.42),segments=10)


def face(m,skin,trim,abyss,corrosive,tier):
    h = "Head"
    m.ell("Low skull under the flower jaws",h,skin,(0,.002,.028),(.155,.105,.135),segments=15)
    m.ell("Black recessed mouth behind jaws",h,"ink",(0,0,.142),(.083,.077,.045),segments=13)
    m.ell("Corrosive throat deep inside opening",h,"venom" if corrosive else "soul",
          (0,0,.179),(.042,.040,.014),segments=10)
    for s in (-1,1):
        m.ell("Slanted side eye cavity",h,"ink",(s*.119,.045,.083),(.041,.033,.047),segments=10)
        m.ell("Bioluminescent side eye",h,"venom" if corrosive else "soul",(s*.143,.045,.107),
              (.024,.020,.023),segments=9)
        m.tube("Armored brow above side eye",h,trim,[(s*.067,.082,.096),(s*.126,.078,.105),
               (s*.163,.068,.050)],[.018,.026,.013],n=7)
        if abyss and tier==2:
            leaf(m,"Swept abyssal crown horn","HeadGear","acid_ivory",(s*.065,.065,.007),
                 (s*.180,.267,-.090),.088)


def petal(m,node,x,y,skin,trim,abyss,corrosive,tier):
    r = .139+.032*abyss
    def at(radius,z,tangent=0):
        return (x*radius-y*tangent,y*radius+x*tangent,z)
    m.tube("Separate articulated petal root",node,skin,[at(.068,.061),at(.106,.116),at(r,.177)],
           [.052,.049,.040],n=9)
    m.tube("Curved split jaw petal",node,trim,[at(.096,.109),at(r,.190),
           at(r*.89,.265),at(.080,MUZZLE[2])],[.044,.055+.015*abyss,.041,.003],n=10)
    m.tube("Inner tooth on each jaw petal",node,"acid_ivory",[at(r*.76,.207,.018),
           at(.075,.249,.017),at(.052,.305,.007)],[.021,.017,.003],n=7)
    if corrosive:
        m.tube("Bioluminescent vein on opening petal",node,"venom",[
            at(.125,.144),at(r+.038,.199),at(r*.96+.018,.274),at(.081,.343),
        ],[.010,.013,.013,.002],n=7)
        if tier==2:
            m.tube("Secondary cutting petal prong",node,"sprout",[at(.104,.172,-.033),
                   at(.123,.247,-.051),at(.102,.332,-.039)],[.024,.029,.003],n=8)


def legs(m,side,s,skin,trim):
    thigh,shin,foot,arm,fore,hand = (f"{p}{side}" for p in
                                     ("Thigh","Shin","Foot","UpperArm","Forearm","Hand"))
    m.tube("Long stalker thigh",thigh,skin,[(0,.035,0),(s*.029,-.106,.025),(0,-.242,0)],
           [.054,.071,.038],n=9)
    m.tube("Long stalker shin",shin,"pine",[(0,.037,0),(s*.021,-.088,-.020),(0,-.243,0)],
           [.049,.044,.032],n=9)
    for x in (-.028,.028):
        m.tube("Split stalker foot",foot,"slag",[(x,.007,-.032),(x,-.026,.051),(x*1.46,-.045,.149)],
               [.028,.030,.004],n=8)
    m.tube("Small raised shoulder limb",arm,skin,[(0,.034,0),(s*.025,-.044,0),(0,-.119,0)],
           [.046,.050,.031],n=8)
    m.tube("Folded lower shoulder limb",fore,trim,[(0,.020,0),(s*.013,-.063,0),(0,-.108,0)],
           [.035,.037,.023],n=8)
    m.tube("Small inner hook",hand,trim,[(0,.020,0),(s*.023,-.034,.026),(s*.006,-.044,.088)],
           [.028,.026,.004],n=7)
