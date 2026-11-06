"""Forge the skeleton knight, hollow chest furnace and heated war hammer."""

from math import cos, sin, tau


def bloodfurnace(model, tier, branch, *, production=False):
    infernal = tier > 0 and branch == 0
    banked = tier > 0 and branch == 1
    furnace(model, tier, infernal, banked)
    head(model, tier, infernal, banked)
    armor(model, tier, infernal, banked)
    hammer(model, tier, infernal, banked)


def furnace(model, tier, infernal, banked):
    model.loft("Rear furnace housing","Chest","slag",[
        (-.054,.115,.086,-.039),(.036,.174,.122,-.052),(.191,.202,.148,-.043),
        (.293,.164,.118,-.040),(.346,.095,.081,-.018),
    ],n=16)
    radius = .130 if infernal else .092
    model.ell("Black refractory lining","Chest","ink",(0,.168,.076),
              (.148,.136,.098),segments=14,rings=7)
    model.ell("Fire in the chest","Chest","sap" if infernal and tier==2 else "forge",
              (0,.166,.116),(radius,radius*.83,radius*.68),segments=14,rings=7)
    for s in (-1,1):
        model.tube("Forged clavicle bridge","Chest","oldbone",[
            (s*.035,.334,.070),(s*.139,.323,.033),(s*.200,.304,.017),(s*.244,.304,.002),
        ],[.037,.041,.039,.028],n=8)
        for y in (.068,.134,.201,.268):
            model.tube("Charred outer rib","Chest","oldbone",[
                (s*.059,y,-.100),(s*.144,y,-.060),(s*.188,y,.020),(s*.153,y-.005,.117),
                (s*.091,y-.014,.146),
            ],[.024,.028,.026,.022,.018],n=8)
        model.tube("Rib cage outer iron band","Chest","iron",[
            (s*.115,.037,.120),(s*.167,.112,.123),(s*.176,.215,.111),(s*.136,.303,.098),
        ],[.020,.026,.024,.019],n=8)
    if infernal and tier==2:
        for i in range(8):
            a = tau*i/8
            x,y = cos(a),sin(a)
            model.tube("Radiant flame ray","Chest","sap",[
                (x*.090,.166+y*.083,.178),(x*.132,.166+y*.111,.188),
                (x*.193,.166+y*.158,.148),
            ],[.026,.022,.004],n=7)
    elif banked:
        model.plate("Banked-fire furnace door","Chest","furnace_verdigris",[
            (-.122,.101),(.123,.101),(.122,.224),(.088,.253),(-.087,.253),(-.122,.224),
        ],(.154,.184),bevel=.010)
        for y in (.148,.202):
            model.box("Glowing slit in furnace door","Chest","forge",(0,y,.188),(.161,.015,.011),.003)
        model.ell("Round furnace draft dial","Chest","edge",(0,.267,.179),
                  (.041,.039,.014),segments=10,rings=5)
        model.ell("Dark dial face","Chest","slag",(0,.267,.192),
                  (.030,.030,.008),segments=9,rings=4)
        model.tube("Draft dial needle","Chest","forge",[(0,.257,.200),(.013,.281,.200)],
                   [.004,.004],n=5)
        for s in (-1,1):
            model.tube("Furnace side pipe","Chest","furnace_jade" if tier==2 else "furnace_verdigris",[
                (s*.142,.078,.117),(s*.202,.110,.116),(s*.206,.219,.107),(s*.166,.267,.102),
            ],[.026,.026,.023,.018],n=8)
        model.loft("Leather bellows pack","BackSpring","hide",[
            (-.105,.091,.054,-.076),(-.074,.131,.068,-.081),(.052,.126,.074,-.081),
            (.098,.085,.051,-.080),
        ],n=10,fold=.08)
    else:
        for x in (-.056,0,.056):
            model.tube("Grate bar","Chest","iron",[(x,.068,.165),(x,.173,.196),(x,.267,.151)],
                       [.011,.016,.010],n=7)
        if infernal:
            model.tube("Red copper furnace collar","Chest","copper",[
                (-.139,.221,.125),(-.104,.286,.144),(0,.302,.146),(.104,.286,.144),(.139,.221,.125),
            ],[.026,.028,.031,.028,.026],n=9)


def head(model, tier, infernal, banked):
    node = "Head"
    model.loft("Ash-stained skull",node,"bone",[
        (-.104,.062,.048,.032),(-.074,.091,.072,.028),(.001,.118,.105,.012),
        (.072,.118,.096,-.004),(.129,.083,.080,-.013),(.153,.047,.052,-.019),
    ],n=16)
    for s in (-1,1):
        model.ell("Deep eye socket",node,"ink",(s*.056,.031,.103),(.042,.030,.015),segments=11,rings=5)
        model.ell("Banked eye ember",node,"sap" if infernal else "forge",
                  (s*.055,.026,.119),(.018 if infernal else .010,.011,.007),segments=8,rings=4)
        model.tube("Chiseled cheekbone",node,"oldbone",[
            (s*.111,-.011,.056),(s*.093,-.052,.101),(s*.064,-.056,.113),
        ],[.021,.022,.010],n=7)
    model.ell("Hollow nasal cavity",node,"ink",(0,-.022,.117),(.015,.023,.011),segments=8,rings=4)
    model.plate("Soot between the teeth",node,"ink",
                [(-.061,-.082),(.061,-.082),(.063,-.055),(-.063,-.055)],(.097,.119),bevel=.003)
    for x in (-.049,-.030,-.011,.011,.030,.049):
        model.box("Separate skull tooth",node,"bone",(x,-.066,.121-abs(x)*.16),(.014,.032,.018),.002)
    if infernal and tier==2:
        model.loft("Gilt furnace crown band","HeadGear","gold",[
            (.107,.116,.100,-.010),(.124,.132,.111,-.011),(.151,.102,.094,-.014),
        ],n=14)
    count = 5 if infernal and tier==2 else 2 if banked else 3
    for i in range(count):
        x = (i-(count-1)/2)*.047
        y = (.217 if banked else .261)+.037*cos(i*1.54)+(.052 if infernal else 0)
        model.tube("Leaning flame crest","HeadGear","sap" if infernal and tier==2 else "forge",[
            (x,.115,-.023),(x,.162,-.030),(x+.012,y-.046,-.057),(x+.026,y,-.083),
        ],[.031,.036,.017,.002],n=8)
    if banked:
        model.loft("Lower furnace helmet rim","HeadGear","furnace_verdigris",[
            (.087,.117,.101,-.010),(.102,.127,.112,-.012),(.134,.099,.088,-.015),
        ],n=14,opening=.75)


def armor(model, tier, infernal, banked):
    armor = "furnace_verdigris" if banked else "copper" if infernal else "slag"
    edge = "furnace_jade" if banked else "gold" if infernal else "iron"
    model.loft("Cast-iron hip armor","Hips","slag",[
        (-.090,.140,.104,0),(-.044,.163,.123,0),(.041,.157,.112,0),
    ],n=14)
    model.loft("Hinged waist guard","Hips","furnace_verdigris" if banked else "wine",[
        (-.210,.146,.115,0),(-.132,.153,.116,0),(-.041,.143,.106,0),
    ],n=12,opening=.56,fold=.085)
    for s,side in ((-1,"R"),(1,"L")):
        arm,fore,hand = f"UpperArm{side}",f"Forearm{side}",f"Hand{side}"
        model.tube("Scorched upper arm bone",arm,"oldbone",[
            (0,.043,0),(0,-.049,0),(s*.006,-.172,.008),(0,-.259,0),
        ],[.054,.055,.043,.035],n=9)
        model.loft("Raised cast shoulder shell",arm,armor,[
            (-.074,.075,.074,0),(-.042,.107,.104,0),(.028,.099,.098,0),(.065,.058,.061,0),
        ],n=12)
        model.tube("Iron forearm bracer",fore,armor,[(0,.019,0),(0,-.090,.002),(0,-.224,.003)],
                   [.041,.060,.042],n=10)
        model.ell("Hammer grip gauntlet",hand,"slag",(0,-.011,.016),(.050,.057,.047),segments=10)
        for joint,length in ((f"Thigh{side}",.191),(f"Shin{side}",.191)):
            model.tube("Greaved skeleton leg",joint,"iron",[(0,.030,0),(0,-.096,.004),(0,-length,0)],
                       [.063,.072,.051],n=9)
        model.loft("Broad soot-black sabaton",f"Foot{side}","slag",[
            (-.060,.075,.116,.057),(-.042,.082,.118,.055),(.012,.070,.108,.042),(.059,.054,.060,.005),
        ],n=12)
        if tier:
            model.loft("Colored rolled pauldron edge",arm,edge,[
                (-.063,.099,.101,.004),(-.051,.111,.110,.006),(-.035,.104,.102,.006),
            ],n=12)
            model.plate("Heat shield on gauntlet",fore,edge,
                        [(-.039,-.159),(.039,-.159),(.044,-.059),(0,-.031),(-.044,-.059)],
                        (.047,.061),bevel=.005)
        if banked and tier==2:
            lantern(model,arm,s)
        if infernal and tier==2:
            model.tube("Flaring shoulder crest",arm,"forge",[(0,.030,.016),
                       (s*.026,.106,.001),(s*.062,.180,-.023)],[.034,.028,.004],n=8)


def lantern(model, arm, side):
    x = side*.079
    model.tube("Hanging lantern chain",arm,"iron",[(x,-.038,.043),(x+side*.020,-.106,.069),
               (x+side*.022,-.157,.074)],[.010,.011,.009],n=7)
    model.ell("Ember lantern center",arm,"forge",(x+side*.035,-.210,.087),
              (.042,.057,.038),segments=10,rings=5)
    for y in (-.161,-.257):
        model.loft("Cage cap for hanging lantern",arm,"furnace_jade",[
            (y-.011,.042,.040,.087),(y+.010,.046,.041,.087),
        ],n=9,transform=lambda p:(p[0]+x+side*.035,p[1],p[2]))
    for z in (.046,.123):
        model.tube("Lantern cage strap",arm,"iron",[(x,-.143,z),(x+side*.047,-.182,z),
                   (x+side*.034,-.258,z)],[.006,.009,.006],n=6)


def hammer(model, tier, infernal, banked):
    node = "WeaponR"
    model.tube("Ash hammer haft",node,"darkwood",[(0,0,-.111),(0,0,.156),(0,0,.529)],
               [.026,.030,.033],n=9)
    model.tube("Leather hammer binding",node,"hide",[(0,0,-.093),(0,0,.095)],[.033,.035],n=9)
    width = .272 if infernal and tier==2 else .228
    model.box("Cast furnace hammer head",node,
              "furnace_verdigris" if banked else "copper" if infernal else "iron",
              (0,0,.510),(width,.126,.142),.019)
    for s in (-1,1):
        model.box("Forged hammer striking face",node,
                  "furnace_jade" if banked else "brass" if infernal else "iron",
                  (s*width*.45,0,.510),(.037,.135,.153),.009)
        model.box("Inset heated hammer vent",node,"sap" if infernal and tier==2 else "forge",
                  (s*width*.24,.067,.510),(.051,.012,.093),.006)
    if tier:
        model.box("Reinforced hammer collar",node,"brass" if infernal else "slag",
                  (0,0,.401),(.087,.079,.052),.008)
    if infernal:
        model.tube("Copper hammer pick",node,"gold" if tier==2 else "copper",[
            (-width*.36,0,.51),(-width*.59,0,.514),(-width*.82,.021,.537),
            (-width*(1.10 if tier==2 else .93),.022,.552),
        ],[.067,.055,.033,.004],n=8)
    if banked:
        for z in (.435,.579):
            model.box("Hammer cage crossbar",node,"edge",(0,.014,z),(.233,.158,.021),.006)
        if tier==2:
            for x in (-.090,.090):
                model.tube("Wrapped hammer cage rib",node,"furnace_jade",[
                    (x,-.077,.440),(x,-.082,.505),(x,-.080,.578),
                ],[.014,.015,.014],n=7)
