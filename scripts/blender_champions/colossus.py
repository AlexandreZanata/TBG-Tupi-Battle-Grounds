"""Carve the monumental stone god and a single rigid carried obelisk."""

from bloomkeeper import leaf
from completion_people import limbs

IMPACT=(0,1.27,0)
CARRY=(0,.10,-.27)


def colossus(m,tier,branch,*,production=False):
    solar=tier > 0 and branch == 0
    bastion=tier > 0 and branch == 1
    stone="fortress_basalt" if bastion else "star_ceramic" if solar else "crag_limestone"
    shadow="steel_dark" if bastion else "slate"
    trim="brass" if bastion else "gold"
    limbs(m,shadow,stone,arm=.30,leg=.46,width=.084,skin=stone)
    m.loft("Monument torso core","Chest",shadow,[
        (-.062,.122,.092,0),(.067,.161,.116,.006),(.208,.235,.131,.008),
        (.298,.254,.121,0),(.365,.111,.088,0),
    ],n=12)
    m.tube("Stone neck seat","Chest",stone,[(0,.296,0),(0,.420,0)],[.099,.084],n=10)
    m.box("Carved waist plinth","Hips",shadow,(0,.002,0),(.394,.140,.229),bevel=.031)
    m.box("Gilded girdle atop the plinth","Hips",trim,(0,.039,.013),(.422,.062,.244),bevel=.014)
    stone_mask(m,stone,shadow,trim,tier,solar,bastion)
    plating(m,stone,shadow,trim,tier,solar,bastion)
    pillar(m,stone,shadow,trim,tier,solar,bastion)


def stone_mask(m,stone,shadow,trim,tier,solar,bastion):
    h="Head"
    m.loft("Chiseled god face",h,stone,[
        (-.147,.079,.067,.029),(-.092,.114,.090,.027),(.040,.133,.112,.006),
        (.106,.126,.107,-.006),(.142,.093,.081,-.009),
    ],n=10)
    m.box("Recessed mouth cut",h,shadow,(0,-.083,.110),(.125,.023,.024),bevel=.004)
    m.plate("Chiseled nose and chin ridge",h,trim,[(-.023,.046),(.023,.046),(.040,-.055),
            (.022,-.086),(-.027,-.086),(-.041,-.051)],(.110,.161))
    for s in (-1,1):
        m.box("Carved eye socket",h,shadow,(s*.066,.037,.110),(.068,.034,.031),rotation=-s*.17)
        m.box("Amber eyes in the carved sockets",h,"sap",(s*.066,.034,.128),
              (.049,.011,.014),bevel=.003,rotation=-s*.17)
        leaf(m,"Separate temple tablet",h,trim,(s*.099,.038,.056),
             (s*.183,.160,-.034),.047)
        if bastion:
            m.box("Crenellated temple buttress",h,stone,(s*.122,.151,-.007),
                  (.080,.094+.048*(tier==2),.100),bevel=.009)
    if solar:
        leaf(m,"Gilded central sun crest",h,trim,(0,.089,.077),
             (0,.200+.061*(tier==2),-.063),.057,facing=(1,0,0))


def plating(m,stone,shadow,trim,tier,solar,bastion):
    for s,side in ((-1,"R"),(1,"L")):
        upper=f"UpperArm{side}"
        m.ell("Stone shoulder key seated in chest","Chest",shadow,(s*.243,.282,0),
              (.100,.092,.093),segments=10)
        m.plate("Sculpted separate pectoral block","Chest",stone,
                [(s*.023,.292),(s*.174,.311),(s*.226,.237),(s*.164,.114),
                 (s*.075,.147),(s*.022,.204)],(.112,.177),bevel=.013)
        m.box("Plinth-fitted shoulder armor",upper,stone,(s*.028,.024,0),
              (.196+.055*bastion,.105,.200),bevel=.024)
        m.box("Carved forward shin shell",f"Shin{side}",stone,(0,-.107,.058),
              (.139,.164,.087),bevel=.018)
        if solar:
            leaf(m,"Offensive sun shoulder blade",upper,trim,(0,.017,.063),
                 (s*(.129+.038*(tier==2)),.122,-.047),.056)
        if bastion:
            m.box("Stacked rampart block",upper,stone,(s*.033,.091,.014),
                  (.141,.070+.036*(tier==2),.150),bevel=.010)
        leaf(m,"Independent fitted waist guard","Hips",stone,(s*.073,.022,.115),
             (s*.160,-.214,.079),.065)


def pillar(m,stone,shadow,trim,tier,solar,bastion):
    n="WeaponR"
    width=.27 if solar else .23
    m.loft("Single continuous obelisk shaft",n,stone,[
        (-.409,width*.80,width*.69,0),(-.345,width*.77,width*.64,0),
        (.181,width*.70,width*.59,0),(.688,width*.64,width*.55,0),
        (1.158,width*.54,width*.48,0),
    ],n=4)
    m.tube("Pyramidion around native impact tip",n,trim,[(0,1.145,0),IMPACT,(0,1.47,0)],
           [width*.54,width*.41,.002],n=4)
    for y,ratio in ((-.405,1.00),(-.233,.90),(.191,.83),(.610,.76),(1.044,.67)):
        m.box("Separate structural obelisk collar",n,trim,(0,y,0),
              (width*1.61*ratio,.043,width*1.37*ratio),bevel=.009)
        if y<1:
            leaf(m,"Inset gilded monument glyph",n,"sap" if solar else "gold",
                 (0,y+.059,width*.65*ratio),(0,y+.172,width*.62*ratio),.028)
    for s in (-1,1):
        if solar:
            for y in (.072,.481,.878):
                leaf(m,"External gilded obelisk cutting fin",n,trim,(s*.113,y,.013),
                     (s*(.205+.053*(tier==2)),y+.176,-.037),.038)
        if bastion:
            m.box("Long dark monument buttress",n,shadow,(s*.138,.331,0),
                  (.064,.738,.148),bevel=.010)
            if tier==2:
                m.box("Crowned fortress cap",n,trim,(s*.107,1.159,0),
                      (.105,.084,.134),bevel=.010)
