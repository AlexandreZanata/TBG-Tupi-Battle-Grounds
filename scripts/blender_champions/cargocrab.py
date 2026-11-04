"""Build the mechanical carrier around its existing claws and passenger seat."""

from math import cos, sin, tau

SEAT_TOP = .245
CLAW_TIP = (0,-.28,0)


def cargocrab(model, tier, branch, *, production=False):
    armor = tier > 0 and branch == 0
    turbo = tier > 0 and branch == 1
    enamel = "crab_armor" if armor else "patina" if turbo else "crab_enamel"
    shell(model,enamel,tier,armor,turbo)
    saddle(model,tier,armor)
    for side, sign in (("L",1),("R",-1)):
        pistons(model,side,sign,enamel,turbo)
        pincer(model,side,sign,enamel)
    if turbo:
        exhausts(model,tier)


def shell(m, enamel, tier, armor, turbo):
    h = "Hips"
    m.loft("Cast chassis belly",h,"slag",[
        (-.188,.24,.20,-.025),(-.111,.366,.306,-.005),(.028,.405,.343,0),
        (.068,.341,.277,0),
    ],n=18)
    m.loft("Segmented enamel carapace",h,enamel,[
        (-.015,.375,.308,0),(.043,.423,.352,0),(.118,.392,.321,0),
        (.185,.300,.249,-.02),(.198,.195,.146,-.035),
    ],n=20)
    rim = [(.42*sin(i*tau/24),.046,.35*cos(i*tau/24)) for i in range(25)]
    m.tube("Carapace rolled rim",h,"edge" if armor else "brass",rim,[.014]*25,n=6)
    m.box("Chamfered grille housing",h,"steel_dark",(0,-.012,.333),(.34,.126,.10),.018)
    for i in range(-3,4):
        m.box("Inset air intake vane",h,enamel,(i*.040,-.010,.391),(.019,.074,.019),.004)
    for s in (-1,1):
        m.tube("Periscope eye stalk",h,"brass",[(s*.12,.077,.277),(s*.15,.204,.295),
                 (s*.16,.255,.322)],[.032,.026,.030],n=8)
        m.ell("Optic barrel",h,"slag",(s*.16,.263,.333),(.062,.052,.065),segments=10)
        m.ell("Optic bezel",h,"brass",(s*.16,.263,.387),(.042,.040,.016),segments=10)
        m.ell("Amber optic",h,"forge",(s*.16,.263,.399),(.031,.029,.011),segments=10)
        for z in (-.19,.015,.20):
            m.ell("Rim rivet",h,"edge" if armor else "brass",(s*.375,.105,z),
                  (.021,.022,.022),segments=8,rings=5)
        if armor:
            m.tube("Layered armor shoulder",h,"crab_armor",[(s*.30,.19,-.24),
                   (s*.382,.136,-.12),(s*.447,.083,.05),(s*.36,.126,.23)],
                   [.033,.073,.066,.035],n=8)
            m.tube("Outboard deflection spike",h,"edge",[(s*.398,.12,.06),
                   (s*.465,.165,.073),(s*.526,.179,.056)],[.042,.028,.003],n=7)
        if turbo:
            m.tube("Exposed copper fuel feed",h,"copper",[(s*.23,.123,.23),
                   (s*.30,.185,.081),(s*.30,.171,-.161),(s*.22,.128,-.31)],
                   [.018,.024,.024,.019],n=7)


def saddle(m, tier, armor):
    c = "Chest"
    m.box("Passenger seat bridge",c,"darkwood",(0,.158,-.035),(.36,.074,.31),.016)
    m.box("Passenger saddle cushion",c,"leather",(0,.207,-.035),(.302,2*(SEAT_TOP-.207),.272),.024)
    for s in (-1,1):
        m.tube("Grab rail along saddle",c,"brass",[(s*.177,.166,-.167),
               (s*.184,.252,-.145),(s*.186,.254,.037),(s*.181,.185,.077)],
               [.013]*4,n=7)
        if armor and tier == 2:
            m.tube("Open shield-frame buttress",c,"crab_armor",[(s*.212,.13,-.18),
                   (s*.257,.273,-.184),(s*.252,.367,-.195),(s*.134,.427,-.206)],
                   [.033,.036,.026,.018],n=8)
            m.ell("Shield generator lens",c,"sap",(s*.256,.311,-.166),
                  (.026,.045,.020),segments=9,rings=5)


def pistons(m, side, s, enamel, turbo):
    joint = f"Thigh{side}"
    for z in (-.227,0,.226):
        a,b,c = (s*.112,-.012,z*.73),(s*.322,-.091,z*.95),(s*.438,-.295,z*1.23)
        m.tube("Radial piston housing",joint,"steel_dark",[a,b],[.064,.056],n=9)
        m.ell("Radial piston swivel",joint,enamel,b,(.068,.067,.060),segments=10)
        m.tube("Extended piston rod",joint,"brass" if turbo else "edge",[b,c],[.025,.026],n=8)
        m.tube("Tapered shin cover",joint,enamel,[(s*.365,-.173,z*1.07),c],[.049,.034],n=8)
        m.box("Radial rubber foot",joint,"slag",(c[0],-.314,c[2]),(.105,.044,.102),.012)
        if turbo:
            m.ell("Jade servo on piston",joint,"soul",(b[0],b[1]+.031,b[2]+.046),
                  (.024,.030,.013),segments=8,rings=5)


def pincer(m, side, s, enamel):
    upper, fore, hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
    for joint in (upper,fore):
        m.tube("Claw actuator cylinder",joint,"steel_dark",[(0,.024,0),(0,-.128,0),(0,-.232,0)],
               [.063,.055,.041],n=9)
        m.ell("Actuator enamel sleeve",joint,enamel,(0,-.082,.028),
              (.075,.103,.064),segments=10,rings=6)
    m.ell("Claw knuckle",hand,"slag",(0,-.035,0),(.093,.073,.065),segments=12)
    for sign, reach in ((s,-.300),(-s,-.258)):
        m.tube("Tapered hooked claw jaw",hand,enamel,[(sign*.037,-.049,0),
               (sign*.095,-.117,.006),(sign*.096,-.198,.002),(sign*.044,reach,CLAW_TIP[2])],
               [.058,.071,.049,.012],n=9)
        m.tube("Inset pincer cutting edge",hand,"edge",[(sign*.035,-.129,.039),
               (sign*.049,-.200,.029),(sign*.037,reach+.002,.001)],[.016,.019,.005],n=7)


def exhausts(m, tier):
    for s in (-1,1):
        m.tube("Rear turbo exhaust stack","Hips","copper",[(s*.28,.076,-.19),
               (s*.34,.157,-.31),(s*.34,.210,-.42)],[.043,.047,.057],n=10)
        m.ell("Open exhaust bore","Hips","ink",(s*.34,.215,-.459),(.037,.035,.012),segments=9)
        if tier == 2:
            m.tube("Spoiler bracket","Hips","steel_dark",[(s*.24,.16,-.20),
                   (s*.27,.257,-.315),(s*.29,.331,-.418)],[.027,.032,.019],n=7)
    if tier == 2:
        m.box("Raised rear spoiler","Hips","patina",(0,.344,-.413),(.71,.041,.167),.014)
