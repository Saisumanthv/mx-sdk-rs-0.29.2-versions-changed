use crate::model::{BytesValue, U64Value};

use super::DctInstance;

#[derive(Debug, Default)]
pub struct DctObject {
    pub token_identifier: Option<BytesValue>,
    pub instances: Vec<DctInstance>,
    pub last_nonce: Option<U64Value>,
    pub roles: Vec<BytesValue>,
    pub frozen: Option<U64Value>,
}
