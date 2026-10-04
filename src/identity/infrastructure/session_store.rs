use async_trait::async_trait;
use fred::prelude::{Expiration, KeysInterface, Pool, SetOptions};
use tower_sessions::SessionStore;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store::{Error, Result};

/// Browser sessions in Redis under `{prefix}:session:{id}`, the key layout every
/// application on the shared Redis uses, so one instance can serve them all.
#[derive(Debug, Clone)]
pub struct RedisSessionStore {
	pool: Pool,
	prefix: String,
}

impl RedisSessionStore {
	pub fn new(pool: Pool, key_prefix: &str) -> Self {
		Self {
			pool,
			prefix: format!("{key_prefix}:session:"),
		}
	}

	fn key(&self, id: &Id) -> String {
		format!("{}{id}", self.prefix)
	}

	async fn write(&self, record: &Record, options: SetOptions) -> Result<bool> {
		let payload = rmp_serde::to_vec(record).map_err(|err| Error::Encode(err.to_string()))?;
		self.pool
			.set(
				self.key(&record.id),
				payload.as_slice(),
				Some(Expiration::EXAT(record.expiry_date.unix_timestamp())),
				Some(options),
				false,
			)
			.await
			.map_err(|err| Error::Backend(err.to_string()))
	}
}

#[async_trait]
impl SessionStore for RedisSessionStore {
	async fn create(&self, record: &mut Record) -> Result<()> {
		while !self.write(record, SetOptions::NX).await? {
			record.id = Id::default();
		}
		Ok(())
	}

	async fn save(&self, record: &Record) -> Result<()> {
		self.write(record, SetOptions::XX).await?;
		Ok(())
	}

	async fn load(&self, id: &Id) -> Result<Option<Record>> {
		let payload: Option<Vec<u8>> = self
			.pool
			.get(self.key(id))
			.await
			.map_err(|err| Error::Backend(err.to_string()))?;
		payload
			.map(|bytes| rmp_serde::from_slice(&bytes).map_err(|err| Error::Decode(err.to_string())))
			.transpose()
	}

	async fn delete(&self, id: &Id) -> Result<()> {
		let _: i64 = self
			.pool
			.del(self.key(id))
			.await
			.map_err(|err| Error::Backend(err.to_string()))?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use fred::prelude::{ClientLike, Config, KeysInterface, Pool};
	use testcontainers_modules::redis::Redis;
	use testcontainers_modules::testcontainers::ImageExt;
	use testcontainers_modules::testcontainers::runners::AsyncRunner;
	use tower_sessions::SessionStore;
	use tower_sessions::cookie::time::{Duration, OffsetDateTime};
	use tower_sessions::session::{Id, Record};

	use super::RedisSessionStore;

	#[tokio::test]
	async fn sessions_live_under_the_application_prefix() {
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
		let store = RedisSessionStore::new(pool.clone(), "test");
		let mut record = Record {
			id: Id::default(),
			data: [("tokens".to_string(), serde_json::json!("value"))].into(),
			expiry_date: OffsetDateTime::now_utc() + Duration::minutes(5),
		};

		store.create(&mut record).await.unwrap();

		let key = format!("test:session:{}", record.id);
		assert!(pool.exists::<bool, _>(&key).await.unwrap());
		assert_eq!(store.load(&record.id).await.unwrap().unwrap().data, record.data);
		store.delete(&record.id).await.unwrap();
		assert!(store.load(&record.id).await.unwrap().is_none());
	}
}
