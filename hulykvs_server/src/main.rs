//
// Copyright © 2025 Hardcore Engineering Inc.
//
// Licensed under the Eclipse Public License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License. You may
// obtain a copy of the License at https://www.eclipse.org/legal/epl-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::pin::Pin;

use actix_cors::Cors;
use actix_web::{
    App, HttpMessage, HttpServer,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{self, Next},
    web::{self, Data},
};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres as pg;
use tracing::info;

mod config;
mod handlers;
mod token;

use config::CONFIG;
use token::ServiceRequestExt;

pub type Pool = bb8::Pool<PostgresConnectionManager<tokio_postgres::NoTls>>;

fn initialize_tracing(level: tracing::Level) {
    use tracing_subscriber::{filter::targets::Targets, prelude::*};

    let filter = Targets::default()
        .with_target(env!("CARGO_BIN_NAME"), level)
        .with_target("actix", level);
    let format = tracing_subscriber::fmt::layer().compact();

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .init();
}

async fn interceptor(
    request: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let claims = request.extract_claims(CONFIG.token_secret.as_bytes())?;

    request.extensions_mut().insert(claims.to_owned());

    next.call(request).await
}

#[derive(Debug)]
struct ConnectionCustomizer;

impl bb8::CustomizeConnection<pg::Client, pg::Error> for ConnectionCustomizer {
    fn on_acquire<'a>(
        &'a self,
        client: &'a mut pg::Client,
    ) -> Pin<Box<dyn Future<Output = Result<(), pg::Error>> + Send + 'a>> {
        Box::pin(async {
            client
                .execute("set search_path to $1", &[&CONFIG.db_namespace])
                .await
                .unwrap();
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    initialize_tracing(tracing::Level::DEBUG);

    tracing::info!("{}/{}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));

    let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
        &CONFIG.db_connection,
        tokio_postgres::NoTls,
    )?;

    let pool = bb8::Pool::builder()
        .max_size(15)
        .connection_customizer(Box::new(ConnectionCustomizer))
        .build(manager)
        .await?;
    {
        refinery::embed_migrations!("etc/migrations");

        let mut connection = pool.dedicated_connection().await?;

        // query params cannot be bound in ddl statements
        connection
            .execute(
                &format!("create schema if not exists {}", CONFIG.db_namespace),
                &[],
            )
            .await?;

        let report = migrations::runner()
            .set_migration_table_name("migrations")
            .run_async(&mut connection)
            .await?;

        for m in report.applied_migrations().iter() {
            info!(migration = m.to_string(), "applied migration")
        }
    }

    let socket = std::net::SocketAddr::new(CONFIG.bind_host.as_str().parse()?, CONFIG.bind_port);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(
                web::scope("/api")
                    .wrap(middleware::from_fn(interceptor))
                    .route("/{bucket}", web::get().to(handlers::list))
                    .route("/{bucket}/{id}", web::get().to(handlers::get))
                    .route("/{bucket}/{id}", web::post().to(handlers::post))
                    .route("/{bucket}/{id}", web::delete().to(handlers::delete)),
            )
            .route("/status", web::get().to(async || "ok"))
    })
    .bind(socket)?
    .run();

    server.await?;

    Ok(())
}
