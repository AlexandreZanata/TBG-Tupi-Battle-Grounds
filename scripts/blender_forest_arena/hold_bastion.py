"""Model the Bastion curtain wall, gatehouse and Sentinel deck."""

from math import cos, sin, tau

from hold_parts import banner, ballista, blocks, bore, footing, layer, mechanism, pier, plaque
from mesh import box, tube
from structures import arch, turret


def bastion(root,form,seat):
    wall = layer(root,"Bastion masonry")
    footing(wall,2.05,2.57)
    blocks(wall,-.57,-.22,.16,.62,1.75,1.11,rows=5,columns=2)
    blocks(wall,.40,-.22,.16,.61,1.75,1.11,rows=5,columns=2)
    box(wall,"Inner gate chamber",(-.12,-.16,.64),(.73,1.35,1.02),"mortar",.052)
    arch(wall,-1.136)
    box(wall,"Curtain-wall walk",(-.16,-.10,1.16),(1.74,2.16,.16),"stone-light",.036)
    for side in (-1,1):
        for i in range(6):
            if side==1 and i==3:
                continue
            box(wall,"Curtain-wall merlon",(-.15+side*.84,-.97+i*.34,1.34),
                (.215,.226,.30),"stone",.026)
    for x in (-.77,.47):
        shield(wall,x,-1.103,.80,seat)
    box(wall,"Rear keep",(-.35,.31,1.49),(.76,.83,.63),"stone-cool",.065)
    box(wall,"Keep cornice",(-.35,.31,1.83),(.90,.95,.14),"stone-light",.036)
    for x in (-.64,-.06):
        for y in (.03,.59):
            box(wall,"Keep merlon",(x,y,1.97),(.24,.22,.24),"stone",.025)
    banner(wall,seat,-.36,-.115,1.79,.27,.45)

    defense = layer(root,"Bastion gun port")
    port(defense)
    mechanism(defense,"Falconet",(.52,-.35,.88),lambda g: bore(g,.27,.082,-.15),
              muzzle=(.27,0,0),motion="Aim and deploy through shutter")

    if form in ("A1","A2"):
        fortress(layer(root,"Fortress ramparts"),form=="A2",seat)
    if form in ("B1","B2"):
        sentinel(layer(root,"Sentinel watchtower"),form=="B2",seat)


def shield(group,x,y,z,key):
    outline = ((-.16,.13),(-.15,-.025),(0,-.213),(.15,-.025),(.16,.13))
    plaque(group,"Iron-bound kite shield",[(x+a,z+b) for a,b in outline],y,.054,"gunmetal")
    plaque(group,"Seat-colored shield enamel",[(x+a*.80,z+b*.79) for a,b in outline],
           y-.037,.027,key,.008)
    box(group,"Raised shield boss",(x,y-.061,z-.028),(.081,.040,.082),"bronze",.017,.22)


def port(group):
    x,y,z = .792,-.35,.88
    for sign in (-1,1):
        box(group,"Gun port stone jamb",(x,y+sign*.131,z),(.177,.081,.29),"stone-light",.021)
        box(group,"Gun port lintel or sill",(x,y,z+sign*.14),(.177,.239,.071),"stone-light",.020)
    box(group,"Deep gun port opening",(.731,y,z),(.060,.229,.223),"soot",.015)
    hinge = mechanism(group,"Gun port shutter",(.828,y,1.0),
                      lambda g: box(g,"Oak shutter leaf",(0,0,-.115),(.064,.247,.233),"oak",.019),
                      motion="Hatch")
    hinge.rotation_euler.y = -.96


def fortress(group,imperial,seat):
    for x in (-.68,.67):
        for y in (-.77,.78):
            turret(group,x,y,.335,1.69 if imperial else 1.47)
            if imperial:
                tube(group,"Imperial iron crown cap",[(x,y,1.79),(x,y,1.89),(x,y,2.18)],
                     [.324,.295,.027],"gunmetal",sides=10,accent="silver")
    if imperial:
        for x in (-.65,.58):
            blocks(group,x,-1.11,.12,.34,.27,.83,rows=3,columns=1,key="stone-cool")
            box(group,"Reinforcing iron strap",(x,-1.257,.67),(.102,.05,.90),"gunmetal",.015)
        banner(group,seat,0,-1.06,1.59,.43,.46)


def sentinel(group,royal,seat):
    y = .98
    tube(group,"Tapered Sentinel shaft",[(0,y,.13),(0,y,.39),(0,y,1.40),(0,y,1.95)],
         [.398,.359,.290,.295],"stone-cool",sides=12,accent="stone-light")
    for z in (.48,.84,1.23,1.59):
        tube(group,"Watchtower belt course",[(0,y,z),(0,y,z+.061)],
             [.352-(z-.48)*.040]*2,"stone-light",sides=12)
    top = 2.45 if royal else 1.95
    if royal:
        tube(group,"Ballista stage",[(0,y,1.88),(0,y,2.27),(0,y,top)],
             [.302,.299,.433],"stone",sides=12,accent="stone-light")
    tube(group,"Observation deck",[(0,y,top-.048),(0,y,top+.14)],
         [.51,.56],"stone-light",sides=12)
    for i in range(8):
        angle = tau*i/8
        x,yy = .459*cos(angle),y+.459*sin(angle)
        box(group,"Watchtower deck merlon",(x,yy,top+.22),(.151,.16,.19),"stone",.016,angle)
    banner(group,seat,0,y-.483,top+.13,.29,.43)
    if royal:
        mechanism(group,"Royal Sentinel ballista",(0,y,2.67),ballista,
                  muzzle=(.47,0,.24),motion="Aim, release and recoil")
    else:
        for x in (-.33,.33):
            for yy in (y-.30,y+.30):
                pier(group,x,yy,top+.15,.085,.48,rows=1)
        tube(group,"Tiled Sentinel watch roof",[(0,y,top+.59),(0,y,top+.70),(0,y,top+1.03)],
             [.568,.528,.032],"tile-dark",sides=8,accent="tile-light")
