"""Tailor the ritualist and build the spindle around its native release."""

from math import cos, sin, tau

from mathutils import Vector

RELEASE = (0,.84,.03)


def bloodweaver(model, tier, branch, *, production=False):
    offering = tier > 0 and branch == 0
    circulation = tier > 0 and branch == 1
    robe(model,tier,offering,circulation)
    head(model,tier,offering,circulation)
    spindle(model,"WeaponR",tier,offering,circulation)
    if offering:
        chalice(model,tier)
    elif circulation and tier == 2:
        spindle(model,"OffhandL",tier,False,True,short=True)
        for x,y,z in ((-.245,.27,.035),(.245,.18,.020),(0,.41,-.044)):
            model.ell("Suspended blood drop","BackSpring","blood_glow",(x,y,z),
                      (.028,.050,.029),segments=9,rings=5)
    else:
        for x,y,z in ((-.017,.101,.062),(.023,.182,.050)):
            model.ell("Suspended blood drop","OffhandL","blood_glow",(x,y,z),
                      (.021,.035,.021),segments=9,rings=5)


def robe(model,tier,offering,circulation):
    cloth = "ritual_indigo" if circulation else "wine"
    hem = "ritual_indigo" if circulation else "plum"
    trim = "edge" if circulation else "gold" if offering else "oldbone"
    model.loft("Tailored ritual bodice","Chest",cloth,[
        (-.052,.117,.087,0),(.067,.143,.111,0),(.151,.155,.110,0),
        (.250,.163,.106,-.005),(.328,.137,.094,-.006),(.369,.075,.069,0),
    ],n=15,fold=.059)
    model.loft("Waist-bound pleated skirt","Hips",hem,[
        (-.369,.221,.173,.005),(-.325,.212,.162,.005),(-.195,.170,.132,0),
        (-.068,.143,.111,0),(.043,.128,.094,0),
    ],n=16,opening=.27,fold=.109)
    model.loft("Ritual belt","Hips",trim,[(-.020,.133,.109,0),(.023,.136,.110,0)],n=14)
    for side,sign in (("R",-1),("L",1)):
        arm,fore,hand = (f"{name}{side}" for name in ("UpperArm","Forearm","Hand"))
        model.tube("Full ritual sleeve",arm,cloth,[(0,.042,0),(-sign*.022,-.052,.008),(0,-.171,0)],
                   [.061,.073,.038],n=9)
        model.tube("Gathered ritual cuff",fore,hem,[(0,.025,0),(0,-.078,.005),(0,-.151,.005)],
                   [.047,.051,.044],n=9)
        for i in range(3 if circulation else 1):
            y = -.134+i*.041
            model.loft("Spindle winding cuff",fore,"ritual" if circulation else trim,[
                (y-.009,.054,.049,.005),(y+.008,.055,.051,.005),
            ],n=10)
        model.ell("Bony palm",hand,"bone",(0,-.013,.010),(.039,.050,.032),segments=9,rings=5)
        for x in (-.026,0,.026):
            model.tube("Separate finger bone",hand,"oldbone",[(x,-.021,.025),(x,-.062,.035),(x,-.068,.015)],
                       [.010,.010,.006],n=6)
        model.tube("Shrouded leg",f"Shin{side}","hide",[(0,.021,0),(0,-.112,0),(0,-.184,0)],
                   [.048,.048,.038],n=8)
        model.loft("Pointed leather shoe",f"Foot{side}","hide",[
            (-.060,.055,.109,.042),(-.047,.062,.116,.046),(.028,.047,.073,.014),(.050,.039,.050,0),
        ],n=10)
        if offering:
            model.tube("Ivory ceremonial stole","Chest","bone",[
                (sign*.165,.304,.023),(sign*.123,.299,.118),(sign*.096,.200,.138),
                (sign*.076,.088,.111),
            ],[.045,.039,.030,.023],n=9)
        if circulation:
            model.tube("Thread running over the robe","Chest","ritual",[
                (sign*.169,.284,.039),(sign*.131,.267,.112),(sign*.106,.181,.130),
                (sign*.050,.127,.126),
            ],[.009,.014,.013,.005],n=6)
    if offering and tier == 2:
        c = Vector((0,.195,.143))
        for edge in range(4):
            a = edge*tau/4
            p = [c+Vector((cos(a)*.070*sin(i*tau/12),.083*cos(i*tau/12),
                           sin(a)*.051*sin(i*tau/12))) for i in range(7)]
            model.tube("Gilt heart reliquary rib","Chest","gold",p,[.009]*7,n=6)
        model.ell("Blood heart","Chest","blood_glow",(0,.191,.151),(.046,.058,.035),
                  segments=11,rings=6)


def head(model,tier,offering,circulation):
    cloth = "ritual_indigo" if circulation else "wine"
    model.loft("Tapered skull","Head","bone",[
        (-.111,.049,.055,.020),(-.079,.079,.078,.017),(-.010,.111,.101,.010),
        (.074,.106,.090,-.008),(.128,.079,.064,-.011),
    ],n=15)
    model.loft("Pointed ritual hood","HeadGear",cloth,[
        (-.124,.110,.106,-.025),(-.068,.140,.125,-.023),(.077,.147,.139,-.012),
        (.151,.120,.109,-.015),(.222,.036,.064,-.036),
    ],n=16,opening=.85,fold=.068)
    for s in (-1,1):
        model.ell("Carved eye hollow","Head","ink",(s*.051,.022,.104),
                  (.030,.027,.015),segments=10,rings=5)
        model.ell("Ember in the eye hollow","Head","blood_glow",(s*.051,.019,.117),
                  (.012,.012,.007),segments=8,rings=4)
        model.tube("Bound hood edge","HeadGear","edge" if circulation else "gold" if offering else "oldbone",
                   [(s*.038,.164,.106),(s*.108,.102,.106),(s*.119,.010,.085),
                    (s*.097,-.089,.074)],[.009,.014,.012,.007],n=7)
    model.ell("Nasal hollow","Head","ink",(0,-.034,.118),(.013,.023,.009),segments=8,rings=4)
    for x in (-.038,-.019,0,.019,.038):
        model.box("Carved tooth","Head","linen",(x,-.077,.103),(.013,.024,.019),.002)
    if offering and tier == 2:
        model.tube("Reliquary hood finial","HeadGear","gold",[(0,.155,.092),(0,.208,.077),(0,.256,.046)],
                   [.018,.025,.003],n=8)


def spindle(model,node,tier,offering,circulation,short=False):
    y = .53 if short else RELEASE[1]
    z = RELEASE[2]
    trim = "edge" if circulation else "gold" if offering else "bone"
    model.tube("Turned spindle shaft",node,"darkwood",[(0,-.196,0),(0,.077,0),(0,y-.135,.015),(0,y,z)],
               [.024,.026,.026,.024],n=9)
    model.loft("Knotted grip winding",node,"hide",[(-.077,.034,.032,0),(.089,.033,.034,0)],n=10)
    model.ell("Blood-thread spindle center",node,"blood_glow",(0,y,z),
              (.055,.084,.051),segments=12,rings=6)
    for i in range(2 if circulation else 1):
        path=[((.078+i*.017)*cos(j*tau/18),y-.076+.158*j/18,
               z+(.063+i*.017)*sin(j*tau/18)) for j in range(19)]
        model.tube("Wound spindle thread",node,"ritual" if circulation else trim,path,
                   [.008]*19,n=5)
    for s in (-1,1):
        model.tube("Crown of the spindle",node,trim,[(s*.018,y-.103,z),(s*.070,y-.026,z),
                   (s*.066,y+.061,z),(s*.021,y+.120,z)], [.021,.018,.013,.003],n=7)


def chalice(model,tier):
    node = "OffhandL"
    model.loft("Gilt offering chalice",node,"gold",[
        (-.014,.051,.042,.049),(.009,.048,.042,.049),(.044,.018,.018,.049),
        (.069,.022,.019,.049),(.091,.052,.048,.049),(.135,.067,.056,.049),
    ],n=14)
    model.ell("Blood in offering cup",node,"blood_glow",(0,.139,.049),
              (.058,.012,.046),segments=12,rings=4)
    if tier==2:
        for s in (-1,1):
            model.tube("Chalice handle",node,"gold",[(s*.054,.129,.049),
                       (s*.103,.107,.050),(s*.057,.079,.049)],[.009,.012,.009],n=7)
