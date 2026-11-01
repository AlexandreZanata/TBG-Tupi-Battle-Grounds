//! Ranking DOM presentation. Public player data only enters text nodes and encoded profile links.
(() => {
  'use strict';
  const D=globalThis.TbgRankingData;
  const I=()=>globalThis.TBG_I18N,t=(key,args)=>I().t(key,args);
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
      else {link.textContent=t(place===1?'ranking.web.be_first':'ranking.web.open_place');link.removeAttribute('href');}
      card.querySelector('[data-podium-rating]').textContent=p?p.rating:'—';
      card.querySelector('[data-podium-record]').textContent=p?[p.wins,p.losses,p.draws].join(' · '):'—';
      card.setAttribute('aria-label',p?t('ranking.web.podium_label',{rank:p.rank,name:p.name,rating:p.rating}):t('ranking.web.podium_empty',{rank:place}));
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
    if(!rows.length){const row=node('tr'),cell=node('td',t(data.total?'ranking.web.page_empty':'ranking.web.none'),'ranking-empty');cell.colSpan=7;row.append(cell);rows.push(row);}
    get(root,'rows').replaceChildren(...rows);
    text(root,'status',data.total?I().plural('ranking.web.status',data.total,{n:I().number(data.total),from:at+1,to:at+data.rows.length}):t('ranking.web.first_champion'));
    get(root,'status').classList.remove('ranking-error');
    const date=new Date(data.updated_at);
    text(root,'updated',Number.isNaN(date.getTime())?'':t('ranking.web.updated',{date:new Intl.DateTimeFormat(I().lang,{dateStyle:'medium'}).format(date)}));
  }
  function pager(root,at,total,loading){
    const p=D.pages(at,total),buttons=[];
    const add=(label,offset,disabled,current)=>{
      const b=node('button',label);b.type='button';b.dataset.rankingOffset=offset;b.disabled=loading||disabled;
      if(current)b.setAttribute('aria-current','page');
      buttons.push(b);return b;
    };
    add('«',0,at===0).setAttribute('aria-label',t('ranking.web.first_page'));
    add('‹',p.previous,at===0).setAttribute('aria-label',t('ranking.web.previous_page'));
    for(const offset of p.numbered)add(String(offset/D.PAGE+1),offset,false,offset===at);
    add('›',p.next,at>=p.last).setAttribute('aria-label',t('ranking.web.next_page'));
    add('»',p.last,at>=p.last).setAttribute('aria-label',t('ranking.web.last_page'));
    get(root,'pager').replaceChildren(...buttons);
    root.querySelector('[data-ranking-action="refresh"]').disabled=loading;
    root.querySelector('.ranking-table').setAttribute('aria-busy',String(loading));
  }
  function recent(root,p,game){
    const rows=(p?p.recent:[]).slice(0,5).map(m=>{
      // `result` is the stored W/L/D code; only its label is translated.
      const classes={W:'ranking-wins',L:'ranking-losses',D:'ranking-draw'};
      const li=node('li');
      const opponent=node('a');profileLink(opponent,m.opponent,game);opponent.textContent=t('ranking.web.vs',{name:m.opponent});
      li.append(opponent,node('b',t('ranking.result.'+m.result.toLowerCase()),classes[m.result]),node('span',D.delta(m.delta),'ranking-delta'));return li;
    });
    if(!rows.length)rows.push(node('li',t(p?'ranking.web.recent_none':'ranking.web.recent_signin'),'ranking-empty-recent'));
    get(root,'recent').replaceChildren(...rows);
  }
  function profile(root,p,name,game,status){
    text(root,'profile-title',t(!game&&name?'ranking.web.player_record':'ranking.web.your_rank'));
    text(root,'player-name',p?p.name:name||t('ranking.web.guest'));
    text(root,'rank',status?t(status):(!p||p.rank===null?t('ranking.web.unranked'):t('ranking.web.global_rank',{rank:p.rank})));
    text(root,'rating',p?p.rating:'—');text(root,'wins',p?p.wins:'—');text(root,'losses',p?p.losses:'—');text(root,'win-rate',p?D.percent(p.win_rate):'—');
    text(root,'profile-note',p?(p.games?I().plural('ranking.web.matches',p.games):t('ranking.web.enter')):t('ranking.web.build'));
    const play=get(root,'play');
    text(root,'play-label',t(game?(name?'ranking.web.play_ranked':'ranking.web.create_account'):'ranking.web.enter_game'));
    if(game)play.dataset.action=name?'ranked':'register';
    const link=get(root,'profile-link');link.hidden=!p;
    if(p){link.href='/ranking/player.html?name='+encodeURIComponent(p.name);if(game){link.target='_blank';link.rel='noopener';}}
    recent(root,p,game);
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
