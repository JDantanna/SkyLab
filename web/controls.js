'use strict';
function updateLanguageFlag(){const language=$('language').value,flag=$('current-flag');flag.src='/flags/'+language+'.svg';flag.alt={en:'English','pt-BR':'Português (Brasil)',da:'Dansk'}[language];for(const button of $('language-menu').querySelectorAll('[data-language]'))button.setAttribute('aria-checked',String(button.dataset.language===language));}
function closeLanguages(){$('language-menu').hidden=true;$('language-toggle').setAttribute('aria-expanded','false');}
$('language-toggle').onclick=()=>{const menu=$('language-menu');menu.hidden=!menu.hidden;$('language-toggle').setAttribute('aria-expanded',String(!menu.hidden));if(!menu.hidden)menu.querySelector(`[data-language="${$('language').value}"]`).focus();};
$('language-menu').onclick=e=>{const button=e.target.closest('[data-language]');if(!button)return;$('language').value=button.dataset.language;$('language').dispatchEvent(new Event('change'));updateLanguageFlag();closeLanguages();$('language-toggle').focus();};
$('language').addEventListener('change',updateLanguageFlag);
document.addEventListener('pointerdown',e=>{if(!e.target.closest('.language-control'))closeLanguages();});
document.addEventListener('keydown',e=>{if(e.key==='Escape')closeLanguages();});
const demoAction=$('demo').onclick;$('demo').onclick=()=>{$('help-dialog').close();demoAction();};
updateLanguageFlag();

const appendOption=$('append-info').closest('.append-option'),appendHelp=$('append-explanation');
function hideAppendHelp(){appendHelp.hidden=true;$('append-info').setAttribute('aria-expanded','false');}
function showAppendHelp(){appendHelp.hidden=false;const anchor=appendOption.getBoundingClientRect(),height=appendHelp.offsetHeight;appendHelp.style.left=Math.max(8,Math.min(anchor.left,innerWidth-appendHelp.offsetWidth-8))+'px';appendHelp.style.top=Math.max(8,Math.min(anchor.top-height-9>=8?anchor.top-height-9:anchor.bottom+9,innerHeight-height-8))+'px';$('append-info').setAttribute('aria-expanded','true');}
appendOption.addEventListener('pointerenter',showAppendHelp);
appendOption.addEventListener('pointerleave',()=>{if(!appendOption.contains(document.activeElement))hideAppendHelp();});
appendOption.addEventListener('focusin',showAppendHelp);
appendOption.addEventListener('focusout',event=>{if(!appendOption.contains(event.relatedTarget))hideAppendHelp();});
$('append-info').onclick=showAppendHelp;
document.addEventListener('keydown',event=>{if(event.key==='Escape')hideAppendHelp();});
document.addEventListener('pointerdown',event=>{if(!event.target.closest('.append-option'))hideAppendHelp();});
window.addEventListener('scroll',hideAppendHelp,true);window.addEventListener('resize',hideAppendHelp);

function closeProductMenu(){$('product-menu').hidden=true;$('product-menu-toggle').setAttribute('aria-expanded','false');}
$('product-menu-toggle').onclick=()=>{closeLanguages();const menu=$('product-menu');menu.hidden=!menu.hidden;$('product-menu-toggle').setAttribute('aria-expanded',String(!menu.hidden));if(!menu.hidden)menu.querySelector('button:not(:disabled)').focus();};
$('product-menu').addEventListener('click',e=>{if(e.target.closest('button')&&!e.target.closest('button').disabled)closeProductMenu();});
document.addEventListener('pointerdown',e=>{if(!e.target.closest('.product-menu-anchor'))closeProductMenu();});
for(const id of ['product-menu','language-menu'])$(id).addEventListener('keydown',e=>{const buttons=[...$(id).querySelectorAll('button:not(:disabled)')];if(['ArrowDown','ArrowUp','Home','End'].includes(e.key)){e.preventDefault();const index=buttons.indexOf(document.activeElement);buttons[e.key==='Home'?0:e.key==='End'?buttons.length-1:(index+(e.key==='ArrowUp'?-1:1)+buttons.length)%buttons.length].focus();}if(e.key==='Escape'){e.preventDefault();id==='product-menu'?closeProductMenu():closeLanguages();$(id==='product-menu'?'product-menu-toggle':'language-toggle').focus();}});
