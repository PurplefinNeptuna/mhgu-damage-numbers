from paths import MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
#!/usr/bin/env python3
"""Composite real ImGui axis-aligned glyph/image quads with linear texture sampling."""
import json
from pathlib import Path
import numpy as np
from PIL import Image
from paths import ROOT, MOD, SCRIPTS, FORGE, DEPS, PYTHON, CONTAINER_STORAGE, input_path
P=ROOT/'work/damage-numbers/open-visual'
def render(output=None):
    global P
    if output is not None:P=output
    doc=json.loads((P/'draw.json').read_text());textures={}
    for id,w,h in doc['textures']:
        textures[id]=np.frombuffer((P/f'texture-{id}.rgba').read_bytes(),dtype=np.uint8).reshape(h,w,4).astype(float)/255
    target=np.ones((220,620,4));target[:,:,:3]=(39/255,47/255,49/255)
    for tex,idx,n,base in doc['commands']:
        image=textures[tex];h,w=image.shape[:2]
        for i in range(idx,idx+n,6):
            vertices=np.array([doc['vertices'][base+j] for j in sorted(set(doc['indices'][i:i+6]))])
            assert len(vertices)==4 and len(set(vertices[:,4]))==1,'Unexpected non-quad draw'
            x0,y0=vertices[:,:2].min(axis=0);x1,y1=vertices[:,:2].max(axis=0)
            if x1<=x0 or y1<=y0:continue
            l,t=max(0,int(np.ceil(x0-.5))),max(0,int(np.ceil(y0-.5)))
            r,b=min(620,int(np.ceil(x1-.5))),min(220,int(np.ceil(y1-.5)))
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
