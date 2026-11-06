"""Tailor an empty-hood oracle and its independently rotating staff ring."""

from math import cos, sin, tau

from bloomkeeper import leaf


def luminousoracle(m,tier,branch,*,production=False):
    restore = tier > 0 and branch == 0
    aura = tier > 0 and branch == 1
    cloth = "oracle_saffron" if restore else "oracle_lagoon" if aura else "ritual_indigo"
    robe(m,cloth,restore,aura,tier)
    hood(m,cloth,restore,aura)
    staff(m,restore,aura,tier)


def robe(m,cloth,restore,aura,tier):
    m.loft("Tailored ceremonial bodice","Chest",cloth,[
        (-.040,.121,.097,-.005),(.047,.133,.105,-.002),(.154,.146,.110,0),
        (.239,.169,.108,-.007),(.324,.099,.079,-.008),
    ],n=15,fold=.045)
    m.loft("Long priestly robe on hip pivots","Hips",cloth,[
        (-.382,.183,.142,0),(-.276,.163,.127,-.002),(-.126,.147,.113,0),(.013,.132,.105,0),
    ],n=17,opening=.22,fold=.105)
    m.loft("Layered oracle cape","BackSpring","sun_navy" if not aura else "oracle_lagoon",[
        (-.534,.188,.082,-.102),(-.407,.172,.077,-.093),(-.239,.138,.058,-.052),
        (.016,.133,.045,-.008),(.078,.091,.044,.007),
    ],n=13,opening=1.58,fold=.12)
    for s in (-1,1):
        m.tube("Long contrasting robe seam","Hips","gold",[(s*.067,-.011,.101),
               (s*.082,-.165,.115),(s*.099,-.304,.129),(s*.110,-.374,.135)],
               [.009,.010,.012,.010],n=7)
        m.tube("Patterned priestly stole","Chest","sun_cream" if restore else "brass",[
            (s*.076,.320,.050),(s*.094,.242,.098),(s*.077,.130,.114),(s*.092,-.022,.103),
        ],[.019,.029,.023,.018],n=8)
        if aura:
            leaf(m,"Split lagoon shoulder shell","Chest","sun_teal",(s*.114,.306,.003),
                 (s*.237,.333,-.034),.059)
            if tier==2:
                leaf(m,"Ivory shell on lower shoulder","Chest","sun_cream",(s*.147,.279,.013),
                     (s*.252,.235,.037),.042)
        side = "L" if s>0 else "R"
        arm,fore,hand,thigh,shin,foot = (f"{p}{side}" for p in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Bishop sleeve upper",arm,cloth,[(0,.031,0),(0,-.053,0),(0,-.175,0)],
               [.063,.073,.048],n=9)
        m.tube("Bishop sleeve forearm",fore,cloth,[(0,.026,0),(0,-.058,0),(0,-.154,0)],
               [.053,.059,.049],n=9)
        m.tube("Worked sleeve cuff",fore,"gold",[(0,-.114,0),(0,-.151,0)],[.055,.053],n=9)
        m.ell("Oracle glove",hand,"sun_navy",(0,-.018,.008),(.042,.046,.035),segments=10)
        for joint in (thigh,shin):
            m.tube("Robed hidden leg",joint,"sun_navy",[(0,.027,0),(0,-.083,0),(0,-.193,0)],
                   [.049,.047,.038],n=8)
        m.ell("Pointed oracle shoe",foot,"brass",(0,-.019,.039),(.048,.043,.087),segments=10)


def hood(m,cloth,restore,aura):
    h = "Head"
    m.loft("Sculpted open oracle hood",h,"sun_cream" if restore else cloth,[
        (-.108,.087,.082,-.006),(-.032,.128,.115,-.010),(.055,.141,.122,-.015),
        (.146,.126,.117,-.021),(.218,.077,.090,-.053),(.260,.019,.047,-.097),
    ],n=17,opening=.64,fold=.026)
    m.ell("Recessed unlit hood interior",h,"ink",(0,.045,.029),(.098,.128,.044),segments=14)
    for s in (-1,1):
        m.tube("Gold edge following the open hood",h,"brass",[(s*.060,-.101,.059),
               (s*.089,-.006,.078),(s*.094,.105,.082),(s*.060,.179,.061),(0,.218,.016)],
               [.013,.015,.014,.011,.007],n=7)
    if aura:
        m.tube("Turquoise hood brow trim",h,"sun_teal",[(-.069,.157,.061),(0,.193,.067),
               (.069,.157,.061)],[.012,.018,.012],n=8)


def staff(m,restore,aura,tier):
    w,r = "WeaponR","HeadGear"
    lift = .10 if restore and tier==2 else 0
    radius = .157 if restore else .129
    m.tube("Tall oracle staff on the hand socket",w,"darkwood",[(0,-.354,0),(0,0,0),
           (0,.420,0),(0,.49+lift,0)],[.022,.026,.021,.026],n=10)
    m.tube("Staff grip and binding",w,"brass",[(0,-.066,0),(0,.059,0)],[.030,.030],n=10)
    for s in (-1,1):
        m.tube("Staff ring cradle",w,"gold",[(0,.431+lift,0),(s*.065,.493+lift,-.007),
               (s*.082,.545+lift,.008)],[.024,.025,.012],n=8)
    m.rim("Independent staff halo",r,"gold" if not aura else "sun_teal",
          (0,lift,.024),(radius,radius),.019)
    m.ell("Staff light at its release plane",r,"sap" if not aura else "soul",
          (0,lift,.037),(.040,.041,.043),segments=13)
    if restore:
        for i in range(6):
            a = i*tau/6
            leaf(m,"Sun-ray calyx on the staff ring",r,"sun_cream",
                 (.091*sin(a),lift+.091*cos(a),.030),
                 (.209*sin(a),lift+.209*cos(a),.032),.036)
        if tier==2:
            leaf(m,"Second tier sun-ray finial",r,"gold",(0,lift+.113,.026),
                 (0,lift+.285,.016),.050)
    if aura and tier==2:
        for s in (-1,1):
            leaf(m,"Paired aura prong around staff ring",r,"sun_cream",
                 (s*.091,lift-.118,.033),(s*.183,lift+.123,.039),.041)
