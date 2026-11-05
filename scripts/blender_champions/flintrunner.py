"""Knapp a narrow runner's pair of blades and differentiate both Evolution silhouettes."""

from bloomkeeper import leaf
from ogre_parts import ogre


def flintrunner(m,tier,branch,*,production=False):
    trail = tier > 0 and branch == 0
    spark = tier > 0 and branch == 1
    trim = "sun_cream" if trail else "forge" if spark else "bone"
    shell = "fortress_sand" if trail else "judge_obsidian" if spark else "horn_stone"
    ogre(m,"flintrunner","hide",shell,skin="oldbone",jaw=.84)
    for node in ("WeaponR","OffhandL"):
        blade(m,node,shell,trim,tier,trail,spark)
    for s,side in ((-1,"R"),(1,"L")):
        m.tube("Laced trail harness","Chest",shell,[(s*.133,.281,.087),(s*.086,.210,.125),
               (s*.060,.087,.130),(s*.098,.036,.098)],[.031,.028,.028,.026],n=8)
        for y in (-.058,-.107,-.155):
            m.loft("Layered ankle wraps",f"Shin{side}",trim,[(y-.009,.070,.072,0),
                   (y+.008,.073,.072,0)],n=10)
        if spark and tier==2:
            leaf(m,"Obsidian shoulder spur",f"UpperArm{side}",shell,(0,-.029,.040),
                 (s*.137,.095,-.008),.065)
    for i in range(2+(tier==2)):
        leaf(m,"Knapped ridge on the crown","Head",shell,(0,.106,.064-i*.061),
             (0,.192+.035*trail,-.027-i*.061),.049,facing=(1,0,0))


def blade(m,n,shell,trim,tier,trail,spark):
    tip=.480 if trail and tier==2 else .402 if trail else .341+.035*(tier==2)
    m.tube("Finger-fitted flint grip",n,"wood",[(0,0,-.070),(0,0,.067)],[.031,.027],n=9)
    m.ell("Blade stone guard",n,shell,(0,0,.066),(.065,.041,.024),segments=9)
    leaf(m,"Knapped running blade",n,trim if trail else shell,(0,0,.057),(0,0,tip),
         .063 if trail else .052,facing=(0,1,0))
    for s in (-1,1):
        m.tube("Visible knapped edge" if not spark else "Ember fracture in obsidian",n,trim,[
            (s*.018,.006,.097),(s*.046,.010,.189),(s*.029,.009,.249),(0,.006,tip-.029),
        ],[.009,.010,.009,.003],n=7)
