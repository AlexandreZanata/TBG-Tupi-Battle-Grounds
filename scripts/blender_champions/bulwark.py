"""Build a walking stone citadel and its aligned two-handed striking tool."""

from math import cos, sin, tau

SUPPORT = (0,.12,0)


def bulwark(m,tier,branch,*,production=False):
    citadel = tier > 0 and branch == 0
    earth = tier > 0 and branch == 1
    stone = "fortress_sand" if citadel else "fortress_basalt" if earth else "fortress_moss"
    chassis(m,stone,citadel,earth,tier)
    legs_and_arms(m,stone,earth)
    hammer(m,stone,earth,tier)


def chassis(m,stone,citadel,earth,tier):
    c,h = "Chest","Head"
    m.loft("Riveted dome underframe",c,"steel_dark",[
        (-.029,.271,.205,-.055),(.072,.374,.295,-.080),(.195,.427,.339,-.114),
        (.281,.397,.306,-.139),(.400,.292,.255,-.157),(.505,.171,.169,-.173),
    ],n=18)
    for row,(y,r,z,w) in enumerate(((.13,.382,-.117,.121),(.238,.391,-.130,.122),
                                    (.345,.337,-.153,.100),(.435,.260,-.168,.087))):
        for k in range(8):
            angle = (k-3.5+(.31 if row%2 else 0))*tau/10
            x = r*sin(angle)
            dz = r*.77*cos(angle)
            slab = m.ell("Overlapping quarried dome block",c,stone,(x,y,z+dz),
                        (w,.076,.060),segments=9,rings=5)
            for vertex in slab.data.vertices:
                dx,dz_local = vertex.co.x-x,vertex.co.z-z-dz
                vertex.co.x = x+dx*cos(angle)+dz_local*sin(angle)
                vertex.co.z = z+dz-dx*sin(angle)+dz_local*cos(angle)
    m.ell("Rounded roof keystone",c,stone,(0,.482,-.172),(.156,.081,.150),segments=12)
    m.box("Angular front sight box",h,"steel_dark",(0,-.034,.180),(.264,.164,.279),.032)
    m.box("Deep recessed face plate",h,"ink",(0,-.045,.330),(.191,.086,.040),.010)
    for s in (-1,1):
        m.box("Pilot light in slit",h,"forge",(s*.058,-.025,.354),(.045,.020,.010),.004)
        m.tube("Chest truss strut",c,"copper" if earth else "iron",[(s*.205,.048,.162),
               (s*.245,.205,.224),(s*.230,.396,.177)],[.027,.033,.024],n=8)
    m.box("Vented helmet brow",h,stone,(0,.056,.301),(.296,.084,.141),.018)
    if citadel:
        fortress(m,tier)
    if earth:
        for s in (-1,1):
            m.tube("Basalt shoulder counterweight",c,stone,[(s*.288,.330,-.182),
                   (s*.431,.262,-.094),(s*.456,.182,.012)],[.048,.084,.035],n=8)


def fortress(m,tier):
    for s in (-1,1):
        m.box("Riveted shield side wall","Chest","fortress_sand",(s*.377,.236,-.064),
              (.125,.236,.316),.027)
        m.box("Fortress rampart cap","Chest","brass",(s*.379,.359,-.064),(.153,.043,.340),.010)
        if tier==2:
            for z in (-.183,-.062,.059):
                m.box("Crenellation block","Chest","fortress_sand",(s*.379,.408,z),(.119,.084,.068),.013)
    if tier==2:
        m.box("Citadel central keep","Chest","fortress_sand",(0,.539,-.190),(.244,.209,.231),.026)
        for x in (-.094,.094):
            m.box("Keep corner merlon","Chest","fortress_sand",(x,.657,-.191),(.068,.071,.249),.009)
        m.box("Keep arrow slit","Chest","ink",(0,.552,-.071),(.028,.091,.012),.003)


def legs_and_arms(m,stone,earth):
    for side,s in (("L",1),("R",-1)):
        upper,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Armor-linked upper arm",upper,"slag",[(0,.055,0),(s*.043,-.065,.013),(0,-.192,0)],
               [.089,.106,.071],n=9)
        m.tube("Arm piston with iron sleeve",fore,"iron",[(0,.019,0),(0,-.123,.012),(0,-.174,0)],
               [.077,.083,.060],n=9)
        m.box("Tool grip glove",hand,"steel_dark",(0,-.019,.006),(.118,.105,.114),.018)
        for joint,length in ((thigh,.15),(shin,.15)):
            m.tube("Short iron support leg",joint,"steel_dark",[(0,.033,0),(0,-.065,0),(0,-length,0)],
                   [.093,.086,.067],n=9)
        m.box("Bastion track shoe",foot,"slag",(0,-.021,.044),(.199,.091,.247),.020)
        m.box("Carved toe casing",foot,stone,(0,.015,.107),(.177,.066,.137),.017)
        if earth:
            for z in (-.004,.067,.137):
                m.box("Ground cleat",foot,"copper",(0,-.052,z),(.205,.028,.029),.005)


def hammer(m,stone,earth,tier):
    w = "WeaponR"
    z = .13 if earth else .10
    m.tube("Wrapped two-hand tool haft",w,"darkwood",[(0,-.241,0),(0,0,0),SUPPORT,(0,.228,0)],
           [.033,.035,.033,.029],n=9)
    for y in (-.036,.039,.102,.148,.205):
        m.tube("Haft binding",w,"brass",[(0,y-.012,0),(0,y+.011,0)],[.037,.037],n=8)
    m.box("Quarried hammer block",w,stone,(0,-.221,z-.120),(.301 if earth else .237,.160,.230),.027)
    m.box("Hammer front striking plane",w,"iron",(0,-.220,z-.014),
          (.266 if earth else .214,.118,.028),.011)
    if earth and tier==2:
        m.box("Broadened earthbreaker flange",w,"copper",(0,-.297,z-.093),(.359,.042,.232),.012)
        for s in (-1,1):
            m.tube("Cast earthbreaker edge spike",w,"iron",[(s*.129,-.230,z-.073),
                   (s*.180,-.230,z-.073),(s*.207,-.219,z-.023)],[.031,.030,.003],n=7)
