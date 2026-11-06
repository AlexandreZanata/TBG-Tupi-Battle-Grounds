"""Build a five-quill hunter with a separately loaded wrist needle."""

from bloomkeeper import leaf
from completion_people import limbs

RELEASE = (0,0,.25)


def spinehurler(m,tier,branch,*,production=False):
    lance = tier > 0 and branch == 0
    volley = tier > 0 and branch == 1
    coat = "spine_slate" if not lance else "slate"
    bone = "sun_cream" if lance else "soul" if volley else "oldbone"
    trunk(m,coat,bone,lance,volley,tier)
    mask(m,coat,bone,lance,volley,tier)
    limbs(m,coat,"pine",skin="spine_slate")
    launcher(m,bone,lance,volley,tier)


def trunk(m,coat,bone,lance,volley,tier):
    m.loft("Hunter's fitted bark thorax","Chest",coat,[
        (-.043,.110,.089,0),(.060,.130,.111,-.009),(.177,.159,.119,-.012),
        (.289,.178,.116,-.016),(.339,.105,.076,-.019),
    ],n=15,fold=.052)
    m.box("Quill rack's lower strap","Hips","hide",(0,.034,.067),(.235,.061,.117),.013)
    for s in (-1,1):
        m.tube("Shoulder-to-quiver binding","Chest","leather",[(s*.128,.306,-.038),
               (s*.115,.243,.089),(s*.103,.104,.101)],[.018,.025,.013],n=8)
        if lance:
            leaf(m,"Ivory arming plate","Chest","sun_cream",(s*.061,.207,.084),
                 (s*.190,.354,.020),.063)
    for i in range(-2,3):
        x=i*.052
        y=.124+.029*(2-abs(i))
        m.tube("Tapered quill mounted on the back","BackSpring",bone,[
            (x*.54,-.096,-.044),(x,y,-.073),(x*1.36,y+.171,-.124),
            (x*(1.82 if volley else 1.55),y+.261+.069*lance,-.157),
        ],[.026,.042,.026,.004],n=8)
        if volley:
            m.tube("Brass quill rack socket","BackSpring","copper",[(x*.76,.015,-.054),
                   (x*.80,.063,-.069)],[.035,.035],n=8)
    if volley and tier==2:
        for s in (-1,1):
            leaf(m,"Secondary teal quill rack vane","BackSpring","crag_fissure",(s*.066,.029,-.091),
                 (s*.251,.275,-.144),.065)


def mask(m,coat,bone,lance,volley,tier):
    h="Head"
    m.ell("Hooded hunter skull",h,coat,(0,.018,-.022),(.126,.132,.116),segments=14)
    m.tube("Chiseled beak-like face guard",h,"pine",[(0,.023,.067),(0,-.019,.125),
           (0,-.052,.219)],[.087,.078,.014],n=10)
    for s in (-1,1):
        m.ell("Dark socket in the mask",h,"ink",(s*.071,.026,.112),(.046,.027,.027),segments=9)
        m.ell("Quill hunter eye",h,"soul" if volley else "sap",(s*.080,.027,.133),
              (.019,.015,.009),segments=8)
    if tier==2:
        leaf(m,"Swept forehead quill",h,bone,(0,.093,.053),(0,.284,-.098),.047,facing=(1,0,0))


def launcher(m,bone,lance,volley,tier):
    w="WeaponR"
    m.tube("Bone-quill carrier guide",w,"hide",[(0,0,-.091),(0,0,.028),(0,0,.119)],
           [.031,.032,.028],n=9)
    if volley:
        for s in (-1,1):
            m.tube("Carrier side-guide",w,"crag_fissure",[(s*.024,0,-.026),(s*.046,0,.107)],
                   [.011,.013],n=7)
    n="HeadGear"
    rear = -.252 if lance else -.182
    m.tube("Loaded removable needle",n,bone,[(0,0,rear),(0,0,-.083),(0,0,.169),RELEASE],
           [.012,.024,.017,.003],n=9)
    for s in (-1,1):
        leaf(m,"Loaded quill fletch",n,"sun_cream" if lance else "linen",
             (0,0,rear+.123),(s*.045,0,rear),.022,facing=(0,1,0))
