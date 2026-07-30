use crate::base::cli_error::CliError;
use crate::workspace::workspace::Workspace;

pub trait TemplateGenerator {
    fn generate(&self, workspace: &Workspace) -> Result<(), CliError>;
}