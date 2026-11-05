"""Build a dockside fighter with hand-mounted hooks, split coat and rope hardware."""

from bloomkeeper import leaf
from ogre_parts import ogre


def hookbrawler(m,tier,branch,*,production=False):
    anchor = tier > 0 and branch == 0
    chain = tier > 0 and branch == 1
    cloth = "field_slate" if not chain else "gunner_indigo"
    trim = "sun_cream" if anchor else "bell_eye" if chain else "linen"
    ogre(m,"hookbrawler",cloth,"leather",jaw=1.0)
    m.loft("Knotted dockside headscarf","Head",cloth,[
        (.082,.142,.106,-.004),(.111,.139,.110,-.010),(.143,.086,.087,-.011),
    ],n=15)
    for s in (-1,1):
        m.tube("Headscarf cloth tail","HeadGear",trim,[(s*.082,.107,-.079),(s*.116,.069,-.170),
               (s*.131,-.020,-.222)],[.032,.027,.013],n=7)
        m.plate("Independently fitted split coat tail","Hips",cloth,
                [(s*.023,-.005),(s*.158,.006),(s*.202,-.182),(s*.077,-.245)],(.088,.115))
        m.tube("Heavy coat lapel","Chest",trim,[(s*.126,.288,.073),(s*.161,.234,.108),
               (s*.092,.135,.129),(s*.037,.074,.126)],[.024,.025,.017,.013],n=7)
    hook(m,"WeaponR",trim,tier,anchor,chain)
    if chain:
        hook(m,"OffhandL",trim,tier,False,True)
        m.rim("Chain buckle on the chest strap","Chest","edge",(0,.113,.146),(.054,.042),.015,segments=12)
    else:
        for y in (.089,.114,.139):
            m.rim("Layered coiled rope","BackSpring",trim,(0,y,-.076),(.130,.105),.014,segments=16)
    if anchor and tier==2:
        m.plate("Anchor crew shoulder guard","UpperArmR","iron",
                [(-.111,.018),(-.064,.091),(.060,.085),(.099,-.069),(0,-.106)],(.067,.113))


def hook(m,n,trim,tier,anchor,chain):
    m.tube("Hook wrapped grip",n,"darkwood",[(0,0,-.070),(0,0,.118)],[.028,.034],n=9)
    m.tube("Forward hook shackle",n,"iron",[(0,0,.069),(0,0,.201),(0,0,.261)],[.039,.035,.023],n=10)
    reach=.053*(tier==2)
    m.tube("Swept long steel hook",n,"steel" if anchor else "steel_dark",[
        (0,0,.193),(0,-.128,.211),(0,-.243-reach,.263),
        (0,-.267-reach,.342),(.014,-.211-reach,.401),(.014,-.138,.384),
    ],[.041,.043,.047,.042,.027,.004],n=10)
    m.rim("Shackle link",n,trim,(0,-.060,.234),(.056,.044),.012,segments=11)
    if anchor:
        for s in (-1,1):
            leaf(m,"Twin forged anchor flukes",n,"sun_cream",(0,-.219-reach,.329),
                 (s*.066,-.161-reach,.392),.029,facing=(0,1,0))
    if chain and tier==2:
        for y in (-.079,-.147,-.215):
            m.rim("Decorative rotating chain link",n,trim,(.046,y,.236),(.029,.044),.009,segments=10)
