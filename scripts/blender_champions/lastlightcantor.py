"""Tailor a skeletal cantor with a hollow bell, bone missal and five mourning forms."""

from math import cos, sin, tau

from bloomkeeper import leaf

MUZZLE = (0,0,.25)


def lastlightcantor(m,tier,branch,*,production=False):
    vigil = tier > 0 and branch == 0
    requiem = tier > 0 and branch == 1
    cloth = "cantor_gold" if vigil else "cantor_amethyst" if requiem else "ritual_indigo"
    robe(m,cloth,vigil,requiem,tier)
    head(m,cloth,vigil,requiem,tier)
    bell(m,vigil,requiem,tier)
    m.box("Bone missal cover","OffhandL","oldbone",(0,-.004,.030),(.137,.049,.155),.010)
    m.box("Missal leaf pages","OffhandL","linen",(0,.011,.035),(.117,.033,.138),.005)
    m.box("Missal spine","OffhandL","hide",(-.069,.006,.029),(.029,.058,.148),.007)


def robe(m,cloth,vigil,requiem,tier):
    m.loft("Tapered mourning bodice","Chest",cloth,[
        (-.041,.105,.078,0),(.068,.122,.089,0),(.180,.142,.103,0),(.265,.160,.092,-.007),
        (.328,.088,.069,-.009),
    ],n=14)
    m.loft("Layered choir robe skirt","Hips",cloth,[(-.361,.183,.132,0),(-.214,.165,.115,0),
           (-.068,.142,.101,-.006),(.029,.120,.092,-.006)],n=15,opening=.33,fold=.11)
    for s in (-1,1):
        m.tube("Gold choir stole","Chest","sun_cream" if vigil else "brass",[(s*.070,.320,.057),
               (s*.088,.222,.102),(s*.071,.121,.111),(s*.090,.022,.097)],
               [.019,.024,.023,.016],n=8)
        if vigil:
            m.ell("Gilt hanging vestment bell","Chest","gold",(s*.118,.109,.111),
                  (.033,.049,.030),segments=9)
        if requiem:
            leaf(m,"Layered violet choir collar","Chest","cantor_amethyst",(s*.103,.286,.035),
                 (s*.228,.338,-.014),.056)
        side="L" if s>0 else "R"
        upper,fore,hand,thigh,shin,foot = (f"{p}{side}" for p in
                                           ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Choir sleeve with rolled edge",upper,cloth,[(0,.032,0),(0,-.069,0),(0,-.175,0)],
               [.061,.068,.042],n=9)
        m.tube("Bishop sleeve taper",fore,cloth,[(0,.025,0),(0,-.062,0),(0,-.153,0)],
               [.052,.061,.041],n=9)
        m.ell("Bone hand of the cantor",hand,"bone",(0,-.016,.010),(.044,.044,.036),segments=9)
        for node in (thigh,shin):
            m.tube("Choir hidden lower limb",node,"hide",[(0,.022,0),(0,-.092,0),(0,-.190,0)],
                   [.048,.047,.038],n=8)
        m.ell("Soft pointed choir shoe",foot,"hide",(0,-.018,.044),(.050,.045,.091),segments=10)


def head(m,cloth,vigil,requiem,tier):
    h = "Head"
    m.ell("Singer's recessed skull",h,"bone",(0,.007,.011),(.106,.117,.091),segments=13)
    m.ell("Skull's dark opened singing mouth",h,"ink",(0,-.070,.082),(.048,.046,.021),segments=11)
    for s in (-1,1):
        m.ell("Cantor's deep eye socket",h,"ink",(s*.045,.015,.091),(.032,.030,.016),segments=9)
        m.ell("Soft cantor pupil",h,"ritual" if requiem else "sap",(s*.044,.015,.104),
              (.013,.012,.006),segments=8)
    for i in range(-2,3):
        x = i*.044
        y = .200-.022*abs(i)+.038*requiem
        leaf(m,"Pleated mourning headdress panel","HeadGear",cloth,
             (x*.61,-.027,-.045),(x*1.74,y,-.069),.044)
        leaf(m,"Headdress panel's raised bright border","HeadGear","gold" if vigil else "oldbone",
             (x*.61,-.019,-.032),(x*1.74,y-.012,-.052),.013)
    if requiem and tier==2:
        m.rim("Violet mourning halo","HeadGear","ritual",(0,.057,-.098),(.203,.208),.010,segments=22)


def bell(m,vigil,requiem,tier):
    w = "WeaponR"
    r = .166 if vigil else .129
    metal = "gold" if vigil else "cantor_amethyst" if requiem else "brass"
    m.tube("Bell handle on the wrist socket",w,"hide",[(0,.076,0),(0,.012,0),(0,-.036,0)],
           [.025,.028,.024],n=9)
    profile = ((-.160,r),(-.124,r*.86),(-.056,r*.63),(-.023,r*.42),
               (-.049,r*.37),(-.083,r*.53),(-.132,r*.73),(-.165,r*.82))
    n = 20
    points = [(radius*sin(i*tau/n),y,radius*cos(i*tau/n)) for y,radius in profile for i in range(n)]
    faces = []
    for row in range(len(profile)):
        a,b = row*n,((row+1)%len(profile))*n
        for i in range(n):
            j = (i+1)%n
            faces.append((a+i,a+j,b+j,b+i))
    m.mesh("Hollow cast bell",w,metal,points,faces)
    m.tube("Swinging bell clapper stem",w,"iron",[(0,-.071,0),(0,-.171,0)],[.014,.014],n=8)
    m.ell("Separate clapper strike ball",w,"gold",(0,-.173,0),(.028,.034,.028),segments=10)
    m.tube("Echo outlet brace in front of bell",w,"brass",[(0,-.084,.057),(0,-.019,.142),
           (0,0,.212)],[.023,.032,.030],n=8)
    m.rim("Front echo outlet rim",w,"sun_cream" if vigil else "brass",(0,0,.235),(.044,.044),.009,segments=16)
    m.ell("Bell echo at native release",w,"ritual" if requiem else "sap",(0,0,.228),
          (.035,.035,MUZZLE[2]-.228),segments=11)
    if tier==2:
        for s in (-1,1):
            m.ell("Paired upper bell finial" if vigil else "Second choir chime",w,metal,
                  (s*(r+.028),-.122,.031),(.033,.061,.039),segments=10)
