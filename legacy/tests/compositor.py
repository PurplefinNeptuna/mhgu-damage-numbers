"""Analytic checks for tilted coverage, seam ownership, UV/color and clipping."""
import numpy as np
from render_damage_numbers_visual import triangle

def run():
    white=np.ones((1,1,4));target=np.ones((10,10,4));target[:,:,:3]=0
    points=[(.5,.5),(8.5,2.5),(6.5,8.5),(-1.5,6.5)]
    uv=[(0,0),(1,0),(1,1),(0,1)]
    v=np.array([(*p,*q,0x800000ff) for p,q in zip(points,uv)])
    triangle(target,white,v[[0,1,2]],[0,0,10,10]);triangle(target,white,v[[0,2,3]],[0,0,10,10])
    filled=target[:,:,0]>0
    assert filled.sum()>30 and np.allclose(target[:,:,0][filled],128/255), 'diagonal seam double blend or hole'
    assert target[4,3,0]==128/255 and target[0,8,0]==0, 'tilted shape coverage'
    clip=np.ones((10,10,4));clip[:,:,:3]=0
    triangle(clip,white,v[[0,1,2]],[2,2,5,6]);triangle(clip,white,v[[0,2,3]],[2,2,5,6])
    assert np.all(clip[:2,:,:3]==0) and np.all(clip[6:,:,:3]==0) and np.all(clip[:,:2,:3]==0) and np.all(clip[:,5:,:3]==0)
    # Constant white UV texel times barycentric vertex colors.
    tri=np.array([[0,0,0,0,0xff0000ff],[8,0,1,0,0xff00ff00],[0,8,0,1,0xffff0000]])
    color=np.ones((10,10,4));color[:,:,:3]=0
    triangle(color,white,tri,[0,0,10,10])
    assert np.allclose(color[2,2,:3],[.375,.3125,.3125]), 'RGBA interpolation'
    texture=np.ones((8,8,4));texture[:,:,0]=np.arange(8)[None,:]/7
    tex=np.ones((10,10,4));tex[:,:,:3]=0
    tri[:,4]=0xffffffff;triangle(tex,texture,tri,[0,0,10,10])
    assert abs(tex[2,2,0]-2/7)<1e-9, 'UV interpolation and bilinear sample'
    print('PASS: tilted triangle coverage, no shared-edge double blend, UV/RGBA interpolation and command clipping')
if __name__=='__main__':run()
