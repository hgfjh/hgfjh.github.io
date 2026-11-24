"""Independent stdlib reference check; not a Rust execution or build test."""
from collections import Counter, defaultdict
from math import atan2, cos, sin, pi, sqrt, isfinite, exp
import json
import gzip

A, B, C, R, L = 2.65, 1.35, 1.12, .53, .43


def waist(x):
    e = exp(-(x/.65)**2)
    de = -2*x/.65**2 * e
    return 1-.60*e, -.60*de, 1.12*(1-.40*e), -1.12*.40*de


def deform(p, n):
    x,y,z = p
    nx,ny,nz = n
    s,ds,t,dt = waist(x)
    normal = (nx-ds*y*ny/s-dt*z*nz/t, ny/s, nz/t)
    length = sqrt(sum(v*v for v in normal))
    return (x,s*y,t*z), tuple(v/length for v in normal)


def generate(arc, seam, radial):
    n = arc + seam
    corner = atan2(B, -C)
    pts, boundary, on_shared_seam = [], [], []
    for i in range(n):
        theta = (-corner + 2 * corner * i / arc if i <= arc else
                 corner + (2*pi-2*corner)*(i-arc)/seam)
        dx, dy = cos(theta), sin(theta)
        on_seam = i == 0 or i >= arc
        alpha = (dx/A)**2 + (dy/B)**2
        beta = 2*C*dx/A**2
        gamma = (C/A)**2-1
        outer = -C/dx if on_seam else (-beta+sqrt(beta**2-4*alpha*gamma))/(2*alpha)
        for j in range(radial+1):
            t = (1-cos(pi*j/radial))/2
            rho = R+t*(outer-R)
            x, y = C+rho*dx, rho*dy
            shared = on_seam and j == radial
            if shared:
                x = 0.
                if i == 0: y = -B
                if i == arc: y = B
            pts.append((x,y))
            boundary.append(j == 0 or (j == radial and i <= arc))
            on_shared_seam.append(shared)
    mirror = []
    for i, (x,y) in enumerate(pts.copy()):
        if on_shared_seam[i]: mirror.append(i)
        else:
            mirror.append(len(pts))
            pts.append((-x,y))
            boundary.append(boundary[i])
    faces = []
    for i in range(n):
        for j in range(radial):
            a = i*(radial+1)+j
            b = a+1
            c = ((i+1)%n)*(radial+1)+j+1
            d = c-1
            for u,v,w in [(a,b,c),(a,c,d)]:
                faces += [(u,v,w),(mirror[u],mirror[w],mirror[v])]
    edges = Counter()
    areas = []
    for a,b,c in faces:
        x,y = pts[a]; u,v = pts[b]; s,t = pts[c]
        areas.append((u-x)*(t-y)-(v-y)*(s-x))
        assert not all(boundary[k] for k in (a,b,c))
        edges.update(tuple(sorted(e)) for e in [(a,b),(b,c),(c,a)])
    assert min(areas)>0
    assert all((n,boundary[a] and boundary[b]) in [(1,True),(2,False)] for (a,b),n in edges.items())
    # Extend the planar warp over each hole by a fan, as in the Rust check.
    warped = [(x, waist(x)[0]*y) for x,y in pts]
    def area(a,b,c):
        return (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    new_areas = [area(warped[a],warped[b],warped[c]) for a,b,c in faces]
    fans = []
    for a,b,c in faces:
        for u,v in [(a,b),(b,c),(c,a)]:
            if edges[tuple(sorted((u,v)))] != 1: continue
            for cx in [-C,C]:
                if all(abs((pts[k][0]-cx)**2+pts[k][1]**2-R**2)<1e-10 for k in (u,v)):
                    assert area(pts[v],pts[u],(cx,0))>0
                    fans.append(area(warped[v],warped[u],(cx,0)))
    assert len(fans)==2*n and min(new_areas+fans)>0
    print(f'waist: min projected area={min(new_areas):.6g}, min fan area={min(fans):.6g}')
    xyz, normals, upper, lower = [], [], [], []
    residual = 0
    for (x,y),rim in zip(pts,boundary):
        q0=1-(x/A)**2-(y/B)**2; q1=(x-C)**2+y*y-R*R; q2=(x+C)**2+y*y-R*R
        p=q0*q1*q2
        px=-2*x/A**2*q1*q2+q0*2*(x-C)*q2+q0*q1*2*(x+C)
        py=-2*y/B**2*q1*q2+q0*2*y*(q1+q2)
        assert isfinite(p) and (abs(p)<1e-10 if rim else p>0)
        z=0 if rim else L*sqrt(p)
        residual=max(residual,abs(z*z-L*L*p))
        upper.append(len(xyz))
        for h in [z] if rim else [z,-z]:
            norm=(-L*L*px,-L*L*py,2*h)
            length=sqrt(sum(v*v for v in norm)); assert length>0
            position, normal = deform((x,y,h), tuple(v/length for v in norm))
            xyz.append(position)
            normals.append(normal)
        lower.append(len(xyz)-1)
    triangles=[]
    for a,b,c in faces:
        triangles.extend([(upper[a],upper[b],upper[c]),(lower[a],lower[c],lower[b])])
    ec=Counter(); oriented=Counter(); links=defaultdict(lambda:defaultdict(set)); adjacency=defaultdict(set)
    assert len({tuple(sorted(f)) for f in triangles})==len(triangles)
    for a,b,c in triangles:
        for v,x,y in [(a,b,c),(b,c,a),(c,a,b)]:
            edge=tuple(sorted((v,x)))
            ec[edge]+=1; oriented[edge]+=1 if v<x else -1
            links[v][x].add(y); links[v][y].add(x)
            adjacency[v].update((x,y))
    assert all(n==2 and oriented[e]==0 for e,n in ec.items())
    def connected(graph):
        todo=[next(iter(graph))]; seen=set()
        while todo:
            v=todo.pop()
            if v not in seen:
                seen.add(v); todo.extend(graph[v]-seen)
        return len(seen)==len(graph)
    assert len(links)==len(xyz) and connected(adjacency)
    assert all(len(link)>=3 and all(len(v)==2 for v in link.values()) and connected(link) for link in links.values())
    V,E,F=len(xyz),len(ec),len(triangles)
    assert V-E+F==-2
    assert max(abs(sum(v*v for v in n)-1) for n in normals)<1e-12
    stats=dict(vertices=V,edges=E,faces=F,euler=-2,genus=2)
    print(f'{arc=}, {seam=}, {radial=}: {stats}, min twice area={min(areas):.3g}, max residual={residual:.3g}')
    return dict(positions=xyz,normals=normals,triangles=triangles,stats=stats)


if __name__=='__main__':
    for resolution in [(8,4,2),(17,7,5),(32,16,8)]:
        generate(*resolution)
    mesh=generate(64,32,16)
    raw=json.dumps(mesh,separators=(',',':')).encode()
    print(f'Python reference JSON: {len(raw):,} bytes, gzip: {len(gzip.compress(raw)):,} bytes')
