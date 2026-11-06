"""Build a banded hornet, paired wings, venom pistol and four moving hose sections."""

from bloomkeeper import leaf
from insect_legs import legs

HOSE = ("OffhandL","Nock","StringTop","StringBottom")
HOSE_START = (-.10,-.035,-.18)
HOSE_END = (-.075,.03,.015)


def hornet(m,tier,branch,*,production=False):
    venom = tier > 0 and branch == 0
    swoop = tier > 0 and branch == 1
    shell = "sun_cream" if venom else "steel_dark" if swoop else "wasp_amber"
    trunk(m,shell,venom,swoop,tier)
    head(m,shell)
    pistol(m,venom,swoop,tier)
    for side,s,node in (("L",1,"HeadGear"),("R",-1,"Tabard")):
        wings(m,node,s,swoop,tier)
        arms(m,side,shell)
        legs(m,side,.19,"steel_dark",shell,.047)
    for node in HOSE:
        m.tube("Unit-length venom hose section",node,"hide",[(0,-.50,0),(0,.50,0)],
               [.022,.022],n=10)


def trunk(m,shell,venom,swoop,tier):
    m.loft("Compact hornet thorax","Chest",shell,[
        (-.052,.082,.069,-.007),(.045,.112,.093,-.007),(.162,.159,.103,0),
        (.272,.163,.099,0),(.334,.102,.075,.008),
    ],n=14)
    m.tube("Tapered segmented wasp abdomen","BackSpring","wasp_amber",[
        (0,.011,.025),(0,-.014,-.088),(0,-.031,-.184),(0,-.075,-.271),(0,-.131,-.345),
    ],[.061,.119,.126,.089,.012],n=13)
    for z,y,r in ((-.079,-.013,.118),(-.148,-.026,.132),(-.220,-.045,.117)):
        m.tube("Dark transverse abdomen band","BackSpring","ink",[(0,y+.007,z-.021),
               (0,y-.002,z+.021)],[r,r],n=12)
    m.ell("Venom reservoir near its hose joint","BackSpring","venom",
          HOSE_START,(.043,.049 if not venom else .072,.060),segments=11)
    m.ell("Far abdomen reservoir","BackSpring","venom",(.103,-.031,-.187),
          (.047,.053,.059),segments=11)
    if venom:
        for s in (-1,1):
            m.ell("Separate chest venom ampoule","Chest","venom",(s*.116,.146,.095),
                  (.040,.068,.026),segments=9)
    if swoop:
        for s in (-1,1):
            leaf(m,"Aqua swept hip carapace","Hips","wasp_aqua",(s*.080,.043,.016),
                 (s*.188,-.092,-.094),.054)


def head(m,shell):
    h = "Head"
    m.ell("Rounded hornet helmet",h,shell,(0,.015,.016),(.135,.129,.121),segments=15)
    for s in (-1,1):
        m.ell("Black hornet compound eye",h,"ink",(s*.092,.031,.098),
              (.060,.065,.043),segments=12)
        m.ell("Compound eye reflected lobe",h,"steel",(s*.108,.047,.128),
              (.030,.035,.013),segments=9)
        m.tube("Forward forked hornet antenna",h,"hide",[(s*.052,.109,.051),
               (s*.101,.175,.063),(s*.112,.229,.113)],[.018,.014,.004],n=7)
        leaf(m,"Faceted hornet cheek",h,"wasp_amber",(s*.121,.001,.074),
             (s*.061,-.088,.112),.034)


def wings(m,node,s,swoop,tier):
    for i in range(2):
        root = (-s*.046,.017-i*.066,.019)
        tip = (s*(.322+.063*swoop-.085*i),.210-.168*i+.035*swoop,-.110-.039*i)
        leaf(m,"Opaque faceted flying wing",node,"wasp_aqua" if swoop else "wasp_wing",
             root,tip,.077+.009*swoop)
        leaf(m,"Raised vein across flying wing",node,"sun_cream",
             (root[0],root[1],root[2]+.019),(tip[0]*.95,tip[1]-.012,tip[2]+.010),.012)
        if swoop and tier==2:
            leaf(m,"Reinforced trailing wing spur",node,"gold",(s*.146,tip[1]*.65,-.052),
                 (s*(.377-.061*i),tip[1]-.105,tip[2]-.061),.033)


def arms(m,side,shell):
    arm,fore,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
    for node,length in ((arm,.23),(fore,.207)):
        m.tube("Segmented hornet arm",node,shell,[(0,.029,0),(0,-length*.39,.009),(0,-length,0)],
               [.053,.059,.039],n=9)
    m.ell("Closed chitin hand",hand,"ink",(0,-.019,.011),(.043,.048,.036),segments=10)


def pistol(m,venom,swoop,tier):
    w = "WeaponR"
    z = .53 if venom and tier==2 else .46
    m.box("Venom pistol grip",w,"hide",(0,-.044,.019),(.067,.117,.088),.013)
    m.box("Beveled stinger receiver",w,"steel_dark",(0,.054,.132),(.155,.137,.250),.022)
    m.tube("Tapered venom pistol barrel",w,"brass",[(0,.06,.208),(0,.06,z-.092),(0,.06,z-.019)],
           [.061,.053,.062],n=13)
    m.rim("Venom muzzle rim at release",w,"edge",(0,.06,z-.010),(.049,.049),.010,segments=17)
    m.ell("Recessed stinger bore",w,"ink",(0,.06,z-.018),(.038,.038,.004),segments=10)
    m.ell("Hose socket on the receiver",w,"brass",HOSE_END,(.032,.032,.034),segments=10)
    m.ell("Ready venom chamber",w,"venom",(.076,.077,.147),(.031,.049,.081),segments=11)
    if venom and tier==2:
        m.rim("Secondary reservoir brace",w,"gold",(0,.06,.321),(.067,.067),.015)
    if swoop:
        leaf(m,"Aqua airflow fin on the receiver",w,"wasp_aqua",(0,.107,.106),(0,.169,.314),.043,
             facing=(1,0,0))
