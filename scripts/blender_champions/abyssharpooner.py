"""Fit a two-hand harpoon carrier with a guided shaft and a rear spear rack."""

from math import cos, sin, tau

from abyssal_parts import fins, mantle, marine
from bloomkeeper import leaf

RELEASE=(0,.12,.28)
SUPPORT=(.24,-.01,.11)


def abyssharpooner(m,tier,branch,*,production=False):
    sight = tier > 0 and branch == 0
    hunt = tier > 0 and branch == 1
    frame = "star_ceramic" if sight else "judge_obsidian" if hunt else "wood"
    edge = "prism_ice" if sight else "beetle_coral" if hunt else "fish_silver"
    marine(m,"abyssharpooner","gunner_indigo","shark_belly","leech_plum",
           face=(.104,.098,.131),eyes=.077,snout=.155)
    fins(m,"leech_plum",edge,.164+.026*hunt)
    mantle(m,"ritual_indigo" if not hunt else "pyre_enamel",edge,.158,.308)
    launcher(m,frame,edge,tier,sight,hunt)
    if sight:
        m.rim("Recessed diver's sight", "Head",frame,(-.077,.035,.171),(.044,.039),.010,segments=12)
        if tier==2:
            m.tube("Side sight hinge rising from the headset","Head",edge,
                   [(-.092,.058,.128),(-.138,.109,.074),(-.102,.157,-.026)],[.021,.019,.015],n=8)
    if hunt:
        for s in (-1,1):
            m.tube("Stored barbed spear attached to the back rack","BackSpring",edge,[
                (s*.079,-.156,-.069),(s*.091,.121,-.076),(s*(.112+.044*(tier==2)),.330,-.095),
            ],[.017,.024,.004],n=8)
            leaf(m,"Rack spear barb","BackSpring",frame,(s*.106,.277,-.088),
                 (s*.143,.214,-.098),.034)


def launcher(m,frame,edge,tier,sight,hunt):
    receiver(m,frame,edge,hunt)
    draw_rail(m,frame,edge,tier,sight,hunt)
    harpoon(m,edge,tier,sight,hunt)


def receiver(m,frame,edge,hunt):
    n="WeaponR"
    m.plate("Contoured one-piece launcher receiver",n,frame,[
        (.085,-.144),(.105,-.086),(.100,.120),(.106,.257),
        (.071,.287),(.017,.239),(.004,.121),(-.018,.018),(.026,-.115),
    ],(-.049,.049),axis="x",bevel=.012)
    m.tube("Gasketed walnut handle through the main palm",n,"darkwood",
           [(0,.051,-.028),(0,.007,0),(0,-.063,.025)],
           [.040,.032,.027],n=9)
    m.plate("Shaped rear shoulder pad",n,"hide",[
        (.085,-.153),(.096,-.125),(.022,-.122),(.006,-.155),
    ],(-.045,.045),axis="x",bevel=.008)
    for s in (-1,1):
        m.tube("Receiver perimeter trim",n,edge,[(s*.047,.081,-.112),
               (s*.052,.070,-.044),(s*.054,.059,.181),(s*.044,.077,.257)],
               [.007,.009,.008,.005],n=7)
        for z in (-.066,.165,.212):
            m.ell("Flush marine receiver rivet",n,"brass" if not hunt else "steel",
                  (s*.052,.057,z),(.008,.010,.009),segments=8)
    m.tube("Upper strut of the hand-fitted support bridge",n,"steel_dark",[
        (.036,.038,.061),(.100,.031,.076),(.180,.013,.082),(.241,.009,.094),
    ],[.023,.019,.015,.018],n=8)
    m.tube("Lower strut joining the receiver to the second grip",n,"iron",[
        (.037,.003,.182),(.111,-.027,.151),(.188,-.039,.122),SUPPORT,
    ],[.022,.018,.016,.025],n=8)
    m.tube("Wrapped support grip at the original left palm",n,"hide",[
        (.239,.009,.094),SUPPORT,(.257,-.021,.128),
    ],[.022,.029,.020],n=9)
    m.tube("Riveted reel spindle",n,"brass",[(.051,.036,.056),(.086,.036,.056),
           (.101,.036,.056)],[.053,.053,.038],n=12)
    m.tube("Dark spool inside the reel side plates",n,"pine",[(.074,.036,.056),
           (.094,.036,.056)],[.042,.042],n=10)
    points=[(.098,.036+.031*cos(i*tau/15),.056+.031*sin(i*tau/15)) for i in range(16)]
    m.tube("Coiled retrieval line on the reel",n,"linen",points,[.005]*len(points),n=6)


def draw_rail(m,frame,edge,tier,sight,hunt):
    n="WeaponR"
    m.loft("Open guide sleeve with recessed bore",n,"steel_dark",[
        (.212,.036,.034,0),(.258,.048,.047,0),(.280,.047,.046,0),
        (.280,.032,.031,0),(.233,.028,.027,0),
    ],n=14,transform=lambda p:(p[0],.12-p[2],p[1]))
    m.rim("Machined guide lip at the native release origin",n,edge,RELEASE,(.047,.046),
          .008,segments=14)
    for s in (-1,1):
        m.tube("Separate polished draw rail",n,"edge",[(s*.021,.105,-.061),
               (s*.025,.113,.039),(s*.033,.108,.192),(s*.036,.115,.268)],
               [.007,.009,.010,.009],n=7)
        m.box("Breech rail shoe",n,frame,(s*.034,.087,.018),(.039,.031,.102),bevel=.008)
        if hunt:
            leaf(m,"Coral front guide blade",n,edge,(s*.040,.104,.206),
                 (s*(.080+.018*(tier==2)),.143,.277),.030,facing=(0,1,0))
    m.box("Low mechanical sight saddle",n,"steel_dark",(0,.143,.146),(.070,.048,.071),bevel=.008)
    if sight:
        m.tube("Raised precision sight dovetail",n,edge,[(0,.161,.144),(0,.186,.159)],
               [.023,.017],n=8)
        m.rim("Aperture on the precision launcher",n,edge,(0,.199,.166),(.027,.026),
              .008,segments=12)
    if sight and tier==2:
        m.tube("Front range marker seated on the guide",n,edge,[(0,.158,.255),
               (0,.190,.262),(0,.202,.262)],[.016,.013,.006],n=7)


def harpoon(m,edge,tier,sight,hunt):
    n="WeaponR"
    tip=.627 if sight else .552
    m.tube("Harpoon shaft through the native launch origin",n,"steel",[
        (0,.12,.048),RELEASE,(0,.12,tip-.142),(0,.12,tip-.057),
    ],[.013,.013,.015,.011],n=9)
    m.tube("Machined lance ferrule",n,"brass" if not hunt else "copper",
           [(0,.12,tip-.171),(0,.12,tip-.119)],[.024,.023],n=9)
    leaf(m,"Forged faceted spearhead",n,"edge",(0,.12,tip-.149),(0,.12,tip),
         .048 if not hunt else .052,facing=(0,1,0))
    for s in (-1,1):
        leaf(m,"Swept load-bearing harpoon barb",n,edge,(s*.009,.119,tip-.034),
             (s*(.075+.017*(hunt and tier==2) if hunt else .054),.12,tip-.147),
             .027,facing=(0,1,0))
        if hunt and tier==2:
            leaf(m,"Second coral barb at the ferrule",n,"beetle_coral",(s*.010,.12,tip-.101),
                 (s*.060,.12,tip-.191),.022,facing=(0,1,0))
