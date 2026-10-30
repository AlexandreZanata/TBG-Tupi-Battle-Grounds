//! Ranking DOM presentation. Public player data only enters text nodes and encoded profile links.
(() => {
  'use strict';
  const D=globalThis.TbgRankingData;
  const node=(tag,text,cls)=>{const n=document.createElement(tag);if(text!==undefined)n.textContent=String(text);if(cls)n.className=cls;return n;};
  const get=(root,key)=>root.querySelector('[data-r="'+key+'"]');
  const text=(root,key,value)=>{get(root,key).textContent=String(value);};
  function profileLink(anchor,name,game){
    anchor.textContent=name;
    anchor.href='/ranking/player.html?name='+encodeURIComponent(name);
    if(game){anchor.target='_blank';anchor.rel='noopener';}
  }
  function podium(root,leaders,game){
    root.querySelectorAll('[data-podium]').forEach(card=>{
      const place=Number(card.dataset.podium),p=leaders.find(r=>r.rank===place);
      const link=card.querySelector('[data-podium-name]');
      card.classList.toggle('is-empty',!p);
      if(p) profileLink(link,p.name,game);
      else {link.textContent=place===1?'Be the first':'Open place';link.removeAttribute('href');}
      card.querySelector('[data-podium-rating]').textContent=p?p.rating:'—';
      card.querySelector('[data-podium-record]').textContent=p?[p.wins,p.losses,p.draws].join(' · '):'—';
      card.setAttribute('aria-label',p?'Rank '+p.rank+': '+p.name+', '+p.rating+' Rating':'Rank '+place+' awaits a ranked player');
    });
  }
  function table(root,data,at,ownName,crest,game){
    const rows=data.rows.map(p=>{
      const row=node('tr');row.dataset.rankPosition=p.rank;
      row.classList.toggle('ranking-own-row',p.name.toLowerCase()===ownName.toLowerCase());
      const rank=node('td',p.rank,'ranking-rank-cell');
      if(p.rank<=3) rank.classList.add('ranking-place-'+p.rank);
      const player=node('td',undefined,'ranking-player-cell');
      const image=node('img');image.src=crest;image.alt='';image.width=28;image.height=28;
      const link=node('a');profileLink(link,p.name,game);player.append(image,link);
      row.append(rank,player,node('td',p.rating,'ranking-rating'),node('td',p.wins,'ranking-wins'),node('td',p.losses,'ranking-losses'),node('td',p.draws),node('td',D.percent(p.win_rate)));
      return row;
    });
    if(!rows.length){const row=node('tr'),cell=node('td',data.total?'No players on this page.':'No ranked players yet. Play Ranked to earn a place.','ranking-empty');cell.colSpan=7;row.append(cell);rows.push(row);}
    get(root,'rows').replaceChildren(...rows);
    text(root,'status',data.total?data.total+' ranked players · '+(at+1)+'–'+(at+data.rows.length):'The first champion is still to come');
    get(root,'status').classList.remove('ranking-error');
    const date=new Date(data.updated_at);
    text(root,'updated',Number.isNaN(date.getTime())?'':'Last ranked result · '+new Intl.DateTimeFormat('en',{dateStyle:'medium'}).format(date));
  }
  function pager(root,at,total,loading){
    const p=D.pages(at,total),buttons=[];
    const add=(label,offset,disabled,current)=>{
      const b=node('button',label);b.type='button';b.dataset.rankingOffset=offset;b.disabled=loading||disabled;
      if(current)b.setAttribute('aria-current','page');
      buttons.push(b);return b;
    };
    add('«',0,at===0).setAttribute('aria-label','First ranking page');
    add('‹',p.previous,at===0).setAttribute('aria-label','Previous ranking page');
    for(const offset of p.numbered)add(String(offset/D.PAGE+1),offset,false,offset===at);
    add('›',p.next,at>=p.last).setAttribute('aria-label','Next ranking page');
    add('»',p.last,at>=p.last).setAttribute('aria-label','Last ranking page');
    get(root,'pager').replaceChildren(...buttons);
    root.querySelector('[data-ranking-action="refresh"]').disabled=loading;
    root.querySelector('.ranking-table').setAttribute('aria-busy',String(loading));
  }
  function recent(root,p){
    const rows=(p?p.recent:[]).slice(0,5).map(m=>{
      const words={W:['Win','ranking-wins'],L:['Loss','ranking-losses'],D:['Draw','ranking-draw']};
      const [word,cls]=words[m.result],li=node('li');
      const opponent=node('a','vs '+m.opponent);opponent.href='/ranking/player.html?name='+encodeURIComponent(m.opponent);
      li.append(opponent,node('b',word,cls),node('span',D.delta(m.delta),'ranking-delta'));return li;
    });
    if(!rows.length)rows.push(node('li',p?'No ranked matches yet.':'Sign in to see your recent ranked matches','ranking-empty-recent'));
    get(root,'recent').replaceChildren(...rows);
  }
  function profile(root,p,name,game,status){
    text(root,'profile-title',!game&&name?'Player Record':'Your Rank');
    text(root,'player-name',p?p.name:name||'Guest');
    text(root,'rank',status||(!p||p.rank===null?'Unranked':'#'+p.rank+' Global'));
    text(root,'rating',p?p.rating:'—');text(root,'wins',p?p.wins:'—');text(root,'losses',p?p.losses:'—');text(root,'win-rate',p?D.percent(p.win_rate):'—');
    text(root,'profile-note',p?(p.games?p.games+' ranked '+(p.games===1?'match':'matches'):'Play Ranked to enter the global ranking'):'Play Ranked to build your record');
    const play=get(root,'play');
    text(root,'play-label',game?(name?'Play Ranked':'Create account'):'Enter game');
    if(game)play.dataset.action=name?'ranked':'register';
    const link=get(root,'profile-link');link.hidden=!p;
    if(p){link.href='/ranking/player.html?name='+encodeURIComponent(p.name);if(game){link.target='_blank';link.rel='noopener';}}
    recent(root,p);
  }
  function navigation(lobby,open){
    if(!lobby)return;
    lobby.dataset.lobbyView=open?'ranking':'lobby';
    for(const action of ['menu','ranking']){
      const button=lobby.querySelector('.lobby-tabs [data-action="'+action+'"]');
      const current=open?action==='ranking':action==='menu';button.classList.toggle('active',current);
      if(current)button.setAttribute('aria-current','page');else button.removeAttribute('aria-current');
    }
  }
  globalThis.TbgRankingRender=Object.freeze({get,text,podium,table,pager,profile,navigation});
})();
