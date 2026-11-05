"""Fit sculpted ogre underlayers to the native shoulder, hand and hip pivots."""

from bloomkeeper import leaf
from rig import BODY_PLANS


def ogre(m,unit,cloth,guard,*,skin="ogre",jaw=1.0):
    leg,width,_,_,_,arm = BODY_PLANS[unit]
    m.loft("Wide fitted ogre torso","Chest",cloth,[
        (-.045,width*.31,.094,0),(.067,width*.38,.127,.005),
        (.172,width*.47,.132,0),(.254,width*.56,.118,-.005),
        (.310,width*.45,.094,-.002),(.354,.099,.079,0),
    ],n=16)
    m.ell("Neck under the head pivot","Chest",skin,(0,.363,.003),(.101,.081,.089),segments=12)
    m.loft("Hip leather underlayer","Hips","hide",[
        (-.073,width*.36,.119,.003),(-.021,width*.46,.133,.007),
        (.030,width*.40,.134,.008),(.071,width*.31,.106,.007),
    ],n=14)
    m.loft("Full wrapped waist belt","Hips",guard,[
        (.016,width*.435,.139,.005),(.041,width*.419,.139,.005),
    ],n=14)
    m.box("Waist belt buckle","Hips","brass",(0,.029,.146),(.062,.041,.018))
    face(m,skin,jaw)
    for s,side in ((-1,"R"),(1,"L")):
        # The deltoid on the rotating arm intersects the chest saddle through the stance.
        m.ell("Shoulder saddle joined to torso","Chest",skin,(s*(width*.47+.015),.270,0),
              (.096,.078,.100),segments=12)
        limb_set(m,side,skin,cloth,guard,leg,arm)


def face(m,skin,jaw):
    h="Head"
    m.loft("Sculpted ogre skull",h,skin,[
        (-.134,.075*jaw,.067,.048),(-.092,.115*jaw,.100,.045),
        (-.030,.134,.120,.019),(.041,.142,.124,.003),
        (.111,.127,.104,-.010),(.147,.073,.075,-.017),
    ],n=17)
    m.ell("Broad upturned ogre nose",h,skin,(0,-.004,.143),(.042,.034,.035),segments=12)
    m.ell("Ogre mouth seam",h,"hide",(0,-.067,.151),(.072*jaw,.014,.013),segments=10)
    for s in (-1,1):
        leaf(m,"Pointed ogre ear",h,skin,(s*.101,.024,-.006),(s*.218,.063,-.039),.042)
        m.ell("Ear inset",h,"ogre_shade",(s*.155,.038,.012),(.028,.018,.011),segments=8)
        m.ell("Recessed eye socket",h,"ogre_shade",(s*.063,.037,.117),(.035,.024,.020),segments=10)
        m.ell("Ivory visible eye",h,"sun_cream",(s*.062,.034,.132),(.023,.014,.013),segments=10)
        m.ell("Ogre pupil",h,"ink",(s*.059,.034,.143),(.009,.011,.004),segments=8)
        m.tube("Angular brow shelf",h,skin,[(s*.032,.052,.138),(s*.066,.067,.128),
               (s*.096,.062,.109)],[.014,.020,.013],n=8)
        m.tube("Tusk rooted in the jaw",h,"bone",[(s*.087*jaw,-.091,.131),
               (s*.094*jaw,-.052,.168),(s*.092*jaw,-.018,.170)],[.022,.017,.003],n=9)


def limb_set(m,side,skin,cloth,guard,leg,arm):
    upper,fore,hand,thigh,shin,foot=(f"{part}{side}" for part in
                                    ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
    m.tube("Shaped ogre upper arm",upper,skin,[(0,.056,0),(0,-.044,.004),(0,-arm*.58,.011),
           (0,-arm,0)],[.073,.101,.090,.057],n=11)
    m.tube("Elbow and lower arm",fore,skin,[(0,.030,0),(0,-arm*.39,.006),
           (0,-arm*.91,0)],[.060,.075,.051],n=10)
    m.loft("Fitted vambrace",fore,guard,[
        (-arm*.81,.062,.059,.003),(-arm*.52,.075,.067,.004),(-arm*.40,.068,.061,.005),
    ],n=10)
    m.ell("Closed ogre hand",hand,skin,(0,-.012,.011),(.059,.061,.053),segments=11)
    for node in (thigh,shin):
        m.tube("Tailored leg under the kit",node,cloth,[(0,.048,0),(0,-leg*.23,.007),
               (0,-leg*.5,0)],[.078,.078,.056],n=10)
    m.ell("Wide fitted leather boot",foot,"hide",(0,-.023,.045),(.085,.056,.123),segments=11)
    m.ell("Raised toe cap",foot,guard,(0,-.020,.111),(.074,.038,.058),segments=10)
