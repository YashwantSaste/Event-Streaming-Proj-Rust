use std::path::Path;
use common::error::application_error::ApplicationError;
use crate::base::command_result::CommandResult;

pub struct Initializer<'a>{
    workspace_name: &'a str,
    path: Option<&'a Path>,
}


impl<'a> Initializer<'a>{

    pub fn exist(&self) -> bool{
        return self.path.unwrap().exists();
    }

    pub fn init(&self) -> Result<CommandResult, ApplicationError>{
        if self.exist(){
            ApplicationError::new ( format!(
                "Workspace already exists at: {}",
                self.path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| ".".to_string())
            ));
        }

        // few other command
        Ok(CommandResult{
            success: true,
            exit_code: 0,
            message: format!(
                "Workspace has been successfully initialized at: {}",
                self.path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| ".".to_string())
            ),
        })
    }

    // other generic methods for workspace init
}