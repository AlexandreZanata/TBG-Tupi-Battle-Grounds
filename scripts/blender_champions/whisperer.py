"""Sculpt a floating bone bell with independently posed eye and tendrils."""

from math import cos, sin, tau

MUZZLE = (0, 0, .15)
TENDRILS = (
    ("UpperArmL","ForearmL","HandL",1),
    ("UpperArmR","ForearmR","HandR",-1),
    ("ThighL","ShinL","FootL",1),
    ("ThighR","ShinR","FootR",-1),
)


def whisperer(m,tier,branch,*,production=False):
    fear = tier > 0 and branch == 0
    pulse = tier > 0 and branch == 1
    shell = "acid_ivory" if fear else "bell_amethyst"
    trim = "gold" if fear else "bell_rose" if pulse else "brass"
    bell(m,shell,trim,pulse,tier)
    eye(m,shell,trim,fear,pulse,tier)
    for upper, lower, tip, side in TENDRILS:
        tentacle(m,upper,lower,tip,side,shell,pulse)
    for node, side in (("WeaponR",1),("OffhandL",-1)):
        m.tube("Outer bell tentacle",node,"violet_dark",[(0,.014,0),(side*.057,-.082,.002),
               (side*.102,-.213,.018),(side*.085,-.302,.049)],[.051,.048,.035,.022],n=9)
        m.tube("Hook of the outer tentacle",node,trim,[(side*.085,-.26,.034),
               (side*.093,-.337,.060),(side*.043,-.377,.117)],[.026,.025,.004],n=8)


def bell(m,shell,trim,pulse,tier):
    node = "Chest"
    profile = (
        (-.050,.248),(.018,.301),(.110,.303),(.213,.267),(.315,.202),
        (.390,.113),(.421,.035),(.373,.030),(.327,.102),(.280,.160),
        (.185,.211),(.087,.251),(.015,.259),(-.040,.224),
    )
    n = 24
    verts = [(r*sin(a*tau/n),y,r*cos(a*tau/n)*.91) for y,r in profile for a in range(n)]
    faces = [(i*n+a,i*n+(a+1)%n,j*n+(a+1)%n,j*n+a)
             for i in range(len(profile)) for j in ((i+1)%len(profile),) for a in range(n)]
    m.mesh("Hollow bone bell shell",node,shell,verts,faces)
    rim = [(.293*sin(a*tau/n),.022,.270*cos(a*tau/n)) for a in range(n+1)]
    m.tube("Rolled bell lip",node,trim,rim,[.019]*(n+1),n=7)
    for i in range(8):
        a = i*tau/8
        path = [(r*sin(a),y,r*cos(a)*.91) for y,r in ((.07,.305),(.165,.289),(.245,.249),(.315,.197))]
        m.tube("Raised engraved bell rib",node,trim,path,[.009,.014,.014,.004],n=6)
    if shell == "acid_ivory":
        for s in (-1,1):
            m.tube("Gilded fear-bell cheek hook",node,"gold",[(s*.193,.217,.126),
                   (s*.259,.117,.175),(s*.241,.022,.184),(s*.194,-.073,.175)],
                   [.022,.030,.024,.005],n=8)
    if pulse:
        resonators(m,tier)
    m.tube("Suspended bell clapper","Tabard","steel_dark",[(0,.010,-.10),(0,-.24,-.10),
           (0,-.30,-.11)],[.022,.019,.019],n=8)
    m.ell("Bell clapper drop","Tabard",trim,(0,-.327,-.11),(.052,.070,.051),segments=10)


def resonators(m,tier):
    for i in range(6):
        a = (i+.5)*tau/6
        p = lambda r,y: (r*sin(a),y,r*cos(a)*.91)
        m.tube("Arched rose resonator cage","Chest","bell_rose",[p(.227,.241),p(.312,.171),
               p(.333,.067),p(.298,-.056)],[.024,.033,.029,.012],n=8)
        if tier == 2:
            x,y,z = p(.329,.112)
            m.ell("Luminous pulse resonator","Chest","bell_eye",(x,y,z),
                  (.041,.060,.041),segments=9,rings=5)


def eye(m,shell,trim,fear,pulse,tier):
    m.ell("Black eye socket","Head","ink",(0,.001,.028),(.167,.142,.047),segments=16)
    path = [(.148*cos(i*tau/24),.130*sin(i*tau/24),.073) for i in range(25)]
    m.tube("Carved eye frame","Head",trim,path,[.029]*25,n=8)
    m.ell("Single living eye","Back","bell_eye" if pulse else "soul",(0,0,.079),
          (.104,.088,MUZZLE[2]-.079),segments=16,rings=8)
    m.ell("Recessed dark pupil","Back","ink",(0,0,.150),(.022,.060,.007),segments=10)
    for s in (-1,1):
        end = .346 if fear else .200
        m.tube("Swept orbital horn","Head","acid_ivory" if fear else shell,
               [(s*.112,.089,.030),(s*.177,.166,.013),(s*.173,end,-.022)],
               [.049,.045,.004],n=9)
        if fear and tier==2:
            m.tube("Ivory face blade","Head","acid_ivory",[(s*.157,.122,.021),
                   (s*.212,.012,.033),(s*.224,-.123,.069),(s*.156,-.208,.091)],
                   [.041,.059,.039,.004],n=9)


def tentacle(m,upper,lower,tip,s,shell,pulse):
    m.tube("Upper segmented tendril",upper,"violet_dark",[(0,.034,0),(s*.026,-.064,.018),(0,-.170,0)],
           [.064,.052,.036],n=9)
    m.tube("Lower segmented tendril",lower,shell,[(0,.020,0),(s*.022,-.078,.016),(0,-.186,0)],
           [.043,.046,.025],n=8)
    m.ell("Tendril luminous joint",lower,"bell_eye" if pulse else "soul",(s*.021,-.091,.034),
          (.030,.043,.025),segments=9,rings=5)
    m.tube("Tendril bone tip",tip,"bone",[(0,.011,0),(s*.026,-.050,.021),(s*.014,-.127,.065)],
           [.035,.033,.004],n=8)
