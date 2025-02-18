// This file was generated by gir (https://github.com/gtk-rs/gir)
// from
// from gir-files (https://github.com/gtk-rs/gir-files.git)
// DO NOT EDIT

use glib::{bitflags::bitflags, prelude::*, translate::*};
use std::fmt;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[doc(alias = "VteFeatureFlags")]
    pub struct FeatureFlags: u32 {
        #[doc(alias = "VTE_FEATURE_FLAG_BIDI")]
        const FLAG_BIDI = ffi::VTE_FEATURE_FLAG_BIDI as _;
        #[doc(alias = "VTE_FEATURE_FLAG_ICU")]
        const FLAG_ICU = ffi::VTE_FEATURE_FLAG_ICU as _;
        #[doc(alias = "VTE_FEATURE_FLAG_SYSTEMD")]
        const FLAG_SYSTEMD = ffi::VTE_FEATURE_FLAG_SYSTEMD as _;
        #[doc(alias = "VTE_FEATURE_FLAG_SIXEL")]
        const FLAG_SIXEL = ffi::VTE_FEATURE_FLAG_SIXEL as _;
        #[doc(alias = "VTE_FEATURE_FLAGS_MASK")]
        const FLAGS_MASK = ffi::VTE_FEATURE_FLAGS_MASK as _;
    }
}

impl fmt::Display for FeatureFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

#[doc(hidden)]
impl IntoGlib for FeatureFlags {
    type GlibType = ffi::VteFeatureFlags;

    #[inline]
    fn into_glib(self) -> ffi::VteFeatureFlags {
        self.bits()
    }
}

#[doc(hidden)]
impl FromGlib<ffi::VteFeatureFlags> for FeatureFlags {
    #[inline]
    unsafe fn from_glib(value: ffi::VteFeatureFlags) -> Self {
        skip_assert_initialized!();
        Self::from_bits_truncate(value)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    #[doc(alias = "VtePtyFlags")]
    pub struct PtyFlags: u32 {
        #[doc(alias = "VTE_PTY_NO_LASTLOG")]
        const NO_LASTLOG = ffi::VTE_PTY_NO_LASTLOG as _;
        #[doc(alias = "VTE_PTY_NO_UTMP")]
        const NO_UTMP = ffi::VTE_PTY_NO_UTMP as _;
        #[doc(alias = "VTE_PTY_NO_WTMP")]
        const NO_WTMP = ffi::VTE_PTY_NO_WTMP as _;
        #[doc(alias = "VTE_PTY_NO_HELPER")]
        const NO_HELPER = ffi::VTE_PTY_NO_HELPER as _;
        #[doc(alias = "VTE_PTY_NO_FALLBACK")]
        const NO_FALLBACK = ffi::VTE_PTY_NO_FALLBACK as _;
        #[doc(alias = "VTE_PTY_NO_SESSION")]
        const NO_SESSION = ffi::VTE_PTY_NO_SESSION as _;
        #[doc(alias = "VTE_PTY_NO_CTTY")]
        const NO_CTTY = ffi::VTE_PTY_NO_CTTY as _;
        #[doc(alias = "VTE_PTY_DEFAULT")]
        const DEFAULT = ffi::VTE_PTY_DEFAULT as _;
    }
}

impl fmt::Display for PtyFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

#[doc(hidden)]
impl IntoGlib for PtyFlags {
    type GlibType = ffi::VtePtyFlags;

    #[inline]
    fn into_glib(self) -> ffi::VtePtyFlags {
        self.bits()
    }
}

#[doc(hidden)]
impl FromGlib<ffi::VtePtyFlags> for PtyFlags {
    #[inline]
    unsafe fn from_glib(value: ffi::VtePtyFlags) -> Self {
        skip_assert_initialized!();
        Self::from_bits_truncate(value)
    }
}

impl StaticType for PtyFlags {
    #[inline]
    fn static_type() -> glib::Type {
        unsafe { from_glib(ffi::vte_pty_flags_get_type()) }
    }
}

impl glib::HasParamSpec for PtyFlags {
    type ParamSpec = glib::ParamSpecFlags;
    type SetValue = Self;
    type BuilderFn = fn(&str) -> glib::ParamSpecFlagsBuilder<Self>;

    fn param_spec_builder() -> Self::BuilderFn {
        |name| Self::ParamSpec::builder(name)
    }
}

impl glib::value::ValueType for PtyFlags {
    type Type = Self;
}

unsafe impl<'a> glib::value::FromValue<'a> for PtyFlags {
    type Checker = glib::value::GenericValueTypeChecker<Self>;

    #[inline]
    unsafe fn from_value(value: &'a glib::Value) -> Self {
        skip_assert_initialized!();
        from_glib(glib::gobject_ffi::g_value_get_flags(value.to_glib_none().0))
    }
}

impl ToValue for PtyFlags {
    #[inline]
    fn to_value(&self) -> glib::Value {
        let mut value = glib::Value::for_value_type::<Self>();
        unsafe {
            glib::gobject_ffi::g_value_set_flags(value.to_glib_none_mut().0, self.into_glib());
        }
        value
    }

    #[inline]
    fn value_type(&self) -> glib::Type {
        Self::static_type()
    }
}

impl From<PtyFlags> for glib::Value {
    #[inline]
    fn from(v: PtyFlags) -> Self {
        skip_assert_initialized!();
        ToValue::to_value(&v)
    }
}
