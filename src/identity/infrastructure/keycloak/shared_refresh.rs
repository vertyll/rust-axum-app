use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use fred::prelude::{Expiration, KeysInterface, Pool, SetOptions};
use sha2::{Digest, Sha256};

use crate::identity::application::session::TokenPair;
use crate::identity::domain::IdentityError;

const LOCK_TTL_SECONDS: i64 = 10;
const RESULT_TTL_SECONDS: i64 = 30;
const WAIT_INTERVAL: Duration = Duration::from_millis(100);
const WAIT_ATTEMPTS: usize = 50;
const SEPARATOR: char = '\n';

/// One refresh per refresh token across replicas. The replica that claims
/// `{prefix}:refresh-lock:{sha256}` calls Keycloak and leaves the new pair under
/// `{prefix}:refresh-result:{sha256}` for thirty seconds; the others take it from
/// there instead of presenting a rotated token twice. Without Redis it refreshes
/// directly, so Redis never becomes a reason a sign-in fails.
#[derive(Clone)]
pub struct SharedRefreshes {
	redis: Option<Pool>,
	key_prefix: Arc<str>,
}

enum Claim {
	Done(TokenPair),
	Leader,
	Follower,
}

impl SharedRefreshes {
	pub fn new(redis: Pool, key_prefix: &str) -> Self {
		Self {
			redis: Some(redis),
			key_prefix: Arc::from(key_prefix),
		}
	}

	pub fn in_process_only() -> Self {
		Self {
			redis: None,
			key_prefix: Arc::from(""),
		}
	}

	pub async fn refresh<F, Fut>(&self, refresh_token: &str, keycloak: F) -> Result<TokenPair, IdentityError>
	where
		F: FnOnce() -> Fut,
		Fut: Future<Output = Result<TokenPair, IdentityError>>,
	{
		let Some(redis) = &self.redis else {
			return keycloak().await;
		};
		let id = sha256(refresh_token);
		let lock_key = format!("{}:refresh-lock:{id}", self.key_prefix);
		let result_key = format!("{}:refresh-result:{id}", self.key_prefix);
		match claim(redis, &lock_key, &result_key).await {
			Err(err) => {
				tracing::warn!("Redis unavailable, refreshing without coordinating replicas: {err}");
				keycloak().await
			}
			Ok(Claim::Done(pair)) => Ok(pair),
			Ok(Claim::Leader) => {
				let result = keycloak().await;
				match &result {
					Ok(pair) => share(redis, &result_key, pair).await,
					Err(_) => release(redis, &lock_key).await,
				}
				result
			}
			Ok(Claim::Follower) => match await_other_replica(redis, &lock_key, &result_key).await {
				Some(pair) => Ok(pair),
				None => keycloak().await,
			},
		}
	}
}

async fn claim(redis: &Pool, lock_key: &str, result_key: &str) -> Result<Claim, fred::prelude::Error> {
	if let Some(pair) = read(redis, result_key).await? {
		return Ok(Claim::Done(pair));
	}
	let claimed: Option<String> = redis
		.set(
			lock_key,
			"1",
			Some(Expiration::EX(LOCK_TTL_SECONDS)),
			Some(SetOptions::NX),
			false,
		)
		.await?;
	Ok(if claimed.is_some() {
		Claim::Leader
	} else {
		Claim::Follower
	})
}

async fn await_other_replica(redis: &Pool, lock_key: &str, result_key: &str) -> Option<TokenPair> {
	for _ in 0..WAIT_ATTEMPTS {
		tokio::time::sleep(WAIT_INTERVAL).await;
		match read(redis, result_key).await {
			Ok(Some(pair)) => return Some(pair),
			Ok(None) => {}
			Err(err) => {
				tracing::warn!("Redis unavailable while waiting for another replica's refresh: {err}");
				return None;
			}
		}
		match redis.exists::<i64, _>(lock_key).await {
			Ok(0) | Err(_) => return None,
			Ok(_) => {}
		}
	}
	None
}

async fn read(redis: &Pool, result_key: &str) -> Result<Option<TokenPair>, fred::prelude::Error> {
	let value: Option<String> = redis.get(result_key).await?;
	Ok(value.and_then(|value| {
		value.split_once(SEPARATOR).map(|(access, refresh)| TokenPair {
			access_token: access.to_string(),
			refresh_token: refresh.to_string(),
		})
	}))
}

async fn share(redis: &Pool, result_key: &str, pair: &TokenPair) {
	let value = format!("{}{SEPARATOR}{}", pair.access_token, pair.refresh_token);
	let stored: Result<(), _> = redis
		.set(result_key, value, Some(Expiration::EX(RESULT_TTL_SECONDS)), None, false)
		.await;
	if let Err(err) = stored {
		tracing::warn!("Could not share the refreshed tokens with other replicas: {err}");
	}
}

async fn release(redis: &Pool, lock_key: &str) {
	if let Err(err) = redis.del::<i64, _>(lock_key).await {
		tracing::warn!("Could not release the refresh lock, it expires on its own: {err}");
	}
}

fn sha256(value: &str) -> String {
	Sha256::digest(value.as_bytes())
		.iter()
		.map(|byte| format!("{byte:02x}"))
		.collect()
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering};

	use fred::prelude::{ClientLike, Config, Pool};
	use testcontainers_modules::redis::Redis;
	use testcontainers_modules::testcontainers::runners::AsyncRunner;
	use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
	use tokio::sync::Notify;

	use super::SharedRefreshes;
	use crate::identity::application::session::TokenPair;
	use crate::identity::domain::IdentityError;

	async fn redis() -> (ContainerAsync<Redis>, Pool) {
		let container = Redis::default().with_tag("8-alpine").start().await.unwrap();
		let port = container.get_host_port_ipv4(6379).await.unwrap();
		let pool = Pool::new(
			Config::from_url(&format!("redis://127.0.0.1:{port}")).unwrap(),
			None,
			None,
			None,
			1,
		)
		.unwrap();
		pool.init().await.unwrap();
		(container, pool)
	}

	fn pair(access: &str, refresh: &str) -> TokenPair {
		TokenPair {
			access_token: access.into(),
			refresh_token: refresh.into(),
		}
	}

	#[tokio::test]
	async fn two_replicas_refreshing_one_token_reach_keycloak_once() {
		let (_container, pool) = redis().await;
		let first = SharedRefreshes::new(pool.clone(), "test");
		let second = SharedRefreshes::new(pool, "test");
		let calls = Arc::new(AtomicUsize::new(0));
		let entered = Arc::new(Notify::new());
		let release = Arc::new(Notify::new());

		let leader = tokio::spawn({
			let (calls, entered, release) = (calls.clone(), entered.clone(), release.clone());
			async move {
				first
					.refresh("refresh-1", || async move {
						calls.fetch_add(1, Ordering::SeqCst);
						entered.notify_one();
						release.notified().await;
						Ok(pair("access-2", "refresh-2"))
					})
					.await
			}
		});
		entered.notified().await;
		let follower = tokio::spawn({
			let calls = calls.clone();
			async move {
				second
					.refresh("refresh-1", || async move {
						calls.fetch_add(1, Ordering::SeqCst);
						Ok(pair("access-3", "refresh-3"))
					})
					.await
			}
		});
		release.notify_one();

		assert_eq!(leader.await.unwrap().unwrap().refresh_token, "refresh-2");
		assert_eq!(follower.await.unwrap().unwrap().refresh_token, "refresh-2");
		assert_eq!(calls.load(Ordering::SeqCst), 1);
	}

	#[tokio::test]
	async fn a_stale_request_receives_the_tokens_already_issued() {
		let (_container, pool) = redis().await;
		SharedRefreshes::new(pool.clone(), "test")
			.refresh("refresh-1", || async { Ok(pair("access-2", "refresh-2")) })
			.await
			.unwrap();

		let stale = SharedRefreshes::new(pool, "test")
			.refresh("refresh-1", || async { Err(IdentityError::SessionExpired) })
			.await
			.unwrap();

		assert_eq!(stale.access_token, "access-2");
	}

	#[tokio::test]
	async fn a_refused_refresh_is_not_shared() {
		let (_container, pool) = redis().await;
		let replica = SharedRefreshes::new(pool, "test");
		let refused = replica
			.refresh("refresh-1", || async { Err(IdentityError::SessionExpired) })
			.await;
		assert!(refused.is_err());

		let retried = replica
			.refresh("refresh-1", || async { Ok(pair("access-2", "refresh-2")) })
			.await
			.unwrap();

		assert_eq!(retried.refresh_token, "refresh-2");
	}

	#[tokio::test]
	async fn keys_carry_the_application_prefix_and_never_the_token() {
		let (_container, pool) = redis().await;
		SharedRefreshes::new(pool.clone(), "test")
			.refresh("secret-refresh", || async { Ok(pair("access", "refresh")) })
			.await
			.unwrap();

		let (_, keys): (String, Vec<String>) = pool
			.next()
			.custom(fred::cmd!("SCAN"), vec!["0", "COUNT", "100"])
			.await
			.unwrap();
		assert_eq!(
			keys.iter()
				.filter(|key| key.starts_with("test:refresh-result:"))
				.count(),
			1
		);
		assert!(
			keys.iter()
				.all(|key| key.starts_with("test:") && !key.contains("secret-refresh"))
		);
	}
}
