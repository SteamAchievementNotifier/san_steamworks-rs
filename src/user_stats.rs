mod stat_callback;
pub mod stats;

pub use self::stat_callback::*;
use super::*;

/// Access to the steam user interface
pub struct UserStats<Manager> {
    pub(crate) user_stats: *mut sys::ISteamUserStats,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 1100;

impl<Manager> UserStats<Manager> {
    /// Triggers a [`UserStatsReceived`](./struct.UserStatsReceived.html) callback.
    pub fn request_current_stats(&self) {
        unsafe {
            sys::SteamAPI_ISteamUserStats_RequestCurrentStats(self.user_stats);
        }
    }

    /// Asynchronously fetch the data for the percentage of players who have received each achievement
    /// for the current game globally.
    /// 
    /// You must have called `request_current_stats()` and it needs to return successfully via its
    /// callback prior to calling this!*
    /// 
    /// **Note: Not sure if this is applicable, as the other achievement functions requiring
    /// `request_current_stats()` don't specifically need it to be called in order for them to complete
    /// successfully. Maybe it autoruns via `Client::init()/init_app()` somehow?*
    pub fn request_global_achievement_percentages<F>(&self, cb: F)
    where
        F: FnOnce(Result<GameId, SteamError>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamUserStats_RequestGlobalAchievementPercentages(
                self.user_stats,
            );
            register_call_result::<sys::GlobalAchievementPercentagesReady_t, _, _>(
                &self.inner,
                api_call,
                // `CALLBACK_BASE_ID + <number>`: <number> is found in Steamworks `isteamuserstats.h` header file
                // (Under `struct GlobalAchievementPercentagesReady_t {...};` in this case)
                CALLBACK_BASE_ID + 10,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(GameId(v.m_nGameID))
                    })
                },
            );
        }
    }

    /// Send the changed stats and achievements data to the server for permanent storage.
    ///
    /// * Triggers a [`UserStatsStored`](../struct.UserStatsStored.html) callback if successful.
    /// * Triggers a [`UserAchievementStored`](../struct.UserAchievementStored.html) callback
    ///   if achievements have been unlocked.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn store_stats(&self) -> Result<(), ()> {
        let success = unsafe { sys::SteamAPI_ISteamUserStats_StoreStats(self.user_stats) };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Resets the current users stats and, optionally achievements.
    pub fn reset_all_stats(&self, achievements_too: bool) -> Result<(), ()> {
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_ResetAllStats(self.user_stats, achievements_too)
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Gets the value of a given stat for the current user
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn get_stat_i32(&self, name: &str) -> Result<i32, ()> {
        let name = CString::new(name).unwrap();

        let mut value: i32 = 0;
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_GetStatInt32(
                self.user_stats,
                name.as_ptr() as *const _,
                &mut value,
            )
        };
        if success {
            Ok(value)
        } else {
            Err(())
        }
    }

    /// Sets / updates the value of a given stat for the current user
    ///
    /// This call only changes the value in-memory and is very cheap. To commit the stats you
    /// must call [`store_stats()`](#method.store_stats)
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn set_stat_i32(&self, name: &str, stat: i32) -> Result<(), ()> {
        let name = CString::new(name).unwrap();

        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_SetStatInt32(
                self.user_stats,
                name.as_ptr() as *const _,
                stat,
            )
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Gets the value of a given stat for the current user
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn get_stat_f32(&self, name: &str) -> Result<f32, ()> {
        let name = CString::new(name).unwrap();

        let mut value: f32 = 0.0;
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_GetStatFloat(
                self.user_stats,
                name.as_ptr() as *const _,
                &mut value,
            )
        };
        if success {
            Ok(value)
        } else {
            Err(())
        }
    }

    /// Sets / updates the value of a given stat for the current user
    ///
    /// This call only changes the value in-memory and is very cheap. To commit the stats you
    /// must call [`store_stats()`](#method.store_stats)
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn set_stat_f32(&self, name: &str, stat: f32) -> Result<(), ()> {
        let name = CString::new(name).unwrap();

        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_SetStatFloat(
                self.user_stats,
                name.as_ptr() as *const _,
                stat,
            )
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Access achievement API for a given achievement 'API Name'.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    #[inline]
    #[must_use]
    pub fn achievement(&self, name: &str) -> stats::AchievementHelper<'_, Manager> {
        stats::AchievementHelper {
            name: CString::new(name).unwrap(),
            parent: self,
        }
    }

    /// Get the number of achievements defined in the App Admin panel of the Steamworks website.
    ///
    /// This is used for iterating through all of the achievements with GetAchievementName.
    ///
    /// Returns 0 if the current App ID has no achievements.
    /// 
    /// *Note: Returns an error for AppId `480` (Spacewar)!*
    pub fn get_num_achievements(&self) -> Result<u32,()> {
        unsafe {
            let num = sys::SteamAPI_ISteamUserStats_GetNumAchievements(
                self.user_stats,
            );
            if num != 0 {
                Ok(num)
            } else {
                Err(())
            }
        }
    }

    /// Returns an array of all achievement names for the current AppId.
    /// 
    /// Returns an empty string for an achievement name if `iAchievement` is not a valid index,
    /// and the current AppId must have achievements.
    pub fn get_achievement_names(&self) -> Option<Vec<String>> {
        let num = self.get_num_achievements().expect("Failed to get number of achievements");
        let mut names = Vec::new();

        for i in 0..num {
            unsafe {
                let name = sys::SteamAPI_ISteamUserStats_GetAchievementName(
                    self.user_stats,
                    i
                );

                let c_str = CStr::from_ptr(name).to_string_lossy().into_owned();

                names.push(c_str);
            }
        }
        Some(names)
    }
}
