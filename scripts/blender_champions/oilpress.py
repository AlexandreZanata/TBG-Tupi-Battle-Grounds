"""Build an ogre's walking boiler and double impact press."""

from math import cos, sin, tau


def oilpress(model,tier,branch,*,production=False):
    hull = tier > 0 and branch == 0
    tar = tier > 0 and branch == 1
    boiler(model,tier,hull,tar)
    helmet(model,tier,hull,tar)
    limbs(model,tier,hull,tar)
    back(model,tier,hull,tar)


def boiler(model,tier,hull,tar):
    shell = "brass" if hull else "patina" if tar else "copper"
    model.loft("Riveted pressure boiler","Chest",shell,[
        (-.053,.172,.121,0),(.029,.248,.160,0),(.102,.281,.195,0),
        (.254,.282,.195,0),(.323,.241,.168,0),(.371,.151,.105,-.012),
    ],n=18)
    for y in (.079,.274):
        model.loft("Boiler band","Chest","edge" if tar else "gold" if hull else "iron",[
            (y-.017,.259,.178,0),(y,.287,.201,0),(y+.019,.279,.196,0),
        ],n=18)
        for a in (-1.07,-.68,-.28,.29,.68,1.10):
            model.ell("Boiler rivet","Chest","iron" if hull else "brass",
                      (.280*sin(a),y,.200*cos(a)),(.014,.014,.013),segments=8,rings=4)
    model.ell("Brass pressure gauge housing","Chest","brass",(.106,.222,.188),
              (.061,.060,.029),segments=14,rings=6)
    model.ell("Pressure dial face","Chest","linen",(.106,.222,.215),
              (.049,.049,.010),segments=14,rings=5)
    model.tube("Black gauge needle","Chest","ink",[(.103,.212,.227),(.126,.247,.227)],
               [.008,.004],n=6)
    model.box("Recessed firebox","Chest","slag",(-.037,.119,.192),(.178,.114,.043),.013)
    for i in range(3):
        model.box("Firebox slit","Chest","forge" if hull and tier==2 else "wood",
                  (-.038,.092+i*.030,.217),(.136,.010,.010),.003)
    model.loft("Boiler hip casing","Hips","slag",[
        (-.111,.172,.123,0),(-.071,.231,.155,0),(.045,.217,.138,0),
    ],n=15)
    if hull:
        model.plate("Riveted hull breastplate","Chest","steel",[
            (-.236,.291),(-.181,.360),(.179,.360),(.233,.290),(.155,.273),(-.157,.273),
        ],(.137,.187),bevel=.012)
        if tier==2:
            model.plate("Foundry bulwark","Hips","brass",[
                (-.259,.024),(.259,.024),(.239,-.113),(.143,-.159),(-.141,-.159),(-.239,-.113),
            ][::-1],(.122,.181),bevel=.013)
    if tar:
        for x in (-.150,-.062,.041):
            model.tube("Viscous oil trail","Chest","oil_slick",[(x,.301,.161),(x-.021,.235,.188),
                       (x-.016,.175,.193),(x+.014,.132,.194)],[.022,.019,.015,.005],n=8)


def helmet(model,tier,hull,tar):
    model.loft("Sealed ogre pressure helmet","HeadGear","patina" if tar else "brass" if hull else "iron",[
        (-.094,.079,.076,.005),(-.055,.116,.104,.003),(.046,.126,.108,0),
        (.102,.113,.099,-.009),(.141,.068,.063,-.014),
    ],n=16)
    model.plate("Recessed visor slot","Head","ink",[
        (-.096,.019),(.094,.019),(.091,.053),(-.094,.058),
    ],(.092,.116),bevel=.006)
    for s in (-1,1):
        model.ell("Amber behind the visor","Head","forge",(s*.046,.039,.120),
                  (.022,.011,.007),segments=8,rings=4)
    model.loft("Vented helmet valve","HeadGear","brass",[
        (.125,.048,.048,0),(.155,.040,.040,0),(.185,.047,.043,0),
    ],n=10)
    if tar:
        model.tube("Oil breathing pipe","HeadGear","oil_slick",[
            (.084,.034,.061),(.149,.022,.031),(.149,-.066,-.024),(.107,-.088,-.050),
        ],[.025,.024,.025,.019],n=8)


def limbs(model,tier,hull,tar):
    for side,s in (("L",1),("R",-1)):
        arm,fore,hand = (f"{name}{side}" for name in ("UpperArm","Forearm","Hand"))
        model.tube("Connected boiler shoulder","Chest","steel_dark",[(s*.235,.277,.014),
                   (s*.299,.305,.011),(s*.330,.318,0)],[.088,.089,.071],n=10)
        model.ell("Press shoulder bearing",arm,"brass" if hull else "patina" if tar else "iron",
                  (0,-.008,0),(.116,.098,.111),segments=12)
        model.tube("Pressure cylinder",arm,"iron",[(0,.017,0),(0,-.092,0),(0,-.196,0)],
                   [.075,.085,.059],n=10)
        model.tube("Sliding press piston",fore,"edge",[(0,.043,.004),(0,-.106,.004),(0,-.167,.003)],
                   [.041,.050,.039],n=10)
        model.tube("Pressure hose",fore,"oil_slick" if tar else "hide",[
            (s*.050,.029,-.004),(s*.085,-.048,-.020),(s*.077,-.128,-.019),(s*.029,-.181,-.007),
        ],[.018,.021,.020,.014],n=7)
        model.box("Press fist housing",hand,"patina" if tar else "iron",(0,-.017,0),
                  (.162,.101,.152),.018)
        model.box("Machined flat strike plate",hand,"oil_slick" if tar and tier==2 else "edge",
                  (0,-.102,0),(.211,.076,.204),.016)
        for z in (-.063,.063):
            model.box("Pressed tool rib",hand,"brass" if hull else "iron",(0,-.084,z),(.224,.041,.041),.008)
        for joint,length in ((f"Thigh{side}",.191),(f"Shin{side}",.191)):
            model.tube("Load-bearing leg",joint,"steel_dark",[(0,.038,0),(0,-.083,0),(0,-length,0)],
                       [.076,.083,.064],n=9)
        foot = f"Foot{side}"
        model.loft("Flat anchor boot",foot,"iron",[
            (-.060,.096,.143,.047),(-.040,.103,.147,.051),(.014,.093,.112,.033),(.057,.069,.078,0),
        ],n=12)
        if hull and tier==2:
            for x in (-.070,.070):
                model.tube("Ground anchor spur",foot,"brass",[(x,-.035,.130),
                           (x,-.031,.184),(x,-.035,.215)],[.029,.021,.003],n=7)


def back(model,tier,hull,tar):
    node = "BackSpring"
    for x in (-.105,.105):
        model.tube("Smokestack neck",node,"iron",[(x,-.006,-.076),(x,.104,-.126),(x,.296,-.118)],
                   [.045,.044,.042],n=11)
        model.loft("Smokestack rolled lip",node,"brass",[
            (.267,.058,.053,-.118),(.290,.061,.057,-.118),(.303,.052,.045,-.118),
        ],n=12,transform=lambda p:(p[0]+x,p[1],p[2]))
        model.ell("Smokestack dark bore",node,"ink",(x,.303,-.118),
                  (.039,.006,.034),segments=10,rings=4)
    if tar:
        cylinders = ((0,.168),) if tier==2 else ((-.129,.083),(.129,.083))
        for x,r in cylinders:
            model.loft("Tar storage tank",node,"patina",[
                (-.212,r*.75,r*.58,-.164),(-.173,r,r*.76,-.169),
                (.114,r,r*.76,-.166),(.163,r*.72,r*.57,-.159),
            ],n=14,transform=lambda p:(p[0]+x,p[1],p[2]))
            for y in (-.154,.093):
                model.loft("Tar tank retaining strap",node,"edge",[
                    (y-.013,r*1.04,r*.80,-.167),(y+.013,r*1.04,r*.80,-.167),
                ],n=14,transform=lambda p:(p[0]+x,p[1],p[2]))
            model.tube("Overflow tube",node,"oil_slick",[(x,-.183,-.175),(x+.039,-.217,-.242),
                       (x+.059,-.252,-.261)],[.021,.016,.003],n=7)
