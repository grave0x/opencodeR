use opencode_core::*;
use opencode_schema::project::ProjectInfo;

pub struct AppState {
    pub agent: Box<dyn AgentService + Send + Sync>,
    pub catalog: Box<dyn CatalogService + Send + Sync>,
    pub session: Box<dyn SessionService + Send + Sync>,
    pub pty: Box<dyn PtyService + Send + Sync>,
    pub permission: Box<dyn PermissionService + Send + Sync>,
    pub question: Box<dyn QuestionService + Send + Sync>,
    pub filesystem: Box<dyn FileSystemService + Send + Sync>,
    pub integration: Box<dyn IntegrationService + Send + Sync>,
    pub credential: Box<dyn CredentialService + Send + Sync>,
    pub command: Box<dyn CommandService + Send + Sync>,
    pub skill: Box<dyn SkillService + Send + Sync>,
    pub reference: Box<dyn ReferenceService + Send + Sync>,
    pub event: Box<dyn EventService + Send + Sync>,
    pub project_copy: Box<dyn ProjectCopyService + Send + Sync>,
    pub projects: Vec<ProjectInfo>,
}

impl AppState {
    pub fn new() -> Self {
        let (
            agent, catalog, session, pty, permission, question, filesystem,
            integration, credential, command, skill, reference, event, project_copy,
        ) = opencode_core::memory::default_services();
        Self {
            agent,
            catalog,
            session,
            pty,
            permission,
            question,
            filesystem,
            integration,
            credential,
            command,
            skill,
            reference,
            event,
            project_copy,
            projects: Vec::new(),
        }
    }
}
