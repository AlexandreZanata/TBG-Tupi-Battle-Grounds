"""Tailor the Hexer's crooked hat, ragged mantle and ritual gear."""

from math import cos, sin, tau

from characters import BONE_FACE, CAPES, face_transform


def hexer(model, tier, branch, *, production=False):
    doom = tier > 0 and branch == 0
    plague = tier > 0 and branch == 1
    glow = "soul" if plague else "ritual"
    robe(model, tier, plague)
    head(model, tier, doom, plague, glow)
    staff(model, tier, doom, glow)
    if plague:
        censer(model, tier)


def robe(model, tier, plague):
    cloth = "pine" if plague else "violet"
    lining = "moss" if plague else "violet_dark"
    model.loft("Fitted ritual coat", "Chest", cloth, [
        (-.039,.105,.083,0), (.075,.120,.100,0), (.202,.155,.110,-.008),
        (.278,.150,.098,-.015), (.320,.092,.071,-.012), (.360,.061,.053,0),
    ], n=13, fold=.06)
    model.loft("Worn robe hem", "Hips", cloth, [
        (-.328,.177,.119,-.017), (-.288,.186,.129,-.014),
        (-.161,.162,.117,-.005), (.009,.122,.096,0), (.043,.109,.088,0),
    ], n=14, opening=.25, fold=.17,
       transform=lambda p:(p[0],p[1]+.032*cos(p[0]*37)*max(0,-p[1])/.34,p[2]))
    model.loft("Twisted rope belt", "Hips", "hide", [(.016,.119,.094,0),(.045,.115,.088,0)], n=12)
    model.tube("Ritual coat edge", "Chest", lining, [
        (-.075,.046,.076), (-.062,.157,.100), (-.028,.236,.102), (.030,.307,.078),
    ], [.011,.013,.014,.012], n=6)
    model.loft("High tattered mantle", "BackSpring", lining, CAPES, n=12,
               opening=1.16, fold=.17,
               transform=lambda p:(p[0]*(.80+.10*tier),p[1]*.95,p[2]-.006))
    for side, sign in (("L",1),("R",-1)):
        model.tube("Mantle shoulder bridge", "Chest", lining, [
            (sign*.107,.257,-.007),(sign*.165,.291,-.009),(sign*.231,.290,-.009),
        ], [.047,.057,.042])
        model.loft("Drooping sleeve", f"UpperArm{side}", cloth, [
            (-.179,.054,.047,0), (-.084,.062,.056,-.007), (.016,.051,.050,0),
        ], n=10, fold=.08)
        model.loft("Ritual sleeve cuff", f"Forearm{side}", lining, [
            (-.122,.046,.045,0), (-.078,.053,.053,0), (.018,.044,.047,0),
        ], n=10)
        hand = f"Hand{side}"
        model.ell("Ancient knuckles", hand, "oldbone", (0,-.027,.011), (.042,.055,.037), segments=9)
        for x in (-.023,0,.023):
            model.tube("Bent claw", hand, "bone", [
                (x,-.045,.024),(x,-.087,.036),(x*.79,-.100,.056 if side=="L" else .031),
            ], [.011,.008,.003], n=5)
        model.tube("Exposed shin", f"Shin{side}", "oldbone",
                   [(0,-.02,0),(0,-.15,0),(0,-.191,.005)],[.039,.029,.042], n=7)
        model.tube("Femur under the robe", f"Thigh{side}", "violet_dark" if not plague else "pine",
                   [(0,0,0),(0,-.115,0),(0,-.191,0)],[.035,.029,.037], n=7)
        model.ell("Worn split boot", f"Foot{side}", "hide", (0,-.018,.042), (.049,.048,.083), segments=9)


def head(model, tier, doom, plague, glow):
    model.loft("Hollow-cheek skull", "Head", "bone", BONE_FACE, n=18,
               transform=lambda p:face_transform(p,True))
    for side in (-1,1):
        x = side*.053
        model.ell("Black eye socket", "Head", "ink", (x,.032,.091), (.036,.028,.017), segments=10)
        model.ell("Ritual pupil", "Head", glow, (x,.031,.104), (.014,.011,.009), segments=8)
    model.ell("Nose cavity", "Head", "ink", (0,-.023,.103), (.015,.022,.010), segments=8)
    for x in (-.040,-.020,0,.020,.040):
        model.ell("Broken ivory tooth", "Head", "oldbone", (x,-.088,.103-abs(x)*.25),
                  (.009,.016,.010), segments=7)
    hat(model, tier, doom, plague)
    if plague and tier == 2:
        model.mesh("Hammered plague beak", "Head", "linen", [
            (-.061,.014,.100),(0,.034,.112),(.061,.014,.100),(.048,-.070,.108),
            (-.048,-.070,.108),(0,-.053,.252),
        ], [(0,1,2,3,4),(1,0,5),(2,1,5),(3,2,5),(4,3,5),(0,4,5)], smooth=False)
        for s in (-1,1):
            model.tube("Beak stitching", "Head", "leather",
                       [(s*.051,-.019,.115),(s*.033,-.034,.171),(s*.008,-.050,.224)],
                       [.004]*3, n=5)


def hat(model, tier, doom, plague):
    shell = "pine" if plague else "violet_dark"
    model.loft("Wavy broad hat brim", "Head", shell, [
        (.079,.123,.113,0), (.071,.221,.182,-.008),
        (.089,.224,.184,-.008), (.111,.130,.124,0),
    ], n=16, transform=lambda p:(p[0],p[1]+.018*cos(p[0]*16)+.022*p[2],p[2]))
    model.loft("Bent crown of the hat", "HeadGear", shell, [
        (.087,.129,.123,-.008), (.131,.128,.120,-.012), (.218,.092,.096,-.034),
        (.285,.063,.064,-.076), (.323,.042,.042,-.120), (.318,.020,.025,-.174),
        (.283,.005,.008,-.196),
    ], n=14, fold=.09, transform=lambda p:(p[0]+.36*max(0,p[1]-.157),p[1],p[2]))
    model.loft("Twisted hat binding", "Head", "brass" if doom else "hide", [
        (.108,.132,.127,-.008), (.137,.127,.122,-.013),
    ], n=14)
    if doom and tier == 2:
        for s in (-1,0,1):
            x = .091*s
            model.tube("Doom crown thorn", "Head", "gold", [
                (x,.116,.106-abs(x)*.35),(x*1.22,.167,.114-abs(x)*.31),
                (x*1.46,.242-abs(x)*.37,.096-abs(x)*.22),
            ], [.017,.013,.002], n=6)


def staff(model, tier, doom, glow):
    grip = "WeaponR"
    model.tube("Gnarled blackthorn staff", grip, "darkwood", [
        (-.024,-.230,.021),(0,-.144,.032),(-.012,.105,.030),(.028,.38,.030),
        (-.018,.590,.030),(.004,.734,.022),(0,.800,.030),
    ], [.020,.022,.021,.024,.020,.028,.021], n=8)
    model.tube("Leather staff grip", grip, "leather", [
        (-.003,-.070,.030),(-.008,.015,.030),(-.010,.082,.030),
    ], [.024,.027,.024], n=9)
    for side in (-1,1):
        model.tube("Forked lantern cradle", grip, "wood", [
            (0,.685,.025),(side*.049,.766,.029),(side*.073,.836,.044),(side*.058,.896,.030),
        ], [.023,.018,.012,.005], n=6)
    scale = 1.21 if doom and tier==2 else 1
    model.ell("Ivory lantern cranium", grip, "bone", (0,.877,.042), (.070*scale,.071*scale,.054*scale), segments=12)
    model.ell("Lantern jaw", grip, "oldbone", (0,.842,.055), (.052*scale,.032,.040), segments=9)
    model.ell("Cast muzzle in lantern mouth", grip, "ink", (0,.880,.090), (.030,.013,.015), segments=9)
    for side in (-1,1):
        x = side*.028*scale
        model.ell("Lantern eye recess", grip, "ink", (x,.897,.091), (.020,.017,.013), segments=8)
        model.ell("Lantern flame", grip, glow, (x,.898,.103), (.012,.011,.010), segments=8)
    if doom:
        radius = .115 if tier == 1 else .145
        for shift in (0,tau/2):
            arc = [(radius*sin(shift+i*tau/24),.880+radius*cos(shift+i*tau/24),.047) for i in range(11)]
            model.tube("Broken halo of the hex", grip, glow, arc, [.006]*11, n=5)


def censer(model, tier):
    radius = .056 if tier == 1 else .078
    joint = "OffhandL"
    model.tube("Censer chain", joint, "brass", [(0,0,.025),(0,-.094,.036),(0,-.171,.040)],
               [.006,.007,.006], n=6)
    model.ell("Live censer ember", joint, "soul", (0,-.215,.040), (.70*radius,)*3, segments=10)
    model.loft("Censer base cup", joint, "iron", [
        (-.215-radius,.34*radius,.34*radius,.04),(-.252,radius,radius,.04),
        (-.224,.99*radius,.99*radius,.04),
    ], n=10)
    for i in range(5):
        a = i*tau/5
        x,z = radius*sin(a),radius*cos(a)
        model.tube("Censer cage strut", joint, "brass", [
            (x,-.232,.04+z),(x*.81,-.184,.04+z*.81),(0,-.162,.04),
        ], [.008,.008,.005], n=6)
