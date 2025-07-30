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
    HttpResponse, error,
    web::{self, Data, Json, Query},
};
use serde::{Deserialize, Serialize};
use tracing::{error, trace};

use super::Pool;

type BucketPath = web::Path<(String, String)>;
type ObjectPath = web::Path<(String, String, String)>;

//           ____   _____   _____
//          / ___| | ____| |_   _|
//         | |  _  |  _|     | |
//         | |_| | | |___    | |
//          \____| |_____|   |_|
//

pub async fn get(
    path: ObjectPath,
    pool: Data<Pool>,
) -> Result<HttpResponse, actix_web::error::Error> {
    let (workspace, namespace, key) = path.into_inner();
    trace!(workspace, namespace, key, "get request");

    let wsuuid = Uuid::parse_str(workspace.as_str())
	    .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;
    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;

        let statement = r#"
           select value from kvs where workspace=$1 and namespace=$2 and key=$3
        "#;

        let result = connection.query(statement, &[&wsuuid, &nsstr, &keystr]).await?;

        let response = match result.as_slice() {
            [] => HttpResponse::NotFound().finish(),
            [found] => HttpResponse::Ok().body(found.get::<_, Vec<u8>>("value")),
            _ => panic!("multiple rows found, unique constraint is probably violated"),
        };

        Ok(response)
    }()
    .await
    .map_err(|error| {
        error!(op = "get", workspace, namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}

//          ____   ___  ____ _____                                                 _
//         |  _ \ / _ \/ ___|_   _|            _   _   _ __    ___    ___   _ __  | |_
//         | |_) | | | \___ \ | |     _____   | | | | | '_ \  / __|  / _ \ | '__| | __|
//         |  __/| |_| |___) || |    |_____|  | |_| | | |_) | \__ \ |  __/ | |    | |_
//         |_|    \___/|____/ |_|              \__,_| | .__/  |___/  \___| |_|     \__|
//                                                    |_|
//

pub async fn post(
    path: ObjectPath,
    pool: Data<Pool>,
    body: web::Bytes,
) -> Result<HttpResponse, actix_web::error::Error> {
    let (workspace, namespace, key) = path.into_inner();
    trace!(workspace, namespace, key, "post request");

    let wsuuid = Uuid::parse_str(workspace.as_str())
	    .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;
    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;

        let md5 = md5::compute(&body);

        let statement = r#"
            INSERT INTO kvs(workspace, namespace, key, md5, value)
            VALUES($1, $2, $3, $4, $5)
            ON CONFLICT (workspace, namespace, key)
            DO UPDATE SET
                md5 = excluded.md5,
                value = excluded.value
        "#;

        connection
            .execute(statement, &[&wsuuid, &nsstr, &keystr, &&md5[..], &&body[..]])
            .await?;

        Ok(HttpResponse::NoContent().finish())
    }()
    .await
    .map_err(|error| {
        error!(op = "upsert", workspace, namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}


//          ____   ___  ____ _____             _                               _
//         |  _ \ / _ \/ ___|_   _|           (_)  _ __    ___    ___   _ __  | |_
//         | |_) | | | \___ \ | |     _____   | | | '_ \  / __|  / _ \ | '__| | __|
//         |  __/| |_| |___) || |    |_____|  | | | | | | \__ \ |  __/ | |    | |_
//         |_|    \___/|____/ |_|             |_| |_| |_| |___/  \___| |_|     \__|
//

pub async fn insert(
    path: ObjectPath,
    pool: Data<Pool>,
    body: web::Bytes,
) -> Result<HttpResponse, actix_web::error::Error> {
    let (workspace, namespace, key) = path.into_inner();
    trace!(workspace, namespace, key, "insert request");

    let wsuuid = Uuid::parse_str(workspace.as_str())
        .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;
    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;
        let md5 = md5::compute(&body);

        let statement = r#"
            INSERT INTO kvs(workspace, namespace, key, md5, value)
            VALUES($1, $2, $3, $4, $5)
        "#;

        connection
            .execute(statement, &[&wsuuid, &nsstr, &keystr, &&md5[..], &&body[..]])
            .await?;

        Ok(HttpResponse::Created().finish())
    }()
    .await
    .map_err(|error| {
        error!(op = "insert", workspace, namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}



//          ____   ___  ____ _____                                 _           _
//         |  _ \ / _ \/ ___|_   _|            _   _   _ __     __| |   __ _  | |_    ___
//         | |_) | | | \___ \ | |     _____   | | | | | '_ \   / _` |  / _` | | __|  / _ \
//         |  __/| |_| |___) || |    |_____|  | |_| | | |_) | | (_| | | (_| | | |_  |  __/
//         |_|    \___/|____/ |_|              \__,_| | .__/   \__,_|  \__,_|  \__|  \___|
//                                                    |_|
//

pub async fn update(
    path: ObjectPath,
    pool: Data<Pool>,
    body: web::Bytes,
) -> Result<HttpResponse, actix_web::error::Error> {
    let (workspace, namespace, key) = path.into_inner();
    trace!(workspace, namespace, key, "update request");

    let wsuuid = Uuid::parse_str(workspace.as_str())
        .map_err(|e| error::ErrorBadRequest(format!("Invalid UUID in workspace: {}", e)))?;
    let nsstr = namespace.as_str();
    let keystr = key.as_str();

    async move || -> anyhow::Result<HttpResponse> {
        let connection = pool.get().await?;
        let md5 = md5::compute(&body);

        let statement = r#"
            UPDATE kvs
            SET md5 = $4, value = $5
            WHERE workspace = $1 AND namespace = $2 AND key = $3
        "#;

        let updated = connection
            .execute(statement, &[&wsuuid, &nsstr, &keystr, &&md5[..], &&body[..]])
            .await?;

        if updated == 0 {
            return Err(anyhow::anyhow!("Not found"));
        }

        Ok(HttpResponse::NoContent().finish())
    }()
    .await
    .map_err(|error| {
        error!(op = "update", workspace, namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}









//          ____    _____   _       _____   _____   _____
//         |  _ \  | ____| | |     | ____| |_   _| | ____|
//         | | | | |  _|   | |     |  _|     | |   |  _|
//         | |_| | | |___  | |___  | |___    | |   | |___
//         |____/  |_____| |_____| |_____|   |_|   |_____|
//

pub async fn delete(
    path: ObjectPath,
    pool: Data<Pool>,
) -> Result<HttpResponse, actix_web::error::Error> {
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

        let response = match connection.execute(statement, &[&wsuuid, &nsstr, &keystr]).await? {
            1 => HttpResponse::NoContent(),
            0 => HttpResponse::NotFound(),
            _ => panic!("multiple rows deleted, unique constraint is probably violated"),
        };

        Ok(response.into())
    }()
    .await
    .map_err(|error| {
        error!(op = "delete", workspace, namespace, key, ?error, "internal error");
        error::ErrorInternalServerError("")
    })
}

//          _       ___   ____    _____
//         | |     |_ _| / ___|  |_   _|
//         | |      | |  \___ \    | |
//         | |___   | |   ___) |   | |
//         |_____| |___| |____/    |_|
//

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
    path: BucketPath,
    pool: Data<Pool>,
    query: Query<ListInfo>,
) -> Result<Json<ListResponse>, actix_web::error::Error> {
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

            connection.query(statement, &[&wsuuid, &nsstr, &pattern]).await?
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
