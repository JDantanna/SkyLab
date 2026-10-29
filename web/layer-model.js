(function(root){
 'use strict';
 const key=line=>JSON.stringify([line.layer??null,line.name]);
 function prepare(data,name,source,occupied=[]){
  let next=Math.max(0,...occupied.map(layer=>layer.id))+1;const mapping=new Map(),layers=[];
  for(const layer of data.layers||[]){const id=next++;mapping.set(layer.id,id);layers.push({...layer,id});}
  for(const layer of layers)if(layer.parent!=null)layer.parent=mapping.get(layer.parent)??null;
  let fallback=null;const assign=geometry=>{let id=mapping.get(geometry.layer);if(id===undefined){if(fallback===null){fallback=next++;layers.push({id:fallback,warnings:[...data.warnings],name:name.replace(/[\x00-\x1f\x7f]/g,' ').slice(0,255)||'Project',source});}id=fallback;}return {...geometry,layer:id};};
  return {...data,version:Math.max(3,data.version||1),lines:data.lines.map(assign),polygons:data.polygons.map(assign),layers};
 }
 function ensure(data,name='Project'){const ids=new Set(data.layers?.map(l=>l.id));if(ids.size&&[...data.lines,...data.polygons].every(g=>ids.has(g.layer)))return data;return prepare(data,name,'Project');}
 function merge(existing,incoming,name,source){const base=ensure(existing),added=prepare(incoming,name,source,base.layers);return {...base,version:Math.max(3,base.version,added.version),crs:added.crs,layers:[...base.layers,...added.layers],polygons:[...base.polygons,...added.polygons],lines:[...base.lines,...added.lines],warnings:[...new Set([...base.warnings,...added.warnings])]};}
 const caches=new WeakMap();
 function index(data){let value=caches.get(data);if(!value){value=new Map((data.layers||[]).map(l=>[l.id,l]));caches.set(data,value);}return value;}
 function visible(data,id){const map=index(data),seen=new Set();while(id!=null){const node=map.get(id);if(!node)return true;if(node.hidden||seen.has(id))return false;seen.add(id);id=node.parent;}return true;}
 function descendants(data,id){const ids=new Set([id]);for(let n=0;n<32;n++){const before=ids.size;for(const l of data.layers||[])if(ids.has(l.parent))ids.add(l.id);if(ids.size===before)break;}return ids;}
 function move(data,id,parent=null,before=null){const map=index(data),node=map.get(id);if(!node||parent===id||parent!=null&&map.get(parent)?.source!=='Group'||parent!=null&&descendants(data,id).has(parent))throw Error('Hierarquia inválida.');const layers=data.layers.filter(l=>l.id!==id),moved={...node,parent};let at=before==null?layers.length:layers.findIndex(l=>l.id===before&&(l.parent??null)===(parent??null));if(at<0)throw Error('Hierarquia inválida.');layers.splice(at,0,moved);return {...data,version:Math.max(5,data.version),layers};}
 function group(data,name,parent=null){const id=Math.max(0,...data.layers.map(l=>l.id))+1;if(parent!=null&&index(data).get(parent)?.source!=='Group')throw Error('Hierarquia inválida.');return {...data,version:Math.max(5,data.version),layers:[...data.layers,{id,name,source:'Group',parent,hidden:false,warnings:[]}]};}
 function toggle(data,kind,id){if(kind==='line'){if(!data.lines[id])throw Error('Seleção inválida.');return {...data,version:Math.max(5,data.version),lines:data.lines.map((l,i)=>i===id?{...l,hidden:!l.hidden}:l)};}if(!index(data).has(id))throw Error('Hierarquia inválida.');return {...data,version:Math.max(5,data.version),layers:data.layers.map(l=>l.id===id?{...l,hidden:!l.hidden}:l)};}
 const api={prepare,ensure,merge,key,visible,descendants,move,group,toggle};if(typeof module!=='undefined'&&module.exports)module.exports=api;else root.SkyLabLayers=api;
})(typeof window==='undefined'?globalThis:window);
