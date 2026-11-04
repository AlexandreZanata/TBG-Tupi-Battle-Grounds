"""Sculpt a countershaded shark and a two-hand trident on the native weapon line."""

from bloomkeeper import leaf

REAR = -.40
SUPPORT = (0,0,.18)
TIP = (0,0,1.12)


def trident(m,tier,branch,*,production=False):
    ocean = tier > 0 and branch == 0
    tide = tier > 0 and branch == 1
    trunk(m,ocean,tide,tier)
    head(m,tide)
    limbs(m,ocean,tide,tier)
    spear(m,ocean,tide,tier)


def trunk(m,ocean,tide,tier):
    m.loft("Countershaded shark torso","Chest","shark_hide",[
        (-.060,.160,.124,-.022),(.025,.203,.153,-.019),(.145,.241,.167,-.015),
        (.275,.264,.156,-.005),(.387,.209,.128,0),(.432,.141,.107,.005),
    ],n=17)
    m.ell("Pale ventral countershade","Chest","sun_cream" if ocean else "shark_belly",(0,.168,.126),
          (.171,.240,.085),segments=16)
    belt = "shark_reef" if tide else "hide"
    m.loft("Wide leather fighting sash","Hips",belt,[(.001,.195,.150,0),(.043,.226,.163,0),
           (.095,.204,.152,0)],n=15,fold=.068)
    m.box("Wide shark sash buckle","Hips","brass",(0,.048,.164),(.089,.074,.034),.012)
    for s in (-1,1):
        m.tube("Hip guard below sash","Hips",belt,[(s*.130,.025,.129),
               (s*.174,-.091,.146),(s*.144,-.174,.098)],[.043,.053,.024],n=8)
    height = .57 if ocean and tier==2 else .44
    leaf(m,"Swept jointed dorsal fin","BackSpring","sun_cream" if ocean and tier==2 else "shark_reef" if tide else "shark_hide",
         (0,-.141,.030),(0,height,-.210),.137,facing=(1,0,0))
    if tide:
        for s in (-1,1):
            m.tube("Bronze reef harness on shoulder","Chest","copper",[(s*.221,.369,-.058),
                   (s*.196,.300,.104),(s*.154,.104,.165)],[.026,.034,.020],n=8)


def head(m,tide):
    h,j = "Head","HeadGear"
    m.loft("Hammered shark head profile",h,"shark_hide",[
        (-.110,.149,.117,.006),(-.053,.190,.181,.048),(.041,.207,.229,.070),
        (.135,.175,.167,.025),(.196,.113,.109,-.026),
    ],n=17)
    m.ell("Projected wedge of shark snout",h,"shark_hide",(0,.006,.215),(.181,.089,.146),segments=15)
    m.ell("Recessed dark mouth opening",h,"ink",(0,-.052,.256),(.153,.034,.101),segments=15)
    m.ell("Hinged pale lower jaw",j,"shark_belly",(0,-.019,.106),(.161,.047,.125),segments=15)
    for x in (-.112,-.071,-.033,.033,.071,.112):
        z = .318-abs(x)*.28
        m.tube("Visible upper shark tooth",h,"bone",[(x,-.041,z),(x,-.062,z+.006),
               (x,-.080,z+.005)],[.017,.015,.002],n=7)
    for s in (-1,1):
        m.ell("Dark predator eye socket",h,"ink",(s*.162,.058,.175),(.048,.037,.043),segments=11)
        m.ell("Amber iris under armored brow",h,"forge" if tide else "sap",(s*.184,.056,.196),
              (.021,.019,.019),segments=9)
        m.tube("Hard shark brow ridge",h,"shark_hide",[(s*.113,.099,.184),(s*.168,.091,.185),
               (s*.199,.063,.146)],[.025,.035,.016],n=8)
        for z in (-.081,-.035,.011):
            m.tube("Sunk gill line",h,"pine",[(s*.177,.051,z-.016),(s*.200,.001,z),
                   (s*.178,-.056,z+.016)],[.009,.013,.006],n=6)


def limbs(m,ocean,tide,tier):
    for side,s in (("L",1),("R",-1)):
        arm,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Heavy shark upper arm",arm,"shark_hide",[(0,.055,0),(s*.014,-.102,0),
               (s*.012,-.244,.011),(0,-.352,0)],[.098,.115,.090,.062],n=10)
        m.tube("Tapered shark forearm",fore,"shark_hide",[(0,.041,0),(s*.015,-.131,.010),
               (0,-.307,0)],[.070,.079,.056],n=9)
        leaf(m,"Trailing forearm fin",fore,"shark_reef" if tide else "shark_hide",
             (s*.053,-.058,-.033),(s*.158,-.231,-.120),.054)
        m.ell("Webbed polearm grip",hand,"shark_hide",(0,-.022,.009),(.066,.064,.054),segments=11)
        m.ell("Grip thumb curled over shaft",hand,"shark_belly",(s*.044,-.003,.042),
              (.028,.040,.027),segments=9)
        for joint in (thigh,shin):
            m.tube("Stout shark leg",joint,"shark_hide",[(0,.042,0),(0,-.095,.008),(0,-.231,0)],
                   [.098,.091,.065],n=9)
        m.ell("Wide leather sea boot",foot,"hide",(0,-.016,.061),(.086,.054,.130),segments=12)
        if ocean or (tide and tier==2):
            m.ell("Reef armor shoulder cap",arm,"sun_cream" if ocean else "copper",
                  (s*.019,-.016,.024),(.135,.085,.118),segments=12)
        if ocean and tier==2:
            m.box("Tapered ivory shin guard",shin,"sun_cream",(0,-.093,.065),(.135,.176,.055),.019)


def spear(m,ocean,tide,tier):
    w = "WeaponR"
    m.tube("Single trident shaft",w,"darkwood",[(0,0,REAR),(0,0,0),SUPPORT,(0,0,.85)],
           [.026,.024,.024,.032],n=11)
    for z in (0,SUPPORT[2]):
        m.tube("Leather anchor grip",w,"hide",[(0,0,z-.046),(0,0,z+.046)],[.037,.037],n=10)
        for u in (-.034,.032):
            m.tube("Grip retaining collar",w,"brass",[(0,0,z+u-.006),(0,0,z+u+.006)],
                   [.041,.041],n=9)
    width = .237 if tide else .174
    m.tube("Cast fork crossbar",w,"copper" if tide else "brass",[(-width,0,.833),
           (-width*.54,0,.813),(0,0,.785),(width*.54,0,.813),(width,0,.833)],
           [.032,.038,.047,.038,.032],n=9)
    for s in (-1,0,1):
        x = s*width
        tip = TIP if s==0 else (x-s*.035,0,1.073)
        leaf(m,"Forged trident blade",w,"sun_cream" if ocean else "horn_stone" if tide else "edge",
             (x,0,.825),tip,.054 if s==0 else .043,facing=(0,1,0))
        if tide and tier==2 and s:
            leaf(m,"Reverse hook on outer tine",w,"brass",(x,0,.879),(s*(width+.065),0,.732),
                 .033,facing=(0,1,0))
    if ocean and tier==2:
        m.tube("Amber spearhead coupling",w,"sap",[(0,0,.688),(0,0,.718)],[.038,.038],n=10)
