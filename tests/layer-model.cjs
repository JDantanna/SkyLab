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
 const context=vm.createContext({window:{},$,WeakMap,Set,documents:{active:1},tr:s=>s,layerName:l=>l.name,renderList(){},status(){},api:()=>new Promise(r=>resolve=r)});
 vm.runInContext('let project={polygons:[{},{}],layers:[{id:1,name:"Source"}],lines:[{layer:1},{layer:1},{layer:1}]},lineScope={kind:"polygon",index:0};',context);
 vm.runInContext(source.slice(source.indexOf('const scopeAssignments='),source.indexOf('let renameTarget=')),context);
 assert.equal(context.window.skyScopeIndices().size,0);resolve([0,1,0]);await new Promise(r=>setImmediate(r));assert.deepEqual(Array.from(context.window.skyScopeIndices()),[0,2]);
 context.window.skyLineScope(2);assert.equal(vm.runInContext('lineScope.index',context),0);
 context.window.skyLineScope(1);assert.equal(vm.runInContext('lineScope.index',context),1);assert.deepEqual(Array.from(context.window.skyScopeIndices()),[1]);
 context.window.skyPolygonScope(0);assert.deepEqual(Array.from(context.window.skyScopeIndices()),[0,2]);
 vm.runInContext('project={...project};',context);context.window.skyScopeIndices();vm.runInContext('project={polygons:[],layers:[{id:9,name:"New"}],lines:[{layer:9}]};lineScope=null;',context);context.window.skyScopeIndices();resolve([1,1,1]);await new Promise(r=>setImmediate(r));assert.deepEqual(Array.from(context.window.skyScopeIndices()),[0]);assert.equal(vm.runInContext('lineScope.id',context),9);
});
