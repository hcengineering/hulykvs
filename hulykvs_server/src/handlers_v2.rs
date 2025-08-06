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

use uuid::Uuid;

use actix_web::{
    Error, HttpMessage, HttpRequest, HttpResponse, error,
    web::{self, Data, Json, Query},
};

use hulyrs::services::jwt::Claims;

use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use super::Pool;

type BucketPath = web::Path<(String, String)>;
type ObjectPath = web::Path<(String, String, String)>;

pub async fn get(
    req: HttpRequest,
    path: ObjectPath,
    pool: Data<Pool>,
) -> Result<HttpResponse, actix_web::error::Error> {
    workspace_owner(&req)?; // Check workspace

    let (workspace, namespace, key) = path.into_inner();
    trace!(workspace, namespace, key, "get request");

    let wsuuid = Uuid::parse_str(workspace.as_str())
        .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;
    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;

        let statement = r#"
           select value, md5 from kvs where workspace=$1 and namespace=$2 and key=$3
        "#;

        let result = connection
            .query(statement, &[&wsuuid, &nsstr, &keystr])
            .await?;

        let response = match result.as_slice() {
            [] => HttpResponse::NotFound().finish(),
            [row] => {
                let value: Vec<u8> = row.get("value");
                let md5: &[u8] = row.get("md5");
                let md5_hex = hex::encode(md5);
                HttpResponse::Ok()
                    .insert_header(("ETag", md5_hex))
                    .body(value)
            }
            _ => panic!("multiple rows found, unique constraint is probably violated"),
        };

        Ok(response)
    }()
    .await
    .map_err(|error| {
        error!(
            op = "get",
            workspace,
            namespace,
            key,
            ?error,
            "internal error"
        );
        error::ErrorInternalServerError("")
    })
}

pub async fn put(
    req: HttpRequest,
    path: ObjectPath,
    pool: Data<Pool>,
    body: web::Bytes,
) -> Result<HttpResponse, actix_web::error::Error> {
    workspace_owner(&req)?; // Check workspace

    let (workspace, namespace, key) = path.into_inner();
    trace!(workspace, namespace, key, "update request");

    let wsuuid = Uuid::parse_str(workspace.as_str())
        .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;
    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    // header If-Match
    let if_match_header = req.headers().get("If-Match").and_then(|h| h.to_str().ok());

    // header If-None-Match (only *)
    let if_none_match = match req.headers().get("If-None-Match") {
        Some(value) => {
            let value_str = value
                .to_str()
                .map_err(|_| error::ErrorBadRequest("Invalid If-None-Match header encoding"))?;

            if value_str.trim() == "*" {
                true
            } else {
                return Err(error::ErrorBadRequest("If-None-Match must be '*'"));
            }
        }
        None => false,
    };

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;
        let new_md5 = md5::compute(&body);

        // If-None-Match: *
        if if_none_match {
            let result = connection
                .execute(
                    r#"
                    INSERT INTO kvs (workspace, namespace, key, md5, value)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT (workspace, namespace, key) DO NOTHING
                    "#,
                    &[&wsuuid, &nsstr, &keystr, &&new_md5[..], &&body[..]],
                )
                .await?;

            return Ok(if result == 0 {
                HttpResponse::PreconditionFailed().finish()
            } else {
                HttpResponse::Created().finish()
            });
        }

        // If-Match
        if let Some(if_match) = if_match_header {
            if if_match.trim() == "*" {
                // If-Match: *
                let result = connection
                    .execute(
                        r#"
                        UPDATE kvs
                        SET md5 = $4, value = $5
                        WHERE workspace = $1 AND namespace = $2 AND key = $3
                        "#,
                        &[&wsuuid, &nsstr, &keystr, &&new_md5[..], &&body[..]],
                    )
                    .await?;

                return Ok(if result == 0 {
                    HttpResponse::PreconditionFailed().finish()
                } else {
                    HttpResponse::NoContent().finish()
                });
            } else {
                // If-Match: some
                let old_md5 = hex::decode(if_match.trim())
                    .map_err(|_| anyhow::anyhow!("Invalid hex in If-Match"))?;

                let result = connection
                    .execute(
                        r#"
                        UPDATE kvs
                        SET md5 = $5, value = $6
                        WHERE workspace = $1 AND namespace = $2 AND key = $3 AND md5 = $4
                        "#,
                        &[
                            &wsuuid,
                            &nsstr,
                            &keystr,
                            &&old_md5[..],
                            &&new_md5[..],
                            &&body[..],
                        ],
                    )
                    .await?;

                return Ok(if result == 0 {
                    HttpResponse::PreconditionFailed().finish()
                } else {
                    HttpResponse::NoContent().finish()
                });
            }
        }

        // No If-Match, no If-None-Match ==> UPSERT
        let result = connection
            .execute(
                r#"
                INSERT INTO kvs (workspace, namespace, key, md5, value)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (workspace, namespace, key)
                DO UPDATE SET md5 = EXCLUDED.md5, value = EXCLUDED.value
                "#,
                &[&wsuuid, &nsstr, &keystr, &&new_md5[..], &&body[..]],
            )
            .await?;

        Ok(if result == 0 {
            anyhow::bail!("Not found")
        } else {
            HttpResponse::NoContent().finish()
        })
    }()
    .await
    .map_err(|error| {
        error!(
            op = "update",
            workspace,
            namespace,
            key,
            ?error,
            "internal error"
        );
        error::ErrorInternalServerError("internal error")
    })
}

pub async fn delete(
    req: HttpRequest,
    path: ObjectPath,
    pool: Data<Pool>,
) -> Result<HttpResponse, actix_web::error::Error> {
    workspace_owner(&req)?; // Check workspace

    let (workspace, namespace, key) = path.into_inner();
    trace!(workspace, namespace, key, "delete request");

    let wsuuid = Uuid::parse_str(workspace.as_str())
        .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;
    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;

        let statement = r#"
            DELETE FROM kvs WHERE workspace=$1 AND namespace=$2 AND key=$3
        "#;

        let response = match connection
            .execute(statement, &[&wsuuid, &nsstr, &keystr])
            .await?
        {
            1 => HttpResponse::NoContent(),
            0 => HttpResponse::NotFound(),
            _ => panic!("multiple rows deleted, unique constraint is probably violated"),
        };

        Ok(response.into())
    }()
    .await
    .map_err(|error| {
        error!(
            op = "delete",
            workspace,
            namespace,
            key,
            ?error,
            "internal error"
        );
        error::ErrorInternalServerError("")
    })
}

#[derive(Deserialize)]
pub struct ListInfo {
    prefix: Option<String>,
}

#[derive(Serialize)]
pub struct ListResponse {
    workspace: String,
    namespace: String,
    count: usize,
    keys: Vec<String>,
}

pub async fn list(
    req: HttpRequest,
    path: BucketPath,
    pool: Data<Pool>,
    query: Query<ListInfo>,
) -> Result<Json<ListResponse>, actix_web::error::Error> {
    workspace_owner(&req)?; // Check workspace

    let (workspace, namespace) = path.into_inner();
    trace!(workspace, namespace, prefix = ?query.prefix, "list request");

    let wsstr = workspace.as_str();
    let wsuuid = Uuid::parse_str(wsstr)
        .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;

    let nsstr = namespace.as_str();

    async move || -> anyhow::Result<Json<ListResponse>> {
        let connection = pool.get().await?;

        let response = if let Some(prefix) = &query.prefix {
            let pattern = format!("{}%", prefix);
            let statement = r#"
                select key from kvs where workspace=$1 and namespace=$2 and key like $3
            "#;

            connection
                .query(statement, &[&wsuuid, &nsstr, &pattern])
                .await?
        } else {
            let statement = r#"
                select key from kvs where workspace=$1 and namespace=$2
            "#;

            connection.query(statement, &[&wsuuid, &nsstr]).await?
        };

        let count = response.len();

        let keys = response.into_iter().map(|row| row.get(0)).collect();

        Ok(Json(ListResponse {
            keys,
            count,
            namespace: nsstr.to_owned(),
            workspace: wsstr.to_owned(),
        }))
    }()
    .await
    .map_err(|error| {
        error!(op = "list", workspace, namespace, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}

/// Checking workspace in Authorization
pub fn workspace_owner(req: &HttpRequest) -> Result<(), Error> {
    let extensions = req.extensions();

    let claims = extensions
        .get::<Claims>()
        .ok_or_else(|| error::ErrorUnauthorized("Missing auth claims"))?;

    // is_system - allowed to all
    if claims.is_system() {
        return Ok(());
    }

    // else - check workplace
    let jwt_workspace = claims
        .workspace
        .as_ref()
        .ok_or_else(|| error::ErrorForbidden("Missing workspace in token"))?;

    let path_ws = req
        .match_info()
        .get("workspace")
        .ok_or_else(|| error::ErrorBadRequest("Missing workspace in URL path"))?;

    let path_ws_uuid =
        Uuid::parse_str(path_ws).map_err(|_| error::ErrorBadRequest("Invalid workspace UUID"))?;

    if jwt_workspace != &path_ws_uuid {
        return Err(error::ErrorForbidden("Workspace mismatch"));
    }

    Ok(())
}
