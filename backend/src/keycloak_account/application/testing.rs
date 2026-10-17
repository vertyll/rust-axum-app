use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::keycloak_account::application::ports::AccountApi;
use crate::keycloak_account::domain::{KeycloakAccountError, SecondFactor};
use crate::translations::Language;

/// An account API in memory: the second factor it holds, and each language
/// stored, as `<token> <language>`.
#[derive(Clone, Default)]
pub(crate) struct FakeAccountApi {
    second_factor: Arc<Mutex<Option<SecondFactor>>>,
    languages: Arc<Mutex<Vec<String>>>,
    failing: Arc<AtomicBool>,
}

impl FakeAccountApi {
    pub(crate) fn set_second_factor(&self, credential_id: &str) {
        *self.second_factor.lock().unwrap() = Some(SecondFactor {
            credential_id: credential_id.to_owned(),
        });
    }

    pub(crate) fn languages(&self) -> Vec<String> {
        self.languages.lock().unwrap().clone()
    }

    pub(crate) fn fail(&self) {
        self.failing.store(true, Ordering::SeqCst);
    }
}

impl AccountApi for FakeAccountApi {
    async fn second_factor(&self, _access_token: &str) -> Result<Option<SecondFactor>, KeycloakAccountError> {
        Ok(self.second_factor.lock().unwrap().clone())
    }

    async fn set_language(&self, access_token: &str, language: Language) -> Result<(), KeycloakAccountError> {
        if self.failing.load(Ordering::SeqCst) {
            return Err(KeycloakAccountError::Unavailable("down".into()));
        }
        self.languages
            .lock()
            .unwrap()
            .push(format!("{access_token} {language}"));
        Ok(())
    }
}
