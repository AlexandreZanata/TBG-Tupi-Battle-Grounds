"""Sculpt a masked arbalist and a shoulder raven on independent animation nodes."""

from bloomkeeper import leaf

MUZZLE = (0,.035,.34)
SUPPORT = (0,-.02,.09)


def crowcaller(m,tier,branch,*,production=False):
    plague = tier > 0 and branch == 0
    ruin = tier > 0 and branch == 1
    cloth = "crow_plague" if plague else "crow_claret" if ruin else "steel_dark"
    coat(m,cloth,plague,ruin,tier)
    mask(m,cloth,plague,ruin,tier)
    raven(m,plague,ruin,tier)
    bow(m,plague,ruin,tier)


def coat(m,cloth,plague,ruin,tier):
    m.loft("Tailored high-collared coat","Chest",cloth,[
        (-.064,.119,.092,0),(.030,.127,.100,0),(.151,.147,.106,.007),
        (.245,.179,.105,.002),(.317,.129,.090,-.012),(.356,.077,.064,-.010),
    ],n=14)
    m.loft("Fitted split battle skirt","Hips",cloth,[
        (-.213,.175,.134,.002),(-.125,.166,.123,-.004),(-.038,.137,.102,0),(.019,.129,.096,0),
    ],n=14,opening=.43,fold=.11)
    m.loft("Short feather cape","Tabard","ink",[
        (-.338,.173,.069,-.022),(-.227,.149,.056,-.010),(-.103,.115,.044,0),
        (.010,.081,.036,.023),
    ],n=12,opening=1.62,fold=.18)
    for s in (-1,1):
        m.tube("Buckle strap on coat","Chest","leather",[(s*.136,.290,.068),
               (s*.113,.212,.095),(s*.098,.087,.105)],[.019,.022,.014],n=7)
        m.box("Coat clasp","Chest","brass",(s*.106,.165,.106),(.044,.036,.019),.006)
        if plague:
            m.ell("Sealed spore ampoule","Hips","venom",(s*.134,-.033,.113),
                  (.033,.053,.029),segments=9)
            m.box("Spore ampoule brass cap","Hips","brass",(s*.134,.012,.112),(.063,.026,.059),.007)
        if ruin:
            for i in range(2+tier):
                leaf(m,"Layered iron feather mantle","Chest","iron",(s*.137,.319,-.037+i*.041),
                     (s*(.245+.020*i),.226,-.042+i*.045),.045)
        side = "L" if s>0 else "R"
        arm,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Coat sleeve",arm,cloth,[(0,.030,0),(0,-.076,0),(0,-.216,0)],
               [.064,.067,.042],n=9)
        m.tube("Feather-lined bracer",fore,"hide",[(0,.019,0),(0,-.107,0),(0,-.189,0)],
               [.052,.056,.038],n=9)
        m.ell("Thin glove holding the crossbow",hand,"oldbone",(0,-.021,.007),
              (.042,.048,.033),segments=9)
        for node in (thigh,shin):
            m.tube("Slim trousers",node,"ink",[(0,.019,0),(0,-.104,0),(0,-.191,0)],
                   [.055,.051,.040],n=8)
        m.ell("Tapered calfskin boot",foot,"hide",(0,-.021,.046),(.058,.047,.101),segments=10)


def mask(m,cloth,plague,ruin,tier):
    h = "Head"
    m.loft("Deep fitted cowl",h,cloth,[(-.111,.073,.080,-.014),(-.051,.111,.112,-.022),
           (.062,.131,.121,-.023),(.137,.100,.107,-.043),(.179,.058,.073,-.054)],n=15)
    m.ell("Undead porcelain face plate",h,"bone",(0,.006,.093),(.096,.112,.059),segments=14)
    reach = .361 if plague and tier==2 else .273
    m.tube("Long downturned mask beak",h,"acid_ivory" if plague else "bone",[
        (0,.016,.118),(0,-.022,.182),(0,-.064,reach-.065),(0,-.101,reach),
    ],[.061,.062,.033,.004],n=11)
    for s in (-1,1):
        m.ell("Mask black eye socket",h,"ink",(s*.052,.031,.139),(.037,.032,.018),segments=10)
        m.ell("Mask eye lens",h,"venom" if plague else "bell_eye" if ruin else "soul",
              (s*.054,.031,.156),(.018,.017,.009),segments=9)
        m.tube("Sculpted orbital rim",h,"oldbone",[(s*.025,.057,.139),(s*.063,.067,.144),
               (s*.094,.032,.117)],[.013,.019,.009],n=7)
        if plague and tier==2:
            m.tube("Mask respirator line",h,"brass",[(s*.055,-.033,.194),(s*.077,-.063,.208),
                   (s*.083,-.074,.148),(s*.113,-.029,.064)],[.010,.013,.013,.012],n=7)


def raven(m,plague,ruin,tier):
    body, head = "BackSpring", "HeadGear"
    m.ell("Shoulder raven body",body,"ink",(0,.001,.008),(.069,.069,.105),segments=12)
    m.ell("Raven breast",body,"steel_dark",(0,-.007,.070),(.060,.056,.057),segments=10)
    m.ell("Independently aimed raven head",head,"ink",(0,.008,.010),(.045,.050,.048),segments=11)
    m.tube("Hooked raven beak",head,"oldbone",[(0,.002,.036),(0,-.009,.079),(0,-.020,.093)],
           [.023,.014,.003],n=8)
    for s in (-1,1):
        m.ell("Raven eye",head,"venom" if plague else "forge",(s*.032,.016,.043),
              (.013,.012,.009),segments=8)
        m.tube("Raven talon on shoulder",body,"oldbone",[(s*.028,-.047,.024),
               (s*.034,-.080,.027),(s*.034,-.085,.052)],[.009,.009,.004],n=6)
        leaf(m,"Splayed raven tail feather",body,"steel_dark",(s*.019,-.028,-.049),
             (s*.041,-.059,-.167),.034)
    for i in range(5):
        leaf(m,"Articulated raven wing flight feather","Back","crow_plague" if plague else "iron" if ruin else "steel_dark",
             (-.038,-.003,.041-i*.022),(-.128-.009*i,-.050-.012*i,.016-i*.040),.029)
    if plague and tier==2:
        m.ell("Raven spore carrier",body,"venom",(.055,-.026,.018),(.029,.039,.035),segments=9)


def bow(m,plague,ruin,tier):
    w = "WeaponR"
    m.tube("Carved crossbow stock and hand grip",w,"darkwood",[(0,-.027,-.102),(0,-.017,0),
           SUPPORT,(0,-.004,.289)],[.037,.039,.033,.024],n=9)
    m.box("Brass bolt rail",w,"brass",(0,.013,.160),(.060,.032,.305),.007)
    m.box("Palm seat under crossbow",w,"leather",SUPPORT,(.068,.047,.078),.008)
    for s in (-1,1):
        x = .253 if ruin else .210
        m.tube("Crossbow limb",w,"iron" if ruin else "darkwood",[(0,.025,.228),
               (s*.096,.021,.224),(s*x*.75,.028,.193),(s*x,.037,.160)],[.036,.029,.023,.008],n=8)
        m.tube("Crossbow cord",w,"linen",[(s*x,.037,.160),(0,.037,.036)],[.004,.004],n=5)
    if ruin and tier==2:
        m.tube("Iron high sight arch",w,"edge",[(-.058,.032,.121),(-.042,.107,.121),
               (0,.133,.121),(.042,.107,.121),(.058,.032,.121)],[.009]*5,n=7)
    m.tube("Nocked bolt shaft","Nock","wood",[(0,0,-.021),(0,0,.305)],[.008,.008],n=6)
    m.tube("Nocked bolt point at muzzle","Nock","sprout" if plague else "edge",
           [(0,0,.293),(0,0,.310),(0,0,MUZZLE[2])],[.010,.021,.002],n=7)
    for s in (-1,1):
        leaf(m,"Bolt fletching","Nock","linen",(0,0,.016),(s*.024,.022,-.034),.020)
