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

use actix_web::{
    HttpResponse, error,
    web::{self, Data, Json, Query},
};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use super::Pool;

type BucketPath = web::Path<String>;
type ObjectPath = web::Path<(String, String)>;

pub async fn get(
    path: ObjectPath,
    pool: Data<Pool>,
) -> Result<HttpResponse, actix_web::error::Error> {
    let (namespace, key) = path.into_inner();
    trace!(namespace, key, "post request");

    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;

        let statement = r#"
           select value from kvs where namespace=$1 and key=$2
        "#;

        let result = connection.query(statement, &[&nsstr, &keystr]).await?;

        let response = match result.as_slice() {
            [] => HttpResponse::NotFound().finish(),
            [found] => HttpResponse::Ok().body(found.get::<_, Vec<u8>>("value")),
            _ => panic!("multiple rows found, unique constraint is probably violated"),
        };

        Ok(response)
    }()
    .await
    .map_err(|error| {
        error!(op = "get", namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}

pub async fn post(
    path: ObjectPath,
    pool: Data<Pool>,
    body: web::Bytes,
) -> Result<HttpResponse, actix_web::error::Error> {
    let (namespace, key) = path.into_inner();
    trace!(namespace, key, "post request");

    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;

        let md5 = md5::compute(&body);

        let statement = r#"
           insert into kvs(namespace, key, md5, value) 
           values($1, $2, $3, $4)
           on conflict(namespace, key)
           do update set 
            md5=excluded.md5, 
            value=excluded.value
        "#;

        connection
            .execute(statement, &[&nsstr, &keystr, &&md5[..], &&body[..]])
            .await?;

        Ok(HttpResponse::NoContent().finish())
    }()
    .await
    .map_err(|error| {
        error!(op = "upsert", namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}

pub async fn delete(
    path: ObjectPath,
    pool: Data<Pool>,
) -> Result<HttpResponse, actix_web::error::Error> {
    let (namespace, key) = path.into_inner();
    trace!(namespace, key, "delete request");

    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;

        let statement = r#"
           delete from kvs where namespace=$1 and key=$2
        "#;

        let response = match connection.execute(statement, &[&nsstr, &keystr]).await? {
            1 => HttpResponse::NoContent(),
            0 => HttpResponse::NotFound(),
            _ => panic!("multiple rows deleted, unique constraint is probably violated"),
        };

        Ok(response.into())
    }()
    .await
    .map_err(|error| {
        error!(op = "delete", namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}

#[derive(Deserialize)]
pub struct ListInfo {
    prefix: Option<String>,
}

#[derive(Serialize)]
pub struct ListResponse {
    namespace: String,
    count: usize,
    keys: Vec<String>,
}

pub async fn list(
    path: BucketPath,
    pool: Data<Pool>,
    query: Query<ListInfo>,
) -> Result<Json<ListResponse>, actix_web::error::Error> {
    let namespace = path.into_inner();
    trace!(namespace, prefix = ?query.prefix, "enumerate request");

    let nsstr = namespace.as_str();

    async move || -> anyhow::Result<Json<ListResponse>> {
        let connection = pool.get().await?;

        let response = if let Some(prefix) = &query.prefix {
            let pattern = format!("{}%", prefix);
            let statement = r#"
                select key from kvs where namespace=$1 and key like $2
            "#;

            connection.query(statement, &[&nsstr, &pattern]).await?
        } else {
            let statement = r#"
                select key from kvs where namespace=$1
            "#;

            connection.query(statement, &[&nsstr]).await?
        };

        let count = response.len();

        let mut keys = Vec::new();

        for row in response {
            keys.push(row.get::<_, String>(0));
        }

        Ok(Json(ListResponse {
            keys,
            count,
            namespace: nsstr.to_owned(),
        }))
    }()
    .await
    .map_err(|error| {
        error!(op = "list", namespace, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}
