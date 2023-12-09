/// support sync and send
#[macro_export]
macro_rules! impl_sync_send {
    ([$($g:ident),*],$t:ty) => {
        /// support send
        unsafe impl<$($g),*> Send for $t {}

        /// support sync
        unsafe impl<$($g),*> Sync for $t {}
    };
    ($g:ident,$t:ty) => {
        $crate::impl_sync_send!([$g],$t);
    };
    ($t:ty) => {
        /// support send
        unsafe impl Send for $t {}

        /// support sync
        unsafe impl Sync for $t {}
    };
}

/// support debug
#[macro_export]
macro_rules! impl_debug {
    ([$($g:ident),*],$t:ty) => {
        use std::fmt::{Debug, Formatter};

        /// support debug
        impl<$($g: Debug),*> Debug for $t {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Debug::fmt(self.deref(), f)
            }
        }
    };
    ($g:ident,$t:ty) => {
        $crate::impl_debug!([$g],$t);
    };
}

/// support display and debug
#[macro_export]
macro_rules! impl_debug_display {
    ([$($g:ident),*],$t:ty) => {
        use std::fmt::Display;

        $crate::impl_debug!([$($g),*],$t);

        /// support display
        impl<$($g: Display),*> Display for $t {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(self.deref(), f)
            }
        }
    };
    ($g:ident,$t:ty) => {
        $crate::impl_debug_display!([$g],$t);
    };
}

/// support asref<br />
/// use this macro, should be impl DeRef trait, and DeRef type Target equal to $e
#[macro_export]
macro_rules! impl_as_ref {
    ([$($g:ident),*],$e:ty,$t:ty) => {
        /// support asref
        impl<$($g),*> AsRef<$e> for $t {
            fn as_ref(&self) -> &$e {
                self
            }
        }
    };
    ($g:ident,$e:ty,$t:ty) => {
        $crate::impl_as_ref!([$g],$e,$t);
    };
}

/// support default
#[macro_export]
macro_rules! impl_default {
    ([$($g:ident),*],$t:ty,$def:expr) => {
        /// support default
        impl<$($g),*> Default for $t {
            fn default() -> Self { $def }
        }
    };
    ($g:ident,$t:ty,$def:expr) => {
        $crate::impl_default!([$g],$t,$def);
    };
    // $g default impl Default
    ($g:ident =>,$t:ty,$def:expr) => {
        impl<$g: Default> Default for $t {
            fn default() -> Self { $def }
        }
    };
}
