"""Build the reef guard's one-handed trident and layered coral silhouettes."""

from abyssal_parts import fins, marine
from bloomkeeper import leaf


def reeflancer(m,tier,branch,*,production=False):
    guard = tier > 0 and branch == 0
    hunt = tier > 0 and branch == 1
    edge = "sun_cream" if guard else "petal_light" if hunt else "oldbone"
    coral = "beetle_coral" if hunt else "press_ochre"
    cloth = "pyre_enamel" if hunt else "capy_reed"
    marine(m,"reeflancer","shark_hide","fish_silver",cloth,face=(.103,.107,.111),snout=.149)
    fins(m,coral,edge,.216,layers=3)
    trident(m,edge,coral,tier,guard,hunt)
    m.tube("Diagonal reef guard bandolier","Chest",coral,[(-.113,.281,.068),
           (-.056,.197,.124),(.098,.087,.096)],[.023,.028,.023],n=8)
    for s in (-1,1):
        leaf(m,"Fitted hanging shell strip","Hips",coral,(s*.064,.023,.103),
             (s*.124,-.187,.080),.045)
        if hunt:
            leaf(m,"Raised coral shoulder thorn","UpperArmL",edge,(s*.042,.009,.072),
                 (s*(.102+.044*(tier==2)),.109,-.037),.036)
    if guard:
        m.plate("Trident guard shield shell","UpperArmL",edge,
                [(-.095,-.093),(-.118,.039),(0,.091),(.114,.040),(.108,-.089),(0,-.118)],(.043,.075))
    if guard and tier==2:
        leaf(m,"Ivory trident-shaped brow ridge","Head",edge,(0,.061,.103),
             (0,.202,-.072),.048,facing=(1,0,0))


def trident(m,edge,coral,tier,guard,hunt):
    n="WeaponR"
    neck=.475+.045*guard
    m.tube("Curved hardwood trident shaft",n,"wood",[(0,-.237,.003),(0,0,0),(0,.278,.022),
           (0,neck,.029)],[.021,.027,.021,.028],n=9)
    m.box("Trident fork root",n,edge,(0,neck,.028),(.227,.047,.064),bevel=.012)
    for s in (-1,0,1):
        x=s*(.115 if guard else .100)
        m.tube("Thickened independent trident tine",n,edge,[(x,neck,.028),
               (x*1.17,neck+.105,.041),(x*1.10,neck+(.223 if not s else .169),.054)],
               [.028,.021,.002],n=8)
        if hunt and s:
            leaf(m,"Reverse coral barb",n,coral,(x,neck+.065,.042),
                 (s*(.210+.048*(tier==2)),neck+.024,.056),.045)
    m.tube("Forward striking guard seated on the shaft",n,coral,[(0,-.014,.001),
           (0,.018,.113),(0,0,.250)],[.038,.024,.004],n=8)
