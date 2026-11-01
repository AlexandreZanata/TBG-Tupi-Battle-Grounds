// Boundary for future administrative endpoints. The server has no operator identity, roles or
// admin routes yet (docs/planning/WEB-ADMIN-AUTH-CONTRACT.md), so nothing here can succeed and
// the UI renders every mutation as unavailable instead of pretending it worked.

export const ADMIN_AVAILABLE = false;

/** Why each kind of action is unavailable; keys index the UI strings. */
export const REASONS = {
  auth: 'reason.auth',
  balance: 'reason.balance',
  notes: 'reason.notes',
  moderation: 'reason.moderation',
};

export class AdminUnavailable extends Error {
  constructor(action) {
    super(`administrative action unavailable: ${action}`);
    this.action = action;
  }
}

export async function request(action) {
  throw new AdminUnavailable(action);
}
