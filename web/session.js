(function(root){
 'use strict';
 class Documents {
  constructor(limit=3){this.limit=limit;this.items=[];this.active=null;this.nextId=1;}
  add(state){if(this.items.length>=this.limit)throw Error('document_limit');const item={id:this.nextId++,state};this.items.push(item);this.active=item.id;return item;}
  current(){return this.items.find(item=>item.id===this.active);}
  activate(id){const item=this.items.find(item=>item.id===id);if(!item)throw Error('unknown_document');this.active=id;return item;}
  remove(id){const index=this.items.findIndex(item=>item.id===id);if(index<0)return;this.items.splice(index,1);if(this.active===id)this.active=this.items[Math.min(index,this.items.length-1)]?.id??null;}
 }
 // Only the newest result may be displayed; there is never more than one calculation in flight.
 class LatestPreview {
  constructor(run,accept,reject,delay=45){this.run=run;this.accept=accept;this.reject=reject;this.delay=delay;this.revision=0;this.pending=null;this.running=false;this.timer=null;}
  request(value){const revision=++this.revision;this.pending={value,revision};if(!this.running&&!this.timer)this.timer=setTimeout(()=>this.flush(),this.delay);return revision;}
  cancel(){this.revision++;this.pending=null;clearTimeout(this.timer);this.timer=null;}
  async flush(){this.timer=null;if(this.running||!this.pending)return;const job=this.pending;this.pending=null;this.running=true;try{const result=await this.run(job.value);if(job.revision===this.revision)this.accept(result,job.value);}catch(error){if(job.revision===this.revision)this.reject(error,job.value);}finally{this.running=false;if(this.pending)this.timer=setTimeout(()=>this.flush(),this.delay);}}
 }
 const api={Documents,LatestPreview};if(typeof module!=='undefined'&&module.exports)module.exports=api;else root.SkyLabSession=api;
})(typeof window==='undefined'?globalThis:window);
