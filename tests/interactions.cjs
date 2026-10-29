const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const interactions=require('../web/interactions.js');
test('Shift selects the complete interval, in both directions',()=>{
 assert.deepEqual(interactions.range(2,6),[2,3,4,5,6]);assert.deepEqual(interactions.range(6,2),[2,3,4,5,6]);assert.deepEqual(interactions.range(4,4),[4]);
});
test('project shortcuts preserve text editing and avoid acting behind dialogs',()=>{
 assert.equal(interactions.shortcut({ctrlKey:true,key:'z'}),'undo');assert.equal(interactions.shortcut({ctrlKey:true,key:'Z',shiftKey:true}),'redo');assert.equal(interactions.shortcut({ctrlKey:true,key:'y'}),'redo');
 for(const context of [{editing:true},{modal:true},{touring:true},{busy:true}])assert.equal(interactions.shortcut({ctrlKey:true,key:'z'},context),null);
});
function harness(){
 const elements=new Map(),listeners={};const element=id=>{if(!elements.has(id))elements.set(id,{value:'23',textContent:id==='project-title'?'Synthetic project':'Synthetic source',hidden:true});return elements.get(id);};
 const context=vm.createContext({console,Set,structuredClone,SkyLabInteractions:interactions,window:{SkyLabInteractions:interactions,skyAuthority:null},document:{activeElement:{tagName:'BODY'},querySelector:()=>null,addEventListener:(name,handler)=>listeners[name]=handler},$:element,render(){},status(){},refreshSatellite(){},getCrs:()=>({zone:23,south:true}),tr:s=>s});
 vm.runInContext('let selectedPolygon=-1,lineScope=null;let project={polygons:[],lines:Array.from({length:12},(_,i)=>({name:"L"+i,points:[[i,0],[i,100]]}))},selected=4,selection=new Set([4]),selectionAnchor=4,view={x:100,y:200,span:500},history=[],future=[],busy=false,dirty=false;const selectedIndices=()=>[...selection].filter(i=>project?.lines[i]);',context);
 const source=fs.readFileSync(require.resolve('../web/workspace.js'),'utf8');const code=source.slice(source.indexOf('const HISTORY_LIMIT='),source.indexOf("async function selectionMutation"));vm.runInContext(code,context);
 const app=fs.readFileSync(require.resolve('../web/app.js'),'utf8'),start=app.indexOf("document.addEventListener('keydown',"),end=app.indexOf('\nasync function download',start);vm.runInContext(app.slice(start,end),context);
 return {context,listeners,element,run:code=>vm.runInContext(code,context)};
}
test('regression: deleting a line then actual Ctrl+Z handler restores geometry, selection and view',()=>{
 const h=harness();h.run('commit({...project,lines:project.lines.filter((_,i)=>i!==4)},"delete");selection.clear();selected=-1;view={x:900,y:900,span:200};');let prevented=false;
 h.listeners.keydown({ctrlKey:true,key:'z',preventDefault(){prevented=true},stopPropagation(){}});
 assert(prevented);assert.equal(h.run('project.lines.length'),12);assert.equal(h.run('project.lines[4].name'),'L4');assert.equal(h.run('selected'),4);assert.equal(h.run('view.x'),100);
});
test('eleven deletions undo and redo; branching discards redo and memory is bounded to 30 states',()=>{
 const h=harness();h.run('for(let i=0;i<11;i++)commit({...project,lines:project.lines.slice(1)},"delete");');assert.equal(h.run('project.lines.length'),1);
 for(let i=0;i<11;i++)h.listeners.keydown({ctrlKey:true,key:'z',preventDefault(){},stopPropagation(){}});assert.equal(h.run('project.lines.length'),12);
 for(let i=0;i<11;i++)h.run('redo()');assert.equal(h.run('project.lines.length'),1);
 h.run('undo();commit({...project,lines:project.lines.slice(1)},"branch");');assert.equal(h.run('future.length'),0);
 h.run('for(let i=0;i<40;i++)commit({...project},"revision");');assert.equal(h.run('history.length'),30);
});
test('adding a file fits the added geometry, retains existing lines and undo restores the prior camera',async()=>{
 const h=harness();h.element('merge').checked=true;
 h.context.api=async(op,extra)=>op==='inspect-crs'?{crs:{zone:23,south:true},kind:'declared'}:op==='import-kml'?{version:1,crs:{zone:23,south:true},polygons:[],warnings:[],lines:[{name:'NEW',points:[[500000,7800000],[500100,7800100]]}]}:extra.project;
 h.context.importCrs=async()=>({zone:23,south:true});h.context.sameCrs=()=>true;h.context.clone=structuredClone;h.context.draw=()=>{};h.context.transform={w:600,h:400};h.context.satelliteEnabled=false;
 h.run('project.crs={zone:23,south:true};project.version=1;project.polygons=[];project.warnings=[];');
 const source=fs.readFileSync(require.resolve('../web/workspace.js'),'utf8');
 vm.runInContext(source.slice(source.indexOf('function fitGeometry('),source.indexOf('async function fitSatelliteGeometry(')),h.context);
 const layerSource=fs.readFileSync(require.resolve('../web/layers.js'),'utf8');
 h.context.SkyLabLayers=require('../web/layer-model.js');h.context.readLayerFile=file=>file.text();h.context.renderLayers=()=>{};
 const loader=layerSource.slice(layerSource.indexOf('loadFile=async function'),layerSource.indexOf("$('file').onchange="));
 vm.runInContext(loader,h.context);
 await h.context.loadFile({size:100,name:'synthetic.kml',text:async()=>'<kml/>'});
 assert.equal(h.run('project.lines.length'),13);assert.equal(h.run('view.x'),500050);assert.equal(h.run('view.y'),7800050);assert.equal(h.run('view.span'),195);assert.equal(h.element('source-info').textContent,'synthetic.kml');
 assert.equal(h.run('project.layers.length'),2);assert.notEqual(h.run('project.lines[0].layer'),h.run('project.lines[12].layer'));
 h.run('undo()');assert.equal(h.run('project.lines.length'),12);assert.equal(h.run('view.x'),100);
});

test('Ctrl toggles individual lines while Shift selects an interval',()=>{
 const h=harness();h.element('search').value='';h.element('line-list').querySelector=()=>null;
 const source=fs.readFileSync(require.resolve('../web/workspace.js'),'utf8');vm.runInContext(source.slice(source.indexOf('select=function'),source.indexOf('const HISTORY_LIMIT=')),h.context);
 h.run('select(2);select(7,false,true);select(10,false,true)');assert.deepEqual(Array.from(h.run('[...selection]')),[2,7,10]);
 h.run('select(7,false,true)');assert.deepEqual(Array.from(h.run('[...selection]')),[2,10]);
 h.run('select(5);select(8,true)');assert.deepEqual(Array.from(h.run('[...selection]')),[5,6,7,8]);
 h.run('select(5);select(5,false,true)');assert.equal(h.run('selection.size'),0);assert.equal(h.run('selected'),-1);
});
test('batch names follow project order, preserve gaps and reject numeric overflow',()=>{
 const lines=Array.from({length:5},(_,i)=>({name:'OLD'+i})),config={prefix:'L',first:100100,increment:100,digits:6};
 const rows=interactions.sequence(lines,[4,0,2],config);assert.deepEqual(rows,[{index:0,name:'L100100'},{index:2,name:'L100200'},{index:4,name:'L100300'}]);assert.equal(lines[1].name,'OLD1');
 assert.deepEqual(interactions.sequence(lines,[0,1],{...config,first:1,digits:4}).map(r=>r.name),['L0001','L0101']);
 for(const change of [{first:-1},{first:4294967295},{increment:0},{increment:1.5},{digits:0},{prefix:'x\n'}])assert.throws(()=>interactions.sequence(lines,[0,2],{...config,...change}));
});
