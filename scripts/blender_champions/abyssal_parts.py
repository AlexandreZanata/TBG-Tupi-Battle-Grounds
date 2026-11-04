"""Shape fitted marine anatomy, gill fins, and the surfaces shared by Abyssal kits."""

from math import cos, pi, sin, tau

from bloomkeeper import leaf
from rig import BODY_PLANS


def marine(m,unit,skin,belly,cloth,*,face=(.116,.102,.111),snout=.13,eyes=.076,bulk=1.0):
    leg,width,_,_,_,arm=BODY_PLANS[unit]
    m.loft("Shaped marine thorax","Chest",skin,[
        (-.051,width*.27,.075,0),(.064,width*.34,.107,.010),
        (.171,width*.48,.117,.006),(.252,width*.54,.106,0),
        (.310,width*.43,.082,-.002),(.337,.072,.064,-.006),
    ],n=14)
    m.ell("Countershaded breastplate underlayer","Chest",belly,(0,.158,.087),
          (width*.282,.134,.054),segments=13)
    m.tube("Flexible neck rooted in the thorax","Chest",skin,[(0,.278,0),(0,.340,0),(0,.420,-.004)],
           [.073,.069,.052],n=11)
    m.loft("Underwater waist wrap","Hips",cloth,[
        (-.074,width*.36,.092,0),(-.031,width*.46,.112,.004),
        (.037,width*.44,.110,.004),(.066,width*.35,.094,0),
    ],n=12)
    m.loft("Wrapped hip harness","Hips","hide",[
        (.011,width*.464,.116,.006),(.038,width*.460,.119,.006),
    ],n=12)
    m.ell("Sculpted fish skull","Head",skin,(0,.007,.014),face,segments=15)
    m.ell("Pale fish jaw","Head",belly,(0,-.047,snout*.73),
          (face[0]*.69,.045,.057),segments=12)
    m.ell("Mouth crease","Head","pine",(0,-.047,snout+.017),(face[0]*.53,.008,.010),segments=9)
    for s,side in ((-1,"R"),(1,"L")):
        m.ell("Recessed eye socket","Head","pine",(s*eyes,.035,snout-.024),
              (.032,.032,.027),segments=10)
        m.ell("Wet eye surface","Head","gold",(s*eyes,.035,snout-.008),
              (.024,.024,.016),segments=10)
        m.ell("Fish pupil","Head","ink",(s*eyes,.035,snout+.006),(.011,.017,.004),segments=8)
        m.ell("Shoulder seat joined to torso","Chest",skin,(s*(width*.46+.016),.277,0),
              (.076*bulk,.069,.082),segments=11)
        limb_set(m,side,skin,belly,cloth,leg,arm,bulk)
        for i in range(2):
            m.tube("Incised lateral gill","Head","pine",[
                (s*face[0]*.94,.006-i*.023,.053),(s*face[0]*.90,-.006-i*.023,.085),
                (s*face[0]*.82,-.021-i*.023,.105),
            ],[.006,.008,.004],n=6)


def limb_set(m,side,skin,belly,cloth,leg,arm,bulk):
    upper,fore,hand,thigh,shin,foot=(f"{part}{side}" for part in
                                    ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
    m.tube("Articulated upper limb",upper,skin,[(0,.040,0),(0,-arm*.42,.007),(0,-arm,0)],
           [.058*bulk,.071*bulk,.045*bulk],n=9)
    m.tube("Jointed lower arm",fore,skin,[(0,.026,0),(0,-arm*.44,.006),(0,-arm*.9,0)],
           [.046*bulk,.056*bulk,.038*bulk],n=9)
    m.ell("Webbed gripping hand",hand,skin,(0,-.014,.014),(.049*bulk,.049,.048),segments=10)
    for node in (thigh,shin):
        m.tube("Planted tapered leg",node,cloth if node==thigh else skin,[(0,.027,0),
               (0,-leg*.21,.007),(0,-leg*.50,0)],[.062*bulk,.060*bulk,.044],n=9)
    m.ell("Low wide webbed foot",foot,skin,(0,-.021,.044),(.071,.043,.111),segments=10)
    for x in (-.034,.034):
        m.tube("Webbed forefoot ridge",foot,belly,[(x,-.011,.085),(x*1.33,-.023,.143)],
               [.014,.007],n=6)


def fins(m,main,edge,reach,*,layers=2):
    for s in (-1,1):
        for i in range(layers):
            root=(s*.082,.052-.042*i,-.023-.014*i)
            tip=(s*(reach-.021*i),.112-.104*i,-.075-.024*i)
            leaf(m,"Separated cheek fin ray","Head",main if i%2==0 else edge,root,tip,.037+.004*i)
            m.tube("Raised cheek fin spine","Head",edge,[root,tip],[.011,.003],n=6)


def mantle(m,swatch,edge,width,length):
    n="BackSpring"
    for i in (-1,1):
        x=i*width
        points=[
            (0,.085,.012),(x*.70,.092,.023),(x,-.103,-.041),
            (x*.71,-length,-.114),(x*.37,-length*.90,-.147),(0,-length*.84,-.125),
            (x*.44,-length*.43,-.092),
        ]
        front=[(j,(j+1)%6,6) for j in range(6)]
        back=[(c+7,b+7,a+7) for a,b,c in front]
        hem=[(j,j+7,(j+1)%6+7,(j+1)%6) for j in range(6)]
        faces=front+back+hem
        if i>0:
            faces=[tuple(reversed(face)) for face in faces]
        m.mesh("Folded swim mantle panel",n,swatch,points+[(x,y,z-.010) for x,y,z in points],faces)
        m.tube("Mantle fastening joined to shoulder",n,edge,[(x*.39,.053,.089),
               (x*.53,.067,.050),(x*.70,.092,.023)],[.024,.022,.019],n=7)
        m.tube("Bound swim mantle hem",n,edge,[(x*.70,.092,.023),
               (x,-.103,-.041),(x*.71,-length,-.114)],[.012,.011,.006],n=7)


def conch(m,node,swatch,lip,origin,length,radius,*,ribs=True):
    x,y,z=origin
    rings=[(-length,radius*.23),(-length*.73,radius*.51),(-length*.31,radius*.78),
           (0,radius),(0,radius*.78),(-length*.55,radius*.32)]
    m.loft("Hollow flared shell profile",node,swatch,[(a,r,r*.88,0) for a,r in rings],n=16,
           transform=lambda p:(x+p[0],y-p[2],z+p[1]))
    m.rim("Rolled shell lip",node,lip,origin,(radius,radius*.88),.010,segments=16)
    if ribs:
        for i in range(7):
            angle=(i+.25)*tau/7
            points=[(x+cos(angle)*r,y+sin(angle)*r*.88,z+a) for a,r in rings[:4]]
            m.tube("Raised shell flute",node,lip,points,[.004,.007,.009,.008],n=6)


def bow_arc(m,node,color,y,z,rx,ry,*,opening=1.35):
    angles=[opening/2+(tau-opening)*i/13-pi/2 for i in range(14)]
    points=[(rx*cos(a),y,z+ry*sin(a)) for a in angles]
    m.tube("Open bowl-shaped sling arc",node,color,points,[.019]*len(points),n=7)
