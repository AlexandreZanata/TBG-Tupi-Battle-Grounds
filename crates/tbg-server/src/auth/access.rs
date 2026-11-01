//! Access lookup and revocation barriers; natural expiry never revokes a match lease.

use super::*;
use tokio::sync::oneshot;

impl Service {
    pub(crate) fn authenticate(&self, token: [u8; 32]) -> Result<Binding, Error> {
        let now = self.now()?;
        self.access
            .read()
            .map_err(|_| Error::Unavailable)?
            .get(&tokens::hash(&token))
            .filter(|b| b.valid(now))
            .cloned()
            .ok_or(Error::Unauthorized)
    }

    async fn idle(&self, ids: &[FamilyId]) -> Result<(), Error> {
        let (reply, rx) = oneshot::channel();
        tokio::time::timeout(Duration::from_secs(5), async {
            self.lobby
                .send(LobbyCmd::AuthIdle {
                    families: ids.to_vec(),
                    reply,
                })
                .await
                .map_err(|_| Error::Unavailable)?;
            if rx.await.map_err(|_| Error::Unavailable)? {
                Ok(())
            } else {
                Err(Error::InMatch)
            }
        })
        .await
        .map_err(|_| Error::Unavailable)?
    }

    pub(super) async fn revoke(
        &self,
        state: &mut State,
        ids: Vec<FamilyId>,
        ordinary: bool,
        now: i64,
        reason: &'static str,
    ) -> Result<(), Error> {
        if ordinary {
            self.idle(&ids).await?;
        }
        self.revoke_scope(state, RevocationScope::Families(ids), now, reason)
            .await
    }

    async fn revoke_scope(
        &self,
        state: &mut State,
        scope: RevocationScope,
        now: i64,
        reason: &'static str,
    ) -> Result<(), Error> {
        // Retain account metadata with live flags, including naturally expired
        // admitted-match leases no longer present in the Access index.
        for (id, flag) in &state.flags {
            if scope.includes(*id, flag.account) {
                flag.revoked.store(true, Ordering::Release);
            }
        }
        let request = match &scope {
            RevocationScope::Families(ids) => db::Request::Revoke {
                families: ids.clone(),
                now,
                reason,
            },
            RevocationScope::Account(account) => db::Request::RevokeAccount {
                account: *account,
                now,
            },
        };
        self.store()?.auth(request).await?;
        self.access
            .write()
            .map_err(|_| Error::Unavailable)?
            .retain(|_, b| !scope.includes(b.family, b.account));
        state
            .rotations
            .retain(|id, c| !scope.includes(*id, c.grant.family.account));
        state
            .registrations
            .retain(|_, c| !scope.includes(c.grant.family.id, c.grant.family.account));
        let (reply, rx) = oneshot::channel();
        let revoke = scope.clone();
        tokio::time::timeout(Duration::from_secs(5), async {
            self.lobby
                .send(LobbyCmd::AuthRevoke {
                    scope: revoke,
                    reply,
                })
                .await
                .map_err(|_| Error::Unavailable)?;
            if rx.await.map_err(|_| Error::Unavailable)? {
                Ok(())
            } else {
                Err(Error::Unavailable)
            }
        })
        .await
        .map_err(|_| Error::Unavailable)??;
        let removed: Vec<_> = state
            .flags
            .iter()
            .filter(|(id, f)| scope.includes(**id, f.account))
            .map(|(id, _)| *id)
            .collect();
        for id in removed {
            state.flags.remove(&id);
            state.family_rates.remove(&id);
        }
        Ok(())
    }

    /// Account-security changes share this barrier, including admitted-match leases.
    pub(crate) async fn revoke_account(
        &self,
        account: AccountId,
        reason: &'static str,
    ) -> Result<(), Error> {
        let now = self.now()?;
        let mut state = self.state.lock().await;
        self.revoke_scope(&mut state, RevocationScope::Account(account), now, reason)
            .await
    }

    pub(super) async fn revoke_orphan(
        &self,
        state: &mut State,
        id: FamilyId,
        now: i64,
    ) -> Result<(), Error> {
        self.revoke(state, vec![id], false, now, "issuance_failed")
            .await
    }

    fn current_access(&self, bearer: Option<&str>, now: i64) -> Result<Option<Binding>, Error> {
        let Some(token) = bearer else {
            return Ok(None);
        };
        let token = tokens::decode::<32>(token)?;
        Ok(self
            .access
            .read()
            .map_err(|_| Error::Unavailable)?
            .get(&tokens::hash(&token))
            .filter(|b| b.valid(now))
            .cloned())
    }

    pub(super) async fn logout(
        &self,
        channel: Channel,
        refresh: Option<&str>,
        bearer: Option<&str>,
        ip: IpAddr,
    ) -> Result<(), Error> {
        let now = self.now()?;
        let mut state = self.state.lock().await;
        self.prune(&mut state, now);
        self.rate(&mut state, ip, false)?;
        let binding = self.current_access(bearer, now)?;
        let family = if let Some(refresh) = refresh {
            let hash = tokens::hash(&tokens::decode::<32>(refresh)?);
            let family = match self
                .store()?
                .auth(db::Request::LogoutFamily { hash, channel })
                .await
            {
                Ok(db::Reply::LogoutFamily(family)) => family.id,
                Err(db::Error::Unauthorized) if binding.is_some() => {
                    binding.as_ref().ok_or(Error::Unauthorized)?.family
                }
                Err(error) => return Err(error.into()),
                _ => return Err(Error::Unavailable),
            };
            if binding.as_ref().is_some_and(|b| b.family != family) {
                return Err(Error::Unauthorized);
            }
            family
        } else {
            binding.ok_or(Error::Unauthorized)?.family
        };
        self.revoke(&mut state, vec![family], true, now, "logout")
            .await
    }

    pub(super) async fn logout_all(
        &self,
        key: &str,
        bearer: Option<&str>,
        cookie: Option<&str>,
        ip: IpAddr,
    ) -> Result<bool, Error> {
        let (name, secret) = tbg_core::account::parse_key(key).ok_or(Error::Unauthorized)?;
        let now = self.now()?;
        let mut state = self.state.lock().await;
        self.prune(&mut state, now);
        self.rate(&mut state, ip, true)?;
        let binding = self.current_access(bearer, now)?;
        let (id, _, _) = self
            .store()?
            .login_fresh(name.to_ascii_lowercase(), tokens::hash(&secret))
            .await
            .map_err(db::Error::from)?;
        if bearer.is_some() && binding.as_ref().is_none_or(|b| b.account != id) {
            return Err(Error::Unauthorized);
        }
        let clear_cookie = if let Some(cookie) = cookie {
            let hash = tokens::hash(&tokens::decode::<32>(cookie)?);
            matches!(self.store()?.auth(db::Request::LogoutFamily{hash,channel:Channel::Browser}).await,Ok(db::Reply::LogoutFamily(family)) if family.account==id)
        } else {
            false
        };
        drop(state);
        self.revoke_account(id, "logout_all").await?;
        Ok(clear_cookie)
    }
}
