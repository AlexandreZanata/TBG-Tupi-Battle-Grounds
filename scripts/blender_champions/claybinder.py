"""Assemble the clay binder's carved mask, slip pot and sealed casting tool."""

from bloomkeeper import leaf
from ogre_parts import ogre

SEAL_ORIGIN=(0,0,.27)


def claybinder(m,tier,branch,*,production=False):
    kiln = tier > 0 and branch == 0
    mire = tier > 0 and branch == 1
    glaze = "howl_glaze" if mire else "howl_clay"
    trim = "crag_limestone" if kiln else "venom" if mire else "mire_amber"
    ogre(m,"claybinder","mire_reed" if mire else "linen","leather",jaw=.95)
    mask(m,glaze,trim,tier,kiln,mire)
    jar(m,glaze,trim,tier,kiln,mire)
    m.tube("Clay throwing seal shaft","WeaponR","wood",[(0,0,-.068),(0,0,.192)],
           [.034,.030],n=9)
    m.tube("Glazed seal tool casing","WeaponR",glaze,[(0,0,.124),(0,0,.155),
           (0,0,.240),SEAL_ORIGIN],[.033,.067,.069,.057],n=12)
    m.rim("Seal lip at the native cast origin","WeaponR",trim,SEAL_ORIGIN,(.045,.045),.010,segments=10)
    m.ell("Clay charge balanced in the left hand","OffhandL",glaze,(0,.037,.072),
          (.067,.069,.068),segments=12)
    for s in (-1,1):
        m.tube("Pot-carrying strap","Chest","leather",[(s*.107,.281,.073),
               (s*.129,.187,.111),(s*.071,.061,.122)],[.025,.025,.021],n=8)


def mask(m,glaze,trim,tier,kiln,mire):
    h="Head"
    for s in (-1,1):
        m.plate("Split shaped kiln mask",h,trim if kiln else glaze,
                [(s*.024,.075),(s*.067,.089),(s*.120,.070),(s*.131,.019),
                 (s*.104,-.023),(s*.072,-.013),(s*.093,.032),(s*.030,.040)],(.125,.154))
        if mire:
            leaf(m,"Mire mask cheek growth",h,"mire_reed",(s*.108,.049,.108),
                 (s*.175,.084,-.035),.041)
        if kiln and tier==2:
            leaf(m,"Double seal brow crown",h,trim,(s*.047,.084,.087),
                 (s*.090,.203,-.013),.046)
    m.plate("Raised nose bridge on the clay mask",h,glaze,
            [(-.029,.081),(.029,.081),(.046,.011),(0,-.012),(-.046,.011)],(.141,.166))


def jar(m,glaze,trim,tier,kiln,mire):
    n="BackSpring"
    height=.380 if tier==2 else .296
    m.loft("Wheel-thrown clay slip jar",n,glaze,[
        (-.117,.081,.068,-.092),(-.042,.131,.100,-.110),
        (.127,.145,.106,-.114),(.230,.105,.080,-.112),(height,.080,.069,-.115),
    ],n=14)
    for y in (.175,height-.016):
        m.loft("Decorative clay jar band",n,trim,[(y-.010,.095,.084,-.115),
               (y+.009,.103,.086,-.115)],n=12)
    for s in (-1,1):
        m.tube("Glazed jar handle",n,trim,[(s*.102,.202,-.115),(s*.171,.163,-.125),
               (s*.163,.072,-.139),(s*.134,.059,-.125)],[.021,.022,.022,.018],n=8)
        if mire and tier==2:
            leaf(m,"Marsh reed tied to the jar",n,"mire_reed",(s*.061,.186,-.123),
                 (s*.162,.448,-.134),.041,facing=(0,0,-1))
    if kiln:
        m.box("Fired tablet at the waist","Hips",trim,(.123,-.090,.121),(.105,.157,.047),
              bevel=.012,rotation=-.14)
