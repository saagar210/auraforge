use crate::error::AppError;
use crate::types::PlanningTemplate;

const TEMPLATE_FILES: &[&str] = &[
    include_str!("../../templates/saas-web-app.json"),
    include_str!("../../templates/tauri-desktop-app.json"),
    include_str!("../../templates/cli-tool.json"),
    include_str!("../../templates/api-service.json"),
    include_str!("../../templates/backend-service.json"),
    include_str!("../../templates/internal-it-automation.json"),
];

pub fn list_templates() -> Result<Vec<PlanningTemplate>, AppError> {
    TEMPLATE_FILES
        .iter()
        .map(|raw| {
            serde_json::from_str::<PlanningTemplate>(raw)
                .map_err(|err| AppError::Config(format!("Template catalog parse error: {}", err)))
        })
        .collect()
}

pub fn get_template(template_id: &str) -> Result<PlanningTemplate, AppError> {
    let templates = list_templates()?;
    templates
        .into_iter()
        .find(|template| template.id == template_id)
        .ok_or_else(|| {
            AppError::Validation(format!(
                "Unknown template '{}'. Use list_templates to view supported ids.",
                template_id
            ))
        })
}
