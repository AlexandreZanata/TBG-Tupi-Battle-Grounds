"""Build a tracked foundry with independent belts, exhausts and feed barrel."""

from math import cos, sin, tau

from bloomkeeper import leaf


def drillnest(m,tier,branch,*,production=False):
    swarm = tier > 0 and branch == 0
    siege = tier > 0 and branch == 1
    enamel = "sun_cream" if swarm else "pyre_enamel" if siege else "iron"
    chassis(m,enamel,siege,tier)
    feeder(m,enamel,swarm,siege,tier)
    barrel(m,enamel,swarm,siege,tier)
    for s,node in ((1,"HeadGear"),(-1,"Tabard")):
        tread(m,s,node,enamel)


def chassis(m,enamel,siege,tier):
    h = "Hips"
    m.box("Beveled foundry chassis",h,enamel,(0,-.016,0),(.636,.276,.632),.040)
    m.box("Deep chassis plinth",h,"steel_dark",(0,-.114,0),(.618,.079,.667),.020)
    m.box("Wedge front chassis bumper",h,"brass" if siege else "slag",(0,-.031,.334),
          (.558,.126,.078),.024)
    m.box("Armored upper equipment hood","Head",enamel,(0,.005,-.115),(.270,.135,.250),.026)
    m.box("Recessed status window in visor","Head","ink",(0,.007,.014),(.213,.060,.018),.007)
    for x in (-.062,0,.062):
        m.box("Small amber machine optic","Head","forge",(x,.008,.026),(.035,.019,.011),.004)
    for s in (-1,1):
        m.tube("Twin rear vent stack","BackSpring","brass",[(s*.224,.158,-.265),
               (s*.221,.315,-.259),(s*.213,.495,-.262)],[.042,.048,.039],n=10)
        m.ell("Recessed stack mouth","BackSpring","ink",(s*.213,.498,-.262),
              (.029,.009,.029),segments=9)
        if siege and tier==2:
            m.tube("Heavy outward recoil support",h,"copper",[(s*.206,.043,.181),
                   (s*.313,.114,.202),(s*.339,.160,.307)],[.041,.048,.032],n=8)


def tread(m,s,node,enamel):
    x = s*.343
    m.box("Dark tracked drive foundation","Hips","ink",(x,-.037,0),(.139,.207,.760),.034)
    for z in (-.280,-.140,0,.140,.280):
        m.tube("Separate radial track wheel","Hips","slag",[(x-s*.035,-.037,z),(x+s*.076,-.037,z)],
               [.075,.075],n=11)
        m.tube("Drive wheel hub","Hips","brass",[(x+s*.070,-.037,z),(x+s*.088,-.037,z)],
               [.036,.032],n=9)
    for z in (-.323,-.195,-.065,.065,.195,.323):
        m.box("Animated lower tread link",node,"steel_dark",(x,-.142,z),(.170,.038,.112),0)
        m.box("Animated upper tread link",node,enamel,(x,.069,z),(.166,.030,.111),0)
    for z in (-.390,.390):
        m.box("Track return link",node,"iron",(x,-.035,z),(.172,.081,.035),.004)


def feeder(m,enamel,swarm,siege,tier):
    c = "Chest"
    m.box("Raised feed chamber housing",c,"steel_dark",(0,.061,-.052),(.467,.259,.405),.036)
    cells = [(-.134,.101),(0,.101),(.134,.101),(-.069,.216),(.069,.216)]
    if swarm and tier==2:
        cells.extend(((-.219,.189),(.219,.189)))
    for x,y in cells:
        ring = [(x+.057*cos(i*tau/6),y+.057*sin(i*tau/6),.182) for i in range(7)]
        m.tube("Hex feed cell rim",c,enamel,ring,[.011]*7,n=6)
        m.ell("Recessed dark feed cell",c,"ink",(x,y,.175),(.046,.044,.017),segments=9)
        m.tube("Stored drill nose",c,"brass",[(x,y,.176),(x,y,.223),(x,y,.250)],
               [.033,.028,.003],n=8)
    if swarm:
        for s in (-1,1):
            m.box("Ivory guide frame beside honeycomb",c,"sun_cream",(s*.208,.127,-.011),
                  (.101,.267,.335),.021)
            if tier==2:
                m.box("Secondary raised feed cassette",c,"brass",(s*.218,.275,-.025),
                      (.124,.118,.231),.023)


def barrel(m,enamel,swarm,siege,tier):
    w = "WeaponR"
    r = .092 if siege else .066
    m.tube("Independently recoiling barrel",w,enamel,[(0,0,-.025),(0,0,.108),
           (0,0,.267),(0,0,.330)],[r*.86,r,r*.92,r],n=13)
    m.rim("Heavy recoiling bore rim",w,"brass",(0,0,.337),(r,r),.014,segments=18)
    m.ell("Black opening behind barrel rim",w,"ink",(0,0,.330),(r*.82,r*.82,.005),segments=11)
    if siege:
        for s in (-1,1):
            m.tube("Outboard siege recoil rail",w,"copper",[(s*.076,0,.077),
                   (s*.104,0,.169),(s*.104,0,.309)],[.026,.030,.025],n=9)
            if tier==2:
                leaf(m,"Tapered double fork at siege bore",w,"gold",(s*.101,.032,.193),
                     (s*.128,.057,.381),.044,facing=(0,1,0))
