//! Bind socket and resume identity to one credential epoch, renewing without reconnecting.

use super::*;
use crate::{
    auth,
    platform::storage::{self, Scope},
};

impl NetLink {
    pub(super) fn hello(&self) -> ClientMsg {
        let saved = storage::get(Scope::Session, "tbg_resume_family");
        let saved_origin = storage::get(Scope::Session, "tbg_resume_origin");
        let current_origin = crate::platform::http::origin(&crate::config::server_url());
        let resume = self.token.filter(|_| {
            login::resume_matches(saved.as_deref(), self.auth_family.as_deref())
                && login::origin_matches(saved_origin.as_deref(), current_origin.as_deref())
        });
        match self.auth_access {
            Some(access) => ClientMsg::AuthenticatedHello {
                version: PROTOCOL_VERSION,
                resume,
                access,
            },
            None => ClientMsg::Hello {
                version: PROTOCOL_VERSION,
                resume,
            },
        }
    }
    pub(super) fn save_resume_family(&self) {
        if let Some(origin) = crate::platform::http::origin(&crate::config::server_url()) {
            storage::set(Scope::Session, "tbg_resume_origin", &origin);
        } else {
            storage::remove(Scope::Session, "tbg_resume_origin");
        }
        if let Some(family) = &self.auth_family {
            storage::set(Scope::Session, "tbg_resume_family", family);
        } else {
            storage::remove(Scope::Session, "tbg_resume_family");
        }
    }
    pub(super) fn sync_auth(&mut self, now: f64) {
        let view = auth::snapshot();
        if self.auth_epoch != view.epoch {
            self.socket = None;
            self.opened = false;
            self.outq.clear();
            self.inq.clear();
            self.token = None;
            self.in_match = false;
            self.auth_epoch = view.epoch;
            self.auth_access = None;
            self.auth_family = None;
            // Preserve the original reconnect deadline rather than granting more time.
            self.connect(now);
        }
        if self.socket.is_none() && self.lost_at.is_none() {
            self.connect(now);
        }
        if self.opened && view.access.is_some() && self.auth_access != view.access {
            self.auth_access = view.access;
            self.auth_family = view.family;
            self.login_in_flight = true;
            if let Some(access) = view.access {
                self.send(ClientMsg::Reauthenticate { access }, now);
            }
        }
        if self.opened
            && self.auth_access.is_some()
            && view.manual_claim
            && auth::take_manual_claim()
        {
            self.send(ClientMsg::ClaimAccount { take_over: true }, now);
        }
    }
}
