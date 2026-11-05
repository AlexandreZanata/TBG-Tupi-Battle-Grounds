"""Build a six-legged predator with independent joints for both scythes."""

from bloomkeeper import leaf

STRIKE = (-.07,-.03,.43)
CHAINS = (
    ("UpperArmL","ForearmL","HandL",1,.08,True),
    ("UpperArmR","ForearmR","HandR",-1,.08,True),
    ("ThighL","ShinL","FootL",1,0,False),
    ("ThighR","ShinR","FootR",-1,0,False),
    ("WeaponR","Back","StringBottom",1,-.08,False),
    ("OffhandL","StringTop","Nock",-1,-.08,False),
)


def ravager(m,tier,branch,*,production=False):
    hunter = tier > 0 and branch == 0
    frenzy = tier > 0 and branch == 1
    shell = "violet_dark" if frenzy else "ravager_shell"
    accent = "bone" if hunter else "ravager_rust" if frenzy else "oldbone"
    body(m,shell,accent,hunter,frenzy,tier)
    mask(m,shell,accent,hunter,frenzy,tier)
    for chain in CHAINS:
        limb(m,chain,shell,accent,hunter,frenzy,tier)


def body(m,shell,accent,hunter,frenzy,tier):
    m.ell("Low chitin thorax","Chest",shell,(0,.011,.030),(.259,.156,.292),segments=16)
    m.ell("Segmented predator abdomen","Hips",shell,(0,.033,-.143),(.249,.158,.326),segments=16)
    for i in range(4):
        z = .120-i*.114
        y = .125+.018*i
        radius = .213-.013*i
        m.tube("Raised rib across armored back","Spine",accent,[
            (-radius,.052,z),(-radius*.69,y,z+.010),(0,y+.026,z+.026),
            (radius*.69,y,z+.010),(radius,.052,z),
        ],[.026,.033,.039,.033,.026],n=8)
        if i<3:
            for s in (-1,1):
                leaf(m,"Serrated chitin shingle","Spine",accent,(s*.143,y,z-.03),
                     (s*(.286+.037*frenzy),y+.051,z-.094),.048+.016*hunter)
    reach = -.68 if hunter and tier==2 else -.51
    m.tube("Raised jointed abdomen stinger","Hips",accent,[(0,.121,-.324),
           (0,.209,-.399),(.019,.246,-.470),(.013,.198,reach)],[.085,.079,.046,.004],n=10)
    if frenzy and tier==2:
        for s in (-1,1):
            m.tube("Hooked frenzy exhaust fin","Spine","ravager_rust",[(s*.113,.142,-.167),
                   (s*.191,.245,-.217),(s*.228,.315,-.354)],[.062,.060,.004],n=9)


def mask(m,shell,accent,hunter,frenzy,tier):
    h = "Head"
    m.ell("Broad predatory chitin mask",h,shell,(0,.013,.031),(.193,.138,.160),segments=15)
    m.ell("Recessed black mouth",h,"ink",(0,-.041,.158),(.107,.059,.034),segments=12)
    for s in (-1,1):
        m.ell("Angled eye cavity",h,"ink",(s*.108,.038,.135),(.064,.032,.040),segments=10)
        m.ell("Predator eye",h,"forge" if frenzy else "soul",(s*.112,.039,.164),
              (.036,.015,.019),segments=9)
        m.tube("Swept armored brow",h,accent,[(s*.026,.059,.149),(s*.116,.075,.151),
               (s*.172,.100,.112),(s*.213,.158,.047)],[.025,.036,.029,.004],n=8)
        m.tube("Independent-looking cheek mandible",h,accent,[(s*.151,-.045,.070),
               (s*.144,-.062,.167),(s*.072,-.072,.224)],[.044,.033,.005],n=8)
        if hunter:
            m.tube("Ivory hunting cheek blade",h,"bone",[(s*.141,-.023,.065),
                   (s*.211,-.041,.151),(s*.214,-.075,.261)],[.032,.042,.004],n=8)
        if frenzy:
            m.tube("Flared frenzy cheek fin",h,"ravager_rust",[(s*.13,.010,.058),
                   (s*.216,.033,.091),(s*(.297 if tier==2 else .251),.114,.041)],
                   [.045,.052,.004],n=8)
        if hunter and tier==2:
            m.tube("Crest feeler",h,"bone",[(s*.060,.104,.088),(s*.113,.168,.037),
                   (s*.179,.194,-.065)],[.029,.021,.004],n=7)


def limb(m,chain,shell,accent,hunter,frenzy,tier):
    upper, lower, foot, s, z, front = chain
    drop = -.27 if front else -.28
    spread = .08 if front else .09
    joint_z = .09 if front else -.06 if z<0 else .02
    m.tube("Chitin coxa and upper leg",upper,shell,[(0,.023,-z*.12),(s*.081,-.059,z*.54),
           (s*.141,-.162,z*.95),(s*.180,-.208,z)],[.067,.081,.067,.048],n=9)
    m.ell("Chitin hinge cap",lower,accent,(0,.001,.021),(.065,.061,.054),segments=10)
    m.tube("Load-bearing tapered leg",lower,shell,[(0,.040,-joint_z*.16),
           (s*spread*.53,-.130,joint_z*.54),(s*spread,drop,joint_z)],
           [.054,.055,.031],n=9)
    for x in (-.028,.028):
        m.tube("Bifid ground claw",foot,accent,[(x,.005,-.029),(x,-.014,.045),
               (x*1.57,-.026,.104)],[.030,.024,.004],n=7)
    if front:
        scythe(m,lower,s,accent,hunter,frenzy,tier)


def scythe(m,node,s,accent,hunter,frenzy,tier):
    tip = (-s*STRIKE[0],STRIKE[1],STRIKE[2])
    m.tube("Swept striking scythe",node,accent,[(s*.021,.004,.018),(s*.073,.037,.152),
           (s*.044,.020,.302),tip],[.070,.073+.019*hunter,.047,.003],n=10)
    m.tube("Inlaid edge of the striking scythe",node,"forge" if frenzy else "linen",[
        (-s*.036,.038,.117),(-s*.041,.039,.235),(-s*.046,.002,.357),tip,
    ],[.010,.015,.012,.002],n=7)
    if frenzy and tier==2:
        m.tube("Secondary raking hook",node,"ravager_rust",[(s*.057,.015,.118),
               (s*.113,.065,.214),(s*.123,.045,.339)],[.041,.044,.003],n=8)
