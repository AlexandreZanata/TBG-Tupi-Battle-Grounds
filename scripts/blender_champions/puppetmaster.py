"""Build a skeletal showman, split tailcoat and independently held marionettes."""

from mathutils import Vector

CROSS = (0,-.05,.12)


def puppetmaster(m,tier,branch,*,production=False):
    cruel = tier > 0 and branch == 0
    chorus = tier > 0 and branch == 1
    cloth = "showman_ruby" if cruel else "showman_jade" if chorus else "showman_coat"
    tailor(m,cloth,cruel,chorus,tier)
    skull(m,cloth,cruel and tier==2)
    strings(m,"WeaponR",cloth,cruel,cruel and tier==2)
    if chorus:
        strings(m,"OffhandL","linen",False,tier==2)
        if tier==2:
            for s in (-1,1):
                doll(m,"BackSpring",(s*.155,-.177,-.077),"linen",False,.68)
                m.tube("Chorus coat hook","BackSpring","brass",[(s*.105,.036,-.020),
                       (s*.160,-.014,-.070),(s*.155,-.127,-.077)],[.015,.013,.005],n=6)


def tailor(m,cloth,cruel,chorus,tier):
    m.loft("Fitted showman's tailcoat","Chest",cloth,[
        (-.045,.128,.096,0),(.066,.122,.092,0),(.195,.163,.109,.002),
        (.288,.197,.110,0),(.327,.137,.084,-.007),(.350,.089,.071,-.001),
    ],n=14)
    m.loft("Ivory stand collar","Chest","linen",[(.327,.097,.077,0),(.383,.074,.062,0)],n=12)
    m.box("Dickey","Chest","ink",(0,.233,.107),(.095,.211,.028),.011)
    for s in (-1,1):
        outline = [(s*x,y) for x,y in ((.016,.148),(.057,.306),(.152,.330),(.114,.266),(.130,.230))]
        m.plate("Pointed contrasting coat lapel","Chest","gold" if cruel else "linen",outline[::-s],(.099,.128))
        tail = [(s*x,y) for x,y in ((.019,-.19),(.144,-.179),(.146,-.334),
                                   (.179,-.475),(.109,-.516),(.031,-.380))]
        m.plate("Split coat tail","BackSpring",cloth,tail[::-s],(-.060,-.023),bevel=.009)
        for y in (.12,.174,.228):
            m.ell("Engraved coat button","Chest","brass",(s*.060,y,.113),(.016,.016,.010),
                  segments=8,rings=5)
        arm,fore,hand,thigh,shin,foot = (f"{p}{'L' if s>0 else 'R'}" for p in
                                         ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Coat sleeve",arm,cloth,[(0,.042,0),(0,-.043,0),(0,-.151,0),(0,-.207,0)],
               [.062,.078,.056,.043],n=9)
        m.tube("Forearm sleeve",fore,cloth,[(0,.025,0),(0,-.079,0),(0,-.160,0)],
               [.051,.051,.040],n=9)
        m.tube("Tailored cuff",fore,"linen",[(0,-.123,0),(0,-.170,0)],[.049,.048],n=10)
        m.ell("Bony glove palm",hand,"bone",(0,-.019,.005),(.041,.044,.033),segments=9)
        for x in (-.022,0,.022):
            m.tube("Bony glove digit",hand,"oldbone",[(x,-.029,.022),(x,-.063,.031),(x,-.067,.044)],
                   [.009,.009,.006],n=5)
        for joint in (thigh,shin):
            m.tube("Tapered trousers",joint,"ink",[(0,.032,0),(0,-.078,0),(0,-.210,0)],
                   [.054,.053,.037],n=8)
        m.ell("Pointed show shoe",foot,"hide",(0,-.023,.046),(.052,.046,.109),segments=11)
    if chorus:
        m.tube("Jade conductor braid","Chest","gold",[(-.148,.288,.094),(-.099,.115,.134),
               (.015,.078,.126),(.113,.134,.116)],[.011]*4,n=7)
        if tier==2:
            m.ell("Twin-act brooch","Chest","soul",(0,.219,.137),(.029,.036,.016),segments=9)


def skull(m,cloth,mask):
    h = "Head"
    m.ell("Tall cranium",h,"bone",(0,.013,.010),(.115,.133,.103),segments=14,rings=8)
    m.ell("Angular jaw",h,"oldbone",(0,-.078,.041),(.082,.048,.074),segments=12)
    for s in (-1,1):
        m.ell("Deep eye socket",h,"ink",(s*.051,.013,.092),(.041,.030,.023),segments=10)
        m.ell("Violet pupil",h,"ritual",(s*.052,.014,.111),(.013,.012,.007),segments=8)
        m.tube("Skeletal brow",h,"bone",[(s*.018,.047,.102),(s*.054,.049,.115),
               (s*.087,.037,.093)],[.013,.016,.012],n=7)
    m.ell("Nose cavity",h,"ink",(0,-.032,.103),(.018,.021,.010),segments=8)
    for x in (-.045,-.027,-.009,.009,.027,.045):
        m.box("Separated skull tooth",h,"bone",(x,-.067,.105),(.014,.030,.017),.003)
    m.loft("Curved hat brim","HeadGear","ink",[(.098,.109,.104,-.01),(.106,.166,.141,-.01),
           (.124,.156,.134,-.01),(.133,.107,.095,-.01)],n=16)
    m.loft("High showman's crown","HeadGear","violet_dark",[(.120,.105,.090,-.01),
           (.198,.090,.081,-.012),(.278,.113,.092,-.030),(.310,.105,.085,-.033)],n=15)
    m.loft("Branch-colored hat band","HeadGear",cloth,[(.141,.107,.095,-.011),
           (.178,.100,.091,-.012)],n=15)
    if mask:
        m.plate("Porcelain half-mask",h,"acid_ivory",[(.011,-.008),(.020,.065),(.072,.089),
                (.105,.062),(.110,.025),(.089,.041),(.052,.052),(.038,.015)][::-1],(.115,.136))


def strings(m,node,cloth,hooked,lit):
    center = Vector(CROSS)
    rod = "ritual" if lit else "darkwood"
    m.tube("Grip joining hand to control cross",node,"darkwood",[(0,-.021,.007),
           (0,-.039,.069),center],[.020,.016,.013],n=8)
    for a,b in (((-.105,0,0),(.105,0,0)),((0,0,-.075),(0,0,.075))):
        m.tube("Marionette control cross",node,rod,[center+Vector(a),center+Vector(b)],
               [.012,.012],n=8)
    m.ell("Control cross grip",node,"gold",center,(.022,.022,.024),segments=9)
    for s in (-1,1):
        top = (s*.093,CROSS[1],.12)
        bottom = (s*.044,-.291,.143)
        m.tube("Taut marionette string",node,"ritual" if lit else "linen",
               [top,bottom],[.0035,.0035],n=5)
        if hooked:
            m.tube("Metal string hook",node,"gold",[bottom,(s*.063,-.314,.140),
                   (s*.060,-.335,.142),(s*.046,-.327,.143)],[.007,.008,.006,.003],n=6)
    doll(m,node,(0,-.338,.137),cloth,hooked,1)


def doll(m,node,center,cloth,hooked,scale):
    x,y,z = center
    def at(a,b,c):
        return (x+scale*a,y+scale*b,z+scale*c)
    m.ell("Carved wooden puppet body",node,cloth,center,
          tuple(scale*a for a in (.034,.045,.023)),segments=9,rings=5)
    m.ell("Carved puppet skull",node,"bone",at(0,.056,0),
          tuple(scale*a for a in (.032,.033,.024)),segments=9,rings=5)
    for s in (-1,1):
        m.tube("Pegged puppet arm",node,"oldbone",[at(s*.018,.025,0),at(s*.049,.002,.003),
               at(s*.058,-.021,.010)],[.010*scale]*3,n=6)
        m.tube("Pegged puppet leg",node,"oldbone",[at(s*.015,-.028,0),at(s*.021,-.056,0),
               at(s*.020,-.072,.013)],[.011*scale]*3,n=6)
    if hooked:
        m.tube("Puppet dagger",node,"edge",[at(.049,-.019,.012),at(.069,-.012,.025),
               at(.091,.023,.042)],[.009,.015,.001],n=6)
