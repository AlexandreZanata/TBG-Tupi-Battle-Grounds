"""Sculpt a coffin-furnace skeleton with a braced two-hand flame nozzle."""

from bloomkeeper import leaf

SUPPORT = (0,0,.20)


def pyrewarden(m,tier,branch,*,production=False):
    funeral = tier > 0 and branch == 0
    pest = tier > 0 and branch == 1
    enamel = "sun_cream" if funeral else "pine" if pest else "pyre_enamel"
    fire = "venom" if pest else "forge"
    body(m,enamel,fire,funeral,pest)
    coffin(m,enamel,fire,funeral,pest,tier)
    nozzle(m,enamel,fire,funeral,pest,tier)


def body(m,enamel,fire,funeral,pest):
    h,c = "Head","Chest"
    m.loft("Layered furnace coat",c,"hide",[
        (-.053,.144,.109,-.020),(.078,.163,.130,-.012),(.210,.203,.136,.001),(.313,.150,.099,0),
    ],n=14)
    m.box("Riveted breast boiler plate",c,enamel,(0,.161,.134),(.264,.201,.066),.019)
    m.loft("Split furnaceman apron","Hips","leather",[(-.248,.201,.127,0),(-.101,.171,.110,-.007),
           (.046,.145,.113,0)],n=15,opening=.61,fold=.13)
    m.ell("Angular old skull",h,"bone",(0,-.003,.011),(.115,.131,.108),segments=14)
    m.ell("Separated skeletal jaw",h,"oldbone",(0,-.083,.061),(.080,.043,.063),segments=11)
    m.box("Forged skull brow helmet",h,"steel_dark",(0,.096,.016),(.246,.078,.191),.026)
    for x in (-.039,-.013,.013,.039):
        m.box("Visible lower tooth",h,"sun_cream",(x,-.065,.112),(.017,.031,.020),.004)
    for side,s in (("L",1),("R",-1)):
        m.ell("Skull eye recess",h,"ink",(s*.048,.017,.108),(.034,.031,.013),segments=9)
        m.ell("Furnace reflection in the eye",h,fire,(s*.048,.016,.118),(.014,.013,.007),segments=8)
        arm,fore,hand,thigh,shin,foot=(f"{part}{side}" for part in
                                      ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Skeletal upper sleeve",arm,"hide",[(0,.034,0),(0,-.109,.018),(0,-.269,0)],
               [.078,.083,.052],n=9)
        m.ell("Iron shoulder pad",arm,enamel,(s*.023,.006,.011),(.097,.069,.089),segments=10)
        m.tube("Forearm carrying cuff",fore,"leather",[(0,.032,0),(0,-.082,0),(0,-.242,0)],
               [.058,.065,.041],n=9)
        m.ell("Skeleton hand holding nozzle",hand,"oldbone",(0,-.021,.007),(.046,.052,.037),segments=10)
        for node in (thigh,shin):
            m.tube("Wrapped heavy lower limb",node,"steel_dark",[(0,.029,0),(0,-.082,0),(0,-.191,0)],
                   [.068,.065,.045],n=8)
        m.box("Furnace boot",foot,"slag",(0,-.017,.051),(.121,.088,.196),.017)


def coffin(m,enamel,fire,funeral,pest,tier):
    b = "BackSpring"
    outline = [(-.136,-.116),(.136,-.116),(.186,.316),(.118,.471),
               (-.118,.471),(-.186,.316)]
    m.plate("Beveled coffin pressure vessel",b,enamel,outline,(-.382,-.143),bevel=.029)
    m.plate("Inset cast coffin lid",b,"steel_dark",[(x*.81,y*.91+.018) for x,y in outline],
            (-.409,-.380),bevel=.011)
    for s in (-1,1):
        m.tube("Load-bearing stay from shoulder to coffin",b,"steel_dark",
               [(s*.105,.069,.100),(s*.141,.121,.006),(s*.139,.134,-.158)],
               [.031,.045,.037],n=9)
        m.tube("Shoulder stay brass hinge",b,"brass",[(s*.112,.091,.054),(s*.129,.106,.010)],
               [.050,.047],n=9)
        m.tube("Coffin chimney",b,"brass",[(s*.173,.259,-.257),(s*.184,.420,-.258),
               (s*.163,.528,-.247)],[.032,.035,.040],n=9)
        m.ell("Luminous opening in the chimney",b,fire,(s*.163,.535,-.247),
              (.025,.019,.024),segments=9)
        for y in (.008,.092,.176):
            m.box("Furnace vent slit",b,fire,(s*.066,y,-.416),(.080,.022,.009),.004)
        if funeral:
            leaf(m,"Ivory coffin edge reinforcement",b,"sun_cream",(s*.118,-.061,-.412),
                 (s*.156,.366,-.418),.039)
        if pest:
            m.ell("Pestilence bottle on coffin side",b,"venom",(s*.187,.181,-.300),
                  (.046,.088,.047),segments=10)
    if funeral and tier==2:
        m.box("Heavy lid crown",b,"sun_cream",(0,.489,-.269),(.264,.069,.252),.016)


def nozzle(m,enamel,fire,funeral,pest,tier):
    w = "WeaponR"
    z = .604 if funeral and tier==2 else .464
    m.tube("Braced flame nozzle body",w,enamel,[(0,.02,-.045),(0,.02,.095),(0,.02,.271),
           (0,.02,z-.019)],[.046,.061,.061,.071],n=14)
    m.box("Rear nozzle handgrip",w,"wood",(0,-.052,0),(.053,.132,.078),.011)
    m.box("Forward palm brace",w,"hide",SUPPORT,(.105,.070,.082),.014)
    m.rim("Flame bore aligned to the event muzzle",w,"brass",(0,.02,z-.013),(.065,.065),.013)
    m.ell("Recessed black flame bore",w,"ink",(0,.02,z-.021),(.053,.053,.005),segments=12)
    m.tube("Rigid hose elbow on the weapon",w,"steel_dark",[(.044,-.021,-.051),(.101,-.066,-.107),
           (.140,-.123,-.143),(.147,-.198,-.142)],[.028,.026,.027,.025],n=9)
    if pest:
        for s in (-1,1):
            m.tube("Flanking pestilence barrel",w,"pine",[(s*.071,.021,.137),
                   (s*.092,.021,.256),(s*.094,.021,.383 if tier==2 else .290)],
                   [.036,.037,.031],n=10)
            if tier==2:
                m.rim("Secondary pestilence nozzle rim",w,"copper",(s*.094,.021,.380),(.027,.027),.008,
                      segments=14)
