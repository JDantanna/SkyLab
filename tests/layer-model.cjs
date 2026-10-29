const test=require('node:test'),assert=require('node:assert/strict'),layers=require('../web/layer-model.js');
const data=()=>({version:1,crs:{zone:23,south:true},lines:[{name:'L100',points:[[500000,7800000],[500100,7800000]]}],polygons:[],warnings:[]});
test('imported files retain separate layers and names, and do not mutate the prior history state',()=>{
 const old=layers.prepare(data(),'first.mis','MIS'),saved=JSON.stringify(old),incoming=data(),merged=layers.merge(old,incoming,'second.mis','MIS');assert.equal(merged.layers.length,2);assert.notEqual(merged.lines[0].layer,merged.lines[1].layer);assert.equal(merged.lines[0].name,merged.lines[1].name);assert.notEqual(layers.key(merged.lines[0]),layers.key(merged.lines[1]));assert.equal(JSON.stringify(old),saved);assert.equal(incoming.version,1);
});
test('opening a saved layered project retains group names and remaps IDs when adding it to another project',()=>{
 const saved=layers.merge(layers.prepare(data(),'A.mis','MIS'),data(),'B.mis','MIS'),added=layers.prepare(saved,'saved.json','Project',[{id:20}]);assert.deepEqual(added.layers.map(l=>l.name),['A.mis','B.mis']);assert.deepEqual(added.lines.map(l=>l.layer),[21,22]);assert.equal(added.version,3);
});

test('active block filtering switches lists, retains block on line selection, and ignores stale async results',async()=>{
 const fs=require('node:fs'),vm=require('node:vm'),source=fs.readFileSync(require.resolve('../web/layers.js'),'utf8');const elements=new Map(),$=id=>{if(!elements.has(id))elements.set(id,{textContent:''});return elements.get(id);};let resolve;
 const context=vm.createContext({window:{},SkyLabLayers:layers,$,WeakMap,Set,documents:{active:1},tr:s=>s,layerName:l=>l.name,renderList(){},status(){},api:()=>new Promise(r=>resolve=r)});
 vm.runInContext('let project={polygons:[{},{}],layers:[{id:1,name:"Source"}],lines:[{layer:1},{layer:1},{layer:1}]},lineScope={kind:"polygon",index:0};',context);
 vm.runInContext(source.slice(source.indexOf('const scopeAssignments='),source.indexOf('let renameTarget=')),context);
 assert.equal(context.window.skyScopeIndices().size,0);resolve([0,1,0]);await new Promise(r=>setImmediate(r));assert.deepEqual(Array.from(context.window.skyScopeIndices()),[0,2]);
 context.window.skyLineScope(2);assert.equal(vm.runInContext('lineScope.index',context),0);
 context.window.skyLineScope(1);assert.equal(vm.runInContext('lineScope.index',context),1);assert.deepEqual(Array.from(context.window.skyScopeIndices()),[1]);
 context.window.skyPolygonScope(0);assert.deepEqual(Array.from(context.window.skyScopeIndices()),[0,2]);
 vm.runInContext('project={...project};',context);context.window.skyScopeIndices();vm.runInContext('project={polygons:[],layers:[{id:9,name:"New"}],lines:[{layer:9}]};lineScope=null;',context);context.window.skyScopeIndices();resolve([1,1,1]);await new Promise(r=>setImmediate(r));assert.deepEqual(Array.from(context.window.skyScopeIndices()),[0]);assert.equal(vm.runInContext('lineScope.id',context),9);
});

test('folders nest and ungroup without changing file provenance or line coordinates',()=>{
 let p=layers.merge(layers.prepare(data(),'A.mis','MIS'),data(),'B.mis','MIS');const original=p,geometry=JSON.stringify(p.lines);
 p=layers.group(p,'Survey');const group=p.layers.at(-1).id;p=layers.group(p,'Subgroup',group);const sub=p.layers.at(-1).id;p=layers.move(p,1,sub);
 assert.equal(p.version,5);assert.deepEqual([...layers.descendants(p,group)],[group,sub,1]);assert.equal(JSON.stringify(p.lines),geometry);assert.equal(original.layers[0].parent,undefined);
 assert.throws(()=>layers.move(p,group,sub),/Hierarquia/);assert.throws(()=>layers.move(p,2,1),/Hierarquia/);assert.throws(()=>layers.move(p,sub,sub),/Hierarquia/);
 p=layers.move(p,1,null,2);assert.equal(p.layers.find(l=>l.id===1).parent,null);assert(p.layers.findIndex(l=>l.id===1)<p.layers.findIndex(l=>l.id===2));assert.equal(JSON.stringify(p.lines),geometry);
 const imported=layers.prepare(layers.move(p,1,sub),'saved.json','Project',[{id:100}]);const importedSub=imported.layers.find(l=>l.name==='Subgroup');assert.equal(imported.layers.find(l=>l.name==='A.mis').parent,importedSub.id);
});
test('eye state inherits through folders and persists without deleting any line',()=>{
 let p=layers.group(layers.prepare(data(),'A.mis','MIS'),'Group');const group=p.layers.at(-1).id;p=layers.move(p,1,group);const original=p;
 p=layers.toggle(p,'layer',group);assert.equal(layers.visible(p,1),false);assert.equal(layers.visible(original,1),true);assert.equal(p.lines.length,1);assert.deepEqual(p.lines[0].points,original.lines[0].points);
 p=layers.toggle(p,'layer',group);assert.equal(layers.visible(p,1),true);p=layers.toggle(p,'line',0);assert.equal(p.lines[0].hidden,true);assert.equal(JSON.parse(JSON.stringify(p)).lines[0].hidden,true);assert.equal(original.lines[0].hidden,undefined);
});

test('visibility and folder commits preserve an empty selection instead of selecting the first line',async()=>{
 const fs=require('node:fs'),vm=require('node:vm'),source=fs.readFileSync(require.resolve('../web/layers.js'),'utf8'),start=source.indexOf('async function hierarchyCommit('),end=source.indexOf('function toggleMissionVisibility',start);
 const context=vm.createContext({window:{skyIsLineVisible:()=>true,skyIsLayerVisible:()=>true},scopeAssignments:new WeakMap(),api:async(op,{project})=>project,tr:s=>s,render(){},selectedIndices:()=>[],commit:async next=>{context.project=next;context.selection=new Set([0]);context.selected=0;},Set});context.project={lines:[{}],polygons:[]};context.selection=new Set();context.selected=-1;context.selectedPolygon=-1;
 vm.runInContext(source.slice(start,end),context);await vm.runInContext('hierarchyCommit(project,"visible")',context);assert.equal(context.selection.size,0);assert.equal(context.selected,-1);
});
test('pointer drop routes nested files back to the root and ignores cancellation',async()=>{
 const fs=require('node:fs'),vm=require('node:vm'),source=fs.readFileSync(require.resolve('../web/layers.js'),'utf8'),start=source.indexOf('function finishMissionDrag('),end=source.indexOf("document.addEventListener('pointerup'",start);let updates=[];
 const project=layers.move(layers.group(layers.prepare(data(),'A.mis','MIS'),'Group'),1,2);const make=()=>({id:1,project,document:1,pointer:7,node:{releasePointerCapture(){}},moved:true,suppress(){}});
 const context=vm.createContext({project,documents:{active:1},missionDrag:make(),missionDropHover:{id:'mission-root-drop'},clearMissionDrag(){},action:async fn=>fn(),hierarchyCommit:async next=>updates.push(next),SkyLabLayers:layers});vm.runInContext(source.slice(start,end),context);
 vm.runInContext('finishMissionDrag({pointerId:7,preventDefault(){}})',context);await new Promise(r=>setImmediate(r));assert.equal(updates[0].layers.find(l=>l.id===1).parent,null);assert.deepEqual(updates[0].lines,project.lines);
 updates=[];context.missionDrag=make();vm.runInContext('finishMissionDrag({pointerId:7,preventDefault(){}},true)',context);assert.equal(updates.length,0);
});
