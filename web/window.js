'use strict';
(() => {
  const bridge=window.chrome?.webview;if(!bridge)return;
  window.skyMaximizeProject=()=>bridge.postMessage('maximize-project');
  document.documentElement.classList.add('native-window');
  const splash=document.getElementById('startup-screen');
  if(splash){
    splash.hidden=false;splash.style.setProperty('--startup-logo-size',(128/window.devicePixelRatio)+'px');
    bridge.addEventListener('message',event=>{if(event.data!=='startup-ready')return;
      requestAnimationFrame(()=>requestAnimationFrame(()=>{
        splash.classList.add('startup-finished');
        const finish=()=>splash.remove();splash.addEventListener('transitionend',finish,{once:true});setTimeout(finish,1100);
      }));
    });
  }
  async function painted(){
    try{await document.fonts?.ready;await Promise.all([...document.querySelectorAll('#startup-screen img,.home-logo')].map(image=>image.decode?.().catch(()=>{})));}catch{}
    requestAnimationFrame(()=>requestAnimationFrame(()=>bridge.postMessage('startup-painted')));
  }
  if(document.readyState==='complete')painted();else window.addEventListener('load',painted,{once:true});
  const header=document.querySelector('header');if(!header)return;
  header.addEventListener('mousedown',e=>{if(e.button!==0||e.target.closest('button,input,select,textarea,a,label'))return;e.preventDefault();bridge.postMessage(e.detail===2?'maximize':'drag');});
  const layout=()=>bridge.postMessage('caption:'+header.getBoundingClientRect().height+':'+window.devicePixelRatio);
  new ResizeObserver(layout).observe(header);window.addEventListener('resize',layout);layout();
})();
