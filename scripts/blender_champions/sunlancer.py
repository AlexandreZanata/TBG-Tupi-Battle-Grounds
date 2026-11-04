"""Forge a dawn cavalry figure and a long lance on the existing two-hand shaft."""

from bloomkeeper import leaf
from completion_people import face, limbs

FRONT_GRIP = .20
STRIKE = .30


def sunlancer(m,tier,branch,*,production=False):
    pact = tier > 0 and branch == 0
    spark = tier > 0 and branch == 1
    cloth = "lancer_saffron" if pact else "lancer_indigo" if spark else "sun_navy"
    armor = "gold" if pact else "edge" if spark else "sun_cream"
    armor_layers(m,cloth,armor,pact,spark,tier)
    helmet(m,cloth,armor,pact,spark,tier)
    limbs(m,cloth,"brass" if pact else "iron",arm=.20,width=.057)
    lance(m,armor,pact,spark,tier)


def armor_layers(m,cloth,armor,pact,spark,tier):
    c="Chest"
    m.loft("Long fitted lance guard cuirass",c,armor,[
        (-.058,.111,.084,-.002),(.052,.135,.105,0),(.153,.162,.119,0),
        (.249,.171,.107,-.009),(.323,.096,.076,-.004),
    ],n=14)
    m.loft("Long cavalry hip tabard","Hips",cloth,[
        (-.239,.168,.123,.026),(-.141,.155,.112,.015),(.031,.124,.101,0),
    ],n=14,opening=.45,fold=.11)
    for s in (-1,1):
        m.tube("Molded front cuirass flange",c,"brass",[(s*.105,.045,.104),
               (s*.142,.152,.112),(s*.137,.263,.079)],[.016,.023,.017],n=7)
        if pact:
            leaf(m,"Folded golden cavalry shoulder plate",c,"sun_cream",(s*.087,.284,.048),
                 (s*.251,.363,-.023),.069)
        if spark:
            leaf(m,"Indigo pennon on outer shoulder",c,cloth,(s*.097,.277,-.049),
                 (s*.247,.192,-.136),.061)
    if pact and tier==2:
        for s in (-1,1):
            leaf(m,"Tiered solar shoulder lamella",c,"gold",(s*.114,.309,-.027),
                 (s*.281,.286,-.100),.055)


def helmet(m,cloth,armor,pact,spark,tier):
    face(m)
    h="Head"
    m.loft("Cavalry helmet with open face",h,armor,[(.067,.106,.094,-.007),
           (.098,.124,.107,-.009),(.166,.099,.083,-.036),(.205,.047,.048,-.056)],n=14)
    for s in (-1,1):
        leaf(m,"Long protective cheek of cavalry helmet",h,armor,(s*.105,.123,.039),
             (s*.093,-.056,.061),.045)
    leaf(m,"Swept cavalry plume","HeadGear",cloth,(0,.153,-.040),
         (0,.343+.066*(tier==2),-.171),.065 if pact else .046,facing=(1,0,0))


def lance(m,armor,pact,spark,tier):
    w="WeaponR"
    m.tube("Continuous lance shaft",w,"sun_navy",[(0,0,-.290),(0,0,0),
           (0,0,FRONT_GRIP),(0,0,STRIKE),(0,0,.739)],[.022,.024,.024,.022,.027],n=10)
    for z in (0,.141):
        m.tube("Leather two-hand shaft grip",w,"hide",[(0,0,z-.066),(0,0,z+.067)],[.030]*2,n=9)
    m.tube("Lance blade base collar",w,"brass",[(0,0,.661),(0,0,.734)],[.037,.042],n=10)
    leaf(m,"Diamond-section long lance blade",w,armor,(0,0,.717),(0,0,.975),
         .095 if pact else .070,facing=(0,1,0))
    if spark:
        r=.153 if tier==2 else .105
        m.rim("Charged annulus near lance tip",w,"lancer_indigo",(0,0,.634),(r,r),.014)
        if tier==2:
            for s in (-1,1):
                leaf(m,"Flanking spark tip",w,"ritual",(s*.053,0,.710),(s*.115,0,.896),
                     .033,facing=(0,1,0))
