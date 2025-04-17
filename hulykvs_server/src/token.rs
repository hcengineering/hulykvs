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

use std::collections::HashMap;

use actix_web::{dev::ServiceRequest, error};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Claims {
    pub account: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
}

pub trait ServiceRequestExt {
    fn extract_token_raw(&self) -> Result<&str, actix_web::Error>;
    fn extract_claims(&self, secret: &[u8]) -> Result<Claims, actix_web::Error>;
}

impl ServiceRequestExt for ServiceRequest {
    fn extract_token_raw(&self) -> Result<&str, actix_web::Error> {
        self.headers()
            .get("Authorization")
            .and_then(|v| v.to_str().map(|s| s.strip_prefix("Bearer ")).ok())
            .flatten()
            .ok_or_else(|| error::ErrorUnauthorized("NoToken"))
    }

    fn extract_claims(&self, secret: &[u8]) -> Result<Claims, actix_web::Error> {
        use jwt::{Algorithm, DecodingKey, Validation, decode};
        use std::collections::HashSet;

        let token = self.extract_token_raw()?;

        let key = DecodingKey::from_secret(secret);
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims = HashSet::new();

        let claims = decode::<Claims>(token, &key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|_e| error::ErrorUnauthorized("InvalidToken"))?;

        Ok(claims)
    }
}
