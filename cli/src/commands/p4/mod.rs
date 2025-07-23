// Copyright 2020-2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused)]

mod info;
mod root;

use std::path::Path;

use clap::Subcommand;
use jj_lib::config::ConfigFile;
use jj_lib::config::ConfigSource;
use jj_lib::git;
use jj_lib::git::UnexpectedGitBackendError;
use jj_lib::ref_name::RemoteNameBuf;
use jj_lib::ref_name::RemoteRefSymbol;
use jj_lib::store::Store;

use self::info::cmd_p4_info;
use self::info::P4InfoArgs;
use self::root::cmd_p4_root;
use self::root::P4RootArgs;
use crate::cli_util::CommandHelper;
use crate::cli_util::WorkspaceCommandHelper;
use crate::command_error::user_error_with_message;
use crate::command_error::CommandError;
use crate::ui::Ui;

/// Commands for P4 (Perforce) working with P4 remotes and the underlying P4 depot
///
/// See this [perforce]
///
/// [perforce]:
///     https://help.perforce.com/helix-core/server-apps/cmdref/current/Content/CmdRef/Home-cmdref.html
#[derive(Subcommand, Clone, Debug)]
pub enum P4Command {
    Info(P4InfoArgs),
    Root(P4RootArgs),
}

pub fn cmd_p4(
    ui: &mut Ui,
    command: &CommandHelper,
    subcommand: &P4Command,
) -> Result<(), CommandError> {
    match subcommand {
        P4Command::Info(args) => cmd_p4_info(ui, command, args),
        P4Command::Root(args) => cmd_p4_root(ui, command, args),
    }
}

pub fn maybe_add_gitignore(workspace_command: &WorkspaceCommandHelper) -> Result<(), CommandError> {
    if workspace_command.working_copy_shared_with_git() {
        std::fs::write(
            workspace_command
                .workspace_root()
                .join(".jj")
                .join(".gitignore"),
            "/*\n",
        )
        .map_err(|e| user_error_with_message("Failed to write .jj/.gitignore file", e))
    } else {
        Ok(())
    }
}

fn get_single_remote(store: &Store) -> Result<Option<RemoteNameBuf>, UnexpectedGitBackendError> {
    let mut names = git::get_all_remote_names(store)?;
    Ok(match names.len() {
        1 => names.pop(),
        _ => None,
    })
}

/// Sets repository level `trunk()` alias to the specified remote symbol.
fn write_repository_level_trunk_alias(
    ui: &Ui,
    repo_path: &Path,
    symbol: RemoteRefSymbol<'_>,
) -> Result<(), CommandError> {
    let mut file = ConfigFile::load_or_empty(ConfigSource::Repo, repo_path.join("config.toml"))?;
    file.set_value(["revset-aliases", "trunk()"], symbol.to_string())
        .expect("initial repo config shouldn't have invalid values");
    file.save()?;
    writeln!(
        ui.status(),
        "Setting the revset alias `trunk()` to `{symbol}`",
    )?;
    Ok(())
}
