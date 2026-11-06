"""Forge a horned verdict helm and a lance fitted to both existing hand anchors."""

from bloomkeeper import leaf
from completion_people import face
from deity_parts import armor, cloak

SUPPORT=(0,0,.20)
TIPS=(.80,.86,.92,.80,.88)


def astreon(m,tier,branch,*,production=False):
    primordial=tier > 0 and branch == 0
    sentence=tier > 0 and branch == 1
    plate="judge_obsidian" if sentence else "sun_cream" if primordial else "star_ceramic"
    cloth="pyre_enamel" if sentence else "sun_navy"
    light="verdict_ember" if sentence else "storm_arc"
    armor(m,"astreon",cloth,plate,"gold",layers=1+min(tier,2))
    cloak(m,cloth,"brass" if sentence else "gold",.211,.646+.091*(tier==2))
    helm(m,plate,light,tier,primordial,sentence)
    lance(m,plate,light,tier,primordial,sentence)


def helm(m,plate,light,tier,primordial,sentence):
    h="Head"
    face(m,skin="skin",brow="oldbone")
    m.loft("Closed verdict helmet shell",h,plate,[
        (.026,.135,.117,-.018),(.084,.143,.129,-.010),
        (.137,.116,.105,-.011),(.159,.061,.064,-.017),
    ],n=12)
    m.plate("Gilt T-visor brow",h,"gold",[(-.129,.077),(0,.088),(.129,.077),
            (.112,.043),(.018,.056),(.016,-.008),(-.016,-.008),(-.018,.056),(-.112,.043)],
            (.105,.145),bevel=.006)
    for s in (-1,1):
        m.plate("Articulated-looking helmet cheek plate",h,plate,
                [(s*.090,.061),(s*.141,.042),(s*.134,-.071),
                 (s*.089,-.118),(s*.064,-.074)],(.068,.139))
        m.tube("Separate swept verdict horn",h,"brass" if sentence else "gold",[
            (s*.109,.082,-.008),(s*.149,.141,.003),
            (s*(.194+.026*sentence),.230,.019+.049*sentence),
        ],[.031,.030,.004],n=9)
        m.ell("Eye inset under the visor",h,light,(s*.047,.028,.125),(.020,.009,.011),segments=8)
        if sentence:
            leaf(m,"Separate crimson helmet blade",h,"pyre_enamel",(s*.100,.064,.085),
                 (s*(.160+.047*(tier==2)),.182,-.022),.041)
    m.tube("Short carved ivory beard under the visor",h,"fish_silver",[
        (0,-.053,.104),(0,-.098,.125),(0,-.153,.122),
    ],[.052,.048,.019],n=10)
    if primordial:
        leaf(m,"Native spring-mounted helmet fin","HeadGear","gold",(0,-.048,.030),
             (0,.097+.051*(tier==2),-.089),.052,facing=(1,0,0))
    if sentence and tier==2:
        leaf(m,"Crimson forward verdict crest",h,"pyre_enamel",(0,.091,.105),
             (0,.209,.010),.049,facing=(1,0,0))


def lance(m,plate,light,tier,primordial,sentence):
    n="WeaponR"
    tip=TIPS[0 if tier==0 else tier+2*sentence]
    metal="pyre_enamel" if sentence else "gold"
    m.tube("Long rigid verdict lance shaft",n,plate,[(0,0,-.781),(0,0,-.090),
           (0,0,0),SUPPORT,(0,0,.518)],[.029,.030,.033,.033,.037],n=10)
    for z in (-.781,0,SUPPORT[2],.511):
        m.tube("Separate lance grip or end collar",n,"darkwood" if z in (0,SUPPORT[2]) else "gold",
               [(0,0,z-.032),(0,0,z+.032)],[.041,.041],n=9)
    leaf(m,"Continuous forged blade at the native lance tip",n,"edge",(0,0,.511),
         (0,0,tip),.092+.021*primordial,facing=(0,1,0))
    for s in (-1,1):
        sweep=.123+.046*primordial+.038*sentence+.034*(tier==2)
        leaf(m,"Separate swept lance guard wing",n,metal,(s*.029,0,.514),
             (s*sweep,0,.588+.022*tier),.039,facing=(0,1,0))
        if sentence:
            leaf(m,"Forked crimson verdict barb",n,metal,(s*.042,.007,.564),
                 (s*(.092+.042*(tier==2)),.008,tip-.042),.040,facing=(0,1,0))
    if tier==2:
        m.ell("Power crystal seated in the lance guard",n,light,(0,.030,.572),
              (.035,.025,.046),segments=10)
