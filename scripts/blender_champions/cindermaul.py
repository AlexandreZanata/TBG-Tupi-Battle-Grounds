"""Forge two fitted mauls, insulated wear and a portable furnace."""

from bloomkeeper import leaf
from ogre_parts import ogre


def cindermaul(m,tier,branch,*,production=False):
    anvil = tier > 0 and branch == 0
    forge = tier > 0 and branch == 1
    trim = "sun_cream" if anvil else "forge" if forge else "copper"
    ogre(m,"cindermaul","pyre_enamel" if forge else "horn_hide","iron",jaw=1.02)
    for node in ("WeaponR","OffhandL"):
        hammer(m,node,trim,tier,anvil,forge)
    furnace(m,trim,tier,forge)
    m.plate("Heat apron","Hips","pyre_enamel",[(-.117,.014),(.117,.014),(.081,-.189),
            (-.064,-.170)],(.122,.150))
    for s,side in ((-1,"R"),(1,"L")):
        m.tube("Crossed furnace harness","Chest","leather",[(s*.129,.303,.080),
               (s*.036,.181,.134),(-s*.092,.057,.112)],[.027,.025,.021],n=7)
        if anvil:
            m.box("Forged shoulder plate",f"UpperArm{side}","steel",(0,.002,.047),
                  (.195,.121,.155),bevel=.019)
        if forge and tier==2:
            leaf(m,"Furnace vambrace fin",f"Forearm{side}",trim,(0,-.188,.035),
                 (s*.078,-.056,.049),.035)
    if anvil and tier==2:
        m.rim("Raised forge goggles","Head","gold",(0,.086,.111),(.113,.039),.019,segments=14)


def hammer(m,n,trim,tier,anvil,forge):
    m.tube("Wrapped hammer haft",n,"wood",[(0,0,-.084),(0,0,.039),(0,0,.248)],
           [.027,.035,.027],n=10)
    for z in (.025,.061,.102):
        m.tube("Forged grip collar",n,"iron",[(0,0,z-.008),(0,0,z+.007)],[.035,.035],n=9)
    if forge:
        radius=.094+.019*(tier==2)
        m.tube("Cylindrical furnace fist",n,"slag",[(0,0,.150),(0,0,.182),(0,0,.284),
               (0,0,.322)],[radius*.63,radius,radius,radius*.82],n=12)
        for z in (.179,.265):
            m.tube("Heated circumferential maul band",n,trim,[(0,0,z-.009),(0,0,z+.009)],
                   [radius+.004]*2,n=12)
    else:
        m.box("Crosswise anvil head",n,"steel_dark",(0,0,.249),
              (.242+.057*anvil,.138+.024*anvil,.163),bevel=.023)
        for s in (-1,1):
            m.box("Inset anvil striking face",n,trim,(s*(.108+.029*anvil),0,.253),
                  (.032,.104,.121),bevel=.009)
        if anvil and tier==2:
            leaf(m,"Anvil horn profile",n,"sun_cream",(0,.053,.232),(0,.130,.375),.064,facing=(1,0,0))


def furnace(m,trim,tier,forge):
    n="BackSpring"
    m.loft("Insulated firebox pack",n,"slag",[
        (-.107,.081,.058,-.051),(-.070,.121,.082,-.073),
        (.115,.121,.087,-.071),(.180,.084,.063,-.066),
    ],n=10)
    for x in (-.066,.066):
        m.tube("Furnace chimney",n,"copper",[(x,.113,-.064),(x,.205,-.084),
               (x,.238+.066*(forge and tier==2),-.114)],[.031,.027,.033],n=9)
        m.ell("Visible furnace ember",n,trim,(x,.159,-.156),(.023,.028,.011),segments=9)
