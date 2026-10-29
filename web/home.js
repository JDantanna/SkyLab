'use strict';
const home=document.getElementById('home-screen'),workspaceMain=document.querySelector('body>main'),projectTabs=document.querySelector('.project-tabs');
let firstWorkspace=true;
function syncHome(){const active=!!documents.current();home.hidden=active;document.body.classList.toggle("home-active",!active);workspaceMain.hidden=!active;projectTabs.hidden=!active;if(active&&firstWorkspace){firstWorkspace=false;requestAnimationFrame(()=>{draw();if(!localStorage.getItem('skylab.tour.completed')&&!sessionStorage.getItem('skylab.tour.snoozed'))setTimeout(()=>{if(documents.current()&&!document.querySelector('dialog[open]'))startTour();},600);});}window.refreshHotkeyHints?.();}
const beforeHomeTabs=renderTabs;renderTabs=function(){beforeHomeTabs();syncHome();};window.skyLabRefreshTabs=renderTabs;
const beforeHomeCommit=commit;commit=function(...args){if(!documents.current()){documents.add(clone(blankDocument));syncHome();}return beforeHomeCommit(...args);};
const beforeHomeButtons=refreshButtons;refreshButtons=function(){beforeHomeButtons();$('home-new').disabled=busy;$('home-open').disabled=busy;};
$('home-new').onclick=()=>newDocument();
function openProjectFile(){if(!busy)$('file').click();}
$('home-open').onclick=openProjectFile;$('open-file').onclick=openProjectFile;
home.addEventListener('dragover',event=>{event.preventDefault();home.classList.add('drop-active');});
home.addEventListener('dragleave',()=>home.classList.remove('drop-active'));
const beforeHomeTour=startTour;startTour=function(){if(!documents.current())newDocument();beforeHomeTour();};$('start-tour').onclick=startTour;
renderTabs();
