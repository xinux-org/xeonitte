use glib::translate::*;
use io_lifetimes::{BorrowedFd, OwnedFd};
use std::boxed::Box as Box_;
use std::os::unix::io::{IntoRawFd, RawFd};

use std::ptr;

use crate::{ffi, prelude::*, Pty};

impl Pty {
    #[doc(alias = "vte_pty_get_fd")]
    #[doc(alias = "get_fd")]
    pub fn fd(&self) -> BorrowedFd<'_> {
        unsafe {
            let raw_fd = ffi::vte_pty_get_fd(self.to_glib_none().0);
            BorrowedFd::borrow_raw(raw_fd)
        }
    }

    #[doc(alias = "vte_pty_new_foreign_sync")]
    #[doc(alias = "new_foreign_sync")]
    pub fn foreign_sync(
        fd: OwnedFd,
        cancellable: Option<&impl IsA<gio::Cancellable>>,
    ) -> Result<Pty, glib::Error> {
        assert_initialized_main_thread!();
        unsafe {
            let mut error = ptr::null_mut();
            let ret = ffi::vte_pty_new_foreign_sync(
                fd.into_raw_fd(),
                cancellable.map(|p| p.as_ref()).to_glib_none().0,
                &mut error,
            );
            if error.is_null() {
                Ok(from_glib_full(ret))
            } else {
                Err(from_glib_full(error))
            }
        }
    }

    #[doc(alias = "vte_pty_spawn_async")]
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_async<P: FnOnce(Result<glib::Pid, glib::Error>) + 'static, Q: Fn() + 'static>(
        &self,
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
            _source_object: *mut glib::gobject_ffi::GObject,
            res: *mut gio::ffi::GAsyncResult,
            user_data: glib::ffi::gpointer,
        ) {
            let mut error = ptr::null_mut();
            let mut child_pid = 0;
            let _ = ffi::vte_pty_spawn_finish(
                _source_object as *mut _,
                res,
                &mut child_pid,
                &mut error,
            );
            let result = if error.is_null() {
                Ok(from_glib(child_pid))
            } else {
                Err(from_glib_full(error))
            };
            let callback: Box_<glib::thread_guard::ThreadGuard<P>> =
                Box_::from_raw(user_data as *mut _);
            let callback: P = callback.into_inner();
            callback(result);
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
            ffi::vte_pty_spawn_async(
                self.to_glib_none().0,
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

    #[doc(alias = "vte_pty_spawn_async")]
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_future<Q: Fn() + 'static>(
        &self,
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

    /// # Safety
    ///
    /// The map_fds have to make sense.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn spawn_with_fds_async<
        P: FnOnce(Result<glib::Pid, glib::Error>) + 'static,
        Q: Fn() + 'static,
    >(
        &self,
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

        unsafe extern "C" fn spawn_with_fds_trampoline<
            P: FnOnce(Result<glib::Pid, glib::Error>) + 'static,
        >(
            _source_object: *mut glib::gobject_ffi::GObject,
            res: *mut gio::ffi::GAsyncResult,
            user_data: glib::ffi::gpointer,
        ) {
            let mut error = ptr::null_mut();
            let mut child_pid = 0;
            let _ = ffi::vte_pty_spawn_finish(
                _source_object as *mut _,
                res,
                &mut child_pid,
                &mut error,
            );
            let result = if error.is_null() {
                Ok(from_glib(child_pid))
            } else {
                Err(from_glib_full(error))
            };
            let callback: Box_<glib::thread_guard::ThreadGuard<P>> =
                Box_::from_raw(user_data as *mut _);
            let callback = callback.into_inner();
            callback(result)
        }

        let callback = Some(spawn_with_fds_trampoline::<P> as _);

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
            ffi::vte_pty_spawn_with_fds_async(
                self.to_glib_none().0,
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

    // # Safety
    //
    // The map_fds have to make sense.
    #[doc(alias = "vte_pty_spawn_async")]
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn spawn_with_fds_future<Q: Fn() + 'static>(
        &self,
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
        let argv: Vec<String> = argv.iter().map(|x| x.to_string()).collect();
        let envv: Vec<String> = envv.iter().map(|x| x.to_string()).collect();
        let map_fds = map_fds.to_vec();
        Box_::pin(gio::GioFuture::new(self, move |obj, cancellable, send| {
            let argv: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();
            let envv: Vec<&str> = envv.iter().map(|s| s.as_str()).collect();
            obj.spawn_with_fds_async(
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
}
