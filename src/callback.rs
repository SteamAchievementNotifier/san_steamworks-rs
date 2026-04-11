use super::*;
use crate::screenshots::*;

use crate::sys;

use std::sync::{Arc, Weak};

/// A sum type over all possible callback results
#[derive(Debug)]
pub enum CallbackResult {
    FloatingGamepadTextInputDismissed(FloatingGamepadTextInputDismissed),
    GameOverlayActivated(GameOverlayActivated),
    GamepadTextInputDismissed(GamepadTextInputDismissed),
    GameRichPresenceJoinRequested(GameRichPresenceJoinRequested),
    PersonaStateChange(PersonaStateChange),
    ScreenshotRequested(ScreenshotRequested),
    ScreenshotReady(ScreenshotReady),
    UserAchievementStored(UserAchievementStored),
    UserAchievementIconFetched(UserAchievementIconFetched),
    UserStatsReceived(UserStatsReceived),
    UserStatsStored(UserStatsStored),
    NewUrlLaunchParameters(NewUrlLaunchParameters),
}

impl CallbackResult {
    pub unsafe fn from_raw(discriminator: i32, data: *mut c_void) -> Option<Self> {
        Some(match discriminator {
            GameOverlayActivated::ID => {
                Self::GameOverlayActivated(GameOverlayActivated::from_raw(data))
            }
            GamepadTextInputDismissed::ID => {
                Self::GamepadTextInputDismissed(GamepadTextInputDismissed::from_raw(data))
            }
            GameRichPresenceJoinRequested::ID => {
                Self::GameRichPresenceJoinRequested(GameRichPresenceJoinRequested::from_raw(data))
            }
            UserAchievementStored::ID => {
                Self::UserAchievementStored(UserAchievementStored::from_raw(data))
            }
            UserStatsReceived::ID => Self::UserStatsReceived(UserStatsReceived::from_raw(data)),
            UserStatsStored::ID => Self::UserStatsStored(UserStatsStored::from_raw(data)),
            NewUrlLaunchParameters::ID => {
                Self::NewUrlLaunchParameters(NewUrlLaunchParameters::from_raw(data))
            }
            _ => return None,
        })
    }
}

pub unsafe trait Callback {
    const ID: i32;
    unsafe fn from_raw(raw: *mut c_void) -> Self;
}

/// A handle that can be used to remove a callback
/// at a later point.
///
/// Removes the callback from the Steam API context when dropped.
pub struct CallbackHandle {
    id: i32,
    inner: Weak<Inner>,
}

impl Drop for CallbackHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            match inner.callbacks.callbacks.lock() {
                Ok(mut cb) => {
                    cb.remove(&self.id);
                }
                Err(err) => {
                    eprintln!("error while dropping callback: {:?}", err);
                }
            }
        }
    }
}

macro_rules! impl_callback {
    ($fn_arg_name:ident: $sys_ty:ident => $callback_ty:ident $from_raw_impl:tt) => {
        paste::item! {
            unsafe impl Callback for $callback_ty {
                const ID: i32 = steamworks_sys::[<$sys_ty _ k_iCallback>] as i32;

                unsafe fn from_raw(raw: *mut c_void) -> Self {
                    let $fn_arg_name = raw.cast::<steamworks_sys::$sys_ty>().read_unaligned();
                    $from_raw_impl
                }
            }
        }
    };
}

pub(crate) unsafe fn register_callback<C, F>(inner: &Arc<Inner>, mut f: F) -> CallbackHandle
where
    C: Callback,
    F: FnMut(C) + Send + 'static,
{
    {
        inner.callbacks.callbacks.lock().unwrap().insert(
            C::ID,
            Box::new(move |param| {
                let param = C::from_raw(param);
                f(param)
            }),
        );
    }
    CallbackHandle {
        id: C::ID,
        inner: Arc::downgrade(inner),
    }
}

pub(crate) unsafe fn register_call_result<C, F>(
    inner: &Arc<Inner>,
    api_call: sys::SteamAPICall_t,
    f: F,
) where
    F: for<'a> FnOnce(&'a C, bool) + 'static + Send,
{
    inner.callbacks.call_results.lock().unwrap().insert(
        api_call,
        Box::new(move |param, failed| {
            let value = param.cast::<C>().read_unaligned();
            f(&value, failed)
        }),
    );
}
