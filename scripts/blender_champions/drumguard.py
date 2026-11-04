"""Tailor the marching drummer and align sticks with the hip drum."""

from math import cos, sin, tau

DRUM_AT = (.08,.02,.20)


def drumguard(model,tier,branch,*,production=False):
    march = tier>0 and branch==0
    iron = tier>0 and branch==1
    uniform(model,tier,march,iron)
    head(model,tier,march,iron)
    drum(model,tier,march,iron)
    for node in ("WeaponR","OffhandL"):
        model.tube("Turned ash drumstick",node,"wood",[(0,0,-.030),(0,0,.072),(0,0,.193),(0,0,.230)],
                   [.016,.014,.012,.010],n=8)
        model.ell("Rounded drumstick tip",node,"linen",(0,0,.23),(.020,.020,.022),
                  segments=8,rings=5)


def uniform(model,tier,march,iron):
    cloth = "field_slate" if iron else "drum_carmine" if march else "wine"
    edge = "edge" if iron else "gold" if march else "oldbone"
    model.loft("Fitted drummer's coat","Chest",cloth,[
        (-.058,.128,.102,0),(.029,.162,.116,0),(.152,.175,.129,0),
        (.264,.165,.112,-.007),(.330,.125,.096,-.006),(.375,.076,.070,0),
    ],n=16,fold=.034)
    model.loft("Cutaway coat tails","Hips",cloth,[
        (-.186,.165,.116,0),(-.127,.169,.123,0),(-.041,.154,.124,0),(.051,.143,.109,0),
    ],n=14,opening=.52,fold=.069)
    model.loft("Drum belt","Hips","hide",[(-.012,.147,.121,0),(.032,.148,.122,0)],n=13)
    for s,side in ((-1,"R"),(1,"L")):
        arm,fore,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        model.tube("Tailored coat sleeve",arm,cloth,[(0,.038,0),(-s*.016,-.070,.004),(0,-.214,0)],
                   [.055,.069,.043],n=9)
        model.tube("Fitted drum cuff",fore,edge,[(0,.028,0),(0,-.097,.008),(0,-.193,0)],
                   [.045,.053,.039],n=9)
        model.ell("Drumstick grip",hand,"skin",(0,-.015,.012),(.041,.053,.041),segments=10,rings=5)
        model.tube("Crossed linen sling","Chest","linen" if not iron else "leather",[
            (s*.112,.322,.094),(s*.085,.245,.132),(0,.154,.143),(-s*.104,.025,.112),
        ],[.014,.018,.019,.015],n=7)
        for joint,key in ((f"Thigh{side}","linen"),(f"Shin{side}","hide")):
            model.tube("Marching breeches and boot leg",joint,key,[(0,.022,0),(0,-.089,.007),(0,-.189,0)],
                       [.063,.063,.047],n=9)
        model.loft("Rounded marching boot",f"Foot{side}","hide",[
            (-.060,.072,.109,.044),(-.043,.078,.120,.049),(.016,.067,.099,.038),(.053,.047,.065,0),
        ],n=11)
        if iron:
            model.loft("Armored drummer pauldron",arm,"steel",[
                (-.058,.067,.069,0),(-.033,.093,.093,0),(.031,.092,.090,0),(.061,.056,.066,0),
            ],n=11)
            if tier==2:
                model.plate("Shoulder guard lamella",arm,"edge",[
                    (-.075,-.079),(.074,-.078),(.070,-.016),(-.066,-.010),
                ],(.074,.094),bevel=.006)
    if iron:
        model.plate("Rolled steel breastplate","Chest","steel",[
            (-.131,.121),(.136,.119),(.157,.233),(.096,.299),(-.099,.299),(-.157,.235),
        ],(.109,.146),bevel=.011)
    else:
        for y in (.071,.151,.247):
            model.ell("Stamped coat button","Chest",edge,(0,y,.145),
                      (.014,.014,.008),segments=8,rings=4)


def head(model,tier,march,iron):
    model.loft("Drummer's rounded face","Head","skin",[
        (-.111,.066,.056,.017),(-.073,.104,.090,.010),(.004,.124,.107,0),
        (.091,.114,.098,-.009),(.134,.079,.072,-.010),
    ],n=15)
    for s in (-1,1):
        model.ell("Drummer eye","Head","ink",(s*.047,.028,.109),(.021,.013,.009),segments=8,rings=4)
        model.tube("Trimmed marching moustache","Head","hide",[
            (s*.007,-.038,.139),(s*.035,-.046,.145),(s*.077,-.043,.113),
        ],[.010,.015,.003],n=7)
    model.ell("Rounded drummer nose","Head","skin",(0,-.009,.116),(.030,.030,.028),segments=10)
    if iron:
        model.loft("Riveted kettle helmet","HeadGear","steel",[
            (.053,.139,.129,-.007),(.071,.150,.141,-.004),(.133,.130,.109,-.008),
            (.171,.092,.084,-.012),(.181,.055,.056,-.012),
        ],n=16)
        if tier==2:
            model.tube("Iron rhythm helmet comb","HeadGear","edge",[
                (0,.152,.086),(0,.189,.030),(0,.200,-.038),(0,.174,-.080),
            ],[.016,.022,.022,.008],n=7)
    else:
        high = .322 if march else .255
        model.loft("Tall bound shako","HeadGear","drum_carmine" if march else "wine",[
            (.065,.122,.116,-.012),(.089,.132,.122,-.011),(high-.041,.126,.116,-.014),
            (high,.112,.105,-.015),
        ],n=16)
        model.plate("Shako visor peak","HeadGear","hide",
                    [(-.086,.061),(.083,.061),(.082,.080),(-.084,.080)],(.107,.167),bevel=.007)
        model.plate("Gilt shako fitting","HeadGear","gold" if march else "brass",
                    [(-.035,.129),(0,.106),(.034,.129),(.031,.200),(-.033,.200)],(.101,.131),bevel=.006)
        if march:
            for x in ((-.065,.046) if tier==2 else (.045,)):
                model.tube("Swept parade plume","HeadGear","bone",[
                    (x,.254,.012),(x,.359,.005),(x-.019,.429,-.038),(x-.061,.468,-.075),
                ],[.021,.044,.029,.003],n=9)


def drum(model,tier,march,iron):
    r,depth = (.20,.18) if march and tier==2 else (.16,.15) if march else (.14,.13)
    x,y,z=DRUM_AT
    top,bottom = y+depth/2,y-depth/2
    node="Hips"
    trim="edge" if iron else "gold" if march else "brass"
    model.loft("Bent drum shell",node,"steel_dark" if iron else "copper" if march else "wood",[
        (bottom,r*.96,r*.96,z),(bottom+.021,r,r,z),(top-.018,r,r,z),(top,r*.96,r*.96,z),
    ],n=16,transform=lambda p:(p[0]+x,p[1],p[2]))
    model.loft("Tensioned drum membrane",node,"linen",[
        (top-.006,r,r,z),(top+.006,r*.98,r*.98,z),
    ],n=20,transform=lambda p:(p[0]+x,p[1],p[2]))
    for level in (bottom,top):
        model.loft("Rolled drum hoop",node,trim,[
            (level-.009,r*1.055,r*1.055,z),(level+.009,r*1.055,r*1.055,z),
        ],n=18,transform=lambda p:(p[0]+x,p[1],p[2]))
    for i in range(8):
        a=tau*i/8
        points=[(x+r*1.035*sin(a+d),h,z+r*1.035*cos(a+d))
                for d,h in ((-.15,bottom+.011),(.10,y),(.17,top-.009))]
        model.tube("Cross-laced drum tension line",node,trim if iron else "linen",points,
                   [.008,.007,.008],n=5)
    if iron and tier==2:
        for s in (-1,1):
            x0=x+s*.062
            model.tube("Engraved luminous rhythm rune",node,"soul",[
                (x0-.017,y-.021,z+r-.006),(x0,y+.023,z+r+.002),(x0+.017,y-.022,z+r-.006),
            ],[.006,.006,.006],n=5)
