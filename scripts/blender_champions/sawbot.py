"""Build the Sawbot's piston limbs, boiler and hand-driven saws."""

from math import cos, sin, tau


def sawbot(model, tier, branch, *, production=False):
    overclock = tier > 0 and branch == 0
    armored = tier > 0 and branch == 1
    heavy = armored and tier == 2
    casing = "copper" if overclock else "steel_dark" if armored else "iron"
    boiler(model, tier, casing, overclock, armored)
    optics(model, casing, heavy)
    limbs(model, casing, armored, heavy)
    saw(model, "HandR", .22 if heavy else .16, overclock)
    if overclock and tier == 2:
        saw(model, "HandL", .13, True)
    else:
        clamp(model)


def boiler(model, tier, casing, overclock, armored):
    model.loft("Riveted boiler casing", "Chest", casing, [
        (-.02,.084,.084,0),(.047,.124,.107,0),(.178,.154,.130,0),
        (.259,.153,.125,0),(.303,.105,.104,0),(.338,.071,.074,0),
    ], n=12)
    for y,rx,rz in ((.077,.138,.116),(.261,.155,.130)):
        model.loft("Boiler compression ring", "Chest", "brass",
                   [(y-.012,rx-.006,rz-.005,0),(y-.006,rx,rz,0),
                    (y+.008,rx,rz,0),(y+.012,rx-.005,rz-.005,0)], n=12)
    model.plate("Radiator recess", "Chest", "slag",
                [(-.078,.116),(.078,.116),(.086,.223),(.062,.250),(-.062,.250),(-.086,.223)],
                (.113,.142), bevel=.009)
    for y in (.145,.181,.218):
        model.box("Radiator throat", "Chest", "forge" if overclock else "copper",
                  (0,y,.147),(.101,.014,.008),bevel=.003)
    model.loft("Waist coupling", "Hips", "slag",
               [(-.065,.102,.079,0),(-.018,.126,.095,0),(.054,.123,.095,0)], n=10)
    if armored:
        for side in (-1,1):
            outline = [(side*x,y) for x,y in ((.078,.080),(.167,.114),(.180,.232),(.104,.280))]
            model.plate("Bolt-on flank armor", "Chest", "steel",
                        outline if side > 0 else outline[::-1], (.059,.101), bevel=.010)
    for x in ((-.091,.091) if overclock and tier == 2 else (.078,)):
        model.tube("Curved exhaust", "BackSpring", "steel_dark", [
            (x,-.062,-.039),(x,.066,-.083),(x,.185,-.096),(x,.255,-.127),
        ], [.031,.038,.040,.036], n=8)
        model.tube("Rolled exhaust lip", "BackSpring", "brass",
                   [(x,.233,-.117),(x,.255,-.127)], [.044,.044], n=8)
        model.ell("Exhaust core", "BackSpring", "forge" if overclock else "slag",
                  (x,.258,-.129),(.028,.013,.027),segments=8)


def optics(model, casing, heavy):
    model.loft("Cast cyclops helmet", "Head", casing, [
        (-.102,.063,.071,0),(-.074,.114,.088,0),(.045,.128,.112,0),
        (.091,.105,.106,-.004),(.126,.076,.078,-.010),(.138,.046,.049,-.015),
    ], n=12)
    model.tube("Brass optical housing", "Head", "brass",
               [(0,.012,.091),(0,.012,.133),(0,.012,.145)], [.071,.074,.064], n=12)
    model.ell("Dark lens gasket", "Head", "slag", (0,.012,.145),(.053,.054,.019),segments=10)
    model.ell("Furnace eye", "Head", "forge", (0,.012,.161),(.044,.045,.023),segments=12)
    model.plate("Overhanging brow", "Head", "steel_dark",
                [(-.104,.062),(.108,.062),(.084,.104),(-.078,.104)], (.084,.150), bevel=.010)
    model.tube("Signal mast", "Head", "brass", [
        (-.057,.092,-.039),(-.087,.158,-.044),(-.075,.206,-.049),
    ], [.011,.009,.007], n=6)
    model.ell("Mast ember", "Head", "forge", (-.075,.214,-.049),(.014,.017,.014),segments=8)
    if heavy:
        for side in (-1,1):
            model.tube("Forged helmet horn", "Head", "steel", [
                (side*.094,.078,-.004),(side*.145,.115,-.016),
                (side*.170,.172,-.032),(side*.172,.205,-.049),
            ], [.041,.029,.016,.002], n=7)


def limbs(model, casing, armored, heavy):
    for side, sign in (("L",1),("R",-1)):
        shoulder = f"UpperArm{side}"
        lower = f"Forearm{side}"
        model.tube("Shoulder axle", "Chest", "steel_dark", [
            (sign*.105,.281,0),(sign*.167,.306,0),(sign*.226,.306,0),
        ], [.046,.043,.036], n=8)
        model.ell("Shoulder bearing", shoulder, casing, (0,-.015,0),
                  (.093 if heavy else .068,.071 if armored else .057,.071),segments=10)
        model.tube("Upper arm piston", shoulder, "edge", [
            (0,-.041,.004),(0,-.132,.004),(0,-.193,.004),
        ], [.036,.028,.032], n=7)
        model.loft("Forearm actuator casing", lower, casing, [
            (-.149,.042,.046,0),(-.102,.063,.063,0),(-.029,.058,.059,0),(.027,.042,.043,0),
        ], n=9)
        for joint,length in ((f"Thigh{side}",.183),(f"Shin{side}",.190)):
            model.tube("Pinned leg strut", joint, casing,
                       [(0,.021,0),(0,-.091,0),(0,-length,0)], [.047,.046,.038], n=8)
            model.ell("Pinned leg bearing", joint, "brass", (0,-.031,.023),
                      (.040,.035,.028), segments=8)
        model.box("Split iron tread", f"Foot{side}", "steel_dark",
                  (0,-.022,.049),(.106,.087,.160),bevel=.018)
        if armored:
            model.plate("Forward shin guard", f"Shin{side}", "steel",
                        [(-.036,-.149),(.036,-.149),(.051,-.018),(-.051,-.018)], (.026,.064), bevel=.008)


def saw(model, joint, radius, hot):
    model.tube("Blade spindle", joint, "brass", [(0,-.013,0),(0,-.125,0),(0,-.147,0)],
               [.039,.034,.036], n=10)
    perimeter = []
    for tooth in range(14):
        for step, scale in ((0,.81),(.46,1),(.70,.86)):
            angle = (tooth + step) * tau / 14
            perimeter.append((radius * scale * cos(angle), radius * scale * sin(angle)))
    count = len(perimeter)
    vertices = [(x,y,z) for y in (-.139,-.121) for x,z in perimeter]
    faces = [tuple(range(count)),tuple(reversed(range(count,2*count)))]
    faces += [(i,i+count,(i+1)%count+count,(i+1)%count) for i in range(count)]
    model.mesh("Directional carbide teeth and disk", joint, "edge", vertices, faces, smooth=False)
    model.tube("Stamped blade hub", joint, "steel_dark", [(0,-.150,0),(0,-.110,0)],
               [radius*.39]*2, n=10)
    for angle in (0,tau/3,2*tau/3):
        for y in (-.143,-.117):
            model.ell("Visible rotation witness", joint, "forge" if hot else "brass",
                      (radius*.59*cos(angle),y,radius*.59*sin(angle)),
                      (.014,.007,.014),segments=8)


def clamp(model):
    model.box("Clamp palm", "HandL", "iron", (0,-.037,0),(.096,.089,.090),bevel=.010)
    for side in (-1,1):
        model.tube("Forged clamp jaw", "HandL", "brass", [
            (side*.036,-.041,0),(side*.055,-.090,0),
            (side*.051,-.128,.010),(side*.024,-.151,.014),
        ], [.024,.022,.017,.008], n=6)
