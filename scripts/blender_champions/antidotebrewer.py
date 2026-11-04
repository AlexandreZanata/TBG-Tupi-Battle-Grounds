"""Build the herbalist's leaf hood, apron and workshop tools."""

from math import cos, sin, tau

from bloomkeeper import flower, leaf

MOUTH = (0,.07,.03)


def antidotebrewer(model, tier, branch, *, production=False):
    master = tier > 0 and branch == 0
    quick = tier > 0 and branch == 1
    garments(model, tier, master, quick)
    hood(model, tier, master, quick)
    flask(model,"WeaponR",MOUTH,1.18 if master else 1,master)
    if quick and tier == 2:
        flask(model,"OffhandL",MOUTH,1,False,secondary=True)
    workshop(model,tier,master,quick)


def garments(model, tier, master, quick):
    coat = "brew_violet" if quick else "leaf"
    accent = "violet" if quick else "herb_amber" if master else "oldbone"
    model.loft("Herbalist tunic","Chest",coat,[
        (-.049,.135,.091,0),(.057,.159,.117,0),(.161,.175,.118,.004),
        (.267,.156,.112,0),(.334,.126,.086,0),(.369,.078,.074,0),
    ],n=14,fold=.06)
    model.loft("Layered tunic hem","Hips","plum" if quick else "pine",[
        (-.146,.154,.110,0),(-.102,.173,.119,0),(.021,.158,.110,0),(.055,.132,.094,0),
    ],n=12,fold=.08)
    model.plate("Contoured linen apron bib","Chest","linen",[
        (-.12,-.015),(.119,-.015),(.104,.174),(.068,.271),(-.065,.271),(-.101,.174),
    ],(.101,.132),bevel=.010)
    model.plate("Hip-bound linen apron","Hips","linen",[
        (-.136,-.205),(.115,-.209),(.147,-.170),(.113,.015),(-.117,.015),
    ],(.098,.127),bevel=.008)
    for s,side in ((-1,"R"),(1,"L")):
        model.tube("Apron shoulder tie","Chest",accent,[(s*.074,.214,.124),
                   (s*.102,.321,.092),(s*.109,.347,.006),(s*.123,.290,-.106)],
                   [.010,.013,.014,.012],n=7)
        model.tube("Folded leaf sleeve",f"UpperArm{side}",coat,
                   [(0,.030,0),(-s*.018,-.060,.006),(0,-.172,0)],[.055,.071,.045],n=9)
        model.tube("Padded brewing cuff",f"Forearm{side}","leather",[(0,.021,0),(0,-.089,.007),(0,-.151,0)],
                   [.043,.051,.038],n=9)
        model.ell("Sprout fingers",f"Hand{side}","ogre",(0,-.020,.012),(.043,.053,.042),segments=10)
        for j,key in ((f"Thigh{side}","cloth"),(f"Shin{side}","hide")):
            model.tube("Herbalist leggings",j,key,[(0,.024,0),(0,-.105,.003),(0,-.190,0)],
                       [.059,.054,.046],n=8)
        model.loft("Wrapped root boot",f"Foot{side}","leather",[
            (-.060,.063,.097,.043),(-.046,.069,.110,.049),(.008,.065,.099,.041),(.046,.045,.060,0),
        ],n=10)
    if quick:
        model.tube("Slung vial bandolier","Chest","violet_dark",[
            (-.131,.092,.118),(-.044,.144,.158),(.050,.214,.158),(.137,.292,.104)],
            [.020,.027,.026,.018],n=8)
        for x,y,z in ((-.095,.123,.148),(-.035,.166,.171),(.030,.211,.172),(.089,.251,.149)):
            flask(model,"Chest",(x,y+.025,z),.48,False)
    if master or quick:
        model.tube("Dyed apron trim","Hips",accent,[
            (-.131,-.189,.127),(-.067,-.198,.133),(.073,-.200,.132),(.126,-.175,.124)],
            [.011]*4,n=7)
    if master:
        model.box("Waxed herb pouch","Hips","herb_amber",(.124,-.064,.109),(.090,.094,.057),.009)
        leaf(model,"Fresh herb sprig","Hips","sprout",(.127,-.036,.144),(.173,.035,.151),.031)
    if quick and tier==2:
        model.plate("Folded mixing towel","Hips","brew_violet",
                    [(-.139,-.199),(-.077,-.184),(-.067,-.050),(-.136,-.048)],
                    (.122,.139),bevel=.004)


def hood(model, tier, master, quick):
    node = "HeadGear"
    model.loft("Sprout-folk cheeks","Head","ogre",[
        (-.110,.051,.051,.015),(-.069,.101,.087,.020),(-.004,.130,.112,.010),
        (.069,.118,.099,-.004),(.120,.088,.072,-.013),
    ],n=14)
    model.loft("Fitted leaf hood",node,"violet_dark" if quick else "pine",[
        (-.116,.111,.110,-.017),(-.040,.140,.129,-.019),(.085,.149,.135,-.012),
        (.144,.122,.113,-.005),(.178,.075,.067,-.011),
    ],n=16,opening=.84,fold=.05)
    for s in (-1,1):
        model.ell("Warm inset eye","Head","ink",(s*.055,.033,.110),(.019,.017,.012),
                  segments=9,rings=4)
        model.ell("Amber iris","Head","brass",(s*.053,.033,.121),(.008,.009,.004),
                  segments=8,rings=4)
        leaf(model,"Layered hood leaf",node,"brew_violet" if quick else "leaf",
             (s*.072,.117,.097),(s*.137,-.075,.087),.061)
        leaf(model,"Young shoot",node,"sprout",(0,.148,-.015),(s*(.132 if master else .100),.267,-.026),.046)
    model.tube("Small sprout nose","Head","ogre_shade",[(0,.028,.102),(0,-.010,.144),(0,-.033,.122)],
               [.017,.024,.010],n=8)
    model.tube("Half smile","Head","pine",[(-.033,-.056,.103),(0,-.068,.117),(.034,-.055,.105)],
               [.004,.006,.004],n=6)
    if master:
        for x,y,z in ((-.120,.090,.094),(-.055,.135,.122),(.056,.135,.121),(.120,.090,.094)):
            flower(model,node,(x,y,z),.035 if tier == 2 else .027,glow=tier==2)
        model.tube("Amber crown braid",node,"herb_amber",[
            (-.132,.089,.092),(-.064,.140,.121),(0,.154,.123),(.064,.140,.121),(.132,.089,.092)],
            [.011]*5,n=7)


def flask(model,node,mouth,size,master,*,secondary=False):
    x,y,z = mouth
    profile = [(-.135,.024),(-.113,.039),(-.065,.046),(-.045,.033),(-.024,.015),(-.008,.017),(0,.020)]
    model.loft("Faceted remedy vessel",node,"herb_amber" if master else "brew_violet" if secondary else "pine",
               [(y+dy*size,r*size,r*size,z) for dy,r in profile],n=10,
               transform=lambda p:(p[0]+x,p[1],p[2]))
    model.ell("Inset glowing remedy window",node,"soul",(x,y-.079*size,z+.034*size),
              (.031*size,.041*size,.009*size),segments=10,rings=5)
    model.loft("Flask neck collar",node,"gold" if master else "edge" if secondary else "brass",
               [(y-.021*size,.019*size,.019*size,z),(y,.022*size,.022*size,z)],
               n=10,transform=lambda p:(p[0]+x,p[1],p[2]))


def workshop(model,tier,master,quick):
    node = "BackSpring"
    if tier == 2 and master:
        model.ell("Sealed copper still",node,"copper",(0,.038,-.135),
                  (.161,.169,.108),segments=16,rings=8)
        model.ell("Still inspection port",node,"soul",(0,.042,-.240),
                  (.075,.077,.015),segments=12)
        model.tube("Bent condenser",node,"brass",[
            (0,.146,-.132),(0,.285,-.132),(.119,.325,-.115),(.219,.254,-.092),(.209,.136,-.074),
        ],[.025,.025,.023,.021,.019],n=9)
    elif tier == 2 and quick:
        model.loft("Cast back cauldron",node,"iron",[
            (-.124,.089,.072,-.128),(-.092,.130,.098,-.134),(.023,.166,.119,-.136),(.081,.156,.110,-.134),
        ],n=14)
        model.ell("Active cauldron contents",node,"soul",(0,.079,-.134),(.144,.023,.104),segments=14)
        model.tube("Cauldron carrying bail",node,"brass",[
            (-.149,.025,-.143),(-.131,.173,-.140),(0,.243,-.129),(.128,.173,-.140),(.150,.025,-.143),
        ],[.014]*5,n=7)
    else:
        model.loft("Woven herb basket",node,"wicker",[
            (-.154,.100,.072,-.120),(-.104,.138,.085,-.127),(.079,.145,.095,-.130),
        ],n=14)
        for y in (-.096,-.045,.010,.070):
            model.tube("Wicker reinforcing band",node,"oldbone",[
                (.146*sin(i*tau/14),y,-.130+.094*cos(i*tau/14)) for i in range(15)],
                [.009]*15,n=5)
    for s in (-1,1):
        leaf(model,"Dried herb bundle",node,"sprout",(s*.074,.020,-.128),(s*.170,.242,-.183),.057)
