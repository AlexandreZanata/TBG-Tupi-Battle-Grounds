//! Cancellation-safe recovery coordinated with D-97 grant retries and revocation barriers.
use super::*;

impl Service {
    pub(super) async fn recovery_request(
        &self,
        name: String,
        email: String,
        locale: String,
        ip: IpAddr,
    ) {
        let Ok(now) = self.now() else {
            return;
        };
        let Ok(store) = self.store() else {
            return;
        };
        let _ = store
            .release(move |db| {
                crate::store::recovery::rotate::request(
                    db,
                    &name,
                    &email,
                    &locale,
                    &ip.to_string(),
                    now,
                )
            })
            .await;
    }
    #[allow(clippy::too_many_arguments)] // Recovery binds the typed proof, device and retry to one revocable grant.
    pub(super) async fn recover(
        &self,
        name: String,
        code: String,
        device: &str,
        persistent: bool,
        retry: &str,
        channel: Channel,
        ip: IpAddr,
    ) -> Result<Grant, Error> {
        let device = tokens::decode::<16>(device)?;
        let retry = tokens::hash(
            format!("recover:{}", tokens::hex(&tokens::decode::<32>(retry)?)).as_bytes(),
        );
        let fingerprint = tokens::hash(
            format!(
                "{}:{}:{}:{}:{}",
                name.to_ascii_lowercase(),
                code,
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
            let hash = tokens::hash(&tokens::decode::<32>(&cached.grant.refresh)?);
            self.store()?
                .auth(db::Request::Confirm { hash, channel, now })
                .await?;
            return Ok(cached.grant.clone());
        }
        self.cache_capacity(&state, None)?;
        self.capacity(None)?;
        let completed = self
            .store()?
            .release(move |db| crate::store::recovery::rotate::complete(db, &name, &code, now))
            .await
            .ok_or(Error::Unavailable)?
            .map_err(|e| {
                if e == crate::store::recovery::Error::Denied {
                    Error::CodeInvalid
                } else {
                    Error::Unavailable
                }
            })?;
        // Same barrier as revoke_account, already holding the coordinator lock.
        self.revoke_scope(
            &mut state,
            RevocationScope::Account(completed.account),
            now,
            "recovered",
        )
        .await?;
        let grant = self
            .issue(
                &mut state,
                NewFamily {
                    name: completed.name,
                    key_hash: completed.hash,
                    register: false,
                    email: None,
                    device,
                    persistent,
                    channel,
                },
                now,
                Some(completed.key),
            )
            .await?;
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
}
