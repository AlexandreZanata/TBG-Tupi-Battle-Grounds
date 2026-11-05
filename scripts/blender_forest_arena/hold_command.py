"""Model the Command standard, forward outpost and observer platform."""

from math import cos, sin, tau

from hold_parts import banner, braces, deck_rail, disc, footing, ladder, layer, mechanism, plaque
from mesh import box, pebble, surface, tube


def command(root,form,seat):
    tower(layer(root,"Command masonry and balcony"),seat)
    standards(layer(root,"Stitched heraldry"),form,seat)
    guard = layer(root,"Platform defender")
    mechanism(guard,"Archer orbit",(.72,0,1.50),archer,motion="Swivel around Stronghold")
    mechanism(guard,"Archer bow",(.80,0,1.77),bow,
              muzzle=(.20,0,0),motion="Aim and loose")
    if form in ("B1","B2"):
        outpost(layer(root,"Forward command post"),seat)
    if form == "B2":
        observatory(layer(root,"High Command observatory"),seat)


def tower(group,seat):
    footing(group,1.56,1.59)
    for row in range(7):
        z = .15+row*.178
        r = .57-.012*row
        tube(group,"Tapered limestone course",[(0,0,z),(0,0,z+.168)],
             [r,r-.008],"stone" if row%3 else "stone-cool",sides=14,accent="stone-light")
    for x in (-.13,0,.13):
        box(group,"Oak door board",(x,-.553,.387),(.127,.063,.52),"oak",.012)
    for z in (.22,.49):
        box(group,"Iron door strap",(0,-.600,z),(.41,.038,.034),"gunmetal",.010)
    for x in (-.22,.22):
        box(group,"Door jamb",(x,-.532,.408),(.111,.155,.60),"stone-light",.020)
    for i in range(7):
        a = i*tau/7
        x,y = .441*cos(a),.441*sin(a)
        braces(group,(x,y,1.14),(x*1.51,y*1.51,1.44),key="oak",width=.055)
    tube(group,"Corbelled ring balcony",[(0,0,1.365),(0,0,1.435),(0,0,1.51)],
         [.54,.728,.789],"stone-light",sides=14,accent="stone")
    disc(group,"Inset oak deck",(0,0,1.514),.714,.051,"wood",axis="z",sides=14)
    deck_rail(group,0,0,1.774,.734,skip=(0,1,9))
    ladder(group,-.49,-.52,.15,1.32)
    banner(group,seat,.01,-.735,1.51,.37,.49)


def standards(group,form,seat):
    war = form in ("A1","A2")
    crown = form=="A2"
    x,y = -.40,.29
    top = 3.24 if crown else 2.91 if war else 2.67
    tube(group,"War-standard mast",[(x,y,1.49),(x,y,top)],[.038,.022],"oak",sides=8)
    disc(group,"Mast foot socket",(x,y,1.56),.09,.12,"bronze",axis="z",sides=10)
    if war:
        braces(group,(x-.30,y,1.55),(x,y,2.17),width=.044)
        braces(group,(x+.27,y,1.53),(x,y,2.17),width=.044)
    banner(group,seat,x,y-.047,top-.15,.81 if crown else .65 if war else .45,.88 if war else .62)
    if crown:
        z = top-.03
        plaque(group,"Cast Supreme Command crown",
               [(x+a,z+b) for a,b in ((-.28,-.07),(.28,-.07),(.36,.21),
                  (.17,.11),(0,.31),(-.17,.11),(-.36,.21))],y,.088,"gold",.018)
        for sign in (-1,1):
            banner(group,seat,x+sign*.39,y+.019,top-.51,.26,.55)
    else:
        tube(group,"Gilded spear finial",[(x,y,top-.08),(x,y,top),(x,y,top+.18)],
             [.037,.071,.005],"gold",sides=8)


def archer(group):
    for y in (-.075,.072):
        tube(group,"Defender leg",[(0,y,.025),(-.03,y,.15),(0,y,.252)],
             [.055,.054,.066],"oak",sides=7)
        box(group,"Defender boot",(.033,y,.036),(.18,.108,.076),"wood-dark",.015)
    tube(group,"Archer doublet",[(0,0,.225),(-.01,0,.316),(-.031,0,.420),(-.031,0,.466)],
         [.104,.114,.123,.073],"hood",sides=9,accent="leaf-deep")
    for sign in (-1,1):
        y = sign*.093
        tube(group,"Bent archer sleeve",[(-.032,y,.403),(.036,sign*.142,.331),(.105,sign*.064,.312)],
             [.047,.044,.027],"hood",sides=7)
        pebble(group,"Archer hand",(.107,sign*.057,.311),(.034,.033,.034),"skin",sides=7)
    pebble(group,"Archer head",(-.006,0,.520),(.074,.071,.088),"skin",sides=9)
    pebble(group,"Fitted woodland hood",(-.027,.008,.560),(.075,.082,.083),"hood",
           sides=9,accent="leaf-deep")
    tube(group,"Back quiver",[(-.113,.073,.280),(-.134,.117,.498)],
         [.045,.049],"oak",sides=7)


def bow(group):
    tube(group,"Laminated longbow",[(.086,0,-.247),(.163,0,-.144),(.170,0,0),
         (.157,0,.146),(.084,0,.258)],[.009,.024,.026,.023,.009],"wood-end",sides=8)
    tube(group,"Drawn linen bowstring",[(.086,0,-.247),(-.055,0,.002),(.084,0,.258)],
         [.005,.005,.005],"linen",sides=5)
    tube(group,"Nocked arrow",[(-.055,0,0),(.154,0,0),(.20,0,0)],
         [.008,.010,.002],"silver",sides=6)


def outpost(group,seat):
    for y in (-.27,.27):
        tube(group,"Forward bridge support",[(1.33,y,.14),(1.28,y,1.49)],
             [.043,.044],"oak",sides=7)
        braces(group,(.58,y,.94),(1.29,y,1.48),width=.039)
        for x in (.84,1.38):
            braces(group,(x,y,1.49),(x,y,2.10),width=.032)
    for i in range(6):
        box(group,"Forward gangway plank",(.74+i*.118,0,1.493),(.111,.64,.072),"wood",.010)
    box(group,"Campaign map table",(1.10,0,1.72),(.33,.32,.048),"oak",.015)
    box(group,"Folded parchment map",(1.09,-.024,1.75),(.245,.19,.017),"linen",.006,.13)
    vertices = [(x,y,2.10+.075*(1-abs(y)/.37)+.021*sin(x*8))
                for x in (.77,1.0,1.23,1.45) for y in (-.37,-.18,0,.18,.37)]
    faces = [(r*5+j,(r+1)*5+j,(r+1)*5+j+1,r*5+j+1) for r in range(3) for j in range(4)]
    surface(group,"Tensioned campaign canopy",vertices,faces,
            ["trim" if i%4 in (0,3) else seat for i in range(len(faces))])


def observatory(group,seat):
    x,y = -.17,.085
    tube(group,"Observatory chamber",[(x,y,1.50),(x,y,1.62),(x,y,2.50)],
         [.34,.341,.302],"stone",sides=14,accent="stone-light")
    for z in (1.79,2.12):
        disc(group,"Observatory horizontal stone course",(x,y,z),.337,.058,
             "stone-light",axis="z",sides=14)
    box(group,"Observatory window opening",(x,y-.294,2.29),(.224,.057,.24),"soot",.025)
    box(group,"Window bronze sill",(x,y-.325,2.168),(.27,.09,.041),"bronze",.009)
    tube(group,"Seamed bronze dome",[(x,y,2.44),(x,y,2.59),(x,y,2.75),(x,y,2.84)],
         [.379,.370,.282,.067],"bronze",sides=14,accent="gold")
    disc(group,"Dome skirt",(x,y,2.468),.386,.057,seat,axis="z",sides=14)
    mechanism(group,"Observatory telescope",(x+.17,y-.11,2.67),telescope,
              pitch=.43,motion="Observation pivot")


def telescope(group):
    tube(group,"Brass telescope outer tube",[(-.10,0,0),(.23,0,0),(.48,0,0)],
         [.044,.063,.089],"bronze",sides=12,accent="gold")
    disc(group,"Lens retaining ring",(.491,0,0),.103,.052,"gold",axis="x",sides=14)
    disc(group,"Recessed telescope glass",(.518,0,0),.076,.018,"patina",axis="x",sides=14)
