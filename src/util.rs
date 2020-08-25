#![macro_use]
macro_rules! evaluate_fallible_winapi {
    ($e:expr) => {
        let winapi_success: i32 = $e;

        if winapi_success == 0 {
            log::error!(
                "Error from winapi in expression {}: {}",
                stringify!($e),
                winapi::um::errhandlingapi::GetLastError(),
            );

            return Err(());
        }
    };
}
