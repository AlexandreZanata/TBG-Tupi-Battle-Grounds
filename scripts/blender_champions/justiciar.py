"""Build a segmented judgment construct with a moving core and palm apertures."""

from math import cos, sin, tau

from bloomkeeper import leaf

MUZZLE = (0,0,.10)


def justiciar(m,tier,branch,*,production=False):
    fury = tier > 0 and branch == 0
    judgment = tier > 0 and branch == 1
    slab = "judge_flame" if fury else "judge_obsidian" if judgment else "judge_stone"
    energy = "forge" if fury else "ritual" if judgment else "sap"
    shell(m,slab,energy,fury,judgment,tier)
    face(m,slab,energy,fury,tier)
    for side,s in (("L",1),("R",-1)):
        arms(m,side,s,slab,energy,fury,judgment)
        legs(m,side,slab)


def shell(m,slab,energy,fury,judgment,tier):
    c = "Chest"
    m.loft("Dark inner torso stone","Chest","ink",[
        (-.075,.153,.107,-.013),(.031,.175,.132,-.010),(.180,.234,.142,0),
        (.298,.242,.127,-.005),(.362,.130,.112,-.020),
    ],n=14)
    m.box("Segmented hip hinge","Hips","steel_dark",(0,.010,0),(.309,.124,.215),.019)
    for s in (-1,1):
        x = s*.127
        m.box("Independent stone chest slab",c,slab,(x,.177,.113),(.181,.267,.111),.029,rotation=s*-.17)
        m.tube("Metal rail embedded along slab",c,"copper" if fury else "iron",[
            (s*.139,.074,.138),(s*.197,.154,.143),(s*.213,.278,.121),
        ],[.016,.022,.016],n=8)
        if fury:
            leaf(m,"Hot vent in the upper chest slab",c,"forge",(s*.094,.215,.167),
                 (s*.196,.294,.149),.022)
        if judgment:
            leaf(m,"Prismatic vent in upper chest slab",c,"ritual",(s*.118,.207,.160),
                 (s*.187,.267,.154),.021)
            if tier==2:
                leaf(m,"High obsidian shoulder obelisk",c,"judge_obsidian",(s*.220,.222,.032),
                     (s*.389,.514,-.007),.087)
                leaf(m,"Bright ridge of shoulder obelisk",c,"archer_amethyst",(s*.248,.293,.066),
                     (s*.381,.479,.023),.038)
    core(m,slab,energy,judgment)


def core(m,slab,energy,judgment):
    r = "BackSpring"
    m.tube("Moving chest core socket",r,"slag",[(0,0,-.046),(0,0,.004)],[.093,.084],n=14)
    m.rim("Moving judgment core retaining rim",r,"brass" if not judgment else "archer_amethyst",
          (0,0,.022),(.076,.076),.017,segments=18)
    m.ell("Independent luminous chest core",r,energy,(0,0,.029),(.048,.048,.025),segments=12)
    for i in range(6):
        a = tau*i/6
        leaf(m,"Cast stone detent of the core",r,slab,(.056*sin(a),.056*cos(a),.036),
             (.110*sin(a),.110*cos(a),.022),.019)


def face(m,slab,energy,fury,tier):
    h = "Head"
    m.loft("Chiseled construct head",h,slab,[
        (-.110,.079,.078,.018),(-.064,.118,.108,.005),(.066,.118,.103,0),
        (.127,.093,.086,-.012),(.142,.064,.071,-.019),
    ],n=8)
    m.box("Recessed dark visor",h,"ink",(0,.030,.107),(.159,.053,.028),.007)
    for s in (-1,1):
        m.box("Recessed furnace eye",h,energy,(s*.044,.032,.124),(.045,.018,.010),.004)
        m.tube("Carved angular jaw edge",h,"iron",[(s*.100,.008,.064),
               (s*.085,-.076,.081),(s*.034,-.097,.094)],[.015,.023,.013],n=7)
    if fury and tier==2:
        m.rim("Open-centered celestial fury crown","HeadGear","brass",
              (0,.061,-.107),(.232,.241),.023)
        for s in (-1,1):
            leaf(m,"Crown's flame-shaped bronze fin","HeadGear","judge_flame",(s*.136,.172,-.097),
                 (s*.224,.327,-.136),.050)


def arms(m,side,s,slab,energy,fury,judgment):
    arm,fore,hand = (f"{p}{side}" for p in ("UpperArm","Forearm","Hand"))
    m.tube("Construct upper hinge rod",arm,"steel_dark",[(0,.028,0),(0,-.223,0)],[.048,.048],n=9)
    m.box("Chiseled shoulder guard",arm,slab,(s*.034,-.021,.002),(.170,.119,.179),.022,rotation=s*.19)
    if fury:
        leaf(m,"Bronze flame pauldron flange",arm,"copper",(s*.073,.007,.043),
             (s*.168,.122,-.027),.065)
    if judgment:
        leaf(m,"Forked obsidian gauntlet projection",fore,"judge_obsidian",(s*.043,-.162,.051),
             (s*.142,-.024,.034),.047)
        leaf(m,"Crystal edge of the judgment gauntlet",fore,"ritual",(s*.070,-.144,.073),
             (s*.131,-.040,.068),.020)
    m.tube("Visible gauntlet pivot",fore,"iron",[(0,.024,0),(0,-.206,0)],[.048,.043],n=9)
    for y in (-.026,-.095,-.161):
        m.box("Independent layered gauntlet plate",fore,slab,(0,y,.007),
              (.142,.060,.138),.013)
        m.box("Forged bevel band above plate",fore,"brass",(0,y+.023,.023),(.145,.021,.111),.005)
    m.box("Closed stone palm around emitter",hand,"steel_dark",(0,-.016,.028),
          (.115,.113,.106),.019)
    m.rim("Recessed cast ring around palm aperture",hand,"brass",(0,0,.089),
          (.034,.034),.010,segments=16)
    m.ell("Palm light at native release plane",hand,energy,(0,0,.084),(.025,.025,MUZZLE[2]-.084),segments=11)


def legs(m,side,slab):
    thigh,shin,foot = (f"{p}{side}" for p in ("Thigh","Shin","Foot"))
    for node in (thigh,shin):
        m.tube("Exposed leg hinge",node,"steel_dark",[(0,.032,0),(0,-.193,0)],[.052,.051],n=8)
        m.box("Stone leg armor plate",node,slab,(0,-.079,.023),(.143,.159,.132),.019)
    m.box("Construct plated foot",foot,"slag",(0,-.020,.040),(.148,.091,.193),.018)
    m.box("Chiseled toe cap",foot,slab,(0,-.007,.094),(.135,.071,.106),.013)
