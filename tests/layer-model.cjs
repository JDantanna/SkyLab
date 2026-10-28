const test=require('node:test'),assert=require('node:assert/strict'),layers=require('../web/layer-model.js');
const data=()=>({version:1,crs:{zone:23,south:true},lines:[{name:'L100',points:[[500000,7800000],[500100,7800000]]}],polygons:[],warnings:[]});
test('imported files retain separate layers and names, and do not mutate the prior history state',()=>{
 const old=layers.prepare(data(),'first.mis','MIS'),saved=JSON.stringify(old),incoming=data(),merged=layers.merge(old,incoming,'second.mis','MIS');assert.equal(merged.layers.length,2);assert.notEqual(merged.lines[0].layer,merged.lines[1].layer);assert.equal(merged.lines[0].name,merged.lines[1].name);assert.notEqual(layers.key(merged.lines[0]),layers.key(merged.lines[1]));assert.equal(JSON.stringify(old),saved);assert.equal(incoming.version,1);
});
test('opening a saved layered project retains group names and remaps IDs when adding it to another project',()=>{
 const saved=layers.merge(layers.prepare(data(),'A.mis','MIS'),data(),'B.mis','MIS'),added=layers.prepare(saved,'saved.json','Project',[{id:20}]);assert.deepEqual(added.layers.map(l=>l.name),['A.mis','B.mis']);assert.deepEqual(added.lines.map(l=>l.layer),[21,22]);assert.equal(added.version,3);
});
