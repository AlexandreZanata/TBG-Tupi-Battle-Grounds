"""Two original TBG costume sets, modeled as editable Blender components."""

from math import cos, exp, pi, sin, sqrt, tau

HUMAN_FACE = [
    (-.114,.021,.030,.030),(-.103,.053,.060,.019),(-.083,.072,.076,.016),
    (-.058,.083,.096,.005),(-.032,.089,.106,.001),(-.007,.095,.106,.002),
    (.017,.094,.109,0),(.036,.094,.105,0),(.058,.097,.103,-.006),
    (.080,.094,.099,-.011),(.108,.086,.088,-.014),(.137,.059,.060,-.017),
    (.152,.006,.009,-.017),
]
BONE_FACE = [
    (-.123,.022,.030,.045),(-.106,.061,.065,.038),(-.078,.073,.072,.040),
    (-.056,.070,.078,.041),(-.039,.095,.084,.030),(-.021,.120,.102,.012),
    (.005,.119,.111,.007),(.035,.117,.126,-.004),(.065,.120,.123,-.006),
    (.092,.110,.114,-.014),(.127,.095,.095,-.018),(.156,.065,.068,-.020),
    (.170,.006,.009,-.019),
]
HOOD = [
    (-.130,.127,.122,-.012),(-.086,.142,.147,-.020),(-.006,.132,.161,-.017),
    (.060,.128,.161,-.018),(.120,.114,.127,-.024),(.158,.083,.106,-.035),
    (.190,.036,.074,-.064),(.199,.004,.010,-.085),
]
CAPES = [
    (-.367,.237,.113,-.124),(-.324,.244,.119,-.098),(-.215,.219,.094,-.065),
    (-.081,.180,.081,-.032),(.034,.159,.074,-.004),(.104,.094,.050,.010),
]


def mound(x, y, a, b, sx, sy):
    return exp(-((x-a)/sx)**2 - ((y-b)/sy)**2)


def face_transform(point, skull):
    x,y,z=point
    front=max(0,min(1,(z+.014)/.091))**3
    ax=abs(x)
    if skull:
        relief=(.023*mound(ax,y,.061,.062,.044,.015)
                -.045*mound(ax,y,.057,.031,.036,.027)
                -.022*mound(x,y,0,-.026,.020,.026)
                +.015*mound(ax,y,.095,-.026,.029,.018)
                -.010*mound(ax,y,0,-.086,.070,.007))
    else:
        relief=(.029*mound(x,y,0,.015,.017,.041)
                +.028*mound(x,y,0,-.018,.025,.018)
                -.021*mound(ax,y,.048,.033,.024,.015)
                +.010*mound(ax,y,.065,-.010,.028,.025)
                +.008*mound(x,y,0,-.057,.032,.009))
    return (x,y,z+front*relief)


def human_head(m, main, trim, tier, branch):
    m.loft("Sculpted face", "Head", "skin", HUMAN_FACE, n=22, transform=lambda p:face_transform(p,False))
    m.loft("Folded ranger hood", "Head", main, HOOD, n=16, opening=.66, fold=.055)
    for sign in (-1,1):
        seam=[]
        for y,rx,rz,z in HOOD[1:-1]:
            seam.append((sign*(rx*sin(.66)+.004),y,z+rz*cos(.66)+.005))
        m.tube("Hood piping", "Head", trim, seam, [.006]*len(seam),n=5)
        m.ell("Eye", "Head", "linen", (sign*.047,.031,.093), (.014,.005,.009),segments=8)
        m.ell("Pupil", "Head", "ink", (sign*.045,.031,.101), (.006,.005,.004),segments=7)
        m.tube("Brow", "Head", "hide", [(sign*.026,.051,.111),(sign*.047,.055,.099),(sign*.069,.049,.086)], [.0025,.004,.002],n=5)
    m.tube("Mouth line", "Head", "leather", [(-.032,-.062,.101),(0,-.062,.109),(.032,-.062,.101)], [.002,.003,.002],n=5)
    if tier:
        x=.11 if branch==0 else -.11
        m.plate("Ranger's leaf crest", "Head", trim, [(x,.09),(x*1.28,.14),(x*1.62,.24+tier*.018),(x*1.19,.20),(x*.88,.145)],(-.055,-.045),bevel=.003)


def archer(m,tier,branch,*,production=False):
    cloth="moss" if branch==0 else "wine"
    deep="pine" if branch==0 else "plum"
    trim="gold" if tier==2 else "brass"
    human_head(m,deep,trim,tier,branch)
    m.loft("Tailored doublet", "Chest", cloth, [(-.04,.116,.090,0),(.028,.136,.101,0),(.14,.158,.107,0),(.239,.176,.113,0),(.294,.153,.106,0),(.337,.083,.080,0),(.376,.066,.067,0)],n=14,fuse=True)
    m.loft("Split riding tunic", "Hips", cloth, [(-.182,.153,.108,.011),(-.167,.177,.117,.002),(-.066,.151,.115,0),(.033,.133,.104,0),(.067,.111,.093,0)],n=14,fold=.06,fuse=True)
    m.loft("High collar", "Chest", deep, [(.294,.097,.089,0),(.314,.102,.088,0),(.375,.081,.078,0)],n=12)
    m.loft("Leather waist belt", "Hips", "hide", [(.021,.145,.110,0),(.057,.143,.108,0)],n=12)
    m.plate("Engraved buckle", "Hips", trim, [(-.032,.018),(.032,.018),(.043,.044),(.024,.064),(-.026,.064),(-.043,.042)],(.109,.127))
    m.box("Hip pouch", "Hips", "leather", (-.119,-.04,.088),(.087,.103,.058),rotation=-.17)
    m.tube("Chest baldric", "Chest", "leather", [(-.13,.087,.093),(-.070,.143,.111),(.020,.221,.112),(.119,.303,.084)],[.016,.022,.022,.014],n=7)
    m.ell("Baldric clasp", "Chest", trim, (.039,.228,.125), (.021,.022,.010),segments=10)
    m.loft("Layered travel cape", "BackSpring", deep, CAPES, n=14, opening=1.32,fold=.12,
           transform=lambda p:(p[0]*(.77+.1*tier),p[1]*(.69+.13*tier),p[2]))
    for side,s in (("L",1),("R",-1)):
        m.loft("Upper sleeve",f"UpperArm{side}",cloth,[(-.199,.053,.055,0),(-.141,.061,.060,0),(-.056,.073,.068,0),(.017,.069,.061,0),(.042,.041,.041,0)],n=10,fuse=True,transform=lambda p:(p[0]-.018*s,p[1],p[2]))
        m.loft("Lower sleeve",f"Forearm{side}",cloth,[(-.177,.035,.040,0),(-.113,.047,.043,0),(-.027,.057,.051,0),(.020,.052,.052,0)],n=10,fuse=True)
        m.loft("Laced leather bracer",f"Forearm{side}","hide",[(-.161,.043,.052,0),(-.144,.053,.057,0),(-.058,.059,.062,0),(-.036,.052,.055,0)],n=10)
        if side=="L" or tier==2:
            m.plate("Forged vambrace",f"Forearm{side}",trim,[(-.044,-.145),(.036,-.151),(.049,-.083),(.032,-.042),(-.036,-.050)],(.047,.062))
        m.ell("Gloved grip",f"Hand{side}","leather",(0,-.022,.017),(.047,.064,.046),fuse=True,segments=10)
        m.ell("Thumb over grip",f"Hand{side}","skin",(.025*s,-.021,.047),(.023,.039,.024),segments=9)
        m.loft("Breeches",f"Thigh{side}",deep,[(-.181,.046,.054,0),(-.133,.056,.063,0),(-.043,.071,.069,0),(.028,.065,.068,0)],n=10,fuse=True)
        m.loft("Leather gaiter",f"Shin{side}","leather",[(-.190,.043,.053,0),(-.112,.050,.057,0),(-.010,.055,.062,0),(.028,.051,.051,0)],n=10,fuse=True)
        m.loft("Rolled boot edge",f"Shin{side}",deep,[(-.044,.059,.064,0),(-.013,.062,.069,0),(.009,.054,.063,0)],n=10)
        m.ell("Turned boot toe",f"Foot{side}","hide",(0,-.016,.051),(.054,.050,.104),fuse=True,segments=11)
    bow(m,tier,branch,trim,production)
    for x in ([.06] if branch==0 else [-.077,.075]):
        m.tube("Quiver shell","Back","hide",[(x-.035,-.16,-.017),(x,-.038,-.034),(x+.034,.14,-.056)],[.041,.055,.057],n=9)
        m.tube("Quiver rim","Back",trim,[(x+.026,.125,-.056),(x+.034,.153,-.063)],[.059,.059],n=9)
        for k in range(3):
            xx=x+.008+k*.021
            m.tube("Arrow shaft","Back","wood",[(xx,-.02,-.057),(xx+.027,.235+k*.023,-.071)],[.005,.004],n=5)
            m.plate("Arrow fletching","Back","linen",[(xx+.020,.199+k*.023),(xx+.051,.205+k*.023),(xx+.050,.248+k*.023),(xx+.023,.239+k*.023)],(-.075,-.068),bevel=0)


def bow(m,tier,branch,trim,production):
    # The earlier Blender study and the current animated bow use different sockets.
    half=((.64,.74,.70) if branch==0 else (.64,.65,.68))[tier] if production else (.64,.74,.68)[tier]
    center=0 if production else .08
    tip=-.10 if production else -.23
    points=[]
    for i in range(17):
        t=(i-8)/8
        u=abs(t)
        z=tip*u+.44*u*(1-u) if production else tip+.288*(1-t*t)**.83+.028*sin(pi*t)**2
        points.append((0,center+half*t,z))
    m.tube("Laminated yew bow","OffhandL","wood",points,[.013+.010*(1-abs((i-8)/8)) for i in range(17)],n=8)
    m.tube("Dark belly laminate","OffhandL","darkwood",[(x+.004,y,z-.014) for x,y,z in points],[.009]*17,n=6)
    grip=0 if production else .063
    m.tube("Bound bow grip","OffhandL","hide",[(0,center-.079,grip-.009),(0,center+.001,grip),(0,center+.082,grip-.009)],[.025,.031,.025],n=9)
    for sign in (-1,1):
        y=center+sign*half
        m.tube("Nock ferrule","OffhandL",trim,[(0,y-sign*.042,tip+.034),(0,y,tip),(0,y+sign*.016,tip-.008)],[.015,.014,.009],n=7)


def skull(m,tier,branch):
    m.loft("Sculpted ancient skull","Head","bone",BONE_FACE,n=22,transform=lambda p:face_transform(p,True))
    for s in (-1,1):
        m.ell("Orbital shadow","Head","ink",(s*.057,.032,.088),(.032,.024,.013),segments=11)
        m.ell("Soul ember","Head","soul",(s*.057,.033,.101),(.012,.010,.011),segments=9)
        m.tube("Cheek arch","Head","oldbone",[(s*.112,-.008,.052),(s*.095,-.034,.073),(s*.068,-.043,.104)],[.013,.016,.010],n=7)
    m.ell("Nasal hollow","Head","ink",(0,-.024,.111),(.013,.022,.007),segments=9)
    m.tube("Jaw shadow","Head","ink",[(-.063,-.080,.093),(0,-.086,.120),(.063,-.080,.093)],[.003,.007,.003],n=6)
    for i in range(6):
        x=(i-2.5)*.018
        m.ell("Worn tooth","Head","bone",(x,-.081,.121-abs(x)*.31),(.007,.012,.005),segments=7)
    if tier==2 and branch==1:
        m.loft("Reaper's hood","Head","plum",HOOD,n=15,opening=.76,fold=.095,transform=lambda p:(p[0]*1.16,p[1]+.018,p[2]-.014))
    if tier==2 and branch==0:
        for s in (-1,1):
            m.tube("Grave-crown horn","Head","oldbone",[(s*.087,.095,-.002),(s*.132,.136,-.009),(s*.166,.196,-.024),(s*.164,.239,-.051)],[.039,.033,.018,.003],n=8)


def brute(m,tier,branch,*,production=False):
    robe="cloth" if branch==0 else "wine"
    skull(m,tier,branch)
    m.loft("Withered spinal trunk","Chest","slag",[(-.04,.081,.056,-.055),(.056,.118,.078,-.048),(.21,.122,.083,-.045),(.30,.141,.071,-.041),(.374,.085,.053,-.008)],n=12,fuse=True)
    m.loft("Bone pelvis","Hips","oldbone",[(-.094,.102,.071,0),(-.034,.142,.094,0),(.029,.168,.108,0),(.066,.122,.077,-.024)],n=12,fuse=True)
    m.ell("Spectral core","Chest","soul",(0,.215,.014),(.078,.108,.062),segments=14)
    for k in range(5):
        y=.091+k*.055
        w=(.131,.166,.193,.197,.170)[k]
        for s in (-1,1):
            pts=[(s*w*sin(a),y-.032*sin(a*.8),-.019+.115*cos(a)) for a in [j*pi/8 for j in range(9)]]
            m.tube("Curved rib","Chest","bone",pts,[.014,.017,.019,.021,.021,.020,.018,.016,.012],n=6)
    m.tube("Tapered sternum","Chest","oldbone",[(0,.075,.096),(0,.174,.103),(0,.282,.102),(0,.328,.072)],[.013,.020,.026,.017],n=7)
    m.loft("Frayed burial mantle","BackSpring",robe,CAPES,n=15,opening=1.3,fold=.13)
    if tier == 0:
        # The waist edge sits under the belt and shares its pivot.
        m.loft("Belt-bound burial sash", "Hips", robe,
               [(-.294,.149,.085,-.046),(-.260,.143,.084,-.042),
                (-.15,.139,.087,-.026),(.039,.154,.110,0)],
               n=12, opening=1.33, fold=.08, transform=lambda p:(p[0],p[1],-p[2]))
    else:
        m.loft("Draped burial sash","Tabard",robe,[(-.294,.149,.079,0),(-.260,.137,.079,0),(-.15,.135,.087,0),(-.03,.104,.066,0)],n=12,opening=1.33,fold=.15,
               transform=lambda p:(p[0],p[1],-p[2]+.021))
    m.loft("Gravebelt","Hips","leather",[(.024,.175,.121,0),(.059,.163,.112,0)],n=12)
    m.plate("Rune clasp","Hips","patina",[(-.044,.020),(0,.005),(.044,.020),(.041,.060),(0,.079),(-.041,.060)],(.111,.134))
    arm = .32 if production and tier==2 and branch==1 else .25
    for side,s in (("L",1),("R",-1)):
        if tier == 0:
            # End under the pauldron at the arm pivot, where arm rotation cannot open a gap.
            m.tube("Shoulder clavicle", "Chest", "bone",
                   [(s*.062,.330,.044),(s*.138,.349,.034),
                    (s*.225,.350,.021),(s*.300,.342,0)],
                   [.032,.037,.040,.044], n=8)
        for joint,length,r in ((f"UpperArm{side}",arm,.042),(f"Forearm{side}",arm*.9,.034),(f"Thigh{side}",.18,.044),(f"Shin{side}",.18,.038)):
            m.tube("Tapered long bone",joint,"bone",[(0,.028,0),(.006*s,-.024,0),(0,-length*.62,-.009),(-.005*s,-length+.012,0),(0,-length-.017,0)],
                   [r*.87,r*1.29,r*.76,r*1.20,r*.85],n=8,fuse=True)
        m.ell("Metacarpal block",f"Hand{side}","oldbone",(0,-.031,0),(.053,.057,.038),fuse=True,segments=10)
        for n in (-1,0,1):
            x=n*.034
            m.tube("Articulated claw",f"Hand{side}","bone",[(x,-.044,.013),(x,-.093,.029),(x*.97,-.125,.049)],[.012,.010,.003],n=6)
            m.tube("Taloned toe",f"Foot{side}","bone",[(x,-.026,-.042),(x,-.031,.049),(x*1.3,-.037,.114)],[.020,.017,.003],n=6)
        m.ell("Heel bone",f"Foot{side}","oldbone",(0,-.018,-.015),(.047,.045,.063),fuse=True,segments=10)
        m.loft("Forged grave pauldron",f"UpperArm{side}","patina" if tier else "slag",[(-.055,.111,.104,0),(-.031,.118,.109,0),(.021,.096,.094,-.005),(.067,.058,.065,-.015),(.081,.021,.026,-.020)],n=12,transform=lambda p:(p[0]+s*.016,p[1],p[2]))
        m.tube("Pauldron horn",f"UpperArm{side}","bone",[(s*.038,.046,-.009),(s*.080,.111,-.013),(s*.116,.148,-.027)],[.029,.022,.004],n=7)
        m.plate("Riveted bone cuff",f"Forearm{side}","iron",[(-.045,-.163),(.041,-.171),(.048,-.102),(.029,-.073),(-.035,-.082)],(.034,.056))
    if tier==2 and branch==1:
        radius=.45 if production else .43
        m.tube("Reaper ash haft","WeaponR","darkwood",[(0,-.34,0),(0,.32,0),(0,.91,0)],[.024,.029,.025],n=9)
        m.tube("Crescent reaping edge","WeaponR","edge",[(0,.50+radius*cos(a),radius*sin(a)) for a in [i*1.71/14 for i in range(15)]],
               [.043-.032*i/14 for i in range(15)],n=8)
    else:
        cleaver(m,"WeaponR",tier)
    if tier and branch==0:
        if tier==1:
            outline=[(.2*cos(i*tau/12),.19*sin(i*tau/12)) for i in range(12)]
        else:
            outline=[(-.29,-.174),(-.24,-.218),(.31,-.193),(.40,-.120),(.40,.15),(.31,.214),(-.24,.218),(-.32,.151)]
        m.plate("Ossuary shield","OffhandL","patina",outline,(.046,.096),axis="x")
        m.plate("Shield bone rim","OffhandL","bone",[(y*.79,z*.77) for y,z in outline],(.091,.107),axis="x")
        m.ell("Shield umbo","OffhandL","iron",(.119,.07,0),(.048,.057,.068),segments=11)
    if tier==1 and branch==1:
        cleaver(m,"OffhandL",tier)


def cleaver(m,joint,tier):
    m.tube("Cleaver leather grip",joint,"hide",[(0,0,-.094),(0,0,.012),(0,-.008,.162)],[.024,.028,.024],n=8)
    m.ell("Bone pommel",joint,"bone",(0,0,-.097),(.034,.038,.033),segments=9)
    outline=[(-.154,.145),(-.172,.442),(-.143,.523),(.014,.495),(.072,.454),(.056,.211),(.027,.153)]
    m.plate("Bevel-forged blade",joint,"iron",outline,(-.027,.027),axis="x",bevel=.011)
    m.plate("Honed cutting edge",joint,"edge",[(-.153,.159),(-.171,.440),(-.143,.523),(-.111,.513),(-.131,.435),(-.126,.171)],(-.021,.030),axis="x",bevel=.004)
    for z in (.222,.272,.322):
        m.ell("Blade rune inlay",joint,"patina",(.029,-.035,z),(.004,.017,.010),segments=8)
