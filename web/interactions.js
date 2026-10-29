(function(root){
  'use strict';
  const interactions={
    hitIndex(lines,point,tolerance=12){let best=tolerance,index=-1;lines.forEach((points,i)=>{for(let j=1;j<points.length;j++){const a=points[j-1],b=points[j],dx=b[0]-a[0],dy=b[1]-a[1],den=dx*dx+dy*dy,t=den?Math.max(0,Math.min(1,((point[0]-a[0])*dx+(point[1]-a[1])*dy)/den)):0,d=Math.hypot(point[0]-a[0]-t*dx,point[1]-a[1]-t*dy);if(d<best){best=d;index=i;}}});return index;},
    boxHits(lines,a,b){const minx=Math.min(a[0],b[0]),maxx=Math.max(a[0],b[0]),miny=Math.min(a[1],b[1]),maxy=Math.max(a[1],b[1]);return lines.flatMap((points,index)=>{const hit=points.some((point,i)=>{if(!i)return false;const start=points[i-1],dx=point[0]-start[0],dy=point[1]-start[1];let lo=0,hi=1;for(const [p,q]of [[-dx,start[0]-minx],[dx,maxx-start[0]],[-dy,start[1]-miny],[dy,maxy-start[1]]]){if(!p){if(q<0)return false;}else{const t=q/p;if(p<0)lo=Math.max(lo,t);else hi=Math.min(hi,t);if(lo>hi)return false;}}return true;});return hit?[index]:[];});},
    simplifyStroke(points,tolerance=2){if(points.length<3)return points;const keep=new Set([0,points.length-1]),stack=[[0,points.length-1]];while(stack.length){const [first,last]=stack.pop(),a=points[first],b=points[last],dx=b[0]-a[0],dy=b[1]-a[1],den=dx*dx+dy*dy;let far=tolerance,index=-1;for(let i=first+1;i<last;i++){const p=points[i],t=den?Math.max(0,Math.min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/den)):0,d=Math.hypot(p[0]-a[0]-t*dx,p[1]-a[1]-t*dy);if(d>far){far=d;index=i;}}if(index>=0){keep.add(index);stack.push([first,index],[index,last]);}}return [...keep].sort((a,b)=>a-b).map(i=>points[i]);},
    sequence(lines,indices,{prefix,first,increment,digits}){
      if(typeof prefix!=='string'||prefix.length>32||/[\x00-\x1f\x7f]/.test(prefix)||!Number.isInteger(first)||first<0||!Number.isInteger(increment)||increment<1||!Number.isInteger(digits)||digits<1||digits>10||!indices.length||first+(indices.length-1)*increment>4294967295)throw Error('Numeração inválida.');
      const ordered=[...new Set(indices)].sort((a,b)=>a-b);if(ordered.length!==indices.length||ordered.some(i=>!lines[i]))throw Error('Seleção inválida.');
      const rows=ordered.map((index,i)=>({index,name:prefix+String(first+i*increment).padStart(digits,'0')}));if(rows.some(row=>row.name.length>64||!/^[a-zA-Z0-9_.-]+$/.test(row.name)||!Number.isNaN(Number(row.name))))throw Error('Nome inválido para MIS.');return rows;
    },
    range(anchor,end){const out=[];for(let i=Math.min(anchor,end);i<=Math.max(anchor,end);i++)out.push(i);return out;},
    shortcut(event,{modal=false,touring=false,editing=false,busy=false}={}){
      if(modal||touring||editing||busy||event.altKey||!(event.ctrlKey||event.metaKey))return null;
      const key=event.key.toLowerCase();
      if(key==='z')return event.shiftKey?'redo':'undo';
      if(key==='y')return 'redo';
      return null;
    }
  };
  if(typeof module!=='undefined'&&module.exports)module.exports=interactions;
  else root.SkyLabInteractions=interactions;
})(typeof window==='undefined'?globalThis:window);
