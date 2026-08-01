use std::path::PathBuf;

use super::resolver::WorkspaceResolver;
use super::validator::WorkspaceValidator;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::templates::workspace_template_generator::WorkspaceTemplateGenerator;
use crate::workspace::directory::Directory;

/// Coordinates workspace resolution, validation, directory creation, and templates.
pub struct WorkspaceInitializer {
    workspace_name: String,
    path: Option<PathBuf>,
    template_generator: WorkspaceTemplateGenerator,
}

impl WorkspaceInitializer {
    /// Creates a workspace initializer with the selected template strategy.
    pub fn new(
        workspace_name: String,
        path: Option<PathBuf>,
        template_generator: WorkspaceTemplateGenerator,
    ) -> Self {
        Self {
            workspace_name,
            path,
            template_generator,
        }
    }

    /// Initializes a workspace on disk.
    pub fn initialize(&self) -> Result<CommandResult, CliError> {
        let workspace = WorkspaceResolver::resolve(&self.workspace_name, self.path.as_ref())?;
        WorkspaceValidator::validate(&workspace)?;
        Directory::create(&workspace)?;
        self.template_generator.generate(&workspace)?;

        // Set the active workspace
        if let Some(home) = dirs::home_dir() {
            let active_ws_file = home.join(".es_workspace");
            if let Err(e) = std::fs::write(&active_ws_file, workspace.root().display().to_string()) {
                eprintln!("Warning: Failed to set active workspace: {}", e);
            }
        }

        Ok(CommandResult {
            success: true,
            exit_code: 0,
            message: format!("Workspace initialized at {}", workspace.root().display()),
        })
    }
}
