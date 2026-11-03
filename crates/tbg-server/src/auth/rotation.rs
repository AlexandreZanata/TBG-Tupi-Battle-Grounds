//! Atomic issuance and bounded same-response retry; never extend absolute family expiry.

use super::*;

struct ConsumedAttempt {
    old: CredentialHash,
    operation: CredentialHash,
    family: Family,
    at: i64,
    successor: CredentialHash,
}

impl Service {
    pub(super) async fn login(
        &self,
        key: &str,
        device: &str,
        persistent: bool,
        channel: Channel,
        ip: IpAddr,
    ) -> Result<Grant, Error> {
        let (name, secret) = tbg_core::account::parse_key(key).ok_or(Error::Unauthorized)?;
        let device = tokens::decode::<16>(device)?;
        let now = self.now()?;
        let mut state = self.state.lock().await;
        self.prune(&mut state, now);
        self.rate(&mut state, ip, true)?;
        self.maintenance(&mut state, now).await;
        self.issue(
            &mut state,
            NewFamily {
                name,
                key_hash: tokens::hash(&secret),
                register: false,
                email: None,
                device,
                persistent,
                channel,
            },
            now,
            None,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)] // Registration binds enrollment and retry proof to the same channel-specific grant.
    pub(super) async fn register(
        &self,
        name: String,
        address: String,
        locale: String,
        device: &str,
        persistent: bool,
        retry: &str,
        channel: Channel,
        ip: IpAddr,
    ) -> Result<Grant, Error> {
        tbg_core::account::check_name(&name).map_err(|_| Error::NameInvalid)?;
        if address.is_empty() {
            return Err(Error::EmailRequired);
        }
        let email = crate::store::recovery::service::Enrollment {
            email: crate::store::recovery::email::normalize(&address)
                .map_err(|_| Error::EmailInvalid)?,
            locale,
        };
        if !matches!(email.locale.as_str(), "en" | "pt-BR") {
            return Err(Error::EmailInvalid);
        }
        let device = tokens::decode::<16>(device)?;
        let retry = tokens::hash(&tokens::decode::<32>(retry)?);
        let fingerprint = tokens::hash(
            format!(
                "{}:{}:{}:{}:{}:{}",
                name,
                email.email,
                email.locale,
                tokens::hex(&device),
                persistent,
                channel.as_str()
            )
            .as_bytes(),
        );
        let now = self.now()?;
        let mut state = self.state.lock().await;
        self.prune(&mut state, now);
        self.rate(&mut state, ip, false)?;
        if let Some(cached) = state.registrations.get(&retry) {
            if cached.fingerprint != fingerprint {
                return Err(Error::Conflict);
            }
            if state
                .flags
                .get(&cached.grant.family.id)
                .is_some_and(|f| f.revoked.load(Ordering::Acquire))
            {
                return Err(Error::Unauthorized);
            }
            let hash = tokens::hash(&tokens::decode::<32>(&cached.grant.refresh)?);
            self.store()?
                .auth(db::Request::Confirm { hash, channel, now })
                .await?;
            return Ok(cached.grant.clone());
        }
        let address = email.email.clone();
        let locale = email.locale.clone();
        self.store()?
            .release(move |db| crate::store::recovery::service::registration(db, &address, &locale))
            .await
            .ok_or(Error::Unavailable)?
            .map_err(db::Error::from)?;
        self.cache_capacity(&state, None)?;
        if !state.login_limits.check_register(
            ip,
            self.config.register_per_ip_hour.min(u64::from(u32::MAX)) as u32,
            Instant::now(),
        ) {
            return Err(Error::RateLimited);
        }
        let secret = match tokens::random::<10>() {
            Ok(secret) => tbg_core::account::encode_secret(&secret),
            Err(error) => {
                state.login_limits.refund_register(ip);
                return Err(error);
            }
        };
        let key = tbg_core::account::format_key(&name, &secret);
        match self
            .issue(
                &mut state,
                NewFamily {
                    name,
                    key_hash: tokens::hash(&secret),
                    register: true,
                    email: Some(email),
                    device,
                    persistent,
                    channel,
                },
                now,
                Some(key),
            )
            .await
        {
            Ok(grant) => {
                state.registrations.insert(
                    retry,
                    CachedRegistration {
                        fingerprint,
                        until: now + RETRY_SECONDS,
                        deadline: Instant::now() + Duration::from_secs(RETRY_SECONDS as u64),
                        grant: grant.clone(),
                    },
                );
                Ok(grant)
            }
            Err(error) => {
                state.login_limits.refund_register(ip);
                Err(error)
            }
        }
    }

    pub(super) async fn issue(
        &self,
        state: &mut State,
        request: NewFamily,
        now: i64,
        key: Option<String>,
    ) -> Result<Grant, Error> {
        self.capacity(None)?;
        let family = tokens::random::<16>()?;
        let secrets = FreshTokens {
            access: tokens::random::<32>()?,
            refresh: tokens::random::<32>()?,
        };
        let email = request.email;
        let issuance = db::Issuance {
            name: request.name,
            key_hash: request.key_hash,
            register: request.register,
            family,
            device: request.device,
            channel: request.channel,
            persistent: request.persistent,
            refresh_hash: tokens::hash(&secrets.refresh),
            now,
        };
        let request = match email {
            Some(email) => db::Request::IssueEmail {
                issue: issuance,
                email,
            },
            None => db::Request::Issue(issuance),
        };
        let reply = self.store()?.auth(request).await?;
        match self.publish(state, reply, secrets, now, key) {
            Ok(grant) => Ok(grant),
            Err(error) => {
                self.revoke_orphan(state, family, now).await?;
                Err(error)
            }
        }
    }

    pub(super) async fn refresh(
        &self,
        refresh: &str,
        operation: &str,
        channel: Channel,
        ip: IpAddr,
    ) -> Result<Grant, Error> {
        let old = tokens::hash(&tokens::decode::<32>(refresh)?);
        let operation = tokens::hash(&tokens::decode::<16>(operation)?);
        let now = self.now()?;
        let mut state = self.state.lock().await;
        self.prune(&mut state, now);
        self.rate(&mut state, ip, false)?;
        let inspected = self
            .store()?
            .auth(db::Request::Inspect {
                hash: old,
                channel,
                now,
            })
            .await?;
        let family = match inspected {
            db::Reply::Grant { family, .. } => family,
            db::Reply::Consumed {
                family,
                at,
                successor,
                ..
            } => {
                self.family_rate(&mut state, family.id, now)?;
                return self
                    .retry(
                        &mut state,
                        ConsumedAttempt {
                            old,
                            operation,
                            family,
                            at,
                            successor,
                        },
                        now,
                    )
                    .await;
            }
            _ => return Err(Error::Unavailable),
        };
        if state
            .flags
            .get(&family.id)
            .is_some_and(|f| f.revoked.load(Ordering::Acquire))
        {
            return Err(Error::Unauthorized);
        }
        self.family_rate(&mut state, family.id, now)?;
        self.capacity(Some(family.id))?;
        self.cache_capacity(&state, Some(family.id))?;
        let secrets = FreshTokens {
            access: tokens::random::<32>()?,
            refresh: tokens::random::<32>()?,
        };
        let successor = tokens::hash(&secrets.refresh);
        let result = self
            .store()?
            .auth(db::Request::Rotate {
                old_hash: old,
                new_hash: successor,
                operation_hash: operation,
                channel,
                now,
            })
            .await?;
        let db::Reply::Grant { family, .. } = &result else {
            // A cancelled/restarted service may have committed a generation whose
            // raw successor is intentionally unrecoverable from hash-only SQL.
            return Err(Error::RecoveryRequired);
        };
        let id = family.id;
        let grant = match self.publish(&mut state, result, secrets, now, None) {
            Ok(grant) => grant,
            Err(error) => {
                self.revoke_orphan(&mut state, id, now).await?;
                return Err(error);
            }
        };
        state.rotations.insert(
            id,
            CachedRotation {
                old,
                operation,
                successor,
                until: now + RETRY_SECONDS,
                deadline: Instant::now() + Duration::from_secs(RETRY_SECONDS as u64),
                grant: grant.clone(),
            },
        );
        Ok(grant)
    }

    async fn retry(
        &self,
        state: &mut State,
        attempt: ConsumedAttempt,
        now: i64,
    ) -> Result<Grant, Error> {
        let ConsumedAttempt {
            old,
            operation,
            family,
            at,
            successor,
        } = attempt;
        if state
            .flags
            .get(&family.id)
            .is_some_and(|f| f.revoked.load(Ordering::Acquire))
        {
            return Err(Error::Unauthorized);
        }
        if now.saturating_sub(at) >= RETRY_SECONDS {
            self.revoke(state, vec![family.id], false, now, "refresh_reuse")
                .await?;
            return Err(Error::Unauthorized);
        }
        let Some(cached) = state.rotations.get(&family.id) else {
            return Err(Error::RecoveryRequired);
        };
        if cached.old != old || cached.successor != successor {
            return Err(Error::Conflict);
        }
        if cached.operation != operation {
            return Err(Error::Conflict);
        }
        Ok(cached.grant.clone())
    }

    pub(super) async fn confirm(
        &self,
        refresh: &str,
        channel: Channel,
        ip: IpAddr,
    ) -> Result<FamilyId, Error> {
        let hash = tokens::hash(&tokens::decode::<32>(refresh)?);
        let now = self.now()?;
        let mut state = self.state.lock().await;
        self.prune(&mut state, now);
        self.rate(&mut state, ip, false)?;
        let db::Reply::Grant { family, .. } = self
            .store()?
            .auth(db::Request::Confirm { hash, channel, now })
            .await?
        else {
            return Err(Error::Unauthorized);
        };
        if state
            .flags
            .get(&family.id)
            .is_some_and(|flag| flag.revoked.load(Ordering::Acquire))
        {
            return Err(Error::Unauthorized);
        }
        Ok(family.id)
    }
}
