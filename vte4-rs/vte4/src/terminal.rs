// Take a look at the license at the top of the repository in the LICENSE file.

use glib::translate::*;

use std::boxed::Box as Box_;
use std::os::unix::io::{IntoRawFd, RawFd};

use io_lifetimes::OwnedFd;

#[cfg(any(feature = "v0_70", feature = "dox"))]
#[cfg_attr(feature = "dox", doc(cfg(feature = "v0_70")))]
use crate::Regex;
use crate::{prelude::*, PtyFlags, Terminal};

pub trait TerminalExtManual: 'static {
    #[cfg(any(feature = "v0_70", feature = "dox"))]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "v0_70")))]
    #[doc(alias = "vte_terminal_check_regex_array_at")]
    #[doc(alias = "vte_terminal_check_regex_simple_at")]
    #[doc(alias = "check_regex_array_at")]
    fn check_regex_simple_at(
        &self,
        x: f64,
        y: f64,
        regexes: &[&Regex],
        match_flags: u32,
    ) -> Vec<glib::GString>;

    #[doc(alias = "vte_terminal_set_colors")]
    fn set_colors(
        &self,
        foreground: Option<&gdk::RGBA>,
        background: Option<&gdk::RGBA>,
        palette: &[&gdk::RGBA],
    );

    #[doc(alias = "vte_terminal_watch_child")]
    fn watch_child(&self, child_pid: glib::Pid);

    #[doc(alias = "vte_terminal_spawn_async")]
    #[allow(clippy::too_many_arguments)]
    fn spawn_async<P: FnOnce(Result<glib::Pid, glib::Error>) + 'static, Q: Fn() + 'static>(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
        cancellable: Option<&impl IsA<gio::Cancellable>>,
        callback: P,
    );

    #[doc(alias = "vte_terminal_spawn_async")]
    #[allow(clippy::too_many_arguments)]
    fn spawn_future<Q: Fn() + 'static>(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
    ) -> std::pin::Pin<
        Box_<dyn std::future::Future<Output = Result<glib::Pid, glib::Error>> + 'static>,
    >;

    #[doc(alias = "vte_terminal_spawn_with_fds_async")]
    #[allow(clippy::too_many_arguments)]
    unsafe fn spawn_with_fds_async<
        P: FnOnce(Result<glib::Pid, glib::Error>) + 'static,
        Q: Fn() + 'static,
    >(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        fds: Vec<OwnedFd>,
        map_fds: &[RawFd],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
        cancellable: Option<&impl IsA<gio::Cancellable>>,
        callback: P,
    );

    #[doc(alias = "vte_terminal_spawn_async")]
    #[allow(clippy::too_many_arguments)]
    unsafe fn spawn_with_fds_future<Q: Fn() + 'static>(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        fds: Vec<OwnedFd>,
        map_fds: &[RawFd],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
    ) -> std::pin::Pin<
        Box_<dyn std::future::Future<Output = Result<glib::Pid, glib::Error>> + 'static>,
    >;
}

impl<O: IsA<Terminal>> TerminalExtManual for O {
    #[cfg(any(feature = "v0_70", feature = "dox"))]
    #[cfg_attr(feature = "dox", doc(cfg(feature = "v0_70")))]
    fn check_regex_simple_at(
        &self,
        x: f64,
        y: f64,
        regexes: &[&Regex],
        match_flags: u32,
    ) -> Vec<glib::GString> {
        let n_regexes = regexes.len() as _;
        unsafe {
            let mut n_matches = std::mem::MaybeUninit::uninit();
            let ret = FromGlibContainer::from_glib_full_num(
                ffi::vte_terminal_check_regex_array_at(
                    self.as_ref().to_glib_none().0,
                    x,
                    y,
                    regexes.as_ptr() as *mut _,
                    n_regexes,
                    match_flags,
                    n_matches.as_mut_ptr(),
                ),
                n_matches.assume_init() as _,
            );
            ret
        }
    }

    #[doc(alias = "vte_terminal_spawn_async")]
    fn spawn_future<Q: Fn() + 'static>(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
    ) -> std::pin::Pin<
        Box_<dyn std::future::Future<Output = Result<glib::Pid, glib::Error>> + 'static>,
    > {
        let working_directory = working_directory.map(ToOwned::to_owned);
        let argv: Vec<String> = argv.iter().map(|p| p.to_string()).collect();
        let envv: Vec<String> = envv.iter().map(|p| p.to_string()).collect();
        Box_::pin(gio::GioFuture::new(self, move |obj, cancellable, send| {
            let argv: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
            let envv: Vec<&str> = envv.iter().map(|s| s.as_str()).collect();
            obj.spawn_async(
                pty_flags,
                working_directory.as_deref(),
                &argv,
                &envv,
                spawn_flags,
                child_setup,
                timeout,
                Some(cancellable),
                move |res| {
                    send.resolve(res);
                },
            );
        }))
    }

    // # Safety
    //
    // The map_fds have to make sense.
    #[doc(alias = "vte_terminal_spawn_async")]
    unsafe fn spawn_with_fds_future<Q: Fn() + 'static>(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        fds: Vec<OwnedFd>,
        map_fds: &[RawFd],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
    ) -> std::pin::Pin<
        Box_<dyn std::future::Future<Output = Result<glib::Pid, glib::Error>> + 'static>,
    > {
        let working_directory = working_directory.map(ToOwned::to_owned);
        let argv: Vec<String> = argv.iter().map(|p| p.to_string()).collect();
        let envv: Vec<String> = envv.iter().map(|p| p.to_string()).collect();
        let map_fds: Vec<RawFd> = map_fds.to_vec();
        Box_::pin(gio::GioFuture::new(self, move |obj, cancellable, send| {
            let argv: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
            let envv: Vec<&str> = envv.iter().map(|s| s.as_str()).collect();
            obj.spawn_with_fds_async(
                pty_flags,
                working_directory.as_deref(),
                &argv,
                &envv,
                fds,
                &map_fds,
                spawn_flags,
                child_setup,
                timeout,
                Some(cancellable),
                move |res| {
                    send.resolve(res);
                },
            );
        }))
    }

    #[doc(alias = "vte_terminal_set_colors")]
    fn set_colors(
        &self,
        foreground: Option<&gdk::RGBA>,
        background: Option<&gdk::RGBA>,
        palette: &[&gdk::RGBA],
    ) {
        let palette_size = palette.len();

        let palette_vector = palette
            .iter()
            .map(|item| unsafe { *item.to_glib_none().0 })
            .collect::<Vec<gdk::ffi::GdkRGBA>>();

        unsafe {
            ffi::vte_terminal_set_colors(
                self.as_ref().to_glib_none().0,
                foreground.to_glib_none().0,
                background.to_glib_none().0,
                palette_vector.as_ptr(),
                palette_size,
            );
        }
    }

    fn watch_child(&self, child_pid: glib::Pid) {
        unsafe {
            ffi::vte_terminal_watch_child(self.as_ref().to_glib_none().0, child_pid.into_glib());
        }
    }

    fn spawn_async<P: FnOnce(Result<glib::Pid, glib::Error>) + 'static, Q: Fn() + 'static>(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
        cancellable: Option<&impl IsA<gio::Cancellable>>,
        callback: P,
    ) {
        assert_initialized_main_thread!();
        let main_context = glib::MainContext::ref_thread_default();
        let is_main_context_owner = main_context.is_owner();
        let has_acquired_main_context = (!is_main_context_owner)
            .then(|| main_context.acquire().ok())
            .flatten();
        assert!(
            is_main_context_owner || has_acquired_main_context.is_some(),
            "Async operations only allowed if the thread is owning the MainContext"
        );
        assert!(argv.first().is_some(), "Need to pass an argument");
        let child_setup_data: Box_<glib::thread_guard::ThreadGuard<Q>> =
            Box_::new(glib::thread_guard::ThreadGuard::new(child_setup));
        unsafe extern "C" fn child_setup_func<Q: Fn() + 'static>(user_data: glib::ffi::gpointer) {
            let callback: Box_<glib::thread_guard::ThreadGuard<Q>> =
                Box_::from_raw(user_data as *mut _);
            let callback = callback.into_inner();
            callback()
        }
        let child_setup = Some(child_setup_func::<Q> as _);
        let callback_data: Box_<glib::thread_guard::ThreadGuard<P>> =
            Box_::new(glib::thread_guard::ThreadGuard::new(callback));
        unsafe extern "C" fn spawn_async_trampoline<
            P: FnOnce(Result<glib::Pid, glib::Error>) + 'static,
        >(
            _terminal: *mut ffi::VteTerminal,
            pid: glib::ffi::GPid,
            error: *mut glib::ffi::GError,
            user_data: glib::ffi::gpointer,
        ) {
            let pid = from_glib(pid);
            let result = if let Some(err) = Option::<glib::Error>::from_glib_none(error) {
                Err(err)
            } else {
                Ok(pid)
            };
            let callback: Box_<glib::thread_guard::ThreadGuard<P>> =
                Box_::from_raw(user_data as *mut _);
            let callback = callback.into_inner();
            callback(result)
        }
        let callback = Some(spawn_async_trampoline::<P> as _);
        unsafe extern "C" fn child_setup_data_destroy_func<Q: Fn() + 'static>(
            data: glib::ffi::gpointer,
        ) {
            let _callback: Box_<Q> = Box_::from_raw(data as *mut _);
        }
        let destroy_call8 = Some(child_setup_data_destroy_func::<Q> as _);
        let super_callback0: Box_<glib::thread_guard::ThreadGuard<Q>> = child_setup_data;
        let super_callback1: Box_<glib::thread_guard::ThreadGuard<P>> = callback_data;
        unsafe {
            ffi::vte_terminal_spawn_async(
                self.as_ref().to_glib_none().0,
                pty_flags.into_glib(),
                working_directory.to_glib_none().0,
                argv.to_glib_none().0,
                envv.to_glib_none().0,
                spawn_flags.into_glib(),
                child_setup,
                Box_::into_raw(super_callback0) as *mut _,
                destroy_call8,
                timeout,
                cancellable.map(|p| p.as_ref()).to_glib_none().0,
                callback,
                Box_::into_raw(super_callback1) as *mut _,
            );
        }
    }

    // # Safety
    //
    // The map_fds have to make sense.
    unsafe fn spawn_with_fds_async<
        P: FnOnce(Result<glib::Pid, glib::Error>) + 'static,
        Q: Fn() + 'static,
    >(
        &self,
        pty_flags: PtyFlags,
        working_directory: Option<&str>,
        argv: &[&str],
        envv: &[&str],
        fds: Vec<OwnedFd>,
        map_fds: &[RawFd],
        spawn_flags: glib::SpawnFlags,
        child_setup: Q,
        timeout: i32,
        cancellable: Option<&impl IsA<gio::Cancellable>>,
        callback: P,
    ) {
        assert_initialized_main_thread!();
        let main_context = glib::MainContext::ref_thread_default();
        let is_main_context_owner = main_context.is_owner();
        let has_acquired_main_context = (!is_main_context_owner)
            .then(|| main_context.acquire().ok())
            .flatten();
        assert!(
            is_main_context_owner || has_acquired_main_context.is_some(),
            "Async operations only allowed if the thread is owning the MainContext"
        );
        assert!(argv.first().is_some(), "Need to pass an argument");
        let n_fds = fds.len() as _;
        let n_map_fds = map_fds.len() as _;
        let child_setup_data: Box_<glib::thread_guard::ThreadGuard<Q>> =
            Box_::new(glib::thread_guard::ThreadGuard::new(child_setup));
        unsafe extern "C" fn child_setup_func<Q: Fn() + 'static>(user_data: glib::ffi::gpointer) {
            let callback: Box_<glib::thread_guard::ThreadGuard<Q>> =
                Box_::from_raw(user_data as *mut _);
            let callback = callback.into_inner();
            callback()
        }

        let child_setup = Some(child_setup_func::<Q> as _);

        let callback_data: Box_<glib::thread_guard::ThreadGuard<P>> =
            Box_::new(glib::thread_guard::ThreadGuard::new(callback));
        unsafe extern "C" fn spawn_with_fds_async_trampoline<
            P: FnOnce(Result<glib::Pid, glib::Error>) + 'static,
        >(
            _terminal: *mut ffi::VteTerminal,
            pid: glib::ffi::GPid,
            error: *mut glib::ffi::GError,
            user_data: glib::ffi::gpointer,
        ) {
            let pid = from_glib(pid);
            let result = if let Some(err) = Option::<glib::Error>::from_glib_none(error) {
                Err(err)
            } else {
                Ok(pid)
            };
            let callback: Box_<glib::thread_guard::ThreadGuard<P>> =
                Box_::from_raw(user_data as *mut _);
            let callback = callback.into_inner();
            callback(result)
        }
        let callback = Some(spawn_with_fds_async_trampoline::<P> as _);
        unsafe extern "C" fn child_setup_data_destroy_func<Q: Fn() + 'static>(
            data: glib::ffi::gpointer,
        ) {
            let _callback: Box_<Q> = Box_::from_raw(data as *mut _);
        }
        let destroy_call12 = Some(child_setup_data_destroy_func::<Q> as _);
        let super_callback0: Box_<glib::thread_guard::ThreadGuard<Q>> = child_setup_data;
        let super_callback1: Box_<glib::thread_guard::ThreadGuard<P>> = callback_data;
        let fds: Vec<RawFd> = fds.into_iter().map(|x| x.into_raw_fd()).collect();
        unsafe {
            ffi::vte_terminal_spawn_with_fds_async(
                self.as_ref().to_glib_none().0,
                pty_flags.into_glib(),
                working_directory.to_glib_none().0,
                argv.to_glib_none().0,
                envv.to_glib_none().0,
                fds.to_glib_none().0,
                n_fds,
                map_fds.to_glib_none().0,
                n_map_fds,
                spawn_flags.into_glib(),
                child_setup,
                Box_::into_raw(super_callback0) as *mut _,
                destroy_call12,
                timeout,
                cancellable.map(|p| p.as_ref()).to_glib_none().0,
                callback,
                Box_::into_raw(super_callback1) as *mut _,
            );
        }
    }
}
