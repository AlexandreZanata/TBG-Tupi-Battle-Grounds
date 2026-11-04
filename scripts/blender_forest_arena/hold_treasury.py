"""Model the Treasury vault, its mint and the roof mortar."""

from math import cos, sin, tau

from hold_parts import banner, blocks, bore, disc, footing, gear, layer, mechanism, pier
from mesh import box, surface, tube


def treasury(root,form,seat):
    stone = layer(root,"Treasury masonry")
    footing(stone,2.18,1.89)
    blocks(stone,0,0,.16,1.88,1.46,1.17,rows=5)
    for x in (-.81,.81):
        for y in (-.62,.62):
            pier(stone,x,y,.15,.25,1.21)
    door = layer(root,"Vault ironwork")
    vault(door,{"A1":.55,"A2":.65}.get(form,.45))
    roof = layer(root,"Hipped tile roof")
    tiles(roof,form=="A2")
    banner(roof,seat,-.64,-.80,1.26,.21,.31)

    mortar = layer(root,"Coin mortar dormer")
    box(mortar,"Open dormer ring",(.60,0,1.477),(.44,.37,.087),"oak",.018)
    box(mortar,"Recessed dark opening",(.60,0,1.527),(.31,.24,.019),"soot",.009)
    for sign in (-1,1):
        hinge = mechanism(mortar,"Dormer trapdoor",(.60,sign*.12,1.57),
                          lambda g,s=sign: hatch(g,s),motion="Hatch")
        hinge.rotation_euler.x = -sign*.78
    mechanism(mortar,"Coin mortar",(.60,0,1.59),
              lambda g: bore(g,.16,.106,-.134,key="bronze",edge="gold"),
              pitch=.8,muzzle=(.16,0,0),motion="Aim and recoil")

    upgrade = layer(root,"Fortification details")
    if form in ("A1","A2"):
        armored(upgrade,form=="A2")
    else:
        if form in ("B1","B2"):
            press(upgrade,.77)
            mechanism(upgrade,"Mint drive gear",(.94,-.727,.59),
                      lambda g: gear(g,.22,12),motion="Turn")
        if form == "B2":
            press(upgrade,-.77)
            royal_mint(upgrade)
        if form == "Base":
            for x in (-.84,.84):
                box(upgrade,"Trade-goods chest",(x,-.77,.22),(.29,.32,.24),"oak",.032)
                box(upgrade,"Chest lid",(x,-.77,.36),(.32,.33,.063),"wood",.018)


def vault(group,r):
    center = (0,-.805,.754)
    disc(group,"Recessed vault jamb",center,r*1.10,.092,"mortar",sides=20)
    disc(group,"Riveted bronze casing",(0,-.875,.754),r*1.02,.134,"bronze",sides=20)
    disc(group,"Inset forged vault door",(0,-.949,.754),r*.835,.045,"gunmetal",sides=20)
    disc(group,"Vault lock hub",(0,-.989,.754),r*.29,.045,"bronze",sides=12)
    disc(group,"Vault wheel boss",(0,-1.019,.754),r*.15,.05,"gold",sides=10)
    for i in range(8):
        angle = tau*i/8
        x,z = r*.925*cos(angle),.754+r*.925*sin(angle)
        disc(group,"Hand-peened brass rivet",(x,-.952,z),.032,.035,"gold",sides=7)
        tube(group,"Radial lock spoke",[(r*.16*cos(angle),-1.014,.754+r*.16*sin(angle)),
             (r*.57*cos(angle),-1.008,.754+r*.57*sin(angle))],[.025,.017],"bronze",7)
    for z in (.53,.97):
        tube(group,"Door hinge barrel",[(r*.74,-.924,z-.09),(r*.74,-.924,z+.09)],
             [.046,.046],"iron",8)


def tiles(group,armored_roof):
    # The roof stops at the gun-side dormer so the mortar remains visible.
    low,high = 1.35,1.93
    vertices = [(-1.02,-.88,low),(.40,-.88,low),(.40,.88,low),(-1.02,.88,low),
                (-.68,0,high),(.22,0,high)]
    faces = [(0,1,5,4),(2,3,4,5),(3,0,4),(1,2,5)]
    surface(group,"Hipped tile roof shell",vertices,faces,["tile-dark"]*4)
    for sign in (-1,1):
        for row in range(5):
            y = sign*(.09+row*.173)
            z = high-.63*abs(y)
            margin = .058*row
            for j in range(7):
                x = -.69-margin+j*(.92+margin*1.44)/6
                tube(group,"Overlapping clay roof tile",
                     [(x,y-sign*.101,z+.060),(x,y+sign*.087,z-.055)],
                     [.085,.082], "tile-light" if (j+row)%4==0 else "tile",sides=6)
    tube(group,"Capped roof ridge",[(-.75,0,high+.018),(.30,0,high+.018)],
         [.080,.070],"bronze" if armored_roof else "tile-light",sides=9)
    box(group,"Dormer side roof",(.72,.43,1.386),(.51,.66,.12),"tile",.032)
    box(group,"Dormer side roof",(.72,-.43,1.386),(.51,.66,.12),"tile",.032)


def hatch(group,sign):
    box(group,"Hinged hatch leaf",(0,-sign*.079,0),(.36,.159,.053),"oak",.015)
    for x in (-.12,.12):
        box(group,"Hatch iron strap",(x,-sign*.082,.027),(.045,.145,.025),"gunmetal",.005)


def armored(group,grand):
    for x in (-.88,.88):
        box(group,"Iron side band",(x,-.760,.79),(.148,.08,1.08),"gunmetal",.024)
        for z in (.39,.69,1.04,1.28):
            disc(group,"Armor stud",(x,-.813,z),.036,.040,"silver",sides=8)
    if grand:
        for x in (-.90,.91):
            for y in (-.64,.64):
                tube(group,"Grand Vault plated corner column",[(x,y,.19),(x,y,.36),(x,y,1.52)],
                     [.247,.223,.170],"gunmetal",sides=11,accent="silver")
                tube(group,"Plated corner crown",[(x,y,1.49),(x,y,1.58),(x,y,1.84)],
                     [.239,.222,.029],"bronze",sides=11)
        box(group,"Bronze vault lintel",(0,-.87,1.39),(.97,.18,.095),"bronze",.025)


def press(group,x):
    y = -1.12
    box(group,"Mint press stone base",(x,y,.13),(.43,.37,.16),"stone-light",.027)
    for sign in (-1,1):
        box(group,"Screw press upright",(x+sign*.161,y,.56),(.080,.101,.77),"oak",.014)
    box(group,"Press yoke",(x,y,.94),(.45,.17,.093),"wood-end",.018)
    disc(group,"Press die",(x,y,.39),.113,.075,"bronze",axis="z",sides=12)
    tube(group,"Threaded press spindle",[(x,y,.42),(x,y,1.06)],[.038,.034],"gunmetal",8)
    for i in range(5):
        disc(group,"Visible screw thread",(x,y,.57+i*.054),.054,.021,"bronze",axis="z",sides=9)
    mechanism(group,"Mint press lever",(x,y,1.02),
              lambda g: tube(g,"Hand-crank with brass grips",[(-.23,0,0),(0,0,0),(.23,0,0)],
                             [.032,.021,.032],"bronze",sides=8),motion="Turn")


def royal_mint(group):
    x,y = -.56,.36
    pier(group,x,y,1.58,.31,.60,rows=3)
    disc(group,"Forge chimney rim",(x,y,2.24),.184,.12,"bronze",axis="z")
    disc(group,"Dark flue",(x,y,2.308),.125,.014,"soot",axis="z")
    tube(group,"Medallion support",[(0,0,1.93),(0,0,2.58)],[.053,.037],"bronze",8)
    mechanism(group,"Royal mint medallion",(0,0,2.69),medallion,motion="Turn")


def medallion(group):
    disc(group,"Minted gold medallion",(0,0,0),.343,.062,"gold",sides=24)
    disc(group,"Coin border",(0,-.039,0),.299,.026,"bronze",sides=24)
    disc(group,"Embossed coin face",(0,-.056,0),.267,.025,"gold",sides=20)
    for x,z,w,h in ((0,0,.09,.23),(-.11,-.022,.075,.15),(.11,-.022,.075,.15)):
        box(group,"Coin relief stamp",(x,-.077,z),(w,.032,h),"bronze",.016)
