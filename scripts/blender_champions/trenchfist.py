"""Build a hammerhead brawler with a stone ram and separate crystalline salt plates."""

from abyssal_parts import fins, marine
from bloomkeeper import leaf


def trenchfist(m,tier,branch,*,production=False):
    keel = tier > 0 and branch == 0
    salt = tier > 0 and branch == 1
    rock = "crag_fissure" if keel else "ravager_shell" if salt else "judge_stone"
    edge = "crag_limestone" if keel else "bell_eye" if salt else "mire_amber"
    marine(m,"trenchfist","fortress_moss","bone","horn_hide",
           face=(.172,.095,.114),eyes=.152,snout=.129,bulk=1.19)
    fins(m,rock,edge,.255,layers=2)
    m.tube("Diving weight harness","Chest","oilskin",[(-.153,.293,.074),
           (-.044,.207,.150),(.098,.109,.124),(.142,.045,.089)],[.033,.032,.029,.025],n=8)
    m.box("Diver's iron belt weight","Hips","iron",(0,-.012,.132),(.125,.084,.049),bevel=.014)
    ram(m,"WeaponR",rock,edge,tier,keel,salt)
    if salt:
        ram(m,"OffhandL",rock,edge,tier,False,True)
    if keel and tier==2:
        for s in (-1,1):
            leaf(m,"Keel plating around the hammerhead","Head",edge,(s*.105,.059,.065),
                 (s*.229,.132,-.086),.049)


def ram(m,n,rock,edge,tier,keel,salt):
    m.loft("Wrapped weight-bearing fist socket",n,"hide",[
        (-.107,.065,.056,.015),(-.066,.079,.073,.021),
        (.030,.079,.077,.024),(.065,.062,.058,.020),
    ],n=11)
    w=.149 if keel else .116 if salt else .129
    m.plate("Massive irregular stone face",n,rock,[(-w,-.113),(-w*.82,.074),
            (-w*.27,.144),(w*.63,.127),(w,.030),(w*.76,-.156),(0,-.186)],(.058,.216))
    m.tube("Contact ridge at the native strike point",n,edge,[(0,-.095,.192),
           (0,-.043,.235),(0,0,.250)],[.055,.051,.034],n=8)
    for s in (-1,1):
        if salt:
            leaf(m,"Splintered salt crystal knuckle",n,edge,(s*.057,.025,.177),
                 (s*(.092+.018*(tier==2)),.127+.052*(tier==2),.220),.041)
        elif keel:
            m.box("Stone ram corner band",n,edge,(s*.100,-.064,.178),(.052,.146,.088),bevel=.014)
