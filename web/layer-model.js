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
 function ensure(data,name='Project'){const ids=new Set(data.layers?.map(l=>l.id));if(ids.size&&data.lines.every(g=>ids.has(g.layer))&&data.polygons.every(g=>ids.has(g.layer)))return data;return prepare(data,name,'Project');}
 function merge(existing,incoming,name,source){const base=ensure(existing),added=prepare(incoming,name,source,base.layers);return {...base,version:Math.max(3,base.version,added.version),crs:added.crs,layers:[...base.layers,...added.layers],polygons:[...base.polygons,...added.polygons],lines:[...base.lines,...added.lines],warnings:[...new Set([...base.warnings,...added.warnings])]};}
 // Projects and their geometry are immutable between commits; weak keys release undo states.
 const caches=new WeakMap();
 function topology(data){let value=caches.get(data);if(!value){const map=new Map(),children=new Map();for(const layer of data.layers||[]){map.set(layer.id,layer);const parent=layer.parent??null;if(!children.has(parent))children.set(parent,[]);children.get(parent).push(layer);}value={map,children,visible:new Map()};caches.set(data,value);}return value;}
 function index(data){return topology(data).map;}
 function children(data,parent=null){return topology(data).children.get(parent)||[];}
 function visible(data,id){const tree=topology(data),seen=new Set();let result=true;while(id!=null){if(tree.visible.has(id)){result=tree.visible.get(id);break;}const node=tree.map.get(id);if(!node)break;if(node.hidden||seen.has(id)){result=false;break;}seen.add(id);id=node.parent;}for(const key of seen)tree.visible.set(key,result);return result;}
 function descendants(data,id){const ids=new Set([id]),pending=[{id,depth:0}];for(let i=0;i<pending.length;i++){const node=pending[i];if(node.depth>=32)continue;for(const layer of children(data,node.id))if(!ids.has(layer.id)){ids.add(layer.id);pending.push({id:layer.id,depth:node.depth+1});}}return ids;}
 // Aggregate only supplied counts, independent of search and with no repeated geometry scans.
 function totals(data,own){const result=new Map(own);for(const layer of data.layers||[]){const count=own.get(layer.id)||0;if(!count)continue;const seen=new Set([layer.id]);let parent=layer.parent;while(parent!=null&&!seen.has(parent)&&seen.size<=32){seen.add(parent);result.set(parent,(result.get(parent)||0)+count);parent=index(data).get(parent)?.parent;}}return result;}
 function move(data,id,parent=null,before=null){const map=index(data),node=map.get(id);if(!node||parent===id||parent!=null&&map.get(parent)?.source!=='Group'||parent!=null&&descendants(data,id).has(parent))throw Error('Hierarquia inválida.');const layers=data.layers.filter(l=>l.id!==id),moved={...node,parent};let at=before==null?layers.length:layers.findIndex(l=>l.id===before&&(l.parent??null)===(parent??null));if(at<0)throw Error('Hierarquia inválida.');layers.splice(at,0,moved);return {...data,version:Math.max(5,data.version),layers};}
 function group(data,name,parent=null){const id=Math.max(0,...data.layers.map(l=>l.id))+1;if(parent!=null&&index(data).get(parent)?.source!=='Group')throw Error('Hierarquia inválida.');return {...data,version:Math.max(5,data.version),layers:[...data.layers,{id,name,source:'Group',parent,hidden:false,warnings:[]}]};}
 function toggle(data,kind,id){if(kind==='polygon'){if(!data.polygons[id])throw Error('Seleção inválida.');return {...data,version:6,polygons:data.polygons.map((g,i)=>i===id?{...g,hidden:!g.hidden}:g)};}if(kind==='line'){if(!data.lines[id])throw Error('Seleção inválida.');return {...data,version:Math.max(5,data.version),lines:data.lines.map((l,i)=>i===id?{...l,hidden:!l.hidden}:l)};}if(!index(data).has(id))throw Error('Hierarquia inválida.');return {...data,version:Math.max(5,data.version),layers:data.layers.map(l=>l.id===id?{...l,hidden:!l.hidden}:l)};}
 const api={prepare,ensure,merge,key,visible,descendants,children,totals,move,group,toggle};if(typeof module!=='undefined'&&module.exports)module.exports=api;else root.SkyLabLayers=api;
})(typeof window==='undefined'?globalThis:window);
