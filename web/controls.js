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
