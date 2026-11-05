"""Build a stocky wetland guardian with a separately animated belly plate."""

from math import cos, sin, tau

from bloomkeeper import flower, leaf

PALM = (0,0,.12)


def capybara(m,tier,branch,*,production=False):
    stone = tier > 0 and branch == 0
    swamp = tier > 0 and branch == 1
    body(m,stone,swamp,tier)
    head(m,swamp)
    belly(m,stone,swamp,tier)
    for side,s in (("L",1),("R",-1)):
        limbs(m,side,s,stone,swamp)


def body(m,stone,swamp,tier):
    m.loft("Broad short-furred body","Chest","capy_fur",[
        (-.084,.231,.191,-.040),(.018,.317,.238,-.030),(.148,.367,.264,-.026),
        (.267,.344,.240,-.025),(.363,.266,.193,-.008),(.424,.147,.145,.034),
    ],n=18,fold=.034)
    m.ell("Rounded rear haunch","Hips","wood",(0,-.017,-.047),(.299,.142,.235),segments=15)
    m.tube("Leaf-collar vine","Chest","pine",[(-.259,.344,.051),(-.195,.302,.176),
           (0,.279,.233),(.195,.302,.176),(.259,.344,.051)],[.024]*5,n=8)
    for i in range(5):
        x = (i-2)*.091
        y = .305+.040*abs(i-2)
        leaf(m,"Folded collar leaf","Chest","capy_reed" if swamp else "leaf",(x,y,.175),
             (x*1.22,y-.123,.249-.022*abs(i-2)),.049)
    if stone and tier==2:
        for s in (-1,1):
            leaf(m,"Mossy stone shoulder crest","Chest","slate",(s*.207,.306,.039),
                 (s*.428,.456,-.027),.102)
            leaf(m,"Leaf on the stone crest","Chest","sprout",(s*.272,.335,.102),
                 (s*.438,.433,.061),.046)
    if swamp:
        swamp_pack(m,tier)


def swamp_pack(m,tier):
    b = "BackSpring"
    m.ell("Woven wetland pack",b,"wicker",(0,.033,-.109),(.207,.152,.125),segments=13)
    for y in (-.041,.023,.082):
        m.tube("Raised horizontal basket weave",b,"darkwood",[(-.151,y,-.149),
               (-.095,y,-.208),(0,y,-.234),(.095,y,-.208),(.151,y,-.149)],[.015]*5,n=7)
    for x in (-.114,.102):
        m.tube("Tall marsh reed",b,"capy_reed",[(x,.032,-.125),(x*.98,.221,-.163),
               (x*1.35,.358,-.176)],[.012,.011,.006],n=6)
        m.ell("Cattail seed head",b,"wood",(x*1.35,.333,-.173),(.021,.061,.023),segments=9)
    if tier==2:
        flower(m,b,(.096,.175,-.163),.085)
        flower(m,b,(-.120,.096,-.206),.074)
        m.tube("Wrapped willow hip belt","Hips","wicker",[(-.274,.046,.085),(-.173,.046,.215),
               (0,.046,.263),(.173,.046,.215),(.274,.046,.085)],[.034]*5,n=8)


def head(m,swamp):
    h = "Head"
    m.loft("Capybara skull with a broad snout bridge",h,"capy_fur",[
        (-.132,.107,.129,.067),(-.090,.161,.170,.053),(.006,.201,.185,.017),
        (.106,.171,.142,-.012),(.167,.112,.110,-.028),
    ],n=17)
    m.ell("Blocky rounded capybara muzzle",h,"wood",(0,-.035,.196),(.157,.110,.156),segments=16)
    m.ell("Broad satin nose",h,"hide",(0,-.010,.320),(.104,.057,.042),segments=13)
    m.ell("Dark horizontal mouth line",h,"hide",(0,-.103,.294),(.109,.022,.041),segments=12)
    m.box("Paired visible incisors",h,"sun_cream",(0,-.115,.320),(.058,.039,.028),.007)
    m.box("Incisor separation",h,"darkwood",(0,-.114,.335),(.006,.036,.006),.001)
    for s in (-1,1):
        m.ell("Small round capybara ear",h,"capy_fur",(s*.139,.159,-.032),(.057,.064,.034),segments=11)
        m.ell("Ear cup",h,"wood",(s*.141,.168,-.006),(.035,.043,.012),segments=9)
        m.ell("Dark eyes set high on the muzzle",h,"ink",(s*.150,.050,.141),
              (.038,.026,.031),segments=10)
        m.ell("Tiny eye catchlight",h,"linen",(s*.159,.058,.164),(.011,.008,.009),segments=8)
        m.tube("Soft fur ridge above eye",h,"capy_fur",[(s*.106,.081,.155),
               (s*.146,.090,.138),(s*.177,.068,.100)],[.015,.026,.013],n=7)
    if swamp:
        leaf(m,"Reed tucked behind ear",h,"capy_reed",(.121,.150,-.024),(.192,.270,-.034),.038)


def belly(m,stone,swamp,tier):
    node = "Tabard"
    if stone:
        m.ell("Belly stone foundation",node,"fortress_moss",(0,.003,.080),(.257,.235,.102),segments=15)
        for x,y,z,rx,ry in ((0,.065,.174,.133,.121),(-.130,-.030,.153,.113,.103),
                            (.130,-.030,.153,.113,.103),(0,-.123,.154,.107,.067)):
            m.ell("Rounded overlapping granite panel",node,"slate",(x,y,z),(rx,ry,.052),
                  segments=9,rings=5)
            if tier==2:
                leaf(m,"Moss frond rooted in granite",node,"sprout",(x,y+.009,z+.048),
                     (x+.028,y+.073,z+.071),.027)
        return
    m.ell("Soft buff belly",node,"wicker",(0,-.016,.093),(.241,.228,.093),segments=15)
    if swamp:
        for i in range(8):
            a = tau*i/8
            m.tube("Radial willow breast weave",node,"capy_reed",[(.068*sin(a),.064*cos(a),.190),
                   (.151*sin(a+.11),.135*cos(a+.11),.180),(.220*sin(a),.198*cos(a),.143)],
                   [.017,.021,.015],n=7)


def limbs(m,side,s,stone,swamp):
    arm,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                     ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
    m.tube("Heavy shoulder and upper arm",arm,"capy_fur",[(s*.012,.052,0),
           (s*.058,-.055,-.006),(s*.032,-.142,.011),(0,-.202,0)],[.117,.124,.102,.073],n=10)
    m.tube("Tapered forepaw forearm",fore,"capy_fur",[(0,.032,0),(s*.025,-.073,.023),(0,-.177,0)],
           [.083,.100,.070],n=9)
    if stone or swamp:
        m.tube("River guard cuff",fore,"slate" if stone else "capy_reed",[(0,-.083,0),(0,-.131,0)],
               [.101,.096],n=9)
    m.ell("Broad native strike paw",hand,"wood",(0,-.022,.038),(.084,.074,PALM[2]-.038),segments=13)
    for joint in (thigh,shin):
        m.tube("Short planted leg",joint,"capy_fur",[(0,.034,0),(0,-.063,0),(0,-.133,0)],
               [.091,.090,.073],n=9)
    m.ell("Planted webbed foot",foot,"wood",(0,-.022,.064),(.089,.045,.104),segments=11)
    for x in (-.051,0,.051):
        m.tube("Rounded foot claw",foot,"oldbone",[(x,-.020,.120),(x,-.025,.169),(x,-.031,.182)],
               [.019,.016,.003],n=6)
