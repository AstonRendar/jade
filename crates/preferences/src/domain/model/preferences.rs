use crate::domain::value_object::{
    AutoSyncMinutes, Locale, RecentVaults, RelativeVaultPathError, Theme,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Preferences {
    locale: Option<Locale>,
    theme: Theme,
    auto_sync_minutes: AutoSyncMinutes,
    recent_vaults: RecentVaults,
}

impl Preferences {
    pub fn restore(
        locale: Option<Locale>,
        theme: Theme,
        auto_sync_minutes: AutoSyncMinutes,
        recent_vaults: RecentVaults,
    ) -> Self {
        Self {
            locale,
            theme,
            auto_sync_minutes,
            recent_vaults,
        }
    }

    pub fn with_settings(
        &self,
        locale: Option<Locale>,
        theme: Theme,
        auto_sync_minutes: AutoSyncMinutes,
    ) -> Self {
        Self {
            locale,
            theme,
            auto_sync_minutes,
            recent_vaults: self.recent_vaults.clone(),
        }
    }

    pub fn remember_vault(&self, path: &str) -> Result<Self, RelativeVaultPathError> {
        Ok(Self {
            recent_vaults: self.recent_vaults.remember(path)?,
            ..self.clone()
        })
    }

    pub fn forget_vault(&self, path: &str) -> Self {
        Self {
            recent_vaults: self.recent_vaults.forget(path),
            ..self.clone()
        }
    }

    pub fn locale(&self) -> Option<Locale> {
        self.locale
    }

    pub fn theme(&self) -> Theme {
        self.theme
    }

    pub fn auto_sync_minutes(&self) -> AutoSyncMinutes {
        self.auto_sync_minutes
    }

    pub fn recent_vaults(&self) -> &RecentVaults {
        &self.recent_vaults
    }

    pub fn last_vault(&self) -> Option<&str> {
        self.recent_vaults.paths().first().map(String::as_str)
    }
}
