"""Carve a quarry ogre's vertical sling, counterweight and pebble pouches."""

from bloomkeeper import leaf
from ogre_parts import ogre

RELEASE = (0,.25,.03)


def pebbleslinger(m,tier,branch,*,production=False):
    cliff = tier > 0 and branch == 0
    hail = tier > 0 and branch == 1
    accent = "crag_limestone" if cliff else "finn_spray" if hail else "brass"
    ogre(m,"pebbleslinger","oilskin" if not hail else "field_slate","wood",skin="sprout",jaw=.91)
    sling(m,tier,cliff,hail,accent)
    n="OffhandL"
    m.loft("Hand-held closed pebble pouch",n,"leather",[
        (-.065,.034,.033,.035),(-.029,.083,.067,.055),(.035,.080,.066,.051),
        (.069,.049,.042,.039),(.086,.044,.039,.036),
    ],n=12)
    m.tube("Pouch cinch tied around the neck",n,accent,[(0,.061,.037),(0,.074,.037)],[.050,.048],n=12)
    m.loft("Wide quarry pack","BackSpring","ochre_cloth" if not hail else "pine",[
        (-.121,.084,.065,-.044),(-.084,.107,.085,-.066),
        (.103,.109,.086,-.060),(.131,.085,.071,-.055),
    ],n=12)
    if hail:
        for x in (-.099,.099):
            m.ell("Separate stone feed pouch","Hips","finn_storm",(x,-.042,.106),(.062,.079,.040),segments=10)
    if cliff and tier==2:
        leaf(m,"Quarry shoulder slab","UpperArmL",accent,(-.052,.025,.058),(.122,.102,-.023),.059)


def sling(m,tier,cliff,hail,accent):
    n="WeaponR"
    height=.315 if cliff else .255
    spread=.142 if cliff else .129 if hail else .108
    m.tube("Palm-fitted sling grip",n,"darkwood",[(0,-.101,0),(0,.064,0),(0,.100,0)],
           [.028,.037,.040],n=10)
    for s in (-1,1):
        tip=(s*spread,height,.015)
        m.tube("Separate carved sling fork",n,accent,[(0,.049,0),(s*.077,.146,.001),tip],
               [.034,.031,.022],n=9)
        m.tube("Tensioned sling band",n,"hide",[tip,(s*.059,.251,.028),RELEASE],
               [.009,.009,.010],n=7)
        if hail:
            m.ell("Pebble seated in the split pouch",n,"crab_armor",(s*.049,.252,.048),
                  (.030,.032,.024),segments=8)
            if tier==2:
                leaf(m,"Outer hail guide",n,accent,(s*.063,.139,.008),(s*.170,.261,.037),.030)
    if cliff:
        m.ell("Stone counterweight below grip",n,accent,(0,-.139,.003),(.050,.056,.044),segments=9)
    m.ell("Open-sided sling release bed",n,"leather",RELEASE,(.041,.016,.019),segments=8)
