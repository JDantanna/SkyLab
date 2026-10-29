const test=require('node:test'),assert=require('node:assert/strict');
const {Documents,LatestPreview}=require('../web/session.js');

test('untitled labels reuse vacant numbers independently of persistent document IDs',()=>{
 const docs=new Documents(3);assert.equal(docs.nextNumber(),1);const a=docs.add({untitledNumber:docs.nextNumber()});assert.equal(docs.nextNumber(),2);const b=docs.add({untitledNumber:docs.nextNumber()});assert.equal(docs.nextNumber(),3);docs.remove(a.id);assert.equal(docs.nextNumber(),1);docs.remove(b.id);const next=docs.add({untitledNumber:docs.nextNumber()});assert(next.id>1);assert.equal(next.state.untitledNumber,1);
});
test('three documents retain separate geometry, parameters and undo stacks; closing picks a remaining tab',()=>{
 const docs=new Documents(3);const a=docs.add({project:{lines:['A']},history:['undo A'],form:{spacing:100}}),b=docs.add({project:{lines:['B']},history:[],form:{spacing:250}}),c=docs.add({project:null});
 assert.throws(()=>docs.add({}),/document_limit/);docs.activate(a.id);docs.current().state.history.push('edit A');docs.activate(b.id);assert.deepEqual(docs.current().state.history,[]);assert.equal(docs.current().state.form.spacing,250);docs.remove(b.id);assert.equal(docs.active,c.id);docs.activate(a.id);assert.deepEqual(docs.current().state.project.lines,['A']);assert.equal(docs.current().state.history.length,2);docs.add({project:null});assert.equal(docs.items.length,3);
});
const pause=()=>new Promise(resolve=>setImmediate(resolve));
test('rapid preview changes coalesce, stale results never paint, and calculations stay single-flight',async()=>{
 let running=0,peak=0;const jobs=[],paint=[];const queue=new LatestPreview(value=>{peak=Math.max(peak,++running);return new Promise(resolve=>jobs.push({value,resolve:x=>{running--;resolve(x);}}));},(result,value)=>paint.push([result,value]),assert.fail,10000);
 queue.request('old');clearTimeout(queue.timer);const first=queue.flush();queue.request('middle');queue.request('latest');jobs[0].resolve('obsolete');await first;assert.deepEqual(paint,[]);clearTimeout(queue.timer);const last=queue.flush();assert.equal(jobs[1].value,'latest');jobs[1].resolve('current');await last;assert.equal(peak,1);assert.deepEqual(paint,[['current','latest']]);queue.cancel();
});
test('switching document cancels pending success and failure without leaking a preview',async()=>{
 let resolve,reject;const painted=[];const q=new LatestPreview(()=>new Promise((a,b)=>{resolve=a;reject=b;}),x=>painted.push(x),x=>painted.push(x),10000);
 q.request('A');clearTimeout(q.timer);let run=q.flush();q.cancel();resolve('late A');await run;assert.deepEqual(painted,[]);
 q.request('B');clearTimeout(q.timer);run=q.flush();q.cancel();reject(Error('late error'));await run;await pause();assert.deepEqual(painted,[]);
});

function closeHarness(){
 const fs=require('node:fs'),vm=require('node:vm'),source=fs.readFileSync(require.resolve('../web/studio.js'),'utf8'),elements=new Map();
 const $=id=>{if(!elements.has(id))elements.set(id,{open:false,textContent:'',onclick:null,addEventListener(){},showModal(){assert(!this.open);this.open=true;},close(){this.open=false;}});return elements.get(id);};
 const docs=new (require('../web/session.js').Documents)();docs.add({dirty:true});docs.add({dirty:false});
 const context=vm.createContext({$,documents:docs,clone:structuredClone,blankDocument:{dirty:false},clearLivePreview(){},saveActiveDocument(){},applyDocument(s){context.dirty=s.dirty;},activateDocument(id){docs.activate(id);context.dirty=docs.current().state.dirty;},tr:s=>s,status:s=>context.message=s});context.busy=false;context.dirty=false;context.closingDocument=null;context.newDocument=()=>{};
 vm.runInContext(source.slice(source.indexOf('function closeDocument('),source.indexOf('window.skyLabCanClose=')),context);return{context,docs,$,run:code=>vm.runInContext(code,context)};
}
test('closing a tab explains busy state, prompts once, and discards only the requested tab',()=>{
 const h=closeHarness();h.run('busy=true;requestCloseDocument(1)');assert.match(h.context.message,/Aguarde/);assert.equal(h.docs.items.length,2);
 h.run('busy=false;requestCloseDocument(1);requestCloseDocument(1)');assert(h.$('close-project-dialog').open);h.$('discard-project-close').onclick();assert.deepEqual(h.docs.items.map(d=>d.id),[2]);assert(!h.$('close-project-dialog').open);
});
test('cancelled save keeps the tab open and visibly explains why',async()=>{
 const h=closeHarness();h.$('save').onclick=async()=>{};h.run('requestCloseDocument(1)');await h.$('save-project-close').onclick();assert.equal(h.docs.items.length,2);assert(h.$('close-project-dialog').open);assert.match(h.$('project-close-message').textContent,/salvamento/);
});
test('successful save closes the intended tab and restores the remaining active document',async()=>{
 const h=closeHarness();h.$('save').onclick=async()=>{h.context.dirty=false;};h.run('requestCloseDocument(1)');await h.$('save-project-close').onclick();assert.deepEqual(h.docs.items.map(d=>d.id),[2]);assert.equal(h.docs.active,2);
});

test('responsive panel placement scales with the viewport and returns without accumulated drift',()=>{
 const {fitPanel}=require('../web/session.js'),from={width:1000,height:600},to={width:1600,height:960},rect={x:600,y:100,width:320,height:400};
 const large=fitPanel(rect,from,to);assert.equal(large.width,512);assert.equal(large.height,640);assert(large.x>rect.x);assert(large.y>rect.y);
 assert.deepEqual(fitPanel(rect,from,from),rect);
 const small=fitPanel({x:999,y:599,width:900,height:800},from,{width:800,height:500});assert(small.x>=0&&small.y>=0);assert(small.x+small.width<=800);assert(small.y+small.height<=500);
});

test('generation preview sends only its selected polygon, never the loaded flight vertices',async()=>{
 const fs=require('node:fs'),vm=require('node:vm'),source=fs.readFileSync(require.resolve('../web/studio.js'),'utf8'),start=source.indexOf('async function calculatePreview('),end=source.indexOf('const previewQueue=',start);let sent;
 const ctx=vm.createContext({api:async(op,body)=>{sent={op,...body};return{...body.project,polygons:[],lines:[{name:'L100',points:[[1,1],[2,2]]}]};}});vm.runInContext(source.slice(start,end),ctx);
 const polygon={outer:[[1,1],[2,1],[2,2],[1,1]],holes:[],layer:7,name:'Source'},p={version:6,name:'Saved project',crs:{zone:23,south:true},layers:[{id:7}],polygons:[polygon,polygon],lines:Array.from({length:500},()=>({points:Array(50).fill([500000,7800000])}))};
 await ctx.calculatePreview({mode:'generate',project:p,polygon:1,settings:{spacing:100}});assert.equal(sent.op,'generate-preview');assert.equal(sent.project.lines.length,0);assert.equal(sent.project.polygons.length,1);assert.equal(sent.project.polygons[0].name,undefined);assert(JSON.stringify(sent).length<JSON.stringify(p).length/100);assert.equal(p.lines.length,500);
});
