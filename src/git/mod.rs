mod branches;
mod commits;
mod common;
mod forward;
mod log;
mod push;
mod remote;
mod staging;
mod status;
mod tags;
mod types;

pub use branches::{
    create_and_switch_branch, current_branch, list_local_branches, switch_branch, upstream,
};
pub use commits::{commit, commit_amend, last_commit_files, last_commit_message, soft_reset};
pub use common::get_repo_root;
pub use forward::forward_command;
pub use log::log_graph;
pub use push::{push, push_force};
pub use remote::{branch_rename_to_main, has_remote, push_origin_main, remote_add_origin};
pub use staging::{stage_files, unstage_files};
pub use status::get_status;
pub use tags::{create_tag, latest_release_tag, push_tag};
pub use types::{Branch, FileChange, FileStatus, PushResult};
