"""Forge the storm god's crown, beard, two-handed rune hammer and orbiting sparks."""

from math import cos, sin, tau

from bloomkeeper import leaf
from completion_people import face
from deity_parts import armor, cloak

SUPPORT=(0,0,.26)
HEAD_Z=.66
RELEASE=(0,0,.78)


def volthar(m,tier,branch,*,production=False):
    sovereign=tier > 0 and branch == 0
    wrath=tier > 0 and branch == 1
    cloth="sun_navy" if not wrath else "violet_dark"
    plate="prism_ice" if sovereign else "judge_obsidian" if wrath else "lancer_indigo"
    metal="brass" if wrath else "gold"
    armor(m,"volthar",cloth,plate,metal,layers=1+min(tier,2))
    cloak(m,cloth,metal,.219,.580+.095*(tier==2))
    crown(m,metal,tier,sovereign,wrath)
    hammer(m,plate,metal,tier,sovereign,wrath)


def crown(m,metal,tier,sovereign,wrath):
    h="Head"
    face(m,skin="skin",brow="oldbone")
    m.ell("Sculpted sweep of white hair",h,"sun_cream",(0,.031,-.050),
          (.126,.138,.105),segments=14)
    for s in (-1,1):
        m.tube("Layered white beard cheek braid",h,"fish_silver",[(s*.076,-.026,.080),
               (s*.072,-.093,.110),(s*.038,-.164,.119),(s*.016,-.205,.091)],
               [.029,.033,.027,.004],n=8)
        m.tube("Forged swept crown horn",h,metal,[(s*.093,.081,.039),
               (s*.142,.150,-.001),(s*(.190+.044*wrath),.228,-.076)],
               [.040,.033,.005],n=8)
        m.ell("Storm eye under the brow",h,"storm_arc",(s*.043,.027,.121),
              (.018,.008,.008),segments=8)
        if wrath:
            leaf(m,"Long wrath crown spike",h,metal,(s*.091,.090,.082),
                 (s*(.163+.050*(tier==2)),.214,-.052),.048)
    m.tube("Gilt crown circlet",h,metal,[(-.131,.086,.043),(-.085,.103,.101),
           (0,.102,.132),(.085,.103,.101),(.131,.086,.043)],[.019,.025,.025,.025,.019],n=8)
    leaf(m,"Central storm crown prong",h,metal,(0,.094,.104),(0,.191,-.006),.044)
    if sovereign:
        halo_width=.169+.040*(tier==2)
        m.rim("Celestial crown halo",h,"storm_arc",(0,.142,-.095),
              (halo_width,.153+.047*(tier==2)),.011,segments=18)
        for s in (-1,1):
            m.tube("Halo mount joined to crown",h,metal,[(s*.095,.090,.038),
                   (s*(halo_width-.024),.131,-.073),(s*(halo_width-.002),.147,-.095)],[.020,.016,.012],n=7)
    if sovereign and tier==2:
        for x in (-.057,.057):
            leaf(m,"White-gold sovereign diadem vane",h,metal,(x,.086,.085),
                 (x*1.32,.228,-.041),.036)


def hammer(m,plate,metal,tier,sovereign,wrath):
    n="WeaponR"
    m.tube("Continuous fitted hammer haft",n,"darkwood",[(0,0,-.183),(0,0,.031),
           SUPPORT,(0,0,.521),(0,0,HEAD_Z)],[.037,.034,.034,.032,.045],n=10)
    for z in (-.157,0,SUPPORT[2],.535):
        m.tube("Individual leather and metal grip band",n,metal,[(0,0,z-.025),(0,0,z+.025)],
               [.041,.041],n=9)
    w=.375+.032*(tier==2)
    m.box("Crosswise forged rune hammer head",n,plate,(0,0,HEAD_Z),(w,.212,.225),bevel=.027)
    for s in (-1,1):
        m.box("Separate hammer striking cheek",n,metal,(s*w*.46,0,HEAD_Z),
              (.052,.201,.215),bevel=.012)
        if wrath:
            leaf(m,"Bronze wrath hammer spur",n,metal,(s*w*.46,0,.652),
                 (s*(w*.56+.105),0,.671),.063,facing=(0,1,0))
    m.ell("Thunder core centered at the release origin",n,"storm_arc",RELEASE,
          (.055,.051,.021),segments=11)
    m.rim("Conductive rim around the muzzle",n,metal,RELEASE,(.073,.066),.010,segments=14)
    if sovereign:
        m.rim("Raised celestial focusing rim",n,"storm_arc",(0,0,.766),
              (.114,.087),.011,segments=15)
    if wrath and tier==2:
        leaf(m,"Upper tempest hammer flange",n,metal,(0,.068,.636),
             (0,.174,.778),.066,facing=(1,0,0))
    count=4 if tier==2 else 3
    for i in range(count):
        a=i*tau/count
        x,y=.263*cos(a),.157*sin(a)
        leaf(m,"Independently orbiting storm shard","OffhandL","storm_arc",
             (x*.90,y*.90,-.021),(x*1.22,y*1.24,.028),.015)
