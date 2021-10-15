#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate enclose;
#[macro_use]
extern crate async_recursion;
#[macro_use]
pub mod prelude;

const REPO_MOUNT_DIR: &str = ".mnt/borg";

// require borg 1.1
const BORG_MIN_MAJOR: u32 = 1;
const BORG_MIN_MINOR: u32 = 1;

const NON_JOURNALING_FILESYSTEMS: &[&str] = &["exfat", "ext2", "vfat"];

macro_rules! data_dir {
    () => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/data")
    };
}

macro_rules! application_id {
    () => {
        include_str!(concat!(data_dir!(), "/APPLICATION_ID"))
    };
}

pub fn app_id() -> String {
    format!(
        "{}{}",
        application_id!(),
        option_env!("APPLICATION_ID_SUFFIX").unwrap_or_default()
    )
}

pub fn daemon_app_id() -> String {
    format!("{}.Daemon", app_id())
}

pub mod action;
pub mod borg;
pub mod config;
pub mod daemon;
pub mod globals;
pub mod ui;
pub mod utils;
