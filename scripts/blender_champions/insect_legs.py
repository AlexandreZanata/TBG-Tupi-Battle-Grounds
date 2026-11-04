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
