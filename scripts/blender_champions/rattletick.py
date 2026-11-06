"""Sculpt two brothers around one existing kit and independent shoulder joints."""

from math import cos, sin, tau

STRIKE = (0,-.40,0)


def rattletick(m,tier,branch,*,production=False):
    fury = tier > 0 and branch == 0
    tricks = tier > 0 and branch == 1
    cloth = "brother_carmine" if fury else "brother_coast" if tricks else "ochre_cloth"
    bruiser(m,cloth,fury,tricks,tier)
    head(m,cloth,fury,tier)
    club(m,fury,tier)
    rider(m,cloth,fury,tricks,tier)


def bruiser(m,cloth,fury,tricks,tier):
    c = "Chest"
    m.loft("Burly brother's work tunic",c,cloth,[
        (-.061,.167,.129,0),(.045,.193,.140,.003),(.175,.229,.163,.005),
        (.261,.241,.147,0),(.323,.202,.128,-.004),(.360,.106,.098,0),
    ],n=16,fold=.058)
    m.loft("Thick kidney belt","Hips","hide",[(-.037,.159,.122,0),(.020,.189,.148,0),
           (.081,.185,.145,0)],n=14)
    m.tube("Rider harness crossing torso",c,"hide",[(.188,.318,-.003),(.120,.244,.140),
           (.036,.146,.177),(-.089,.077,.153),(-.133,.018,.116)],[.035,.038,.037,.034,.033],n=8)
    m.box("Heavy harness buckle",c,"brass",(.066,.184,.184),(.072,.066,.023),.010,rotation=-.55)
    m.box("Recessed buckle tongue",c,"darkwood",(.066,.184,.199),(.040,.034,.010),.003,rotation=-.55)
    m.box("Tick shoulder saddle and straps",c,"hide",(.267,.352,-.046),(.214,.110,.218),.019)
    if fury:
        for i in range(3):
            m.box("Stacked carmine war pauldron",c,"brother_carmine",(-.202-.025*i,.340-.035*i,.018),
                  (.119,.083,.212-i*.014),.017,rotation=.40)
        if tier==2:
            m.box("Brass reinforced breastplate",c,"brass",(-.087,.175,.149),(.188,.236,.071),.023,rotation=.32)
    if tricks:
        m.box("Mechanic's broad back tool roll","Tabard","brother_coast",(0,.004,-.032),(.277,.182,.147),.028)
        for s in (-1,1):
            m.tube("Stored bronze workshop tool","Tabard","brass",[(s*.098,-.027,-.104),
                   (s*.103,.123,-.122)],[.017,.025],n=8)
            if tier==2:
                m.ell("Extra packed workshop charge","Tabard","copper",(s*.137,-.017,-.020),
                      (.065,.076,.071),segments=10)
    for side,s in (("L",1),("R",-1)):
        arm,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Heavy bare upper arm",arm,"skin",[(0,.025,0),(s*.014,-.047,0),(0,-.178,0)],
               [.092,.103,.069],n=10)
        m.tube("Wrapped lower arm",fore,"leather",[(0,.020,0),(s*.009,-.063,0),(0,-.154,0)],
               [.070,.081,.059],n=9)
        m.ell("Large gripping fist",hand,"skin",(0,-.025,.010),(.065,.063,.053),segments=11)
        for node in (thigh,shin):
            m.tube("Heavy linen trousers",node,"darkwood",[(0,.034,0),(0,-.086,0),(0,-.191,0)],
                   [.090,.087,.062],n=9)
        m.box("Stout work boot",foot,"hide",(0,-.011,.044),(.153,.109,.204),.024)


def head(m,cloth,fury,tier):
    h = "Head"
    m.loft("Broad bruiser skull",h,"skin",[(-.105,.072,.067,.027),(-.061,.113,.104,.021),
           (.019,.119,.107,.010),(.083,.106,.098,0),(.119,.079,.071,-.012)],n=15)
    m.ell("Heavy jaw stubble",h,"darkwood",(0,-.078,.081),(.098,.056,.074),segments=12)
    m.ell("Flattened boxer nose",h,"skin",(0,-.005,.116),(.032,.037,.038),segments=10)
    for s in (-1,1):
        m.ell("Eyelid around bruiser's eye",h,"linen",(s*.048,.025,.102),
              (.025,.014,.016),segments=9)
        m.ell("Bruiser eye pupil",h,"ink",(s*.048,.025,.114),(.010,.011,.007),segments=8)
        m.tube("Heavy angular brow",h,"darkwood",[(s*.020,.038,.110),(s*.063,.060,.105),
               (s*.093,.056,.085)],[.013,.015,.008],n=7)
    m.loft("Thick bruiser headband",h,cloth,[(.061,.111,.104,0),(.098,.111,.104,-.005),
           (.125,.082,.084,-.006)],n=15)
    if fury and tier==2:
        m.tube("Brass brow guard",h,"brass",[(-.117,.067,.043),(-.085,.081,.093),
               (0,.096,.110),(.085,.081,.093),(.117,.067,.043)],[.015,.024,.022,.024,.015],n=7)


def club(m,fury,tier):
    w = "WeaponR"
    m.tube("Wrapped oak club handle",w,"wood",[(0,-.269,0),(0,-.084,0),(0,.036,0),(0,.122,0)],
           [.053,.046,.040,.037],n=10)
    for y in (-.093,-.061,-.029,.003,.035,.067):
        m.tube("Club grip leather wrap",w,"hide",[(0,y-.011,0),(0,y+.010,0)],[.048]*2,n=8)
    m.box("Iron-bound oak club head",w,"darkwood",STRIKE,(.241 if fury else .185,.273,.205),.028)
    for y in (-.503,-.323):
        m.box("Club retaining collar",w,"brass" if fury else "iron",(0,y,0),
              (.259 if fury else .206,.047,.223),.011)
    for s in (-1,1):
        if fury:
            for y in (-.462,-.368):
                end = .178 if tier==2 else .147
                m.tube("Heavy iron club tooth",w,"edge",[(s*.099,y,0),(s*.132,y,0),(s*end,y,0)],
                       [.034,.031,.003],n=7)


def rider(m,cloth,fury,tricks,tier):
    b,h = "BackSpring","HeadGear"
    m.loft("Small shoulder brother's vest",b,cloth,[(-.021,.064,.051,0),(.052,.077,.059,.005),
           (.125,.078,.053,0),(.175,.047,.043,0)],n=12)
    m.ell("Tick's small expressive head",h,"skin",(0,-.003,.014),(.067,.075,.063),segments=13)
    m.ell("Tick's impish grin",h,"hide",(0,-.035,.071),(.028,.012,.010),segments=9)
    for s in (-1,1):
        m.ell("Tick's alert eye",h,"linen",(s*.028,.015,.068),(.016,.012,.008),segments=8)
        m.ell("Tick's pupil",h,"ink",(s*.029,.015,.075),(.007,.008,.005),segments=8)
        m.tube("Rider leg across shoulder",b,"darkwood",[(s*.047,.013,.012),
               (s*.075,-.055,.059),(s*.079,-.097,.076)],[.024,.028,.025],n=8)
    m.loft("Tick's brimmed cap",h,"hide",[(.036,.070,.056,0),(.043,.089,.069,.005),
           (.066,.071,.059,0),(.104,.051,.044,-.005)],n=14)
    if tricks:
        m.box("Tick's workshop goggles",h,"brass",(0,.043,.066),(.117,.031,.024),.009)
        for x in (-.031,.031):
            m.ell("Tick's smoked goggle lens",h,"steel_dark",(x,.045,.081),
                  (.021,.013,.008),segments=8)
    for arm in ("Back","StringTop"):
        m.tube("Independent rider arm",arm,"skin",[(0,.018,0),(0,-.054,.008),
               (0,-.111,-.006)],[.027,.028,.019],n=8)
        m.ell("Rider grip",arm,"skin",(0,-.125,-.006),(.023,.026,.022),segments=8)
    blunderbuss(m,tricks,tier)


def blunderbuss(m,tricks,tier):
    w = "Nock"
    m.box("Rider's weapon grip and stock",w,"wood",(0,-.025,.009),(.053,.066,.104),.009)
    m.tube("Rider's flared blunderbuss barrel",w,"brass" if tricks else "iron",
           [(0,.044,.004),(0,.044,.074),(0,.044,.146),(0,.044,.188)],
           [.027,.026,.032,.043],n=12)
    rim = [(.032*cos(i*tau/12),.044+.032*sin(i*tau/12),.192) for i in range(13)]
    m.tube("Rider blunderbuss open rim",w,"edge",rim,[.008]*13,n=6)
    m.ell("Black inside of rider bore",w,"ink",(0,.044,.186),(.024,.024,.003),segments=10)
    if tricks and tier==2:
        m.box("Upgraded blunderbuss top pressure valve",w,"copper",(0,.089,.119),
              (.069,.064,.074),.012)
