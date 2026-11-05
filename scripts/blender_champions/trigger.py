"""Build a feline gunslinger with a jointed tail and two open revolver bores."""

from math import cos, sin, tau

MUZZLE = (0,.055,.25)


def trigger(m,tier,branch,*,production=False):
    swift = tier > 0 and branch == 0
    ricochet = tier > 0 and branch == 1
    cloth = "gunner_jade" if swift else "gunner_indigo" if ricochet else "field_slate"
    jacket(m,cloth,swift,ricochet,tier)
    feline(m,cloth,swift,ricochet,tier)
    tail(m,cloth,swift,tier)
    for node in ("WeaponR","OffhandL"):
        pistol(m,node,swift,ricochet,tier)


def jacket(m,cloth,swift,ricochet,tier):
    m.loft("Fitted rider's waistcoat","Chest",cloth,[
        (-.049,.119,.093,0),(.053,.137,.101,0),(.177,.160,.114,.004),
        (.274,.175,.097,-.005),(.344,.091,.072,0),
    ],n=14,fold=.045)
    m.loft("Thick revolver belt","Hips","leather",[(.003,.136,.108,0),(.032,.153,.115,0),
           (.084,.149,.116,0)],n=14)
    m.box("Oval rider belt buckle","Hips","brass",(0,.040,.121),(.066,.051,.025),.012)
    for s in (-1,1):
        m.tube("Piped waistcoat edge","Chest","leather",[(s*.095,.288,.083),(s*.074,.192,.116),
               (s*.102,.088,.105)],[.012,.015,.012],n=7)
        m.ell("Undersized western holster","Hips","hide",(s*.137,-.040,.028),
              (.044,.098,.065),segments=10)
        if ricochet:
            for i in range(3):
                m.tube("Bandolier brass cartridge","Chest","brass",[(s*(.094+.019*i),.145+.033*i,.103),
                       (s*(.094+.019*i),.181+.033*i,.119)],[.010,.010],n=7)
        side = "L" if s>0 else "R"
        arm,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Orange upper sleeve and fur",arm,"gunner_fur",[(0,.024,0),(0,-.079,0),(0,-.188,0)],
               [.064,.064,.042],n=9)
        m.tube("Rolled wrist guard",fore,"hide",[(0,.022,0),(0,-.078,0),(0,-.164,0)],
               [.052,.049,.039],n=9)
        m.ell("Feline gloved paw",hand,"gunner_fur",(0,-.020,.008),(.046,.045,.036),segments=10)
        for joint in (thigh,shin):
            m.tube("Riding trousers",joint,"ink",[(0,.031,0),(0,-.083,0),(0,-.193,0)],
                   [.057,.054,.040],n=9)
        m.ell("Rider boot",foot,"hide",(0,-.023,.038),(.060,.050,.106),segments=11)
        if swift:
            m.tube("Brass boot spur",foot,"gold",[(0,-.004,-.057),(0,.007,-.101),(0,.011,-.127)],
                   [.024,.024,.009],n=8)
    if swift:
        m.loft("Jade riding scarf","Chest","gunner_jade",[(.302,.104,.081,0),(.332,.114,.091,.001),
               (.377,.093,.077,0)],n=13)
        if tier==2:
            m.tube("Long riding scarf tail","Chest","gunner_jade",[(.091,.334,-.021),
                   (.171,.316,-.129),(.254,.328,-.249),(.301,.277,-.306)],
                   [.037,.048,.043,.011],n=8)


def feline(m,cloth,swift,ricochet,tier):
    h = "Head"
    m.ell("Wide feline skull",h,"gunner_fur",(0,.026,.011),(.148,.123,.123),segments=16)
    m.ell("Lower feline jaw",h,"gunner_fur",(0,-.067,.050),(.102,.067,.088),segments=13)
    for s in (-1,1):
        m.ell("Twin cream muzzle pad",h,"linen",(s*.045,-.034,.125),(.058,.038,.053),segments=12)
        m.tube("Pointed outer ear",h,"gunner_fur",[(s*.123,.096,-.025),
               (s*.154,.154,-.027),(s*.158,.209,-.035)],[.054,.044,.005],n=9)
        m.tube("Ear lining",h,"skin",[(s*.142,.111,.006),(s*.155,.156,.005),
               (s*.158,.195,-.016)],[.026,.024,.002],n=7)
        m.ell("Amber feline eye",h,"gold",(s*.065,.030,.120),(.039,.023,.013),segments=10)
        m.ell("Feline slit pupil",h,"ink",(s*.067,.030,.133),(.008,.021,.005),segments=8)
        m.tube("Angular feline brow stripe",h,"darkwood",[(s*.027,.049,.125),
               (s*.067,.059,.133),(s*.106,.050,.113)],[.009,.016,.011],n=7)
        for i in range(2):
            m.tube("Dark cheek stripe",h,"darkwood",[(s*.132,.030-i*.031,.036),
                   (s*.134,.018-i*.029,.079),(s*.116,.001-i*.026,.104)],[.008,.011,.005],n=6)
    m.ell("Charcoal feline nose",h,"ink",(0,-.016,.178),(.034,.021,.021),segments=10)
    brim = .197 if swift else .173
    m.loft("Rolled western brim","HeadGear","hide",[(.106,.123,.113,-.011),
           (.119,brim,.145,-.017),(.143,brim*.96,.142,-.017),(.151,.120,.111,-.024)],n=16)
    m.loft("Pinched western crown","HeadGear","darkwood",[(.139,.113,.100,-.025),
           (.183,.105,.091,-.030),(.231,.084,.078,-.033),(.248,.068,.070,-.033)],n=15)
    m.loft("Intrinsic western hat band","HeadGear",cloth,[(.154,.113,.102,-.026),
           (.185,.106,.096,-.030)],n=15)
    if ricochet and tier==2:
        m.ell("Faceted sharpshooter goggles","HeadGear","brass",(0,.159,.076),
              (.084,.028,.026),segments=10)
        for x in (-.041,.041):
            m.ell("Goggle smoked lens","HeadGear","steel_dark",(x,.162,.098),
                  (.030,.023,.011),segments=9)


def tail(m,cloth,swift,tier):
    m.tube("Flexible base of the striped tail","BackSpring","gunner_fur",[(0,.017,.039),
           (.049,-.043,-.092),(.034,-.017,-.212),(0,.04,-.300)],[.049,.052,.045,.044],n=10)
    m.tube("Articulated upturned tail tip","Back","gunner_fur",[(0,-.008,.028),(.030,.050,-.057),
           (.051,.131,-.080),(.031,.211,-.059),(.006,.245,-.025)],[.044,.045,.041,.037,.023],n=10)
    m.tube("Dark tail stripe and tip","Back","darkwood",[(.040,.101,-.080),(.048,.137,-.080)],
           [.042,.041],n=9)
    m.tube("Cream end of the tail","Back","linen",[(.020,.226,-.048),(.006,.251,-.021)],
           [.034,.018],n=9)
    if swift and tier==2:
        m.tube("Rider tail ribbon","Back",cloth,[(.041,.149,-.077),(.087,.181,-.087),
               (.126,.171,-.129)],[.027,.034,.005],n=8)


def pistol(m,node,swift,ricochet,tier):
    m.box("Curved walnut grip",node,"wood",(0,-.040,.006),(.047,.112,.064),.011)
    m.box("Forged revolver body",node,"slag",(0,.036,.052),(.060,.076,.125),.011)
    m.tube("Fluted revolving cylinder",node,"brass" if swift else "iron",
           [(0,.055,.015),(0,.055,.100)],[.044,.044],n=12)
    m.tube("Revolver barrel toward muzzle",node,"iron",[(0,.055,.093),(0,.055,.201),(0,.055,.233)],
           [.027,.024,.029],n=12)
    rim = [(.023*cos(i*tau/16),MUZZLE[1]+.023*sin(i*tau/16),MUZZLE[2]-.005) for i in range(17)]
    m.tube("Open revolver crown",node,"edge",rim,[.005]*17,n=6)
    m.ell("Recessed black bore",node,"ink",(0,.055,.235),(.017,.017,.003),segments=10)
    if ricochet:
        m.box("Faceted ricochet muzzle jacket",node,"brass",(0,.055,.189),
              (.093,.089,.068),.012)
        if tier==2:
            m.box("Tuned tall revolver sight",node,"iron",(0,.106,.175),(.033,.047,.084),.009)
