use crate::TaytayError;
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CredentialState {
    Enrolling,
    Active,
    Rotating,
    Revoked,
    Expired,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceCredential {
    pub id: String,
    pub scope: String,
    pub expires_at: SystemTime,
    pub state: CredentialState,
}
impl DeviceCredential {
    pub fn new(
        id: impl Into<String>,
        scope: impl Into<String>,
        expires_at: SystemTime,
    ) -> Result<Self, TaytayError> {
        let id = id.into();
        let scope = scope.into();
        if id.is_empty() || scope.is_empty() {
            return Err(TaytayError::Configuration(
                "credential ID and scope are required".into(),
            ));
        }
        Ok(Self {
            id,
            scope,
            expires_at,
            state: CredentialState::Enrolling,
        })
    }
    pub fn activate(&mut self, now: SystemTime) -> Result<(), TaytayError> {
        if self.state != CredentialState::Enrolling || self.expires_at <= now {
            return Err(TaytayError::Configuration(
                "credential cannot be activated".into(),
            ));
        }
        self.state = CredentialState::Active;
        Ok(())
    }
    pub fn begin_rotation(&mut self) -> Result<(), TaytayError> {
        if self.state != CredentialState::Active {
            return Err(TaytayError::Configuration(
                "only active credentials can rotate".into(),
            ));
        }
        self.state = CredentialState::Rotating;
        Ok(())
    }
    pub fn revoke(&mut self) {
        self.state = CredentialState::Revoked;
    }
    pub fn usable_for_new_session(&self, now: SystemTime) -> bool {
        self.state == CredentialState::Active && self.expires_at > now
    }
    pub fn refresh_expiry(&mut self, expires_at: SystemTime) -> Result<(), TaytayError> {
        self.refresh_expiry_at(expires_at, SystemTime::now())
    }
    pub fn refresh_expiry_at(
        &mut self,
        expires_at: SystemTime,
        now: SystemTime,
    ) -> Result<(), TaytayError> {
        if self.state != CredentialState::Rotating || expires_at <= now {
            return Err(TaytayError::Configuration(
                "invalid credential rotation expiry".into(),
            ));
        }
        self.expires_at = expires_at;
        self.state = CredentialState::Active;
        Ok(())
    }
    pub fn remaining(&self, now: SystemTime) -> Option<Duration> {
        self.expires_at.duration_since(now).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lifecycle_blocks_revoked_and_expired_sessions() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
        let mut c = DeviceCredential::new("device", "upload:create", now + Duration::from_secs(60))
            .unwrap();
        assert!(!c.usable_for_new_session(now));
        c.activate(now).unwrap();
        assert!(c.usable_for_new_session(now));
        c.begin_rotation().unwrap();
        c.refresh_expiry_at(now + Duration::from_secs(120), now)
            .unwrap();
        c.revoke();
        assert!(!c.usable_for_new_session(now));
    }
}
