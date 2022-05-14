//! Webserver request handlers
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::Extension;
use axum::response::Response;
use comm_types::auth::JwtClaims;

use super::backend::auth::BackendAuthSchema;
use super::backend::BackendSchema;

pub(super) async fn backend_ws_handler(
    ws: WebSocketUpgrade,
    Extension(claims): Extension<JwtClaims>,
) -> Response {
    log::warn!("received ws request from role {:?}", claims.role);
    ws.on_upgrade(|socket| stream_handler(socket, claims))
}

async fn stream_handler(mut socket: WebSocket, claims: JwtClaims) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(t) => {
                    println!("client sent str: {:?}", t);
                }
                Message::Binary(_) => {
                    println!("client sent binary data");
                }
                Message::Ping(_) => {
                    println!("socket ping");
                }
                Message::Pong(_) => {
                    println!("socket pong");
                }
                Message::Close(_) => {
                    println!("client disconnected");
                    return;
                }
            }
        } else {
            println!("client disconnected");
            return;
        }
    }

    loop {
        if socket
            .send(Message::Text(format!("Hi from role: {:?}", claims.role)))
            .await
            .is_err()
        {
            println!("client disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

pub(super) async fn graphql_backend(
    schema: Extension<BackendSchema>,
    req: GraphQLRequest,
    Extension(claims): Extension<JwtClaims>,
) -> GraphQLResponse {
    schema.execute(req.into_inner().data(claims)).await.into()
}

pub(super) async fn graphql_backend_auth(
    schema: Extension<BackendAuthSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
