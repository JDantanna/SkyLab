const test=require('node:test'),assert=require('node:assert/strict'),vm=require('node:vm'),fs=require('node:fs');
const source=fs.readFileSync(require.resolve('../web/cartography.js'),'utf8');
function draftHarness(){const nodes=[{point:[1,2],outgoing:[3,4]},{point:[5,6],incoming:[2,3]},{point:[9,10]}];const draft={anchors:structuredClone(nodes)},ctx=vm.createContext({cartoDraft:draft,cancelCartoGesture(){},paintCartography(){},refreshCartoButtons(){}});const a=source.indexOf('function undoCartoDraft('),b=source.indexOf('async function finishCartoPath(',a);vm.runInContext(source.slice(a,b),ctx);return {ctx,draft,nodes};}
test('pending pen points undo and redo without changing their curve handles',()=>{const h=draftHarness();h.ctx.undoCartoDraft();assert.equal(h.draft.anchors.length,2);assert.deepEqual(h.draft.redo[0],h.nodes[2]);h.ctx.redoCartoDraft();assert.deepEqual(h.draft.anchors,h.nodes);h.ctx.undoCartoDraft();h.ctx.undoCartoDraft();h.ctx.redoCartoDraft();assert.deepEqual(h.draft.anchors[1],h.nodes[1]);});
test('join preview changes the project only when explicitly applied and stale previews are discarded',async()=>{let committed=null,renders=0,canceled=0;const project={lines:[{},{}]},preview={source:project,document:2,project:{lines:[{name:'joined'}]},index:0},ctx=vm.createContext({cartoPreview:preview,project,documents:{active:2},cartoDraft:null,action:async fn=>fn(),commit:async p=>{committed=p;},tr:x=>x,Set,render(){renders++;},cancelCartoPath(){canceled++;}});const a=source.indexOf('async function finishCartoPath('),b=source.indexOf('async function previewLineJoin(',a);vm.runInContext(source.slice(a,b),ctx);assert.equal(committed,null);await ctx.finishCartoPath();assert.equal(committed,preview.project);assert.equal(renders,1);committed=null;ctx.cartoPreview={...preview,document:1};await ctx.finishCartoPath();assert.equal(committed,null);assert.equal(canceled,1);});

test('editing coordinates start inside the map border so dragging does not add a pixel offset',()=>{const js=fs.readFileSync(require.resolve('../web/editing.js'),'utf8'),a=js.indexOf('function editPosition('),b=js.indexOf('\n',a),ctx=vm.createContext({editWrap:{clientLeft:1,clientTop:1,getBoundingClientRect:()=>({left:100,top:200})}});vm.runInContext(js.slice(a,b),ctx);assert.deepEqual(Array.from(ctx.editPosition({clientX:110,clientY:220})),[9,19]);});

test('overlay repaint requests share a frame and exit cancels pending graphics',()=>{
  let next=1,renders=0,canceled=0;const frames=new Map(),ctx=vm.createContext({window:{skyCartography:true},cartoOverlay:{childNodes:[],replaceChildren(){this.childNodes=[];}},requestAnimationFrame(fn){const id=next++;frames.set(id,fn);return id;},cancelAnimationFrame(id){frames.delete(id);canceled++;},renderCartography(){renders++;}});
  const a=source.indexOf('let cartoFrame='),b=source.indexOf('function renderCartography(',a);vm.runInContext(source.slice(a,b),ctx);
  for(let i=0;i<100;i++)ctx.paintCartography();assert.equal(frames.size,1);const [id,fn]=frames.entries().next().value;frames.delete(id);fn();assert.equal(renders,1);
  ctx.paintCartography();ctx.window.skyCartography=false;ctx.cartoOverlay.childNodes=[{}];ctx.paintCartography();assert.equal(frames.size,0);assert.equal(canceled,1);assert.equal(ctx.cartoOverlay.childNodes.length,0);
});
test('anchor reads contain only selected geometry and retain CRS and layer hierarchy',()=>{
  const js=fs.readFileSync(require.resolve('../web/polygons.js'),'utf8'),a=js.indexOf('function anchorReadProject('),b=js.indexOf('function selectedAnchors(',a),ctx=vm.createContext({});vm.runInContext(js.slice(a,b),ctx);
  const project={version:7,crs:{zone:32,south:false},layers:[{id:1,parent:2},{id:2,source:'Group'}],lines:[{name:'A'},{name:'B'}],polygons:[{name:'Block'}]};
  const line=ctx.anchorReadProject(project,'lines',1),polygon=ctx.anchorReadProject(project,'polygons',0);
  assert.deepEqual(Array.from(line.lines),[project.lines[1]]);assert.equal(line.polygons.length,0);assert.equal(line.crs,project.crs);assert.equal(line.layers,project.layers);assert.equal(line.version,7);
  assert.equal(polygon.lines.length,0);assert.deepEqual(Array.from(polygon.polygons),[project.polygons[0]]);assert.equal(project.lines.length,2);
});
test('straight and curved overlay paths preserve endpoints while avoiding redundant projections',()=>{
  const js=fs.readFileSync(require.resolve('../web/polygons.js'),'utf8'),a=js.indexOf('function anchorPath('),b=js.indexOf('function paintPolygonOverlay(',a);let calls=0;const ctx=vm.createContext({screenPoint:p=>{calls++;return p;}});vm.runInContext(js.slice(a,b),ctx);
  assert.equal(ctx.anchorPath([{point:[0,0]},{point:[1,1]},{point:[2,0]}],false),'M0,0 L1,1 L2,0');assert.equal(calls,3);
  calls=0;assert.equal(ctx.anchorPath([{point:[0,0],outgoing:[1,0]},{point:[2,2],incoming:[2,1]}],false),'M0,0 C1,0 2,1 2,2');assert.equal(calls,4);
  assert.equal(ctx.anchorPath([{point:[0,0]},{point:[1,1]},{point:[2,0]}],true),'M0,0 L1,1 L2,0 L0,0 Z');
});
