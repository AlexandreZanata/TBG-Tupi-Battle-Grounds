"""Shared fitted limb underlayers and faces for the completion figures."""


def limbs(m,cloth,bracer,arm=.17,leg=.38,width=.055,skin="skin"):
    for side in ("L","R"):
        upper,fore,hand,thigh,shin,foot = (f"{part}{side}" for part in
                                           ("UpperArm","Forearm","Hand","Thigh","Shin","Foot"))
        m.tube("Fitted articulated sleeve",upper,cloth,[(0,.028,0),(0,-arm*.41,.009),(0,-arm,0)],
               [width*.96,width*1.13,width*.72],n=9)
        m.tube("Forearm bracer",fore,bracer,[(0,.019,0),(0,-arm*.38,.006),(0,-arm*.9,0)],
               [width*.77,width*.94,width*.61],n=9)
        m.ell("Curved grip palm",hand,skin,(0,-.018,.008),(width*.77,width*.84,width*.69),segments=10)
        for node in (thigh,shin):
            m.tube("Tapered joined lower limb",node,cloth,[(0,.027,0),(0,-leg*.21,.007),(0,-leg*.50,0)],
                   [width*.93,width,width*.69],n=8)
        m.ell("Shaped boot",foot,bracer,(0,-.021,.041),(width,.045,.099),segments=10)


def face(m,skin="skin",brow="darkwood"):
    h = "Head"
    m.loft("Shaped face and chin",h,skin,[
        (-.110,.055,.057,.027),(-.069,.087,.083,.013),(.007,.104,.101,.006),
        (.079,.102,.093,0),(.123,.066,.073,-.013),
    ],n=15)
    m.ell("Face nose bridge",h,skin,(0,-.009,.108),(.023,.030,.024),segments=10)
    m.ell("Mouth line",h,brow,(0,-.056,.101),(.038,.009,.008),segments=8)
    for s in (-1,1):
        m.ell("Visible eye",h,"linen",(s*.043,.026,.102),(.025,.014,.013),segments=9)
        m.ell("Eye pupil",h,"ink",(s*.044,.025,.113),(.010,.012,.005),segments=8)
        m.tube("Defined brow",h,brow,[(s*.024,.043,.105),(s*.053,.050,.105),
               (s*.072,.043,.094)],[.008,.011,.007],n=7)
