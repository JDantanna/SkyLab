const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm');
const source=fs.readFileSync(require.resolve('../web/window.js'),'utf8');
test('native splash stays visible until the host is ready and paints before fading; save messages do not dismiss it',()=>{
 const listeners={},frames=[],timeouts=[],classes=[],messages=[];let removed=0;
 const splash={hidden:true,style:{setProperty(name,value){assert.equal(name,'--startup-logo-size');assert.equal(value,'64px');}},classList:{add(name){classes.push(name);}},addEventListener(name,fn){listeners[name]=fn;},remove(){removed++;}};
 const header={addEventListener(){},getBoundingClientRect:()=>({height:50})};
 const bridge={addEventListener(name,fn){listeners[name]=fn;},postMessage:value=>messages.push(value)};
 vm.runInNewContext(source,{window:{chrome:{webview:bridge},devicePixelRatio:2,addEventListener(){}},document:{documentElement:{classList:{add(){}}},getElementById:()=>splash,querySelector:()=>header},ResizeObserver:class{observe(){}},requestAnimationFrame:fn=>frames.push(fn),setTimeout:fn=>timeouts.push(fn)});
 assert.equal(splash.hidden,false);assert.equal(removed,0);assert.equal(messages[0],'caption:50:2');
 listeners.message({data:'file-saved'});assert.equal(frames.length,0);
 listeners.message({data:'startup-ready'});assert.equal(classes.length,0);frames.shift()();assert.equal(classes.length,0);frames.shift()();assert.deepEqual(classes,['startup-finished']);
 assert.equal(removed,0);listeners.transitionend();assert.equal(removed,1);assert.equal(timeouts.length,1);
});
test('regular browser content does not wait for a native startup handshake',()=>{
 vm.runInNewContext(source,{window:{},document:{getElementById(){assert.fail('Native splash should not be activated');}}});
});

test('page-painted handshake waits for logo decoding and two painted frames before native cover can be removed',async()=>{
 const events={},frames=[],messages=[];let finishFont,finishImage;
 const fonts=new Promise(resolve=>finishFont=resolve),image=new Promise(resolve=>finishImage=resolve);
 const splash={hidden:true,style:{setProperty(){}},classList:{add(){}},addEventListener(){},remove(){}};
 vm.runInNewContext(source,{window:{chrome:{webview:{addEventListener(){},postMessage:x=>messages.push(x)}},devicePixelRatio:1,addEventListener:(name,fn)=>events[name]=fn},document:{readyState:'loading',fonts:{ready:fonts},documentElement:{classList:{add(){}}},getElementById:()=>splash,querySelector:()=>null,querySelectorAll:()=>[{decode:()=>image}]},requestAnimationFrame:fn=>frames.push(fn),setTimeout(){}});
 const painted=events.load();assert.deepEqual(messages,[]);assert.equal(frames.length,0);finishFont();await new Promise(setImmediate);assert.equal(frames.length,0);finishImage();await painted;
 assert.equal(frames.length,1);frames.shift()();assert.deepEqual(messages,[]);frames.shift()();assert.deepEqual(messages,['startup-painted']);
});
