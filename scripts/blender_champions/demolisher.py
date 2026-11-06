"""Build a hydraulic brawler whose knuckles land on the native hit anchor."""

from math import cos, sin, tau

TIP = (0,-.05,.25)


def demolisher(m,tier,branch,*,production=False):
    fists = tier > 0 and branch == 0
    shock = tier > 0 and branch == 1
    enamel = "press_ochre" if fists else "press_indigo" if shock else "copper"
    trunk(m,enamel,shock,tier)
    for side,s in (("L",1),("R",-1)):
        arm(m,side,s,enamel,fists,shock,tier)
        legs(m,side,enamel)


def trunk(m,enamel,shock,tier):
    c,h,b = "Chest","Head","BackSpring"
    m.loft("Shoulder-wide cast torso",c,"steel_dark",[
        (-.068,.158,.117,-.023),(.037,.186,.132,-.021),(.166,.246,.153,-.004),
        (.261,.268,.152,-.013),(.318,.199,.116,-.012),(.351,.112,.096,-.025),
    ],n=14)
    m.box("Beveled chest press cover",c,enamel,(0,.179,.133),(.320,.206,.105),.025)
    for x in (-.091,-.046,0,.046,.091):
        m.box("Vertical chest cooling vane",c,"slag",(x,.170,.187),(.024,.100,.019),.004)
    m.box("Heavy waist actuator","Hips","slag",(0,.004,0),(.300,.134,.211),.027)
    m.box("Wedge helmet foundation",h,"steel_dark",(0,-.009,.034),(.237,.186,.204),.032)
    m.box("Inset black visor bed",h,"ink",(0,-.023,.148),(.198,.071,.035),.011)
    m.box("Single recessed visor",h,"soul" if shock else "forge",(0,-.021,.168),(.157,.022,.010),.003)
    m.box("Helmet cast brow",h,enamel,(0,.047,.124),(.255,.060,.104),.012)
    m.tube("Back hydraulic pressure bottle",b,enamel,[(0,-.156,-.109),(0,-.105,-.129),
           (0,.173,-.129),(0,.230,-.119)],[.080,.126,.126,.083],n=14)
    for y in (-.092,.157):
        m.tube("Pressure bottle retaining collar",b,"iron",[(0,y-.019,-.129),(0,y+.019,-.129)],
               [.133,.133],n=13)
    for s in (-1,1):
        m.tube("Bent pressure manifold",b,"copper",[(s*.057,.185,-.177),(s*.124,.227,-.132),
               (s*.200,.163,-.058)],[.024,.028,.018],n=8)
    if shock and tier==2:
        for s in (-1,1):
            coil(m,b,(s*.184,.068,-.143),.058,.245)


def arm(m,side,s,enamel,fists,shock,tier):
    upper,fore,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
    m.ell("Rounded shoulder cylinder",upper,"slag",(s*.025,-.029,-.002),
          (.153,.126,.124),segments=13)
    m.ell("Shoulder enamel shroud",upper,enamel,(s*.039,.019,.004),(.151,.088,.130),segments=13)
    m.tube("Internal upper arm rod",upper,"iron",[(0,-.061,0),(0,-.190,.022),(0,-.259,0)],
           [.074,.083,.062],n=9)
    m.loft("Broad hydraulic forearm barrel",fore,enamel,[
        (-.237,.114,.096,.009),(-.194,.135,.116,.012),(-.045,.145,.123,.012),
        (.016,.123,.102,0),
    ],n=14)
    for x in (-.086,.086):
        m.tube("Exposed chrome piston",fore,"edge",[(x,.016,.103),(x,-.217,.112)],
               [.027,.026],n=8)
    m.box("Articulated square fist",hand,"steel_dark",(0,-.075,.064),(.264,.217,.259),.028)
    for x in (-.082,0,.082):
        m.box("Segmented knuckle pad",hand,"brass" if fists else "iron",(x,TIP[1],.222),
              (.076,.081,2*(TIP[2]-.222)),.008)
    if fists:
        for x in (-.132,.132):
            m.box("Reinforced fist cheek",hand,"press_ochre",(x,-.081,.139),(.047,.166,.181),.013)
        if tier==2:
            m.box("Reinforced heel of fist",hand,"brass",(0,-.186,.104),(.337,.067,.250),.014)
    if shock:
        coil(m,fore,(s*.148,-.097,.005),.046,.154)


def coil(m,node,center,r,length):
    x,y,z = center
    m.tube("Insulated capacitor core",node,"steel_dark",[(x,y-length/2,z),(x,y+length/2,z)],
           [r*.74,r*.74],n=10)
    points = [(x+r*cos(i*tau/10),y+length*(i/30-.5),z+r*sin(i*tau/10)) for i in range(31)]
    m.tube("Helical shock coil",node,"soul",points,[.009]*len(points),n=6)
    for sign in (-1,1):
        end = y+sign*length/2
        m.tube("Coil insulator cap",node,"iron",[(x,end-.013,z),(x,end+.013,z)],[r*1.12]*2,n=10)


def legs(m,side,enamel):
    for part in ("Thigh","Shin"):
        m.tube("Low brawler leg piston",f"{part}{side}","steel_dark",[(0,.032,0),(0,-.074,0),
               (0,-.166,0)],[.078,.086,.063],n=9)
    f = f"Foot{side}"
    m.box("Heavy steel boot",f,"slag",(0,-.012,.035),(.178,.108,.220),.023)
    m.box("Armor boot cap",f,enamel,(0,.014,.092),(.166,.081,.113),.014)
