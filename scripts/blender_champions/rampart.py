"""Build the Rampart's oak bulwark, plate armor and striking gauntlet."""

from siegewalker import ogre_face

WALL_X = .46 * 1.1 * .35
WALL_Y = .34 * .30
WALL_Z = .24 * .90


def rampart(model, tier, branch, *, production=False):
    bastion = tier > 0 and branch == 0
    spikes = tier > 0 and branch == 1
    ram = spikes and tier == 2
    armor = "steel" if bastion else "steel_dark" if spikes else "iron"
    trim = "gold" if bastion and tier==2 else "copper"
    harness(model, armor, trim)
    headgear(model, tier, bastion, spikes)
    limbs(model, armor, trim, ram)
    wall(model, tier, bastion, spikes)


def harness(model, armor, trim):
    model.loft("Massive under-armor gambeson","Chest","hide",[
        (-.049,.178,.110,0),(.063,.235,.151,0),(.199,.257,.146,0),
        (.285,.247,.126,0),(.335,.154,.109,0),
    ],n=13)
    model.loft("Heavy overlapping breastplate","Chest",armor,[
        (.037,.211,.132,0),(.091,.242,.150,0),(.210,.260,.158,0),
        (.274,.247,.140,0),(.304,.209,.125,0),
    ],n=14)
    model.loft("Overlapping leather fauld","Hips","hide",[
        (-.157,.219,.134,0),(-.113,.226,.147,0),(.016,.195,.133,0),(.067,.184,.122,0),
    ],n=12)
    model.loft("Riveted waist strap","Hips","leather",[(.023,.208,.142,0),(.067,.204,.140,0)],n=12)
    for s in (-1,1):
        x=s*.124
        model.box("Hanging thigh guard","Hips",armor,(x,-.061,.134),(.116,.161,.046),
                  bevel=.012,rotation=s*.13)
        model.ell("Strap rivet","Hips",trim,(x,.035,.157),(.016,.016,.010),segments=8)
    model.tube("Wall support hook","BackSpring","steel_dark",
               [(.115,-.087,-.060),(.130,.077,-.064),(.107,.219,-.069),(.084,.246,-.033)],
               [.033,.035,.031,.022],n=8)


def headgear(model, tier, bastion, spikes):
    ogre_face(model,"ogre_shade","ink")
    model.loft("Hammered kettle helmet","Head","steel" if bastion else "steel_dark",[
        (.050,.126,.117,-.009),(.058,.181,.155,-.011),(.078,.184,.157,-.011),
        (.103,.140,.138,-.013),(.147,.115,.116,-.021),(.172,.049,.057,-.021),
    ],n=15)
    for s in (-1,1):
        model.tube("Segmented cheek guard","Head","iron",[
            (s*.125,.069,.065),(s*.136,-.015,.070),(s*.114,-.073,.078),
        ], [.032,.033,.022], n=7)
    if bastion and tier==2:
        model.box("Crest set into the helm","Head","gold",(0,.159,-.013),(.053,.064,.123),bevel=.010)
    if spikes:
        for s in (-1,1):
            model.tube("Raked helmet stud","Head","edge",[
                (s*.128,.087,-.024),(s*.159,.144,-.049),(s*.163,.176,-.063),
            ], [.025,.020,.003],n=7)


def limbs(model, armor, trim, ram):
    for side,s in (("L",1),("R",-1)):
        upper,fore,hand = (f"{name}{side}" for name in ("UpperArm","Forearm","Hand"))
        model.tube("Axle under shoulder plate","Chest","ogre",[
            (s*.150,.271,0),(s*.225,.298,0),(s*.302,.305,0),
        ], [.066,.069,.058],n=8)
        model.loft("Ogre upper arm",upper,"ogre",[
            (-.177,.060,.065,0),(-.087,.087,.086,0),(.023,.073,.071,0),
        ],n=10)
        model.ell("Rolling shoulder plate",upper,armor,
                  (s*.013,.009,-.005),(.118,.086,.109),segments=12)
        model.tube("Pauldron rim",upper,trim,[
            (-s*.065,.019,.090),(s*.004,.032,.105),(s*.079,.018,.080),
        ], [.009,.014,.009],n=7)
        model.loft("Riveted forearm guard",fore,armor,[
            (-.150,.057,.062,0),(-.108,.074,.077,0),(-.026,.078,.075,0),(.016,.064,.065,0),
        ],n=10)
        if side=="L":
            model.ell("Gloved wall grip",hand,"hide",(0,-.036,.016),(.066,.065,.058),segments=11)
        for joint,color in ((f"Thigh{side}","hide"),(f"Shin{side}",armor)):
            model.loft("Layered lower armor",joint,color,[
                (-.190,.063,.070,0),(-.095,.078,.080,0),(.029,.085,.089,0),
            ],n=10)
        model.box("Iron ground plate",f"Foot{side}","steel_dark",(0,-.025,.051),
                  (.164,.087,.198),bevel=.020)
    model.loft("Hammer fist","HandR","steel_dark",[
        (-.150,.102,.098,0),(-.120,.105,.100,0),(-.043,.090,.088,0),(.034,.066,.070,0),
    ],n=12)
    model.loft("Knuckle contact plate","HandR","steel",[
        (-.151,.090,.087,0),(-.140,.106,.102,0),(-.115,.106,.102,0),
    ],n=12)
    if ram:
        model.loft("Battering ram forearm","ForearmR","iron",[
            (-.201,.105,.101,0),(-.156,.119,.114,0),(-.041,.095,.095,0),
        ],n=12)
        model.loft("Heated ram collar","ForearmR","forge",[
            (-.147,.121,.118,0),(-.128,.121,.118,0),
        ],n=12)


def wall(model, tier, bastion, spikes):
    # The kit targets this chest-local grip with the left hand.
    node = "Chest"
    wide, tall = (.56,.70) if bastion else (.50,.62)
    x,y,z = WALL_X,WALL_Y,WALL_Z
    for i in range(4):
        cx = x + (i-1.5)*wide/4
        model.box("Tongue-and-groove oak plank",node,"wood" if i%2 else "darkwood",
                  (cx,y,z+.069),(wide/4-.004,tall,.055),bevel=.012)
    for s in (-1,1):
        model.box("Riveted crossbeam",node,"steel_dark",(x,y+s*tall*.34,z+.098),
                  (wide+.024,.067,.050),bevel=.012)
        for side in (-1,1):
            model.ell("Square-head beam rivet",node,"brass",
                      (x+side*wide*.408,y+s*tall*.34,z+.127),
                      (.020,.019,.012),segments=8)
        if bastion:
            model.box("Reinforced wall edge",node,"gold" if tier==2 else "iron",
                      (x+s*wide*.485,y,z+.086),(.042,tall+.032,.059),bevel=.009)
    model.tube("Left-hand rail",node,"hide",[(x,y-.081,z+.043),(x,y-.054,z),
               (x,y+.049,z),(x,y+.077,z+.040)],[.025]*4,n=8)
    if bastion and tier==2:
        for k in (-.375,-.125,.125,.375):
            model.box("Crenellated stone cap",node,"slate",(x+wide*k,y+tall*.52,z+.071),
                      (.106,.098,.089),bevel=.012)
    if spikes:
        for dx in (-.146,.146):
            for dy in (-.113,.113):
                model.tube("Forged wall spike",node,"edge",[
                    (x+dx,y+dy,z+.099),(x+dx,y+dy,z+.135),(x+dx*1.05,y+dy,z+.212),
                ],[.039,.027,.003],n=7)
