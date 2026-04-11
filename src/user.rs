use super::*;

/// Access to the steam user interface
pub struct User {
    pub(crate) user: *mut sys::ISteamUser,
    pub(crate) _inner: Arc<Inner>,
}

impl User {
    /// Returns the steam id of the current user
    pub fn steam_id(&self) -> SteamId {
        unsafe { SteamId(sys::SteamAPI_ISteamUser_GetSteamID(self.user)) }
    }

    /// Returns the level of the current user
    pub fn level(&self) -> u32 {
        unsafe { sys::SteamAPI_ISteamUser_GetPlayerSteamLevel(self.user) as u32 }
    }

    /// Returns whether the current user's Steam client is connected to the Steam servers.
    pub fn logged_on(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamUser_BLoggedOn(self.user) }
    }

    /// Ends an authentication session that was started with
    /// `begin_authentication_session`.
    ///
    /// This should be called when you are no longer playing with
    /// the specified entity.
    pub fn end_authentication_session(&self, user: SteamId) {
        unsafe {
            sys::SteamAPI_ISteamUser_EndAuthSession(self.user, user.0);
        }
    }

    /// Checks if the user owns a piece of DLC specified by app id.
    ///
    /// This can only be called after authenticating
    /// with the user using `begin_authentication_session`.
    pub fn user_has_license_for_app(&self, user: SteamId, app_id: AppId) -> UserHasLicense {
        unsafe {
            let license_response =
                sys::SteamAPI_ISteamUser_UserHasLicenseForApp(self.user, user.0, app_id.0);

            match license_response {
                sys::EUserHasLicenseForAppResult::k_EUserHasLicenseResultHasLicense => {
                    UserHasLicense::HasLicense
                }
                sys::EUserHasLicenseForAppResult::k_EUserHasLicenseResultDoesNotHaveLicense => {
                    UserHasLicense::DoesNotHaveLicense
                }
                sys::EUserHasLicenseForAppResult::k_EUserHasLicenseResultNoAuth => {
                    UserHasLicense::NoAuth
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Results from [`User::user_has_license_for_app`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserHasLicense {
    /// The user has a license for the specified app.
    HasLicense,
    /// The user does not have a license for the specified app.
    DoesNotHaveLicense,
    /// The user has not been authenticated.
    NoAuth,
}
