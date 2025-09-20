use crate::{AppState, User, error::ErrorOutput, handlers::*, models::*};
use axum::Router;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            );
        }
    }
}

pub(crate) trait OpenApiRouter {
    fn openapi(self) -> Self;
}

#[derive(OpenApi)]
#[openapi(
    paths(
        signup_handler,
        signin_handler,
        list_chat_handler,
        create_chat_handler,
        get_chat_handler,
        list_message_handler,
        send_message_handler,
        list_chat_users_handler,
    ),
    components(schemas(
        User,
        Chat,
        ChatType,
        Message,
        ChatUser,
        Workspace,
        SigninUser,
        CreateUser,
        CreateChat,
        CreateMessage,
        ListMessages,
        AuthOutput,
        ErrorOutput
    )),
    modifiers(&SecurityAddon),
    tags((name = "chat", description = "Chat related operations"))
)]
pub(crate) struct ApiDoc;

impl OpenApiRouter for Router<AppState> {
    fn openapi(self) -> Self {
        self.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openai.json", ApiDoc::openapi()))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    }
}
