"""Sculpt a root-beast with a recessed maw and native-length whip roots."""

from math import cos, sin, tau

from bloomkeeper import leaf

TIP = (0,-.17*1.3*1.4,.06)


def leechroot(model,tier,branch,*,production=False):
    growth=tier>0 and branch==0
    drain=tier>0 and branch==1
    trunk(model,tier,growth,drain)
    maw(model,tier,growth,drain)
    limbs(model,tier,growth,drain)


def trunk(model,tier,growth,drain):
    bark="leech_plum" if drain else "darkwood"
    model.loft("Hunched root trunk","Chest",bark,[
        (-.064,.135,.101,.006),(.033,.181,.135,.005),(.136,.215,.152,-.008),
        (.241,.207,.143,-.029),(.334,.158,.128,-.033),(.373,.087,.093,-.024),
    ],n=16,fold=.085)
    for s in (-1,1):
        model.tube("Twisting bark rib","Chest","oldbone" if growth else "wood",[
            (s*.078,-.038,.105),(s*.124,.073,.146),(s*.109,.166,.154),
            (s*.165,.254,.116),(s*.242,.300,.036),
        ],[.023,.040,.033,.033,.025],n=8)
        model.tube("Root shoulder bridge","Chest",bark,[
            (s*.132,.237,.032),(s*.211,.285,.023),(s*.277,.308,.002),
        ],[.056,.062,.053],n=8)
        if drain:
            model.tube("Siphon vein branching across chest","Chest","leech_pulse",[
                (s*.024,.086,.143),(s*.057,.183,.154),(s*.130,.240,.132),(s*.212,.273,.092),
            ],[.007,.011,.009,.004],n=7)
    for i in range(9):
        a=tau*i/9
        x,z=sin(a),cos(a)
        model.tube("Interwoven rootlet fringe","Hips",bark,[
            (x*.105,.039,z*.080),(x*.152,-.052,z*.099),(x*.185,-.160,z*.129),
            (x*.151,-.228-.027*sin(i*2),z*.141),
        ],[.028,.035,.022,.004],n=7)
    if growth and tier==2:
        for x,y,z,r in ((-.219,.223,-.066,.080),(.211,.244,-.067,.084),(0,.285,-.101,.095)):
            model.tube("Load-bearing root for sap sac","BackSpring","wood",[
                (x*.32,y-.192,z-.028),(x*.62,y-.112,z-.054),(x*.91,y-.042,z-.029),
            ],[.031,.035,.027],n=8)
            model.ell("Bulging amber sap sac","BackSpring","herb_amber",(x,y,z),
                      (r*.86,r*1.24,r*.75),segments=10,rings=6)
            model.ell("Lit sap window","BackSpring","sap",(x,y+.011,z+r*.68),
                      (r*.39,r*.61,r*.14),segments=9,rings=5)
            model.tube("Sac capillary root","BackSpring","oldbone",[
                (x*.54,y-.109,z-.042),(x*.81,y-.055,z+.037),(x,y+.022,z+.051),
            ],[.013,.018,.006],n=7)
    if drain and tier==2:
        model.ell("Visible violet siphon bulb","Chest","leech_pulse",(0,.168,.170),
                  (.081,.084,.046),segments=12,rings=7)
        for s in (-1,1):
            model.tube("Root cage of the siphon bulb","Chest","leech_plum",[
                (s*.022,.082,.169),(s*.078,.147,.206),(s*.064,.236,.186),(s*.015,.254,.158),
            ],[.017,.019,.018,.010],n=8)


def maw(model,tier,growth,drain):
    node="Head"
    bark="leech_plum" if drain else "wood"
    model.loft("Extended wooden head hood",node,bark,[
        (-.113,.058,.062,.030),(-.071,.102,.107,.026),(.008,.127,.133,.026),
        (.089,.106,.118,.014),(.141,.065,.081,-.022),
    ],n=16,fold=.065)
    r = (.105 if tier==2 else .093) if growth else .080
    mouth(model,node,(0,-.005,.205),r,"oldbone" if growth else "wine" if not drain else "violet",
          12 if growth else 8)
    for s in (-1,1):
        model.tube("Knotted muzzle cheek",node,bark,[(s*.123,.028,.093),(s*.111,.012,.162),
                   (s*r*.92,-.041,.204)],[.027,.024,.012],n=8)
        leaf(model,"Root crown leaf","HeadGear","sprout" if growth else "violet" if drain else "leaf",
             (s*.048,.104,.021),(s*(.130 if growth else .095),.247,-.034),.048)
        if growth:
            model.tube("Maw growth horn",node,"oldbone",[(s*.096,.024,.161),
                       (s*.134,.058,.170),(s*.161,.119,.170)],[.025,.019,.003],n=7)
        if drain:
            model.tube("Feeler above the maw",node,"leech_plum",[(s*.102,.041,.148),
                       (s*.143,.109,.161),(s*.158,.144,.116)],[.017,.014,.003],n=7)


def mouth(model,node,center,r,color,count):
    x,y,z=center
    model.ell("Recessed black throat",node,"ink",(x,y,z-.010),(r*.82,r*.82,r*.26),
              segments=14,rings=6)
    path=[(x+r*cos(tau*i/20),y+r*sin(tau*i/20),z+.004*sin(i*tau/5)) for i in range(21)]
    model.tube("Round leech maw rim",node,color,path,[r*.19]*21,n=7)
    for i in range(count):
        a=tau*(i+.31)/count
        dx,dy=cos(a),sin(a)
        model.tube("Inward curved maw tooth",node,"bone",[
            (x+dx*r*.96,y+dy*r*.96,z+.005),
            (x+dx*r*.72,y+dy*r*.72,z+.023),
            (x+dx*r*.44,y+dy*r*.44,z+.028),
        ],[r*.13,r*.095,.0015],n=6)


def limbs(model,tier,growth,drain):
    bark="leech_plum" if drain else "darkwood"
    for side,s in (("R",-1),("L",1)):
        upper,fore,hand=(f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        model.tube("Twisted upper root",upper,bark,[(0,.026,0),(s*.013,-.069,.010),
                   (s*.006,-.154,.021),(0,-.221,0)],[.059,.070,.050,.044],n=9)
        model.tube("Tapered forearm root",fore,bark,[(0,.019,0),(s*.019,-.074,.014),
                   (s*.015,-.134,.013),(0,-.199,0)],[.047,.062,.050,.037],n=9)
        model.tube("Lashing whip root",hand,bark,[(0,.018,0),(s*.026,-.090,.031),
                   (s*.044,-.213,.072),TIP],[.044,.057,.042,.006],n=10)
        model.tube("Raised spiral root fiber",hand,"oldbone" if growth else "leech_pulse" if drain else "wood",[
            (s*.023,-.046,.035),(s*.051,-.132,.055),(s*.041,-.217,.104),(s*.014,-.284,.074),
        ],[.011,.016,.012,.004],n=7)
        if drain:
            model.ell("Raised violet wrist node",fore,"leech_pulse",(s*.034,-.124,.048),
                      (.031,.047,.030),segments=9,rings=5)
            if tier==2:
                mouth(model,hand,(0,-.261,.073),.040,"violet",7)
        for joint in (f"Thigh{side}",f"Shin{side}"):
            model.tube("Crooked root leg",joint,bark,[(0,.031,0),(s*.012,-.082,.015),(0,-.188,0)],
                       [.059,.059,.047],n=8)
        for x in (-.039,0,.039):
            model.tube("Planted root toe",f"Foot{side}",bark,[(x*.65,-.019,-.029),
                       (x,-.052,.065),(x*1.42,-.061,.136)],[.035,.030,.005],n=7)
