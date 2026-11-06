"""Model a six-legged bee healer carrying a separate nectar bowl and spoon."""

from bloomkeeper import leaf
from completion_people import insect_collars, limbs
from insect_legs import auxiliary_legs


def nectarnurse(m,tier,branch,*,production=False):
    honey = tier > 0 and branch == 0
    silk = tier > 0 and branch == 1
    shell = "mantis_jade" if silk else "wasp_amber"
    trim = "fish_silver" if silk else "gold"
    body(m,shell,trim,tier,honey,silk)
    insect_collars(m,shell)
    limbs(m,"pine",shell,arm=.17*.95,width=.046,skin="pine")
    auxiliary_legs(m,shell,trim)
    bowl(m,trim,tier,honey,silk)
    m.tube("Curved nectar spoon handle","WeaponR","wood",[(0,0,-.040),(0,.013,.114),
           (0,.030,.228)],[.021,.022,.012],n=8)
    m.ell("Nectar spoon cup","WeaponR",trim,(0,.032,.237),(.040,.019,.056),segments=11)


def body(m,shell,trim,tier,honey,silk):
    m.loft("Bee thorax","Chest",shell,[
        (-.048,.091,.078,0),(.087,.139,.104,0),(.209,.159,.112,.003),(.302,.104,.068,.012),
    ],n=14)
    m.tube("Rounded honeybee abdomen","Hips",shell,[(0,.016,-.025),(0,.027,-.149),
           (0,.008,-.273),(0,-.046,-.389)],[.085,.143,.113,.022],n=13)
    for y,z,radius in ((.021,-.150,.145),(.006,-.260,.123)):
        m.tube("Bee abdomen stripe","Hips","hide",[(0,y,z-.012),(0,y,z+.016)],[radius,radius],n=13)
    m.ell("Soft bee face","Head",shell,(0,.012,.023),(.121,.115,.106),segments=13)
    for s in (-1,1):
        m.ell("Warm compound eye","Head","pine",(s*.091,.018,.095),(.049,.051,.032),segments=11)
        m.ell("Eye facet","Head","sun_cream",(s*.100,.029,.123),(.015,.019,.008),segments=8)
        m.tube("Curved bee feeler","Head","darkwood",[(s*.048,.087,.065),(s*.075,.156,.090),
               (s*.109,.177,.121)],[.014,.012,.015],n=7)
        leaf(m,"Rounded bee wing","HeadGear","fish_silver" if silk else "wasp_wing",
             (s*.075,.220,-.065),(s*(.373+.043*silk),.405,-.237),.091)
        if honey:
            m.tube("Gold honey lattice edge","Chest",trim,[(s*.106,.252,.068),(s*.126,.182,.114),
                   (s*.072,.093,.132),(0,.065,.118)],[.018,.016,.014,.013],n=7)
        if silk:
            leaf(m,"Silk drape across apron","Chest","sun_cream",(s*.117,.254,.077),
                 (-s*.073,.047,.097),.051)
            leaf(m,"Trailing silk wing layer","HeadGear","wasp_aqua",(s*.088,.199,-.091),
                 (s*.305,.248 if tier==1 else .212,-.284),.052)
    if honey and tier==2:
        m.rim("Hexagonal honey diadem","Head",trim,(0,.097,.092),(.065,.055),.017,segments=6)


def bowl(m,trim,tier,honey,silk):
    n="OffhandL"
    r=.128 if silk else .105
    m.loft("Open hand-carried nectar bowl",n,"mantis_jade" if silk else "wood",[
        (.010,r*.42,r*.42,.048),(.028,r*.75,r*.75,.048),(.082,r,r,.048),
        (.082,r*.85,r*.85,.048),(.043,r*.56,r*.56,.048),
    ],n=14)
    m.ell("Nectar filling",n,"soul" if silk else "sap",(0,.062,.048),(r*.83,.011,r*.83),segments=13)
    for s in (-1,1):
        if honey or silk:
            height = .230 if tier==2 else .131
            m.tube("Shaped bowl handle",n,trim,[(s*r*.86,.056,.048),(s*r*1.16,.120,.052),
                   (s*r*.84,height,.056),(s*r*.63,height-.025,.056)],[.017,.018,.017,.013],n=8)
