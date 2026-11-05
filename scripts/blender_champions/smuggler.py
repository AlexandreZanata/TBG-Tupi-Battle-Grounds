"""Build a coastal courier with a seated revolver bore and detachable cargo."""

from math import cos, sin, tau

from bloomkeeper import leaf

MUZZLE = (0,.04,.30)


def smuggler(m,tier,branch,*,production=False):
    merchant = tier > 0 and branch == 0
    courier = tier > 0 and branch == 1
    cloth = "ochre_cloth" if merchant else "smuggler_coast" if courier else "smuggler_rust"
    clothing(m,cloth,merchant,tier)
    head(m,cloth,merchant,courier,tier)
    revolver(m)
    cargo(m,merchant,courier,tier)
    lantern(m,"Hips",(.185,-.085,.055),.79)


def clothing(m,cloth,merchant,tier):
    m.loft("Cuffed longshore jacket","Chest",cloth,[
        (-.050,.143,.105,.005),(.045,.163,.109,.008),(.157,.188,.122,.008),
        (.257,.208,.116,0),(.306,.183,.100,-.004),(.348,.115,.077,-.007),
    ],n=15,fold=.055)
    m.loft("Rolled longshore collar","Chest","hide",[(.303,.115,.087,.001),(.350,.106,.075,-.003)],n=14)
    m.loft("Gathered hip sash","Hips","wine",[(.017,.161,.116,0),(.052,.190,.137,0),(.079,.183,.131,0)],
           n=14,fold=.09)
    m.tube("Diagonal cargo harness","Chest","leather",[(-.145,.304,.089),(-.075,.211,.127),
           (.023,.127,.137),(.123,.047,.105)],[.026,.026,.028,.023],n=7)
    m.box("Harness buckle","Chest","brass",(-.068,.201,.145),(.057,.053,.018),.005,rotation=.37)
    m.box("Inset buckle leather","Chest","hide",(-.068,.201,.156),(.034,.029,.011),.003,rotation=.37)
    for s in (-1,1):
        arm,fore,hand,thigh,shin,foot = (f"{p}{'L' if s>0 else 'R'}" for p in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Tailored courier sleeve",arm,cloth,[(0,.035,0),(0,-.063,0),(0,-.208,0)],
               [.071,.086,.051],n=10)
        m.tube("Leather forearm bracer",fore,"hide",[(0,.019,0),(0,-.096,0),(0,-.172,0)],
               [.052,.057,.044],n=9)
        m.ell("Gripping hand",hand,"skin",(0,-.023,.008),(.047,.052,.039),segments=10)
        m.ell("Thumb over grip",hand,"skin",(s*.032,-.004,.033),(.022,.030,.020),segments=8)
        for joint in (thigh,shin):
            m.tube("Gathered work trousers",joint,"field_slate",[(0,.033,0),(0,-.086,0),(0,-.190,0)],
                   [.072,.067,.052],n=9)
        m.ell("Squared deck boot",foot,"hide",(0,-.013,.039),(.067,.055,.101),segments=12)
        m.box("Deck boot sole",foot,"ink",(0,-.051,.043),(.124,.025,.165),.007)
        if merchant:
            m.ell("Coin purse","Hips","gold",(s*.107,-.047,.109),(.049,.053,.029),segments=9)
    if merchant and tier == 2:
        m.loft("Short back-spring cape","BackSpring","leather",[
            (-.345,.172,.062,-.068),(-.252,.161,.061,-.051),(-.105,.126,.047,-.032),
            (.054,.111,.027,.001),(.095,.071,.031,.010),
        ],n=13,opening=1.84,fold=.10)


def head(m,cloth,merchant,courier,tier):
    h = "Head"
    m.loft("Weathered courier face",h,"skin",[
        (-.108,.061,.065,.018),(-.087,.088,.081,.016),(-.008,.104,.099,0),
        (.063,.099,.091,-.005),(.112,.073,.070,-.014),
    ],n=14)
    m.ell("Bridge of nose",h,"skin",(0,-.007,.109),(.025,.038,.029),segments=11)
    m.ell("Trimmed pointed beard",h,"darkwood",(0,-.080,.066),(.074,.053,.058),segments=12)
    m.tube("Angular moustache",h,"darkwood",[(-.059,-.049,.092),(0,-.043,.119),(.056,-.052,.090)],
           [.014,.014,.009],n=7)
    m.ell("Exposed sharp eye",h,"linen",(-.048,.026,.091),(.025,.015,.012),segments=9)
    m.ell("Exposed pupil",h,"ink",(-.048,.026,.102),(.010,.012,.005),segments=8)
    m.tube("Eyepatch strap",h,"hide",[(-.099,.055,.029),(-.065,.060,.090),(.057,.025,.102),
           (.101,.015,.040)],[.009]*4,n=6)
    m.ell("Cupped eyepatch",h,"ink",(.048,.025,.094),(.034,.028,.014),segments=9)
    if courier:
        m.loft("Wide coastal bandanna",h,cloth,[(.054,.105,.100,0),(.093,.108,.099,-.005),
               (.123,.083,.073,-.012)],n=14)
        m.tube("Knotted bandanna tail",h,cloth,[(.065,.074,-.071),(.117,.046,-.091),
               (.159,-.033,-.114)],[.028,.032,.005],n=8)
        return
    m.loft("Pinched tricorn crown","HeadGear","hide",[(.074,.101,.088,-.014),
           (.146,.116,.104,-.014),(.169,.092,.084,-.018)],n=14)
    for a in (0,tau/3,2*tau/3):
        pts = [(.156*sin(a+d),.113+.052*cos(d*2),-.022+.129*cos(a+d)) for d in (-.92,-.45,0,.45,.92)]
        m.tube("Rolled tricorn brim edge","HeadGear","gold" if merchant else "leather",pts,[.020]*5,n=7)
        m.tube("Folded tricorn brim","HeadGear","ink",[(x*.90,y-.028,z*.93) for x,y,z in pts],
               [.025,.032,.039,.032,.025],n=8)
    if merchant and tier==2:
        leaf(m,"Merchant's tall gilded plume","HeadGear","gold",(.103,.136,-.019),
             (.238,.345,-.098),.047)


def revolver(m):
    w = "WeaponR"
    m.box("Carved walnut revolver grip",w,"wood",(0,-.054,.011),(.060,.119,.065),.010)
    m.box("Slotted iron receiver",w,"slag",(0,.022,.068),(.071,.077,.123),.009)
    m.tube("Bronze fluted cylinder",w,"brass",[(0,.04,.071),(0,.04,.134)],[.050,.050],n=12)
    for i in range(6):
        a = i*tau/6
        x,y = .048*cos(a),.04+.048*sin(a)
        m.tube("Dark cylinder flute",w,"hide",[(x,y,.078),(x,y,.128)],[.006,.006],n=6)
    m.tube("Tapered steel barrel",w,"iron",[(0,.04,.114),(0,.04,.255),(0,.04,.278)],
           [.034,.027,.032],n=12)
    rim = [(MUZZLE[0]+.026*cos(a*tau/16),MUZZLE[1]+.026*sin(a*tau/16),MUZZLE[2]-.006)
           for a in range(17)]
    m.tube("Hollow muzzle crown",w,"edge",rim,[.007]*17,n=6)
    m.ell("Recessed black muzzle bore",w,"ink",(0,.04,.280),(.020,.020,.003),segments=11,rings=5)
    m.box("Raised front sight",w,"steel_dark",(0,.073,.254),(.013,.019,.028),.003)


def cargo(m,merchant,courier,tier):
    b = "BackSpring"
    if courier:
        m.ell("Rounded courier satchel",b,"leather",(.021,-.102,-.074),(.149,.119,.086),segments=12)
        m.box("Satchel envelope flap",b,"smuggler_coast",(.022,-.063,-.143),(.212,.104,.032),.011)
        m.ell("Satchel button",b,"brass",(.022,-.100,-.168),(.018,.018,.011),segments=8)
        if tier==2:
            crate(m,b,(-.095,-.102,-.137),.76,False)
            crate(m,"OffhandL",(.122,-.025,-.005),.91,False)
            m.tube("Pole for route lantern",b,"wood",[(.139,-.218,-.126),(.173,.188,-.132),
                   (.152,.250,-.081)],[.017,.017,.012],n=8)
            lantern(m,b,(.152,.162,-.080),.72)
        return
    crate(m,b,(0,-.103,-.107),1,merchant and tier==2)
    if merchant:
        crate(m,b,(0,.061,-.112),.72,tier==2)


def crate(m,node,center,scale,gilt):
    x,y,z = center
    trim = "gold" if gilt else "iron"
    m.box("Cargo box",node,"wood",center,tuple(scale*d for d in (.251,.179,.164)),.011)
    for sign in (-1,1):
        m.box("Cargo iron corner band",node,trim,(x+sign*.091*scale,y,z),
              tuple(scale*d for d in (.027,.188,.176)),.006)
    for u in (-.041,.041):
        m.box("Cargo recessed plank seam",node,"darkwood",(x,y+u*scale,z-.083*scale),
              tuple(scale*d for d in (.239,.009,.009)),.002)
    if gilt:
        m.box("Gilded chest lock plate",node,"gold",(x,y,z-.086*scale),
              tuple(scale*d for d in (.064,.065,.021)),.006)


def lantern(m,node,center,scale):
    x,y,z = center
    m.ell("Amber lantern glass",node,"sap",center,tuple(scale*d for d in (.037,.050,.035)),segments=10)
    for sy in (-1,1):
        m.box("Lantern frame cap",node,"brass",(x,y+sy*.049*scale,z),
              tuple(scale*d for d in (.088,.019,.079)),.006)
    for sx,sz in ((-1,-1),(-1,1),(1,-1),(1,1)):
        m.box("Lantern cage post",node,"slag",(x+sx*.033*scale,y,z+sz*.029*scale),
              tuple(scale*d for d in (.011,.096,.012)),.003)
