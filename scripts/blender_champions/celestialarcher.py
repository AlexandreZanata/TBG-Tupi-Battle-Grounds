"""Forge a winged archer with the existing independently drawn string and arrow."""

from math import hypot

from bloomkeeper import leaf

HALF = {(0,0):.50,(1,0):.60,(2,0):.64,(1,1):.54,(2,1):.58}
TIP_Z = -.10
DRAW = .20
ARROW = .42


def celestialarcher(m,tier,branch,*,production=False):
    precision = tier > 0 and branch == 0
    mark = tier > 0 and branch == 1
    cloth = "archer_amethyst" if mark else "sun_navy"
    armor = "sun_cream" if precision else "violet_dark" if mark else "brass"
    body(m,cloth,armor,precision,mark,tier)
    head(m,cloth,precision,mark)
    wings(m,precision,mark,tier)
    bow(m,HALF[tier,branch],precision,mark,tier)


def body(m,cloth,armor,precision,mark,tier):
    c = "Chest"
    m.loft("Close-fitted archer undercoat",c,cloth,[
        (-.052,.114,.086,0),(.031,.126,.091,0),(.143,.144,.102,.004),
        (.258,.168,.100,0),(.332,.095,.080,-.004),
    ],n=14)
    for s in (-1,1):
        leaf(m,"Beveled overlapping breastplate",c,armor,(s*.024,.063,.110),
             (s*.113,.293,.088),.066)
    m.tube("Archer cross-body quiver strap",c,"hide",[(-.136,.280,.077),
           (-.075,.207,.113),(.034,.136,.123),(.114,.055,.103)],[.019,.024,.025,.017],n=8)
    m.loft("Archer hip skirt","Hips",cloth,[(-.133,.152,.118,0),(-.067,.144,.110,0),
           (.026,.124,.099,0),(.057,.131,.106,0)],n=14,opening=.52,fold=.10)
    m.tube("Feather-stamped quiver","Back","violet_dark" if mark else "hide",
           [(0,-.097,-.012),(.015,.063,-.034),(.039,.174,-.046)],[.051,.062,.070],n=11)
    for x in (-.024,.022,.063):
        m.tube("Quiver arrow shafts","Back","wood",[(x,.151,-.042),(x+.029,.324,-.062)],
               [.008,.008],n=6)
        leaf(m,"Quiver arrow fletch","Back","ritual" if mark else "linen",
             (x+.021,.265,-.058),(x+.048,.339,-.066),.020)
    for side,s in (("L",1),("R",-1)):
        arm,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Flexible archery sleeve",arm,cloth,[(0,.033,0),(0,-.069,0),(0,-.202,0)],
               [.062,.062,.042],n=9)
        m.tube("Archer forearm guard",fore,"hide",[(0,.023,0),(0,-.072,0),(0,-.180,0)],
               [.048,.055,.036],n=9)
        if precision:
            leaf(m,"Gilt feather arm guard",fore,"sun_cream",(s*.024,-.142,.027),
                 (s*.073,-.031,.045),.034)
        elif mark:
            leaf(m,"Wrist crystal blade",fore,"archer_amethyst",(0,-.147,.047),
                 (s*.063,-.022,.064),.031)
        m.ell("Bow-hand and string-hand gloves",hand,"skin",(0,-.019,.008),
              (.043,.045,.035),segments=10)
        for joint in (thigh,shin):
            m.tube("Tapered archer trouser leg",joint,cloth,[(0,.030,0),(0,-.083,0),(0,-.191,0)],
                   [.052,.050,.037],n=8)
        m.ell("Light archer boot",foot,"hide",(0,-.020,.044),(.052,.045,.101),segments=10)


def head(m,cloth,precision,mark):
    h = "Head"
    m.loft("Archer face profile",h,"skin",[
        (-.109,.055,.058,.027),(-.072,.082,.083,.018),(.002,.101,.098,.006),
        (.082,.097,.086,-.003),(.115,.061,.070,-.015),
    ],n=15)
    m.ell("Straight bridge of archer nose",h,"skin",(0,-.009,.105),(.020,.030,.025),segments=10)
    m.ell("Fine mouth line",h,"wood",(0,-.052,.097),(.035,.009,.008),segments=8)
    for s in (-1,1):
        m.ell("Archer almond eye",h,"linen",(s*.041,.026,.095),(.025,.013,.012),segments=9)
        m.ell("Archer pupil",h,"ink",(s*.044,.026,.105),(.009,.012,.004),segments=8)
        m.tube("Fitted temple guard",h,"archer_amethyst" if mark else "gold",
               [(s*.094,.098,.036),(s*.103,.025,.063),(s*.085,-.035,.067)],
               [.021,.027,.010],n=7)
    m.loft("Winged archer helmet",h,"sun_cream" if not mark else "violet_dark",[
        (.060,.105,.092,-.007),(.094,.115,.099,-.004),(.145,.109,.093,-.019),
        (.183,.076,.074,-.038),(.209,.033,.050,-.051),
    ],n=15)
    leaf(m,"Central helmet feather",h,"sun_cream" if precision else cloth,
         (0,.103,.086),(0,.253,-.016),.042)
    m.rim("Archer's fine halo","HeadGear","ritual" if mark else "gold",
          (0,.008,-.070),(.177,.184),.012,segments=22)


def wings(m,precision,mark,tier):
    for side,node in ((1,"BackSpring"),(-1,"Tabard")):
        reach = .035*tier if precision or mark else 0
        m.tube("Wing shoulder hinge rooted inside the cuirass",node,
               "archer_amethyst" if mark else "brass",[
            (-side*.070,-.046,.079),(-side*.007,.005,.029),
            (side*.072,.050,-.015),(side*.168,.113,-.057),
        ],[.039,.043,.038,.021],n=9)
        for i in range(4):
            base = (side*(.026+.030*i),.033+.031*i,-.019)
            tip = (side*(.158+.050*i+reach),.125+.047*i+reach,-.030-.018*i)
            leaf(m,"Overlapping rigid celestial feather",node,"archer_amethyst" if mark else "sun_cream",
                 base,tip,.044-.003*i)
            leaf(m,"Metal edge of the feather",node,"ritual" if mark else "brass",
                 (base[0],base[1],base[2]+.018),(tip[0]*.94,tip[1]-.019,tip[2]+.012),.013)
        if precision and tier==2:
            leaf(m,"Extended white flight feather",node,"sun_cream",(side*.103,.089,-.040),
                 (side*.408,.309,-.102),.046)


def bow(m,half,precision,mark,tier):
    w = "OffhandL"
    m.tube("Wrapped bow grip",w,"hide",[(0,-.071,0),(0,.071,0)],[.029,.029],n=9)
    for s in (-1,1):
        tip = (0,s*half,TIP_Z)
        m.tube("Tapered angular bow limb",w,"archer_amethyst" if mark else "gold",[
            (0,s*.064,0),(0,s*.245*half,.061),(0,s*.574*half,.066),
            (0,s*.817*half,-.020),tip,
        ],[.031,.038,.032,.023,.009],n=9)
        leaf(m,"Braced bow-tip nocking plate",w,"sun_cream",(0,s*.805*half,-.013),
             (0,s*(half+.036),TIP_Z-.021),.042,facing=(1,0,0))
        if mark:
            for x in (-.033,.033):
                leaf(m,"Forked amethyst bow blade",w,"archer_amethyst",(x,s*.66*half,.020),
                     (x*(1.50 if tier==2 else 1.12),s*(half+.062),TIP_Z-.023),.030)
    if mark and tier==2:
        for x in (-.068,0,.068):
            leaf(m,"Crowned amethyst prong above the bow",w,"ritual",
                 (x*.47,half-.040,TIP_Z-.009),
                 (x,half+(.121 if x==0 else .089),TIP_Z-.018),.021)
    if precision and tier==2:
        m.rim("Crown ring around bow hand",w,"sun_cream",(0,0,.101),(.080,.103),.012,segments=18)
    length = hypot(half,DRAW)
    for node,s in (("StringTop",-1),("StringBottom",1)):
        m.tube("Independent bowstring half",node,"linen",[(0,0,0),(0,s*length,0)],[.004]*2,n=6)
    m.tube("Nocked arrow shaft","Nock","wood",[(0,0,0),(0,0,ARROW-.024)],[.006]*2,n=7)
    leaf(m,"Nocked arrowhead at release","Nock","sun_cream" if precision else "gold",
         (0,0,ARROW-.075),(0,0,ARROW),.019,facing=(0,1,0))
    for s in (-1,1):
        leaf(m,"Nocked arrow fletching","Nock","linen",(0,0,.093),
             (s*.023,0,.014),.015,facing=(0,1,0))
