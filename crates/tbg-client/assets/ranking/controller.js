//! Lazy ranking page loading, read-only API requests and cancellation across navigation/account changes.
(() => {
  'use strict';
  const root=document.getElementById('tbg-ranking');if(!root)return;
  const lobby=document.getElementById('tbg-lobby'),game=Boolean(lobby);
  const D=globalThis.TbgRankingData,R=globalThis.TbgRankingRender;
  const BASE='/assets/ranking/',REV='?v=ranking-1';
  const query=new URLSearchParams(location.search);
  let active=false,ready=false,mounted,art={},at=0,total=0,leaders=null,loading=false,pageId=0,profileId=0,pageRequest,profileRequest;
  const identity=()=>game?(lobby.dataset.player||''):(D.NAME.test(query.get('player')||'')?query.get('player'):'');
  async function json(path,signal){
    const response=await fetch(path,{signal,headers:{Accept:'application/json'}});
    if(response.status===404)return null;
    if(response.status===429)throw new Error('Too many requests. Please wait a minute and refresh.');
    if(!response.ok)throw new Error('The ranking is unavailable. Please refresh.');
    return response.json();
  }
  function notice(message,retryAllowed=true){
    const box=document.createElement('section');box.className='ranking-notice';
    const title=document.createElement('h1');title.textContent='Global Rankings';
    const status=document.createElement('p');status.setAttribute('role','status');status.textContent=message;
    const retry=document.createElement('button');retry.type='button';retry.dataset.rankingAction='refresh';retry.textContent='Try again';
    box.append(title,status);if(retryAllowed)box.append(retry);root.replaceChildren(box);
  }
  function mount(){
    if(mounted)return mounted;
    mounted=Promise.all([fetch(BASE+'panel.html'+REV).then(r=>{if(!r.ok)throw new Error('template');return r.text();}),json(BASE+'ranking.json'+REV)])
      .then(([html,manifest])=>{
        if(!manifest||!Array.isArray(manifest.assets))throw new Error('art');
        for(const a of manifest.assets){if(!/^[a-z0-9-]+$/.test(a.id)||!/^[a-z0-9-]+\.(webp|svg)$/.test(a.asset.display))throw new Error('art');art[a.id]=BASE+a.asset.display;}
        const template=document.createElement('template');
        // Only the repository-owned template enters HTML; API fields are rendered as text.
        template.innerHTML=html;
        const content=template.content.querySelector('[data-ranking-content]');if(!content)throw new Error('template');
        root.replaceChildren(content);root.dataset.context=game?'game':'web';
        root.querySelectorAll('[data-ranking-art]').forEach(img=>{const source=art[img.dataset.rankingArt];if(!source)throw new Error('art');img.src=source;img.decoding='async';});
        if(game)root.querySelector('.ranking-search').target='_blank';
        ready=true;
      }).catch(error=>{ready=false;mounted=null;throw error;});
    return mounted;
  }
  async function loadPage(offset){
    const id=++pageId;pageRequest?.abort();pageRequest=new AbortController();loading=true;
    R.text(root,'status','Loading rankings…');R.pager(root,at,total,true);
    try{
      const first=offset===0||leaders===null;
      const requests=[json('/api/leaderboard?offset='+offset+'&limit='+D.PAGE,pageRequest.signal)];
      if(offset!==0&&first)requests.push(json('/api/leaderboard?offset=0&limit=3',pageRequest.signal));
      const results=await Promise.all(requests);if(!active||id!==pageId)return;
      const data=D.board(results[0]);
      if(offset>D.last(data.total))return loadPage(D.last(data.total));
      if(offset===0)leaders=data.rows.slice(0,3);
      else if(results[1])leaders=D.board(results[1]).rows;
      at=offset;total=data.total;
      R.podium(root,leaders||[],game);R.table(root,data,at,identity(),art['player-crest'],game);
      if(!game){const url=new URL(location.href);url.searchParams.set('offset',at);history.replaceState(null,'',url);}
    }catch(error){
      if(id!==pageId||!active||error.name==='AbortError')return;
      R.text(root,'status',error.message.startsWith('Too many requests')?error.message:'Could not load the ranking. Please refresh.');R.get(root,'status').classList.add('ranking-error');
    }finally{if(id===pageId&&active){loading=false;R.pager(root,at,total,false);}}
  }
  async function loadProfile(){
    const id=++profileId;profileRequest?.abort();profileRequest=new AbortController();
    const name=identity();R.profile(root,null,name,game,name?'Loading…':undefined);
    if(!name)return;
    try{
      const raw=await json('/api/players/'+encodeURIComponent(name),profileRequest.signal);
      if(!active||id!==profileId||name!==identity())return;
      R.profile(root,raw?D.profile(raw):null,name,game,raw?undefined:'Record not found');
    }catch(error){if(active&&id===profileId&&error.name!=='AbortError')R.profile(root,null,name,game,'Record unavailable');}
  }
  async function open(){
    active=true;root.hidden=false;R.navigation(lobby,true);
    if(!mounted)notice('Loading rankings…',false);
    try{await mount();if(!active)return;await Promise.all([loadPage(game?0:D.offset(query.get('offset'))),loadProfile()]);}
    catch{if(active)notice('Could not load the ranking. Please try again.');}
  }
  function close(){
    active=false;root.hidden=true;pageId++;profileId++;pageRequest?.abort();profileRequest?.abort();R.navigation(lobby,false);
  }
  root.addEventListener('click',event=>{
    const button=event.target.closest('button');if(!button||button.disabled)return;
    if(button.hasAttribute('data-ranking-offset')&&!loading)loadPage(D.offset(button.dataset.rankingOffset));
    if(button.dataset.rankingAction==='refresh'){if(!mounted)open();else if(!loading){loadPage(at);loadProfile();}}
    if(button.dataset.rankingAction==='play'&&!game)location.assign('/?panel=ranking');
  });
  if(game)new MutationObserver(()=>{if(active&&ready){loadPage(at);loadProfile();}}).observe(lobby,{attributes:true,attributeFilter:['data-player','data-rating']});
  globalThis.TbgRanking=Object.freeze({open,close});
  if(!game)open();
})();
