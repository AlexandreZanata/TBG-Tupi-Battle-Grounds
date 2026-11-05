"""Build a living seed thrower with separate socket-mounted pod jaws."""

from bloomkeeper import flower, leaf
from completion_people import limbs

RELEASE=(0,.12,.28)


def podsnapper(m,tier,branch,*,production=False):
    royal=tier > 0 and branch == 0
    harvest=tier > 0 and branch == 1
    bark="horn_stone" if harvest else "darkwood"
    plant="mire_reed" if royal else "beetle_pumpkin" if harvest else "leaf_ant"
    trim="venom" if royal else "sap" if harvest else "sprout"
    limbs(m,bark,"wood",arm=.22,leg=.34,width=.056,skin=bark)
    m.loft("Rootstock body and seated shoulders","Chest",bark,[
        (-.055,.097,.089,0),(.069,.129,.111,.007),(.184,.159,.115,0),
        (.264,.162,.104,-.006),(.315,.090,.072,-.002),
    ],n=13,fold=.08)
    m.tube("Bark collar connecting head and torso","Chest",bark,[(0,.275,0),(0,.340,0),
           (0,.412,0)],[.070,.064,.050],n=9)
    m.ell("Waist root knot","Hips",bark,(0,-.010,0),(.132,.066,.103),segments=11)
    for s,side in ((-1,"R"),(1,"L")):
        m.ell("Root socket at the upper arm","Chest",bark,(s*.168,.276,0),
              (.072,.070,.067),segments=10)
        leaf(m,"Folded shoulder leaf",f"UpperArm{side}",plant,(0,-.067,.044),
             (s*.094,.047,-.043),.047)
        m.tube("Raised bark seam","Chest","wood",[(s*.117,.242,.077),
               (s*.105,.157,.104),(s*.069,.053,.100)],[.020,.023,.017],n=8)
        m.tube("Splayed foot root",f"Foot{side}",bark,[(0,-.011,.037),(s*.058,-.026,.101),
               (s*.084,-.035,.156)],[.034,.028,.003],n=7)
        if harvest:
            leaf(m,"Dried husk hanging from root waist","Hips",plant,(s*.076,.018,.087),
                 (s*.145,-.140,.065),.046)
    head(m,bark,plant,trim,tier,royal,harvest)
    pod(m,plant,trim,tier,royal,harvest)


def head(m,bark,plant,trim,tier,royal,harvest):
    h="Head"
    m.loft("Carved seedkeeper face",h,bark,[
        (-.112,.060,.064,.026),(-.056,.105,.086,.009),(.032,.116,.100,.006),
        (.106,.099,.089,-.007),(.135,.054,.060,-.012),
    ],n=13,fold=.07)
    for s in (-1,1):
        m.ell("Inset seed-shaped eye socket",h,"ink",(s*.052,.016,.104),
              (.032,.026,.016),segments=9)
        m.ell("Seedkeeper eyes",h,"sap",(s*.053,.018,.117),(.021,.017,.012),segments=9)
    m.tube("Curved face notch",h,"wood",[(-.051,-.042,.097),(0,-.059,.100),
           (.048,-.043,.092)],[.006,.010,.007],n=6)
    for i in range(4 if royal or harvest else 3):
        x=(i-1.5)*.051
        leaf(m,"Separate asymmetric crown leaf",h,plant,(x,.087,.012),
             (x*2.35,.241+.024*i,-.072),.044)
    if royal:
        m.tube("Curl growing from the leaf crown",h,trim,[(.045,.119,.002),(.114,.245,-.020),
               (.110,.298,.041),(.068,.278,.071)],[.025,.017,.013,.002],n=8)
    if royal and tier==2:
        flower(m,h,(.061,.264,.063),.067,glow=True)
    if harvest and tier==2:
        leaf(m,"Tall split-husk crown spine",h,plant,(0,.099,.010),(0,.326,-.109),.060,facing=(1,0,0))


def pod(m,plant,trim,tier,royal,harvest):
    n="WeaponR"
    m.tube("Forearm-bound pod stalk",n,"wood",[(0,-.085,-.068),(0,0,0),
           (0,.052,.087)],[.046,.057,.045],n=9)
    for y,z in ((-.079,.147),(.012,.186),(.12,.28)):
        m.ell("Seed exposed between articulated valves",n,"sap" if royal else "bone",
              (0,y,z),(.046,.052,.043),segments=10)
    for s,joint in ((-1,"StringTop"),(1,"StringBottom")):
        root=(s*.035,.033,-.024)
        end=(s*(.115+.038*harvest),.136,.320)
        m.tube("Hinged longitudinal pod valve",joint,plant,[root,(s*.098,.019,.085),
               (s*.125,.063,.213),end],[.036,.076,.074,.008],n=11)
        m.tube("Raised valve seam",joint,trim,[(s*.050,.069,.026),(s*.138,.091,.153),end],
               [.016,.017,.004],n=7)
        if royal:
            leaf(m,"Ironpod reinforcing leaf",joint,"mantis_jade",(s*.041,-.012,.026),
                 (s*.160,.159,.215),.051,facing=(0,0,1))
        if harvest:
            leaf(m,"Separate split-husk flare",joint,"ochre_cloth",(s*.074,.010,.098),
                 (s*(.199+.033*(tier==2)),.112,.270),.056,facing=(0,1,0))
    if harvest and tier==2:
        m.ell("Harvest seed cluster fastened above the grip",n,"sap",(0,.113,.086),
              (.068,.053,.045),segments=10)
