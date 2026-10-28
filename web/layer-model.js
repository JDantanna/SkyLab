(function(root){
 'use strict';
 const key=line=>JSON.stringify([line.layer??null,line.name]);
 function prepare(data,name,source,occupied=[]){
  let next=Math.max(0,...occupied.map(layer=>layer.id))+1;const mapping=new Map(),layers=[];
  for(const layer of data.layers||[]){const id=next++;mapping.set(layer.id,id);layers.push({...layer,id});}
  let fallback=null;const assign=geometry=>{let id=mapping.get(geometry.layer);if(id===undefined){if(fallback===null){fallback=next++;layers.push({id:fallback,warnings:[...data.warnings],name:name.replace(/[\x00-\x1f\x7f]/g,' ').slice(0,255)||'Project',source});}id=fallback;}return {...geometry,layer:id};};
  return {...data,version:3,lines:data.lines.map(assign),polygons:data.polygons.map(assign),layers};
 }
 function ensure(data,name='Project'){const ids=new Set(data.layers?.map(l=>l.id));if(ids.size&&[...data.lines,...data.polygons].every(g=>ids.has(g.layer)))return data;return prepare(data,name,'Project');}
 function merge(existing,incoming,name,source){const base=ensure(existing),added=prepare(incoming,name,source,base.layers);return {...base,version:3,crs:added.crs,layers:[...base.layers,...added.layers],polygons:[...base.polygons,...added.polygons],lines:[...base.lines,...added.lines],warnings:[...new Set([...base.warnings,...added.warnings])]};}
 const api={prepare,ensure,merge,key};if(typeof module!=='undefined'&&module.exports)module.exports=api;else root.SkyLabLayers=api;
})(typeof window==='undefined'?globalThis:window);
