"""Fit armor underlayers, articulated limbs and a split mantle to the deity rig."""

from abyssal_parts import mantle
from bloomkeeper import leaf
from completion_people import limbs
from rig import BODY_PLANS


def armor(m,unit,cloth,plate,metal,*,sleeve="skin",layers=1):
    leg,width,height,depth,_,arm=BODY_PLANS[unit]
    limbs(m,cloth,plate,arm=arm,leg=leg,width=.079,skin=sleeve)
    m.loft("Fitted deity armor underlayer","Chest",cloth,[
        (-.053,width*.28,depth*.36,0),(.059,width*.35,depth*.47,.006),
        (.171,width*.46,depth*.54,.007),(.278,width*.54,depth*.47,0),
        (height+.006,.107,.084,0),
    ],n=14)
    m.tube("Armor collar connecting neck and helm","Chest",sleeve,[(0,.300,0),
           (0,height+.062,0)],[.094,.074],n=11)
    m.loft("Armored deity hips","Hips",cloth,[
        (-.066,width*.36,.107,0),(-.020,width*.47,.122,.004),
        (.045,width*.42,.122,.003),(.064,width*.34,.101,0),
    ],n=13)
    m.loft("Gilded waist harness","Hips",metal,[
        (.007,width*.467,.132,.006),(.054,width*.452,.128,.006),
    ],n=12)
    m.box("Forged belt boss","Hips",metal,(0,.033,.138),(.090,.065,.031),bevel=.011)
    for s,side in ((-1,"R"),(1,"L")):
        m.ell("Shoulder connection under pauldron","Chest",sleeve,(s*(width*.47+.017),.281,0),
              (.097,.087,.084),segments=11)
        m.plate("Separate shaped cuirass wing","Chest",plate,
                [(s*.012,.282),(s*.176,.281),(s*.215,.199),(s*.134,.107),
                 (s*.049,.147),(s*.012,.174)],(.102,.155),bevel=.011)
        m.tube("Cuirass top binding","Chest",metal,[(s*.016,.289,.148),
               (s*.122,.286,.159),(s*.178,.276,.137)],[.012,.014,.012],n=8)
        pauldron(m,f"UpperArm{side}",s,plate,metal,layers)
        m.box("Forged shin shell",f"Shin{side}",plate,(0,-.101,.058),
              (.130,.172,.066),bevel=.016)
        leaf(m,"Independent sculpted hip guard","Hips",plate,(s*.067,.021,.116),
             (s*.149,-.223,.099),.060)


def pauldron(m,n,s,plate,metal,layers):
    for i in range(layers):
        y=.045-i*.048
        x=s*(.016+i*.019)
        m.loft("Layered rounded shoulder shell",n,plate if i else metal,[
            (y-.045,.115-i*.013,.096-i*.010,.009),
            (y,.126-i*.013,.112-i*.008,0),(y+.047,.081-i*.012,.079-i*.008,-.007),
        ],n=10,transform=lambda p:(p[0]+x,p[1],p[2]))


def cloak(m,cloth,metal,width,length):
    mantle(m,cloth,metal,width,length)
    m.tube("Cape clasp and rear collar seam","BackSpring",metal,
           [(-width*.63,.062,.058),(0,.036,.025),(width*.63,.062,.058)],[.025,.030,.025],n=9)
