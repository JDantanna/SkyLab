(function(root){
 'use strict';
 function forward(p){const lat=Math.max(-85.05112878,Math.min(85.05112878,p[1]))*Math.PI/180;return [(p[0]+180)/360,(1-Math.log(Math.tan(Math.PI/4+lat/2))/Math.PI)/2];}
 function inverse(p){return [p[0]*360-180,Math.atan(Math.sinh(Math.PI*(1-2*p[1])))*180/Math.PI];}
 function near(x,center){return x+Math.round(center-x);}
 function fit(points,w,h){if(!points.length||w<=0||h<=0)return null;const first=forward(points[0]),all=points.map(p=>{const q=forward(p);return [near(q[0],first[0]),q[1]];});let minx=Infinity,miny=Infinity,maxx=-Infinity,maxy=-Infinity;for(const p of all){minx=Math.min(minx,p[0]);maxx=Math.max(maxx,p[0]);miny=Math.min(miny,p[1]);maxy=Math.max(maxy,p[1]);}const scale=Math.min(Math.max(1,w-100)/Math.max(maxx-minx,1e-8),Math.max(1,h-100)/Math.max(maxy-miny,1e-8));return {x:(minx+maxx)/2,y:(miny+maxy)/2,z:Math.max(0,Math.min(20,Math.floor(Math.log2(scale/256))))};}
 function screen(p,camera,w,h){const q=forward(p),scale=256*2**camera.z;return [(near(q[0],camera.x)-camera.x)*scale+w/2,(q[1]-camera.y)*scale+h/2];}
 function unscreen(p,camera,w,h){const scale=256*2**camera.z;return inverse([camera.x+(p[0]-w/2)/scale,camera.y+(p[1]-h/2)/scale]);}
 const api={forward,inverse,near,fit,screen,unscreen};if(typeof module!=='undefined'&&module.exports)module.exports=api;else root.SkyLabTiles=api;
})(typeof window==='undefined'?globalThis:window);
