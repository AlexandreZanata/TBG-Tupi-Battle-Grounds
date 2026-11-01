// DEMONSTRATION DATA for the local preview layout only. These are not production players or
// matches: every name starts with "Demo" and every match id with "DEMO-". Replace this module
// with real authorized reads only after the server contract exists.

export const FIXTURE_NOTICE = 'fixture';

export const activity = [
  { at: '2026-11-01T14:05:00Z', text: { 'pt-BR': 'Partida DEMO-0042 encerrada (Casual)', en: 'Match DEMO-0042 finished (Casual)' } },
  { at: '2026-11-01T13:52:00Z', text: { 'pt-BR': 'Jogador Demo Gama entrou na fila Ranqueada', en: 'Player Demo Gama joined the Ranked queue' } },
  { at: '2026-11-01T13:40:00Z', text: { 'pt-BR': 'Nota de atualização de exemplo revisada', en: 'Example patch note reviewed' } },
  { at: '2026-11-01T13:15:00Z', text: { 'pt-BR': 'Relato de exemplo aguardando triagem', en: 'Example report awaiting triage' } },
];

export const matches = [
  { id: 'DEMO-0042', a: 'Demo Alfa', b: 'Demo Beta', mode: 'casual', arena: 'forest', seconds: 412, winner: 'a', at: '2026-11-01T14:05:00Z' },
  { id: 'DEMO-0041', a: 'Demo Gama', b: 'Demo Delta', mode: 'ranked', arena: 'volcanic', seconds: 655, winner: 'b', at: '2026-11-01T13:48:00Z' },
  { id: 'DEMO-0040', a: 'Demo Épsilon', b: 'Demo Alfa', mode: 'challenge', arena: 'celestial', seconds: 301, winner: 'a', at: '2026-11-01T13:30:00Z' },
  { id: 'DEMO-0039', a: 'Demo Beta', b: 'Demo Zeta', mode: 'ranked', arena: 'desert', seconds: 540, winner: 'draw', at: '2026-11-01T13:02:00Z' },
  { id: 'DEMO-0038', a: 'Demo Delta', b: 'Demo Gama', mode: 'casual', arena: 'clockwork', seconds: 377, winner: 'b', at: '2026-11-01T12:41:00Z' },
  { id: 'DEMO-0037', a: 'Demo Zeta', b: 'Demo Épsilon', mode: 'ranked', arena: 'storm', seconds: 468, winner: 'a', at: '2026-11-01T12:10:00Z' },
];

export const players = [
  { name: 'Demo Alfa', rating: 1184, matches: 31, status: 'active' },
  { name: 'Demo Beta', rating: 1102, matches: 24, status: 'active' },
  { name: 'Demo Gama', rating: 1067, matches: 19, status: 'active' },
  { name: 'Demo Delta', rating: 1010, matches: 12, status: 'review' },
  { name: 'Demo Épsilon', rating: 998, matches: 9, status: 'active' },
  { name: 'Demo Zeta', rating: 951, matches: 7, status: 'active' },
];
