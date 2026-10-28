const test=require('node:test'),assert=require('node:assert/strict');
const keys=require('../web/keybindings.js'),tiles=require('../web/tiles.js');
test('editable shortcuts preserve requested order, survive reload and reject duplicate or reserved combinations',()=>{
 assert.deepEqual(Object.keys(keys.defaults),['new','open','save','delete','select','cut']);
 assert.equal(keys.chord({key:'n',ctrlKey:true}),'Ctrl+N');assert.equal(keys.chord({key:'q'}),'Q');
 const changed=keys.assign(keys.defaults,'cut','X');assert.equal(keys.load(JSON.parse(JSON.stringify(changed))).cut,'X');
 assert.throws(()=>keys.assign(changed,'cut','V'),/duplicate/);assert.throws(()=>keys.assign(changed,'cut','F1'),/reserved/);
 assert.deepEqual(keys.load({cut:'bad'}),keys.defaults);
});
test('XYZ projection aligns imagery, click selection and knife coordinates at several zooms',()=>{
 for(const p of [[-45,-20],[12.5,55.7],[-120,60],[179.9,-30]])for(const z of [0,7,14,20]){
  const [x,y]=tiles.forward(p),camera={x:x+.0001,y:y+.0002,z},screen=tiles.screen(p,camera,820,460),result=tiles.unscreen(screen,camera,820,460);
  assert(Math.abs(result[0]-p[0])<1e-8);assert(Math.abs(result[1]-p[1])<1e-8);
 }
});
test('XYZ fits antimeridian areas locally and bounds polar latitude and zoom',()=>{
 const camera=tiles.fit([[179.9,10],[-179.9,10.1]],800,500);assert(camera.z>5&&camera.z<=20);
 for(const p of [[179.9,10],[-179.9,10.1]]){const s=tiles.screen(p,camera,800,500);assert(s[0]>=0&&s[0]<=800);assert(s[1]>=0&&s[1]<=500);}
 assert(tiles.forward([0,90]).every(Number.isFinite));assert.equal(tiles.fit([],800,500),null);
});
