use crate::base::cli_error::CliError;
use crate::templates::template_generator::TemplateGenerator;
use crate::workspace::workspace::Workspace;

pub struct WorkspaceTemplateGenerator {
    generators: Vec<Box<dyn TemplateGenerator>>
}

impl WorkspaceTemplateGenerator {

    pub fn new(generators: Vec<Box<dyn TemplateGenerator>>) -> Self {
        Self {generators}
    }

    pub fn generate(&self, workspace: &Workspace) -> Result<(), CliError> {
        for generator in &self.generators {
            generator.generate(workspace)?;
        }
        Ok(())
    }

}