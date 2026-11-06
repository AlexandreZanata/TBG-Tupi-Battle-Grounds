"""Carve the Undertow Herald's fluted conch and branch-specific horn chambers."""

from abyssal_parts import conch, fins, marine
from bloomkeeper import leaf

RELEASE=(0,.12,.28)
SUPPORT=(.27,0,.05)


def undertowherald(m,tier,branch,*,production=False):
    voice = tier > 0 and branch == 0
    tide = tier > 0 and branch == 1
    shell = "sun_cream" if voice else "brew_violet" if tide else "oldbone"
    edge = "gold" if voice else "bell_eye" if tide else "leather"
    marine(m,"undertowherald","fortress_moss","bone","leech_plum" if tide else "cloth",
           face=(.144,.114,.115),eyes=.091,snout=.143,bulk=1.08)
    fins(m,shell,edge,.219,layers=2)
    horn(m,shell,edge,tier,voice,tide)
    for s in (-1,1):
        m.tube("Long catfish barbel","Head","bone",[(s*.061,-.048,.134),
               (s*.099,-.093,.145),(s*.108,-.150,.166)],[.017,.012,.002],n=7)
        if voice:
            leaf(m,"Raised gold-edged choir collar","Chest",edge,(s*.083,.223,.064),
                 (s*.165,.336,-.055),.043+.014*(tier==2))
        if tide and tier==2:
            leaf(m,"Maelstrom mantle shoulder fin","BackSpring",shell,(s*.087,.092,-.012),
                 (s*.249,.214,-.077),.068)


def horn(m,shell,edge,tier,voice,tide):
    n="WeaponR"
    radius=.150 if voice else .123
    conch(m,n,shell,edge,RELEASE,.327,radius)
    m.tube("Conch grip fitted through the palm",n,"hide",[(0,-.060,-.018),(0,.006,.040),
           (0,.080,.096)],[.036,.033,.029],n=8)
    m.tube("Conch side brace at the offhand",n,"darkwood",[(.037,.061,.153),
           (.113,.049,.073),(.224,.005,.039),SUPPORT],[.033,.028,.026,.026],n=9)
    if voice:
        m.rim("Second gold resonating ring",n,edge,(0,.12,.206),(.120,.105),.013,segments=16)
        if tier==2:
            m.rim("Large outer vocal aperture",n,edge,RELEASE,(.180,.160),.016,segments=18)
            for s in (-1,1):
                m.tube("Reinforcement joining the two vocal rims",n,edge,
                       [(s*.133,.170,.278),(s*.164,.175,.280)],[.015,.016],n=7)
    if tide:
        for s in (-1,1):
            conch(m,n,shell,edge,(s*.124,.134+.028*(tier==2),.259),.183,
                  .054+.025*(tier==2),ribs=False)
