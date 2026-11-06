"""Build a plated six-legged beetle around a glowing abdomen and twin wrist spikes."""

from math import cos, sin, tau

from bloomkeeper import leaf
from completion_people import limbs
from insect_legs import auxiliary_legs

STRIKE=(0,0,.30)


def blastbeetle(m,tier,branch,*,production=False):
    fitted = tier > 0 and branch == 0
    split = tier > 0 and branch == 1
    shell = "sun_cream" if fitted else "beetle_coral" if split else "beetle_pumpkin"
    chassis(m,shell,fitted,split,tier)
    portrait(m,shell,fitted,split,tier)
    limbs(m,"pine",shell,arm=.17*.95,width=.059,skin="pine")
    auxiliary_legs(m,shell,"pine")
    for node,s in (("WeaponR",-1),("OffhandL",1)):
        spike(m,node,s,shell,fitted,split,tier)


def chassis(m,shell,fitted,split,tier):
    m.ell("Wide armored beetle chest","Chest",shell,(0,.174,.003),(.182,.176,.122),segments=15)
    m.ell("Basal chitin under shell","Hips","pine",(0,.004,-.053),(.163,.124,.149),segments=13)
    m.ell("Hind abdomen pressure sac","Hips","forge" if split else "sap",(0,-.001,-.220),
          (.150+.022*fitted,.141+.022*fitted,.167),segments=16,rings=8)
    for s in (-1,1):
        m.tube("Hard protective abdomen cover","Hips",shell,[(s*.089,.093,-.099),
               (s*.148,.086,-.209),(s*.143,.040,-.308),(s*.074,.027,-.374)],
               [.062,.070,.055,.016],n=9)
        if fitted:
            leaf(m,"Ivory reactor protector","Hips","sun_cream",(s*.093,.052,-.156),
                 (s*.175,-.057,-.335),.063)
        if split:
            leaf(m,"Opened coral abdomen shell vane","Hips","beetle_coral",(s*.110,.039,-.166),
                 (s*.227,.058,-.298),.062)
    if tier==2:
        m.rim("Heavy power-ring around rear sac","Hips","gold" if fitted else "forge",
              (0,.004,-.263),(.173,.152),.014)
    if fitted:
        leaf(m,"Ivory thorax horn","Chest","sun_cream",(0,.270,-.026),(0,.450,-.121),
             .061,facing=(1,0,0))


def portrait(m,shell,fitted,split,tier):
    h="Head"
    m.ell("Domed beetle skull",h,shell,(0,.014,.012),(.142,.132,.121),segments=15)
    m.ell("Dark mouth segment",h,"pine",(0,-.066,.100),(.085,.044,.042),segments=10)
    for s in (-1,1):
        m.ell("Black beetle compound eye",h,"ink",(s*.110,.032,.090),(.049,.047,.043),segments=11)
        m.ell("Light in the beetle compound eye",h,"forge" if split else "gold",
              (s*.124,.037,.118),(.024,.023,.016),segments=9)
        m.tube("Curved pressure beetle antenna",h,"pine",[(s*.057,.111,.012),
               (s*.096,.167,-.026),(s*.132,.202,.024)],[.019,.015,.006],n=7)


def spike(m,node,s,shell,fitted,split,tier):
    m.tube("Wrist bracing grip",node,"pine",[(0,0,-.025),(0,0,.043),(0,0,.113)],
           [.036,.049,.044],n=9)
    leaf(m,"Beetle strike spike",node,"sun_cream" if fitted else "beetle_coral" if split else "gold",
         (0,0,.075),STRIKE,.066,facing=(0,1,0))
    if split and tier==2:
        leaf(m,"Side spark prong on the strike spike",node,"forge",(s*.039,.012,.108),
             (s*.117,0,.248),.035,facing=(0,1,0))
