"""Build the Nester's leaf mantle, wicker basket and five clutches."""

from math import cos, sin, tau

from bloomkeeper import leaf


def nester(model, tier, branch, *, production=False):
    brood = tier > 0 and branch == 0
    blight = tier > 0 and branch == 1
    outfit(model, tier, blight)
    head(model, tier, blight)
    nest(model, tier, brood, blight)
    seed = "wood" if not blight else "darkwood"
    model.ell("Seed at the cast point", "WeaponR", seed, (0,.030,.060),
              (.036,.049,.036), segments=11)
    model.tube("Split seed seam", "WeaponR", "venom" if blight else "wicker",
               [(-.020,.013,.085),(0,.030,.094),(.020,.047,.085)], [.004,.006,.004], n=6)


def outfit(model, tier, blight):
    moss = "pine" if blight else "leaf"
    model.loft("Round moss belly", "Chest", moss, [
        (-.068,.135,.111,.019),(.023,.206,.172,.034),(.126,.229,.190,.034),
        (.224,.196,.155,.014),(.296,.142,.108,-.010),(.341,.084,.072,-.013),
    ], n=16, fold=.06)
    model.loft("Root-fiber hip wrap", "Hips", "darkwood", [
        (-.123,.195,.138,.008),(-.086,.212,.157,.009),(.027,.172,.130,0),(.047,.146,.110,0),
    ], n=12, fold=.10)
    for s in (-1,1):
        leaf(model,"Belly mantle leaf","Chest","leaf" if blight else "sprout",
             (s*.128,.256,.101),(s*.193,.083,.145),.052)
        model.tube("Load-bearing basket strap","Chest","hide", [
            (s*.159,.280,-.060),(s*.137,.251,.108),(s*.107,.155,.200),(s*.078,.047,.213),
        ], [.019,.022,.022,.017],n=7)
        model.tube("Shoulder bridge","Chest",moss, [
            (s*.142,.253,0),(s*.216,.280,0),(s*.268,.301,0),
        ], [.052,.061,.050],n=8)
    for side,s in (("L",1),("R",-1)):
        upper,lower,hand = (f"{part}{side}" for part in ("UpperArm","Forearm","Hand"))
        model.loft("Rounded moss sleeve",upper,moss,[
            (-.201,.055,.051,0),(-.112,.065,.062,.010),(-.016,.077,.068,0),(.024,.058,.056,0),
        ],n=10)
        model.tube("Tapered wooden forearm",lower,"wood",
                   [(0,.019,0),(s*.011,-.073,.004),(0,-.181,.008)],[.054,.058,.040],n=8)
        leaf(model,"Overlapping forearm leaf",lower,"leaf" if blight else "sprout",
             (-s*.019,-.022,.048),(s*.057,-.160,.038),.038)
        model.ell("Seed-tending hand",hand,"ogre_shade",(0,-.024,.021),(.051,.055,.043),segments=10)
        if blight and tier==2:
            model.tube("Blight branch shoulder spur",upper,"darkwood",
                       [(s*.023,.010,-.020),(s*.058,.079,-.010),(s*.075,.110,-.038)],
                       [.031,.020,.004],n=7)
        for joint,color in ((f"Thigh{side}","darkwood"),(f"Shin{side}","wood")):
            model.tube("Short root leg",joint,color,[(0,.017,0),(s*.007,-.069,0),(0,-.160,0)],
                       [.055,.050,.049],n=8)
        model.ell("Stubby root foot",f"Foot{side}","hide",(0,-.019,.046),(.068,.049,.092),segments=10)


def head(model, tier, blight):
    model.loft("Rounded wooden face","Head","ogre",[
        (-.115,.055,.049,.038),(-.087,.097,.080,.028),(-.026,.132,.109,.010),
        (.038,.134,.111,.003),(.106,.106,.092,-.014),(.140,.055,.061,-.020),
    ],n=15)
    model.loft("Overlapping broadleaf hood","Head","pine" if blight else "leaf",[
        (.020,.117,.106,-.018),(.071,.156,.129,-.012),(.106,.141,.112,-.016),
        (.154,.107,.089,-.024),(.176,.041,.044,-.023),
    ],n=15,fold=.08)
    for s in (-1,1):
        model.ell("Nester's almond eye","Head","ink",(s*.054,.029,.106),(.021,.013,.011),segments=9)
        model.ell("Eye catchlight","Head","brass",(s*.055,.030,.114),(.008,.008,.005),segments=8)
        model.ell("Rounded leaf cheek","Head","ogre_shade",(s*.090,-.034,.084),(.039,.039,.026),segments=10)
        leaf(model,"Side leaf of the hood","Head","pine" if blight else "sprout",
             (s*.053,.126,.079),(s*.145,.022,.067),.039)
    model.tube("Small smile","Head","hide",
               [(-.033,-.053,.108),(0,-.063,.117),(.036,-.050,.106)],[.005,.006,.005],n=6)
    leaf(model,"Springing hood shoot","HeadGear","sprout" if not blight else "violet",
         (0,.143,-.014),(-.059,.223+.014*tier,-.061),.031)


def wicker_ring(model, name, y, radius, depth, offset=0):
    points = [(radius*cos(i*tau/20),y+.009*sin(i*tau/5+offset),-.142+depth*sin(i*tau/20))
              for i in range(21)]
    model.tube(name,"BackSpring","wicker" if offset==0 else "wood",points,[.010]*21,n=5)


def nest(model, tier, brood, blight):
    node = "BackSpring"
    if brood and tier==2:
        skep(model)
        return
    width = .213+.034*int(tier>0)
    model.loft("Nested split-willow basket",node,"wood",[
        (-.046,width*.57,.082,-.141),(-.012,width*.78,.109,-.141),
        (.055,width*.91,.128,-.141),(.133,width,.139,-.141),
    ],n=15)
    for i,(y,k) in enumerate(((.002,.76),(.043,.87),(.088,.95),(.129,1))):
        wicker_ring(model,"Alternating wicker strand",y,width*k,.139*k,i%2)
    egg_color = "ritual" if blight and tier==2 else "venom" if blight else "linen"
    eggs = [(-.137,.226,-.141),(.127,.244,-.129)]
    if brood or tier==2:
        eggs.append((0,.286,-.166))
    for i,(x,y,z) in enumerate(eggs):
        model.ell("Nested brood egg",node,egg_color,(x,y,z),
                  (.071,.107 if i==2 else .091,.069),segments=12)
        if blight and tier==2:
            for s in (-1,1):
                model.tube("Dark thorn around a blight egg",node,"darkwood",[
                    (x+s*.049,y-.026,z),(x+s*.066,y+.045,z),(x+s*.054,y+.113,z-.025),
                ],[.022,.013,.003],n=6)
        else:
            model.ell("Speckled eggshell",node,"wood" if not blight else "pine",
                      (x+.018,y+.034,z+.060),(.016,.021,.007),segments=8)


def skep(model):
    node = "BackSpring"
    model.loft("High straw skep",node,"wicker",[
        (-.041,.162,.100,-.141),(.030,.221,.137,-.141),(.126,.243,.151,-.141),
        (.239,.214,.147,-.141),(.346,.160,.113,-.141),(.411,.076,.063,-.141),
        (.421,.026,.030,-.141),
    ],n=16)
    for i,(y,r,d) in enumerate(((.017,.212,.131),(.070,.235,.147),(.124,.245,.154),
                                (.181,.230,.152),(.235,.216,.150),(.290,.188,.137),
                                (.338,.164,.119),(.384,.114,.090))):
        wicker_ring(model,"Hand-woven skep course",y,r,d,i%2)
    model.ell("Dark skep entrance",node,"hide",(0,.086,.012),(.069,.051,.019),segments=10)
    for s in (-1,1):
        model.ell("Amber brood pod",node,"sap",(s*.183,.213,-.061),
                  (.050,.074,.047),segments=11)
