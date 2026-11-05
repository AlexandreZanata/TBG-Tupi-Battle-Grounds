"""Shape jointed insect legs with matching clawed foot silhouettes."""


def legs(m,side,span,shell,accent,width=.053):
    thigh,shin,foot = (f"{part}{side}" for part in ("Thigh","Shin","Foot"))
    for node in (thigh,shin):
        m.tube("Tapered chitin leg",node,shell,[(0,.030,0),(0,-span*.43,.013),(0,-span,0)],
               [width*.83,width,width*.65],n=8)
    m.ell("Chitin knee shield",shin,accent,(0,-.028,.030),(width*.90,width*1.12,width*.60),segments=9)
    for x in (-width*.48,width*.48):
        m.tube("Two-toed insect foot",foot,shell,[(x*.72,.009,-.029),(x,-.026,.042),
               (x*1.41,-.039,.107)],[width*.44,width*.41,.004],n=7)


def auxiliary_legs(m,shell,trim):
    span = .38*.55
    for upper,lower,foot,side in (("BackSpring","StringTop","Back",1),("Tabard","StringBottom","Nock",-1)):
        m.tube("Independent auxiliary coxa",upper,shell,[(0,.024,0),(side*.068,-.077,-.027),
               (side*.043,-.148,-.021),(0,-span,0)],[.041,.050,.038,.026],n=8)
        m.tube("Independent auxiliary shin",lower,trim,[(0,.021,0),(side*.027,-.081,-.019),
               (side*.018,-.151,-.020),(0,-span,0)],[.029,.035,.025,.016],n=8)
        for x in (-.014,.014):
            m.tube("Auxiliary planted toe",foot,shell,[(x,.008,-.026),
                   (x+side*.027,-.019,.028),(x+side*.043,-.031,.067)],[.019,.018,.003],n=7)
