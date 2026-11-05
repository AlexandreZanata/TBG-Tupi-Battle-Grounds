"""Build a resonating cicada with a chest-mounted aperture and hinged wing shells."""

from bloomkeeper import leaf
from completion_people import insect_collars, limbs
from insect_legs import auxiliary_legs

APERTURE=(0,0,.30)


def stridulator(m,tier,branch,*,production=False):
    echo = tier > 0 and branch == 0
    chorus = tier > 0 and branch == 1
    shell = "horn_stone" if chorus else "spine_slate"
    trim = "sun_cream" if echo else "wasp_amber" if chorus else "edge"
    m.loft("Heavy cicada thorax","Chest",shell,[
        (-.062,.092,.073,0),(.053,.115,.100,.007),
        (.186,.159,.117,0),(.273,.169,.093,.005),(.324,.096,.069,.010),
    ],n=14)
    m.tube("Short ridged gaster","Hips",shell,[(0,.029,-.019),(0,.033,-.152),(0,-.044,-.331)],
           [.086,.116,.020],n=12)
    insect_collars(m,shell)
    limbs(m,shell,"pine",arm=.17*.95,width=.049,skin=shell)
    auxiliary_legs(m,shell,trim)
    m.ell("Flattened cicada head","Head",shell,(0,.011,.021),(.144,.101,.103),segments=13)
    for s in (-1,1):
        m.ell("Lateral cicada eye","Head","ink",(s*.115,.031,.085),(.045,.042,.044),segments=10)
        m.ell("Cicada eye facet","Head","sap",(s*.130,.038,.106),(.025,.027,.026),segments=9)
        m.tube("Cicada antenna","Head",trim,[(s*.052,.086,.048),(s*.096,.143,.094),
               (s*.124,.147,.141)],[.012,.011,.002],n=7)
        leaf(m,"Folded ribbed cicada wing","HeadGear",trim,(s*.091,.264,-.041),
             (s*(.335+.050*chorus),.085,-.353),.083,facing=(0,1,0))
        m.tube("Wing hinge and raised vein","HeadGear",shell,[(s*.090,.259,-.031),
               (s*.187,.192,-.164),(s*.306,.097,-.306)],[.029,.019,.004],n=8)
        if chorus and tier==2:
            leaf(m,"Nested chorus wing","HeadGear","beetle_pumpkin",(s*.122,.240,-.067),
                 (s*.414,.223,-.276),.067,facing=(0,1,0))
    resonators(m,trim,tier,echo,chorus)


def resonators(m,trim,tier,echo,chorus):
    c="Chest"
    m.tube("Lower resonating chamber mount",c,"pine",[(0,.119,.088),(0,.051,.172),
           (0,0,.247)],[.062,.065,.045],n=10)
    m.loft("Recessed pulse chamber throat",c,"pine",[
        (.215,.047,.043,0),(.272,.049,.041,0),(.300,.053,.045,0),
        (.300,.038,.032,0),(.250,.028,.024,0),
    ],n=14,transform=lambda p:(p[0],-p[2],p[1]))
    m.rim("Pulse aperture at the native chest origin",c,trim,APERTURE,(.052,.045),.014,segments=14)
    for s in (-1,1):
        for i in range(3):
            y=.086+.061*i
            m.tube("Parallel acoustic rib",c,trim,[(s*.063,y,.119),(s*.124,y+.018,.091),
                   (s*.150,y+.018,.042)],[.011,.016,.010],n=7)
        if echo:
            m.tube("Ivory arch around the pulse chamber",c,trim,[(s*.057,-.020,.269),
                   (s*.095,.034,.255),(s*.099,.171,.149),(s*.062,.262,.102)],
                   [.014,.017,.017,.012],n=7)
        if chorus:
            m.rim("Side chamber horn",c,"beetle_pumpkin",(s*.135,.125,.118),(.045,.048),.018,segments=12)
        if echo and tier==2:
            leaf(m,"Crown of the echo chamber","Head",trim,(s*.047,.067,.084),
                 (s*.083,.203,-.019),.036)
