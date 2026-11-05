"""Build a cuttlefish duelist with paired nacre blades and articulated fin armor."""

from abyssal_parts import fins, marine
from bloomkeeper import leaf


def nacreduelist(m,tier,branch,*,production=False):
    tide = tier > 0 and branch == 0
    foam = tier > 0 and branch == 1
    shell = "sun_cream" if tide else "wasp_aqua" if foam else "fish_silver"
    spine = "gold" if tide else "leech_plum"
    cloth = "plum" if not foam else "sun_teal"
    marine(m,"nacreduelist","capy_reed","fish_silver",cloth,face=(.112,.113,.111),snout=.140)
    fins(m,cloth,shell,.175+.035*foam,layers=2+(foam and tier==2))
    for side,s,n in (("R",-1,"WeaponR"),("L",1,"OffhandL")):
        blade(m,n,s,shell,spine,tier,tide,foam)
        leaf(m,"Overlapping waist lamella","Hips",shell,(s*.057,.019,.109),
             (s*.135,-.193 if tide else -.148,.089),.050)
        if tide:
            leaf(m,"Ivory shoulder blade guard",f"UpperArm{side}",shell,(0,-.082,.032),
                 (s*.104,.045,-.041),.051)
        if foam and tier==2:
            leaf(m,"Splayed foam dorsal fin","BackSpring",shell,(s*.042,.071,-.014),
                 (s*.214,.237,-.093),.078)
    if tide and tier==2:
        leaf(m,"Single nacre crown ridge","Head",shell,(0,.076,.086),(0,.228,-.117),.058,facing=(1,0,0))


def blade(m,n,s,shell,spine,tier,tide,foam):
    tip=.460 if tide and tier==2 else .404 if tide else .355
    m.tube("Fitted curled blade handle",n,"hide",[(0,0,-.075),(0,0,.078),(0,0,.122)],
           [.027,.029,.022],n=8)
    m.tube("Nacre cutting spine through native strike point",n,spine,[(0,0,.076),
           (0,.012,.179),(0,0,.250),(s*.028,.055,tip)],[.022,.028,.024,.004],n=8)
    leaf(m,"Curved nacre fin blade",n,shell,(0,0,.075),(s*.029,.067,tip),
         .079 if tide else .056,facing=(1,0,0))
    if foam:
        leaf(m,"Foam hook on blade heel",n,shell,(0,.008,.133),(s*.062,.117,.041),.049,facing=(0,1,0))
