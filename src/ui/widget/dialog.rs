mod archive_prefix;
mod check;
mod check_result;
mod delete_archive;
mod device_missing;
mod exclude;
mod preferences;
mod prune;
mod prune_review;

pub use archive_prefix::ArchivePrefixDialog;
pub use check::CheckDialog;
pub use check_result::CheckResultDialog;
pub use delete_archive::DeleteArchiveDialog;
pub use device_missing::DeviceMissingDialog;
pub use exclude::ExcludeDialog;
pub use preferences::PreferencesDialog;
pub use prune::PruneDialog;
pub use prune_review::PruneReviewDialog;

use glib::prelude::*;

pub fn init() {
    ArchivePrefixDialog::static_type();
    CheckDialog::static_type();
    CheckResultDialog::static_type();
    DeleteArchiveDialog::static_type();
    DeviceMissingDialog::static_type();
    ExcludeDialog::static_type();
    PreferencesDialog::static_type();
    PruneDialog::static_type();
    PruneReviewDialog::static_type();
}
