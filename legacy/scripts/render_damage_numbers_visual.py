from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path, BUILD_ROOT, WORK_ROOT
#!/usr/bin/env python3
"""Composite real ImGui glyphs and arbitrary indexed triangles with linear texture sampling."""
import json
from pathlib import Path
import numpy as np
from PIL import Image
from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
P=WORK_ROOT/'open-visual'
def triangle(target,texture,v,clip):
    """Pixel-center triangle coverage with top-left rule and affine UV/RGBA."""
    height,width=target.shape[:2];th,tw=texture.shape[:2]
    xy=v[:,:2]
    def cross(a,b):return a[0]*b[1]-a[1]*b[0]
    area=cross(xy[1]-xy[0],xy[2]-xy[0])
    if abs(area)<1e-10:return
    if area<0:v=v[[0,2,1]];xy=v[:,:2];area=-area
    lo=xy.min(axis=0);hi=xy.max(axis=0)
    l=max(0,int(np.ceil(max(lo[0],clip[0])-.5)));t=max(0,int(np.ceil(max(lo[1],clip[1])-.5)))
    r=min(width,int(np.ceil(min(hi[0],clip[2])-.5)));b=min(height,int(np.ceil(min(hi[1],clip[3])-.5)))
    if l>=r or t>=b:return
    xx,yy=np.meshgrid(np.arange(l,r)+.5,np.arange(t,b)+.5)
    edges=[];inside=np.ones(xx.shape,dtype=bool)
    for a,z in ((xy[1],xy[2]),(xy[2],xy[0]),(xy[0],xy[1])):
        delta=z-a;e=delta[0]*(yy-a[1])-delta[1]*(xx-a[0])
        top_left=delta[1]<0 or (delta[1]==0 and delta[0]>0)
        inside&=(e>1e-9)|((abs(e)<=1e-9)&top_left);edges.append(e/area)
    weights=np.stack(edges,axis=-1)
    uv=weights@v[:,2:4];xs=np.clip(uv[:,:,0]*tw-.5,0,tw-1);ys=np.clip(uv[:,:,1]*th-.5,0,th-1)
    xa=xs.astype(int);ya=ys.astype(int);xb=np.minimum(xa+1,tw-1);yb=np.minimum(ya+1,th-1)
    fx=(xs-xa)[:,:,None];fy=(ys-ya)[:,:,None]
    sample=(texture[ya,xa]*(1-fx)+texture[ya,xb]*fx)*(1-fy)+(texture[yb,xa]*(1-fx)+texture[yb,xb]*fx)*fy
    colors=np.array([[(int(c)>>(8*j)&255)/255 for j in range(4)] for c in v[:,4]])
    src=sample*(weights@colors);a=src[:,:,3:4]*inside[:,:,None];dst=target[t:b,l:r]
    dst[:,:,:3]=src[:,:,:3]*a+dst[:,:,:3]*(1-a);dst[:,:,3:4]=a+dst[:,:,3:4]*(1-a)

def render(output=None):
    global P
    if output is not None:P=output
    doc=json.loads((P/'draw.json').read_text());textures={}
    for id,w,h in doc['textures']:
        textures[id]=np.frombuffer((P/f'texture-{id}.rgba').read_bytes(),dtype=np.uint8).reshape(h,w,4).astype(float)/255
    width,height=doc.get("display",[620,220])
    target=np.ones((height,width,4));target[:,:,:3]=doc.get("background",[39/255,47/255,49/255])
    for command in doc['commands']:
        tex,idx,n,base=command[:4]
        clip=command[4:8] if len(command)==8 else [0,0,width,height]
        image=textures[tex];h,w=image.shape[:2]
        axis_quads=n%6==0
        if axis_quads:
            for i in range(idx,idx+n,6):
                vs=np.array([doc['vertices'][base+j] for j in sorted(set(doc['indices'][i:i+6]))])
                if len(vs)!=4 or len(set(vs[:,4]))!=1 or len(set(vs[:,0]))!=2 or len(set(vs[:,1]))!=2:
                    axis_quads=False;break
                expected_u=np.where(vs[:,0]==vs[:,0].min(),vs[:,2].min(),vs[:,2].max())
                expected_v=np.where(vs[:,1]==vs[:,1].min(),vs[:,3].min(),vs[:,3].max())
                if not np.array_equal(vs[:,2],expected_u) or not np.array_equal(vs[:,3],expected_v):
                    axis_quads=False;break
        if not axis_quads:
            for i in range(idx,idx+n,3):
                vs=np.array([doc['vertices'][base+j] for j in doc['indices'][i:i+3]])
                triangle(target,image,vs,clip)
            continue
        # Preserve the accepted axis-aligned sampling path byte for byte.
        for i in range(idx,idx+n,6):
            vertices=np.array([doc['vertices'][base+j] for j in sorted(set(doc['indices'][i:i+6]))])
            assert len(vertices)==4 and len(set(vertices[:,4]))==1,'Unexpected non-quad draw'
            x0,y0=vertices[:,:2].min(axis=0);x1,y1=vertices[:,:2].max(axis=0)
            if x1<=x0 or y1<=y0:continue
            l,t=max(0,int(np.ceil(max(x0,clip[0])-.5))),max(0,int(np.ceil(max(y0,clip[1])-.5)))
            r,b=min(width,int(np.ceil(min(x1,clip[2])-.5))),min(height,int(np.ceil(min(y1,clip[3])-.5)))
            if l>=r or t>=b:continue
            u0,v0=vertices[:,2:4].min(axis=0);u1,v1=vertices[:,2:4].max(axis=0)
            xx=(np.arange(l,r)+.5-x0)/(x1-x0);yy=(np.arange(t,b)+.5-y0)/(y1-y0)
            xs=np.clip((u0+xx*(u1-u0))*w-.5,0,w-1);ys=np.clip((v0+yy*(v1-v0))*h-.5,0,h-1)
            xa=xs.astype(int);ya=ys.astype(int);xb=np.minimum(xa+1,w-1);yb=np.minimum(ya+1,h-1)
            fx=(xs-xa)[None,:,None];fy=(ys-ya)[:,None,None]
            sample=(image[ya[:,None],xa[None,:]]*(1-fx)+image[ya[:,None],xb[None,:]]*fx)*(1-fy)+(image[yb[:,None],xa[None,:]]*(1-fx)+image[yb[:,None],xb[None,:]]*fx)*fy
            color=int(vertices[0,4]);tint=np.array([(color>>(8*i)&255)/255 for i in range(4)])
            src=sample*tint;dst=target[t:b,l:r];a=src[:,:,3:4]
            dst[:,:,:3]=src[:,:,:3]*a+dst[:,:,:3]*(1-a);dst[:,:,3:4]=a+dst[:,:,3:4]*(1-a)
    out=P/'preview.png';Image.fromarray(np.clip(target*255,0,255).astype(np.uint8)).convert('RGB').save(out);print(out)
    return out
if __name__=='__main__':render()
