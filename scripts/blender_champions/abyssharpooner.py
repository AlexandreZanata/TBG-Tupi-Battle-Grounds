"""Fit a two-hand harpoon carrier with a guided shaft and a rear spear rack."""

from abyssal_parts import fins, mantle, marine
from bloomkeeper import leaf

RELEASE=(0,.12,.28)
SUPPORT=(.24,-.01,.11)


def abyssharpooner(m,tier,branch,*,production=False):
    sight = tier > 0 and branch == 0
    hunt = tier > 0 and branch == 1
    frame = "star_ceramic" if sight else "judge_obsidian" if hunt else "wood"
    edge = "prism_ice" if sight else "beetle_coral" if hunt else "fish_silver"
    marine(m,"abyssharpooner","gunner_indigo","shark_belly","leech_plum",
           face=(.104,.098,.131),eyes=.077,snout=.155)
    fins(m,"leech_plum",edge,.164+.026*hunt)
    mantle(m,"ritual_indigo" if not hunt else "pyre_enamel",edge,.158,.308)
    launcher(m,frame,edge,tier,sight,hunt)
    if sight:
        m.rim("Recessed diver's sight", "Head",frame,(-.077,.035,.171),(.044,.039),.010,segments=12)
        if tier==2:
            m.tube("Side sight hinge rising from the headset","Head",edge,
                   [(-.092,.058,.128),(-.138,.109,.074),(-.102,.157,-.026)],[.021,.019,.015],n=8)
    if hunt:
        for s in (-1,1):
            m.tube("Stored barbed spear attached to the back rack","BackSpring",edge,[
                (s*.079,-.156,-.069),(s*.091,.121,-.076),(s*(.112+.044*(tier==2)),.330,-.095),
            ],[.017,.024,.004],n=8)
            leaf(m,"Rack spear barb","BackSpring",frame,(s*.106,.277,-.088),
                 (s*.143,.214,-.098),.034)


def launcher(m,frame,edge,tier,sight,hunt):
    n="WeaponR"
    m.loft("Fitted hardwood launcher stock",n,frame,[
        (-.049,.036,.061,.070),(.020,.068,.185,.105),
        (.088,.063,.172,.110),(.111,.046,.136,.126),
    ],n=10)
    m.rim("Open guide at the projectile origin",n,edge,RELEASE,(.049,.043),.012,segments=13)
    end=.627 if sight else .552
    m.tube("Harpoon shaft through the launch guide",n,"iron",[(0,.12,.062),RELEASE,
           (0,.12,end-.110),(0,.12,end)],[.019,.019,.022,.003],n=9)
    for s in (-1,1):
        leaf(m,"Swept harpoon barb",n,edge,(0,.12,end-.018),
             (s*(.082 if hunt else .047),.12,end-.150),.029,facing=(0,1,0))
    m.tube("Rigid support stirrup running from stock to second hand",n,"darkwood",[
        (.031,.019,.107),(.153,-.022,.106),SUPPORT,(.266,.018,.115),
    ],[.027,.025,.029,.018],n=8)
    if sight and tier==2:
        m.rim("Forward raised sight",n,edge,(0,.191,.235),(.027,.027),.010,segments=10)
    if hunt:
        leaf(m,"Spear center spur",n,edge,(0,.13,.340),(0,.200,.467),.029,facing=(1,0,0))
