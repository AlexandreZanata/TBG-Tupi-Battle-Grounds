"""Build a lean mantis with separate hooked blades and a hip-mounted abdomen."""

from bloomkeeper import leaf
from insect_legs import legs

TIP = (0,-.04,.30)


def mantis(m,tier,branch,*,production=False):
    hunt = tier > 0 and branch == 0
    edge = tier > 0 and branch == 1
    shell = "mantis_jade" if edge else "mantis_fern"
    trunk(m,shell,hunt,edge,tier)
    head(m,shell,hunt,tier)
    for side,s,w in (("L",1,"OffhandL"),("R",-1,"WeaponR")):
        arms(m,side,s,shell,edge,tier)
        blade(m,w,s,edge,tier)
        legs(m,side,.18,shell,"sun_cream" if hunt else "pine",.044)
        if hunt:
            leaf(m,"Ivory shin spur",f"Shin{side}","sun_cream",(s*.013,-.140,.037),
                 (s*.061,-.018,.062),.045)


def trunk(m,shell,hunt,edge,tier):
    m.loft("Narrow serrated mantis thorax","Chest",shell,[
        (-.055,.076,.060,0),(.040,.089,.074,-.013),(.146,.103,.082,-.007),
        (.261,.135,.071,.010),(.326,.085,.053,.022),
    ],n=13)
    for s in (-1,1):
        m.tube("Raised rib on mantis thorax","Chest","pine",[(s*.066,.054,.057),
               (s*.093,.182,.074),(s*.102,.289,.051)],[.017,.021,.015],n=7)
    m.tube("Bent hanging mantis abdomen","BackSpring",shell,[(0,.046,.045),(0,.006,-.079),
           (0,-.080,-.202),(0,-.134,-.298)],[.063,.102,.088,.017],n=12)
    for z,y,r in ((-.071,.008,.102),(-.130,-.033,.105),(-.188,-.068,.084)):
        m.tube("Segment band on abdomen","BackSpring","pine",[(0,y+.010,z-.015),(0,y-.010,z+.015)],
               [r,r],n=11)


def head(m,shell,hunt,tier):
    h = "Head"
    m.loft("Inverted triangular mantis mask",h,shell,[
        (-.112,.035,.053,.021),(-.041,.072,.073,.018),(.052,.126,.089,.002),
        (.099,.128,.083,-.009),(.137,.055,.058,-.020),
    ],n=13)
    for s in (-1,1):
        m.ell("Raised compound eye",h,"oldbone",(s*.108,.063,.073),(.058,.061,.055),segments=12)
        m.ell("Dark mantis eye spot",h,"ink",(s*.120,.056,.117),(.019,.026,.011),segments=9)
        height = .355 if hunt and tier==2 else .269
        m.tube("Swept segmented antenna","HeadGear","gold",[(s*.067,.112,-.007),
               (s*.127,.192,-.034),(s*.167,height,-.023)],[.017,.014,.004],n=7)
        if hunt and tier==2:
            leaf(m,"Ivory brow crest","HeadGear","sun_cream",(s*.032,.103,.023),
                 (s*.090,.275,.009),.030)


def arms(m,side,s,shell,edge,tier):
    upper,fore,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
    m.tube("Slender mantis upper arm",upper,shell,[(0,.024,0),(0,-.092,.009),(0,-.234,0)],
           [.043,.058,.035],n=8)
    m.tube("Thick mantis scythe sleeve",fore,shell,[(0,.023,0),(0,-.092,.014),(0,-.208,0)],
           [.051,.066,.037],n=9)
    m.ell("Claw pivot joint",hand,"pine",(0,-.015,.020),(.042,.041,.042),segments=9)
    if edge and tier==2:
        for i in range(3):
            y = -.054-.042*i
            leaf(m,"Serrated elbow crystal",fore,"mantis_jade",(s*.045,y,.029),
                 (s*(.133-.016*i),y-.034,.005),.028)


def blade(m,node,s,edge,tier):
    m.tube("Hooked scythe ridge",node,"pine" if edge else "gold",[
        (0,.005,-.026),(s*.038,-.081,.047),(s*.074,-.108,.148),(s*.044,-.078,.246),TIP,
    ],[.037,.057,.050,.030,.003],n=9)
    leaf(m,"Beveled cutting face at scythe tip",node,"edge" if edge else "sun_cream",
         (s*.034,-.100,.088),TIP,.061 if edge else .046)
    if edge:
        leaf(m,"Secondary jade cutting hook",node,"mantis_jade",(s*.039,-.065,.098),
             (s*(.133 if tier==2 else .104),-.013,.218),.037)
