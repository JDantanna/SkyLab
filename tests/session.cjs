const test=require('node:test'),assert=require('node:assert/strict');
const {Documents,LatestPreview}=require('../web/session.js');
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
