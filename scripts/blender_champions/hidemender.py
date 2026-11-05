"""Build a leatherworker with fitted goggles, layered seams and a hand-held needle."""

from bloomkeeper import leaf
from ogre_parts import ogre


def hidemender(m,tier,branch,*,production=False):
    surgeon = tier > 0 and branch == 0
    spirit = tier > 0 and branch == 1
    cloth = "leech_plum" if spirit else "capy_fur"
    trim = "sun_cream" if surgeon else "soul" if spirit else "linen"
    ogre(m,"hidemender",cloth,"leather",jaw=.89)
    outfit(m,cloth,trim,tier,surgeon,spirit)
    spool(m,cloth,trim,tier,surgeon,spirit)
    needle(m,trim,tier,surgeon,spirit)


def outfit(m,cloth,trim,tier,surgeon,spirit):
    for s in (-1,1):
        m.rim("Tailor's open brass spectacles","Head","brass",(s*.062,.034,.153),
              (.038,.031),.009,segments=12)
        m.tube("Spectacle temple stem","Head","brass",[(s*.097,.045,.155),
               (s*.127,.045,.113),(s*.155,.041,.027)],[.008,.008,.008],n=7)
        if surgeon:
            m.plate("Leatherworker's overlapping apron","Hips",trim,[(s*.007,.016),
                    (s*.140,.019),(s*.123,-.184),(s*.032,-.203)],(.126,.154))
        if spirit:
            leaf(m,"Raised threadwork lapel","Chest","brew_violet",(s*.143,.282,.075),
                 (s*.044,.068,.137),.052)
    m.box("Spectacle bridge","Head","brass",(0,.041,.160),(.046,.015,.013))
    for i in range(5):
        m.box("Individually sewn apron stitch","Chest",trim,(-.097+.046*i,.109+i*.014,.131),
              (.028,.010,.014),bevel=.003,rotation=-.38)
    if surgeon and tier==2:
        m.box("Flat folded leatherworking cap","Head",trim,(0,.141,-.007),(.247,.058,.194),
              bevel=.021)
    if spirit and tier==2:
        for s in (-1,1):
            leaf(m,"Threadwork collar peak","Chest","brew_violet",(s*.099,.207,.096),
                 (s*.182,.365,.018),.052)


def spool(m,cloth,trim,tier,surgeon,spirit):
    n="BackSpring"
    radius=.119+.024*spirit
    y=.098
    m.loft("Back-mounted hardwood spool",n,"wood",[
        (-.054+y,radius,.078,-.082),(-.032+y,radius,.087,-.082),
        (-.023+y,radius*.71,.063,-.082),(.103+y,radius*.71,.063,-.082),
        (.112+y,radius,.087,-.082),(.138+y,radius,.074,-.082),
    ],n=13)
    for offset in (.006,.034,.062,.089):
        m.loft("Separate bundled waxed thread",n,trim,[(y+offset-.008,radius*.74,.068,-.082),
               (y+offset+.009,radius*.76,.069,-.082)],n=12)
    if spirit and tier==2:
        m.tube("Wound spectral thread guard",n,trim,[(.113,.199,-.125),
               (.168,.248,-.151),(.171,.292,-.121),(.119,.286,-.103)],[.012,.014,.015,.007],n=7)
    if surgeon:
        m.box("Spare rolled tool pouch","Hips",cloth,(-.151,-.057,.079),(.096,.164,.078),
              bevel=.015)


def needle(m,trim,tier,surgeon,spirit):
    tip=.40 if surgeon else .33
    n="WeaponR"
    m.tube("Wrapped sewing needle handle",n,"wood",[(0,0,-.069),(0,0,.097)],[.030,.028],n=9)
    m.tube("Tapered sewing needle at the native tip",n,"steel",[(0,0,.067),
           (0,0,.138),(0,0,tip-.030),(0,0,tip)],[.028,.023,.015,.004],n=9)
    m.ell("Sewing needle eye mount",n,"brass",(0,.014,.134),(.028,.034,.027),segments=9)
    m.rim("Eye of the needle",n,trim,(0,.032,.151),(.024,.027),.006,segments=11)
    if spirit:
        m.tube("Independent spectral thread near the needle eye",n,trim,[
            (-.021,.044,.142),(-.055,.026,.180),(-.042,-.022,.224),
            (.014,-.035,.256),(.027,-.008,.281+.018*(tier==2)),
        ],[.008,.008,.007,.006,.003],n=7)
