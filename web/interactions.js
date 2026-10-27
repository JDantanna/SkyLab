(function(root){
  'use strict';
  const interactions={
    range(anchor,end){const out=[];for(let i=Math.min(anchor,end);i<=Math.max(anchor,end);i++)out.push(i);return out;},
    shortcut(event,{modal=false,touring=false,editing=false,busy=false}={}){
      if(modal||touring||editing||busy||event.altKey||!(event.ctrlKey||event.metaKey))return null;
      const key=event.key.toLowerCase();
      if(key==='z')return event.shiftKey?'redo':'undo';
      if(key==='y')return 'redo';
      return null;
    }
  };
  if(typeof module!=='undefined'&&module.exports)module.exports=interactions;
  else root.SkyLabInteractions=interactions;
})(typeof window==='undefined'?globalThis:window);
