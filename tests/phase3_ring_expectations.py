"""Independent test dimensions: torus R10/r2 cut at Y0, detail 2 by 3."""
import math

def expected_ring():
    return [
        dict(valid=True,solids=1,faces=3,bounds=[-12,-12,-2,12,0,2],volume=40*math.pi**2,area=40*math.pi**2+8*math.pi),
        dict(valid=True,solids=0,faces=1,bounds=[0,-10,0,2,-10,3],volume=0,area=6),
    ]

def matches(actual,expected,tolerance=0.001):
    if any(actual.get(k)!=expected[k] for k in ['valid','solids','faces']):return False
    bounds=actual.get('bounds',[])
    if len(bounds)!=6 or any(not math.isfinite(v) or abs(v-e)>tolerance for v,e in zip(bounds,expected['bounds'])):return False
    for key in ['area','volume']:
        value=actual.get(key,float('nan'))
        if not math.isfinite(value) or abs(value-expected[key])>max(tolerance,abs(expected[key])*0.0001):return False
    return True
