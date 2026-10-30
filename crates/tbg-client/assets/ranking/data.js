//! Public ranking data contracts and display paging; the server owns rank order and ratings.
(() => {
  'use strict';
  const PAGE = 10, MAX_OFFSET = 100000;
  const NAME = /^[A-Za-z][A-Za-z0-9_]{2,15}$/;
  function count(value, max = 4294967295) {
    if (!Number.isInteger(value) || value < 0 || value > max) throw new Error('Ranking data is incomplete. Please refresh.');
    return value;
  }
  function name(value) {
    if (typeof value !== 'string' || !NAME.test(value)) throw new Error('Ranking data is incomplete. Please refresh.');
    return value;
  }
  function record(value) {
    if (!value || typeof value !== 'object') throw new Error('Ranking data is incomplete. Please refresh.');
    const rate = value.win_rate;
    if (typeof rate !== 'number' || !Number.isFinite(rate) || rate < 0 || rate > 100) throw new Error('Ranking data is incomplete. Please refresh.');
    return {name:name(value.name),rating:count(value.rating,65535),games:count(value.games),wins:count(value.wins),losses:count(value.losses),draws:count(value.draws),win_rate:rate};
  }
  function board(value) {
    if (!value || !Array.isArray(value.rows) || value.rows.length > 100) throw new Error('Ranking data is incomplete. Please refresh.');
    const rows=value.rows.map(row=>{
      const result={...record(row),rank:count(row.rank)};
      if (result.rank === 0) throw new Error('Ranking data is incomplete. Please refresh.');
      return result;
    });
    return {total:count(value.total),updated_at:typeof value.updated_at==='string'?value.updated_at:'',rows};
  }
  function profile(value) {
    const result=record(value);
    const rank=value.rank===null?null:count(value.rank);
    if (rank===0 || !Array.isArray(value.recent) || value.recent.length>10) throw new Error('Player record is incomplete. Please refresh.');
    const recent=value.recent.map(match=>{
      if (!['W','L','D'].includes(match.result) || !Number.isInteger(match.delta) || Math.abs(match.delta)>65535) throw new Error('Player record is incomplete. Please refresh.');
      return {opponent:name(match.opponent),result:match.result,delta:match.delta};
    });
    return {...result,rank,recent};
  }
  function offset(raw) {
    const value=Number(raw);
    return Number.isFinite(value)?Math.floor(Math.max(0,Math.min(MAX_OFFSET,value))/PAGE)*PAGE:0;
  }
  function last(total) {return Math.min(MAX_OFFSET,Math.floor(Math.max(0,total-1)/PAGE)*PAGE);}
  function pages(at,total) {
    const end=last(total), current=offset(at), start=Math.max(0,Math.min(current-2*PAGE,end-4*PAGE));
    const numbered=[];
    for(let n=start;n<=Math.min(end,start+4*PAGE);n+=PAGE) numbered.push(n);
    return {first:0,previous:Math.max(0,current-PAGE),next:Math.min(end,current+PAGE),last:end,numbered};
  }
  const percent=value=>new Intl.NumberFormat('en',{maximumFractionDigits:1}).format(value)+'%';
  const delta=value=>value>0?'+'+value:value<0?'−'+Math.abs(value):'0';
  const api=Object.freeze({PAGE,NAME,board,profile,offset,last,pages,percent,delta});
  if(typeof module!=='undefined'&&module.exports) module.exports=api;
  else globalThis.TbgRankingData=api;
})();
