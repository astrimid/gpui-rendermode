#[cfg(all(feature = "gpui-unofficial", feature = "gpui-ce"))]
compile_error!("Features 'gpui-unofficial' and 'gpui-ce' are mutually exclusive. Pick one.");

#[cfg(not(any(feature = "gpui-unofficial", feature = "gpui-ce")))]
compile_error!("You must enable at least one backend feature ('gpui-official' or 'gpui-ce').");

#[cfg(feature = "gpui-unofficial")]
pub use gpui_unofficial as gpui;

#[cfg(feature = "gpui-ce")]
pub use gpui_ce as gpui;
