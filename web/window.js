'use strict';
(() => {
  const bridge=window.chrome?.webview;if(!bridge)return;
  document.documentElement.classList.add('native-window');
  const header=document.querySelector('header');if(!header)return;
  header.addEventListener('mousedown',e=>{if(e.button!==0||e.target.closest('button,input,select,textarea,a,label'))return;e.preventDefault();bridge.postMessage(e.detail===2?'maximize':'drag');});
  const layout=()=>bridge.postMessage('caption:'+header.getBoundingClientRect().height+':'+window.devicePixelRatio);
  new ResizeObserver(layout).observe(header);window.addEventListener('resize',layout);layout();
  bridge.addEventListener('message',e=>{if(e.data==='maximized'||e.data==='restored')document.documentElement.classList.toggle('window-maximized',e.data==='maximized');});
})();
