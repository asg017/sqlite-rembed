use sqlite_loadable::{Error, Result};

use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn try_env_var(key: &str) -> Result<String> {
    std::env::var(key)
   .map_err(|_| Error::new_message(format!("{} environment variable not define. Alternatively, pass in an API key with rembed_client_options", DEFAULT_OPENAI_API_KEY_ENV)))
}

#[derive(Clone)]
pub struct OpenAiClient {
    model: String,
    url: String,
    key: String,
}
const DEFAULT_OPENAI_URL: &str = "https://api.openai.com/v1/embeddings";
const DEFAULT_OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";

impl OpenAiClient {
    pub fn new<S: Into<String>>(
        model: S,
        url: Option<String>,
        key: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            model: model.into(),
            url: url.unwrap_or(DEFAULT_OPENAI_URL.to_owned()),
            key: match key {
                Some(key) => key,
                None => try_env_var(DEFAULT_OPENAI_API_KEY_ENV)?,
            },
        })
    }
    pub fn infer_single(&self, input: &str) -> Result<Vec<f32>> {
        let body = serde_json::json!({
            "input": input,
            "model": self.model
        });

        let data: serde_json::Value = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .set("Authorization", format!("Bearer {}", self.key).as_str())
            .send_bytes(
                serde_json::to_vec(&body)
                    .map_err(|error| {
                        Error::new_message(format!("Error serializing body to JSON: {error}"))
                    })?
                    .as_ref(),
            )
            .map_err(|error| Error::new_message(format!("Error sending HTTP request: {error}")))?
            .into_json()
            .map_err(|error| {
                Error::new_message(format!("Error parsing HTTP response as JSON: {error}"))
            })?;
        OpenAiClient::parse_single_response(data)
    }

    pub fn parse_single_response(value: serde_json::Value) -> Result<Vec<f32>> {
        value
            .get("data")
            .ok_or_else(|| Error::new_message("expected 'data' key in response body"))
            .and_then(|v| {
                v.get(0)
                    .ok_or_else(|| Error::new_message("expected 'data.0' path in response body"))
            })
            .and_then(|v| {
                v.get("embedding").ok_or_else(|| {
                    Error::new_message("expected 'data.0.embedding' path in response body")
                })
            })
            .and_then(|v| {
                v.as_array().ok_or_else(|| {
                    Error::new_message("expected 'data.0.embedding' path to be an array")
                })
            })
            .and_then(|arr| {
                arr.iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| {
                                Error::new_message(
                                    "expected 'data.0.embedding' array to contain floats",
                                )
                            })
                            .map(|f| f as f32)
                    })
                    .collect()
            })
    }
}

#[derive(Clone)]
pub struct NomicClient {
    model: String,
    url: String,
    key: String,
}
const DEFAULT_NOMIC_URL: &str = "https://api-atlas.nomic.ai/v1/embedding/text";
const DEFAULT_NOMIC_API_KEY_ENV: &str = "NOMIC_API_KEY";

impl NomicClient {
    pub fn new<S: Into<String>>(
        model: S,
        url: Option<String>,
        key: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            model: model.into(),
            url: url.unwrap_or(DEFAULT_NOMIC_URL.to_owned()),
            key: match key {
                Some(key) => key,
                None => try_env_var(DEFAULT_NOMIC_API_KEY_ENV)?,
            },
        })
    }

    pub fn infer_single(&self, input: &str, input_type: Option<&str>) -> Result<Vec<f32>> {
        let mut body = serde_json::Map::new();
        body.insert("texts".to_owned(), vec![input.to_owned()].into());
        body.insert("model".to_owned(), self.model.to_owned().into());

        if let Some(input_type) = input_type {
            body.insert("input_type".to_owned(), input_type.to_owned().into());
        }

        let data: serde_json::Value = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .set("Authorization", format!("Bearer {}", self.key).as_str())
            .send_bytes(
                serde_json::to_vec(&body)
                    .map_err(|error| {
                        Error::new_message(format!("Error serializing body to JSON: {error}"))
                    })?
                    .as_ref(),
            )
            .map_err(|error| Error::new_message(format!("Error sending HTTP request: {error}")))?
            .into_json()
            .map_err(|error| {
                Error::new_message(format!("Error parsing HTTP response as JSON: {error}"))
            })?;
        NomicClient::parse_single_response(data)
    }
    pub fn parse_single_response(value: serde_json::Value) -> Result<Vec<f32>> {
        value
            .get("embeddings")
            .ok_or_else(|| Error::new_message("expected 'embeddings' key in response body"))
            .and_then(|v| {
                v.get(0).ok_or_else(|| {
                    Error::new_message("expected 'embeddings.0' path in response body")
                })
            })
            .and_then(|v| {
                v.as_array().ok_or_else(|| {
                    Error::new_message("expected 'embeddings.0' path to be an array")
                })
            })
            .and_then(|arr| {
                arr.iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| {
                                Error::new_message(
                                    "expected 'embeddings.0' array to contain floats",
                                )
                            })
                            .map(|f| f as f32)
                    })
                    .collect()
            })
    }
}

#[derive(Clone)]
pub struct CohereClient {
    url: String,
    model: String,
    key: String,
}
const DEFAULT_COHERE_URL: &str = "https://api.cohere.com/v1/embed";
const DEFAULT_COHERE_API_KEY_ENV: &str = "CO_API_KEY";

impl CohereClient {
    pub fn new<S: Into<String>>(
        model: S,
        url: Option<String>,
        key: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            model: model.into(),
            url: url.unwrap_or(DEFAULT_COHERE_URL.to_owned()),
            key: match key {
                Some(key) => key,
                None => try_env_var(DEFAULT_COHERE_API_KEY_ENV)?,
            },
        })
    }

    pub fn infer_single(&self, input: &str, input_type: Option<&str>) -> Result<Vec<f32>> {
        let mut body = serde_json::Map::new();
        body.insert("texts".to_owned(), vec![input.to_owned()].into());
        body.insert("model".to_owned(), self.model.to_owned().into());

        if let Some(input_type) = input_type {
            body.insert("input_type".to_owned(), input_type.to_owned().into());
        }

        let data: serde_json::Value = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .set("Authorization", format!("Bearer {}", self.key).as_str())
            .send_bytes(
                serde_json::to_vec(&body)
                    .map_err(|error| {
                        Error::new_message(format!("Error serializing body to JSON: {error}"))
                    })?
                    .as_ref(),
            )
            .map_err(|error| Error::new_message(format!("Error sending HTTP request: {error}")))?
            .into_json()
            .map_err(|error| {
                Error::new_message(format!("Error parsing HTTP response as JSON: {error}"))
            })?;
        CohereClient::parse_single_response(data)
    }
    pub fn parse_single_response(value: serde_json::Value) -> Result<Vec<f32>> {
        value
            .get("embeddings")
            .ok_or_else(|| Error::new_message("expected 'embeddings' key in response body"))
            .and_then(|v| {
                v.get(0).ok_or_else(|| {
                    Error::new_message("expected 'embeddings.0' path in response body")
                })
            })
            .and_then(|v| {
                v.as_array().ok_or_else(|| {
                    Error::new_message("expected 'embeddings.0' path to be an array")
                })
            })
            .and_then(|arr| {
                arr.iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| {
                                Error::new_message(
                                    "expected 'embeddings.0' array to contain floats",
                                )
                            })
                            .map(|f| f as f32)
                    })
                    .collect()
            })
    }
}
#[derive(Clone)]
pub struct JinaClient {
    url: String,
    model: String,
    key: String,
}
const DEFAULT_JINA_URL: &str = "https://api.jina.ai/v1/embeddings";
const DEFAULT_JINA_API_KEY_ENV: &str = "JINA_API_KEY";

impl JinaClient {
    pub fn new<S: Into<String>>(
        model: S,
        url: Option<String>,
        key: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            model: model.into(),
            url: url.unwrap_or(DEFAULT_JINA_URL.to_owned()),
            key: match key {
                Some(key) => key,
                None => try_env_var(DEFAULT_JINA_API_KEY_ENV)?,
            },
        })
    }

    pub fn infer_single(&self, input: &str) -> Result<Vec<f32>> {
        let mut body = serde_json::Map::new();
        body.insert("input".to_owned(), vec![input.to_owned()].into());
        body.insert("model".to_owned(), self.model.to_owned().into());

        let data: serde_json::Value = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .set("Authorization", format!("Bearer {}", self.key).as_str())
            .send_bytes(
                serde_json::to_vec(&body)
                    .map_err(|error| {
                        Error::new_message(format!("Error serializing body to JSON: {error}"))
                    })?
                    .as_ref(),
            )
            .map_err(|error| Error::new_message(format!("Error sending HTTP request: {error}")))?
            .into_json()
            .map_err(|error| {
                Error::new_message(format!("Error parsing HTTP response as JSON: {error}"))
            })?;
        JinaClient::parse_single_response(data)
    }
    pub fn parse_single_response(value: serde_json::Value) -> Result<Vec<f32>> {
        value
            .get("data")
            .ok_or_else(|| Error::new_message("expected 'data' key in response body"))
            .and_then(|v| {
                v.get(0)
                    .ok_or_else(|| Error::new_message("expected 'data.0' path in response body"))
            })
            .and_then(|v| {
                v.get("embedding").ok_or_else(|| {
                    Error::new_message("expected 'data.0.embedding' path in response body")
                })
            })
            .and_then(|v| {
                v.as_array().ok_or_else(|| {
                    Error::new_message("expected 'data.0.embedding' path to be an array")
                })
            })
            .and_then(|arr| {
                arr.iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| {
                                Error::new_message(
                                    "expected 'data.0.embedding' array to contain floats",
                                )
                            })
                            .map(|f| f as f32)
                    })
                    .collect()
            })
    }
}
#[derive(Clone)]
pub struct MixedbreadClient {
    url: String,
    model: String,
    key: String,
}
const DEFAULT_MIXEDBREAD_URL: &str = "https://api.mixedbread.ai/v1/embeddings/";
const DEFAULT_MIXEDBREAD_API_KEY_ENV: &str = "MIXEDBREAD_API_KEY";

impl MixedbreadClient {
    pub fn new<S: Into<String>>(
        model: S,
        url: Option<String>,
        key: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            model: model.into(),
            url: url.unwrap_or(DEFAULT_MIXEDBREAD_URL.to_owned()),
            key: match key {
                Some(key) => key,
                None => try_env_var(DEFAULT_MIXEDBREAD_API_KEY_ENV)?,
            },
        })
    }

    pub fn infer_single(&self, input: &str) -> Result<Vec<f32>> {
        let mut body = serde_json::Map::new();
        body.insert("input".to_owned(), vec![input.to_owned()].into());
        body.insert("model".to_owned(), self.model.to_owned().into());

        let data: serde_json::Value = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .set("Authorization", format!("Bearer {}", self.key).as_str())
            .send_bytes(
                serde_json::to_vec(&body)
                    .map_err(|error| {
                        Error::new_message(format!("Error serializing body to JSON: {error}"))
                    })?
                    .as_ref(),
            )
            .map_err(|error| Error::new_message(format!("Error sending HTTP request: {error}")))?
            .into_json()
            .map_err(|error| {
                Error::new_message(format!("Error parsing HTTP response as JSON: {error}"))
            })?;
        JinaClient::parse_single_response(data)
    }
    pub fn parse_single_response(value: serde_json::Value) -> Result<Vec<f32>> {
        value
            .get("data")
            .ok_or_else(|| Error::new_message("expected 'data' key in response body"))
            .and_then(|v| {
                v.get(0)
                    .ok_or_else(|| Error::new_message("expected 'data.0' path in response body"))
            })
            .and_then(|v| {
                v.get("embedding").ok_or_else(|| {
                    Error::new_message("expected 'data.0.embedding' path in response body")
                })
            })
            .and_then(|v| {
                v.as_array().ok_or_else(|| {
                    Error::new_message("expected 'data.0.embedding' path to be an array")
                })
            })
            .and_then(|arr| {
                arr.iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| {
                                Error::new_message(
                                    "expected 'data.0.embedding' array to contain floats",
                                )
                            })
                            .map(|f| f as f32)
                    })
                    .collect()
            })
    }
}

#[derive(Clone)]
pub struct OllamaClient {
    url: String,
    model: String,
}
const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434/api/embeddings";
impl OllamaClient {
    pub fn new<S: Into<String>>(model: S, url: Option<String>) -> Self {
        Self {
            model: model.into(),
            url: url.unwrap_or(DEFAULT_OLLAMA_URL.to_owned()),
        }
    }

    pub fn infer_single(&self, input: &str) -> Result<Vec<f32>> {
        let mut body = serde_json::Map::new();
        body.insert("prompt".to_owned(), input.to_owned().into());
        body.insert("model".to_owned(), self.model.to_owned().into());

        let data: serde_json::Value = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .send_bytes(
                serde_json::to_vec(&body)
                    .map_err(|error| {
                        Error::new_message(format!("Error serializing body to JSON: {error}"))
                    })?
                    .as_ref(),
            )
            .map_err(|error| Error::new_message(format!("Error sending HTTP request: {error}")))?
            .into_json()
            .map_err(|error| {
                Error::new_message(format!("Error parsing HTTP response as JSON: {error}"))
            })?;
        OllamaClient::parse_single_response(data)
    }
    pub fn parse_single_response(value: serde_json::Value) -> Result<Vec<f32>> {
        value
            .get("embedding")
            .ok_or_else(|| Error::new_message("expected 'embedding' key in response body"))
            .and_then(|v| {
                v.as_array()
                    .ok_or_else(|| Error::new_message("expected 'embedding' path to be an array"))
            })
            .and_then(|arr| {
                arr.iter()
                    .map(|v| {
                        v.as_f64()
                            .ok_or_else(|| {
                                Error::new_message("expected 'embedding' array to contain floats")
                            })
                            .map(|f| f as f32)
                    })
                    .collect()
            })
    }
}

#[derive(Clone)]
pub struct LlamafileClient {
    url: String,
}
const DEFAULT_LLAMAFILE_URL: &str = "http://localhost:8080/embedding";

impl LlamafileClient {
    pub fn new(url: Option<String>) -> Self {
        Self {
            url: url.unwrap_or(DEFAULT_LLAMAFILE_URL.to_owned()),
        }
    }

    pub fn infer_single(&self, input: &str) -> Result<Vec<f32>> {
        let mut body = serde_json::Map::new();
        body.insert("content".to_owned(), input.to_owned().into());

        let data: serde_json::Value = ureq::post(&self.url)
            .set("Content-Type", "application/json")
            .send_bytes(
                serde_json::to_vec(&body)
                    .map_err(|error| {
                        Error::new_message(format!("Error serializing body to JSON: {error}"))
                    })?
                    .as_ref(),
            )
            .map_err(|error| Error::new_message(format!("Error sending HTTP request: {error}")))?
            .into_json()
            .map_err(|error| {
                Error::new_message(format!("Error parsing HTTP response as JSON: {error}"))
            })?;
        OllamaClient::parse_single_response(data)
    }
}

#[derive(Clone)]
pub struct AmazonBedrockClient {
    model_id: String,
    region: String,
    aws_access_key_id: String,
    aws_secret_access_key: String,
    aws_session_token: String
}
const DEFAULT_AWS_REGION: &str = "us-east-1";

impl AmazonBedrockClient {
    pub fn new<S: Into<String>>(model_id: S, region: Option<String>, aws_access_key_id: Option<String>, aws_secret_access_key: Option<String>, aws_session_token: Option<String>) -> Result<Self> {
        Ok(Self {
            model_id: model_id.into(),
            region: region.unwrap_or(DEFAULT_AWS_REGION.to_owned()),
            aws_access_key_id: aws_access_key_id.unwrap_or(
                std::env::var("AWS_ACCESS_KEY_ID").unwrap()
            ),
            aws_secret_access_key: aws_secret_access_key.unwrap_or(
                std::env::var("AWS_SECRET_ACCESS_KEY").unwrap()
            ),
            aws_session_token: aws_session_token.unwrap_or(
                std::env::var("AWS_SESSION_TOKEN").unwrap_or_default()
            ),
        })
    }

    pub fn sign(&self, key: &[u8], msg: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key).unwrap();
        mac.update(msg);
        let result = mac.finalize();
        result.into_bytes().to_vec()
    }
    
    pub fn get_signing_key(&self, key: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
        let k_date = self.sign(format!("AWS4{key}").as_bytes(), date.as_bytes());
        let k_region = self.sign(&k_date, region.as_bytes());
        let k_service = self.sign(&k_region, service.as_bytes());
        self.sign(&k_service, "aws4_request".as_bytes())
    }
    
    pub fn get_signature(&self, signing_key: &[u8], string_to_sign: &str) -> String {
        let signature = self.sign(signing_key, string_to_sign.as_bytes());
        hex::encode(signature)
    }
    
    pub fn get_canonical_request(&self, http_verb: &str, canonical_uri: &str, canonical_query_string: &str, canonical_headers: &[String], signed_headers: &[&str], payload: &str) -> String {
        let canonical_headers = canonical_headers.join("\n");
        let signed_headers = signed_headers.join(";");
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let hashed_payload = hasher.finalize();
        format!("{http_verb}\n{canonical_uri}\n{canonical_query_string}\n{canonical_headers}\n\n{signed_headers}\n{hashed_payload:x}")
    }
    
    pub fn get_string_to_sign(&self, algorithm: &str, timestamp: &str, credential_scope: &str, canonical_request: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        let canonical_request = hasher.finalize();
        format!("{algorithm}\n{timestamp}\n{credential_scope}\n{canonical_request:x}")
    }
    
    pub fn get_authorization_header(&self, algorithm: &str, credential: &str, scope: &str, signed_headers: &[&str], signature: &str) -> String {
        let signed_headers = signed_headers.join(";");
        format!("{algorithm} Credential={credential}/{scope}, SignedHeaders={signed_headers}, Signature={signature}")
    }

    pub fn infer_single(&self, input: &str, input_type: Option<&str>, truncate: Option<&str>) -> Result<Vec<f32>> {

        // Step 0a. extract model provider

        let model_provider = self.model_id
            .split('.')
            .next()
            .unwrap();
        
        // Step 0b. create payload

        let body = match model_provider {
            "amazon" => ureq::json!({
                "inputText": input.to_owned(),
            }),
            "cohere" => ureq::json!({
                "texts": [input.to_owned()],
                "input_type": input_type.unwrap_or("search_document"),
                "truncate": truncate.unwrap_or("NONE")
            }),
            _ => ureq::json!({})
        };

        // Step 0c. get date and time

        let current_time = chrono::Utc::now();
        let amazon_time = current_time.format("%Y%m%dT%H%M%SZ").to_string();
        let amazon_date = current_time.format("%Y%m%d").to_string();

        // Step 1: create a canonical request

        let canonical_uri = format!("/model/{}/invoke", self.model_id);
        let canonical_query_string = "";
        let service_endpoint: String = format!("bedrock-runtime.{}.amazonaws.com", self.region);
        let endpoint: String = format!("https://{service_endpoint}/model/{}/invoke", self.model_id);

        let mut signed_headers = vec![
            "host",
            "x-amz-date"
        ];

        if !self.aws_session_token.is_empty() {
            signed_headers.push("x-amz-security-token");
        }

        let mut canonical_headers = vec![
            format!("host:{service_endpoint}"),
            format!("x-amz-date:{amazon_time}")
        ];

        if !self.aws_session_token.is_empty() {
            canonical_headers.push(
                format!("x-amz-security-token:{}", self.aws_session_token)
            );
        }

        let canonical_request = self.get_canonical_request(
            "POST",
            &canonical_uri,
            canonical_query_string,
            &canonical_headers,
            &signed_headers,
            &body.to_string()
        );
    
        // Step 2: create string to sign
    
        let algorithm = "AWS4-HMAC-SHA256";
        let service = "bedrock";
        let credential_scope = format!("{amazon_date}/{}/{service}/aws4_request", self.region);
    
        let string_to_sign = self.get_string_to_sign(
            algorithm,
            &amazon_time,
            &credential_scope,
            &canonical_request
        );
    
        // Step 3: calculate signature
    
        let signing_key = self.get_signing_key(
            &self.aws_secret_access_key,
            &amazon_date,
            &self.region,
            service
        );
    
        let signature = self.get_signature(
            &signing_key,
            &string_to_sign
        );
    
        // Step 4: add the signature to the request
    
        let authorization = self.get_authorization_header(
            "AWS4-HMAC-SHA256",
            &self.aws_access_key_id,
            &credential_scope,
            &signed_headers,
            &signature
        );

        // Step 5: send the request

        let request = ureq::post(&endpoint)
            .set("Accept", "application/json")
            .set("X-Amz-Date", &amazon_time)
            .set("Authorization", &authorization);

        let request = if self.aws_session_token.is_empty() {
            request.clone()
        } else {
            request.clone().set("X-Amz-Security-Token", &self.aws_session_token)
        };

        let response = request.clone()
            .send_bytes(
                body.to_string().as_bytes()
            )
            .map_err(
                |error| 
                    Error::new_message(
                        format!("Error sending HTTP request: {error}")
                    )
            )?
            .into_string()
            .map_err(
                |error|
                    Error::new_message(
                        format!("Error parsing HTTP response: {error}")
                    )
            )?;

        let data: serde_json::Value = serde_json::from_str(&response).unwrap();

        AmazonBedrockClient::parse_single_response(self, &data)
    }

    pub fn parse_single_response(&self, value: &serde_json::Value) -> Result<Vec<f32>> {

        let model_provider = self.model_id.split('.').next().unwrap().to_string();

        let output: Result<Vec<f32>>;
        if model_provider == "amazon" {
            output = value
                .get("embedding")
                .ok_or_else(|| Error::new_message("expected 'embedding' key in response body"))
                .and_then(|v| {
                    v.as_array().ok_or_else(|| {
                        Error::new_message("expected 'embedding' path to be an array")
                    })
                })
                .and_then(|arr| {
                    arr.iter()
                        .map(|v| {
                            v.as_f64()
                                .ok_or_else(|| {
                                    Error::new_message(
                                        "expected 'embedding' array to contain floats",
                                    )
                                })
                                .map(|f| f as f32)
                        })
                        .collect()
                });
        } else if model_provider == "cohere" {
            output = value
                .get("embeddings")
                .ok_or_else(|| Error::new_message("expected 'embeddings' key in response body"))
                .and_then(|v: &serde_json::Value| {
                    v.as_array().ok_or_else(|| {
                        Error::new_message("expected 'embeddings' path to be an array")
                    })
                })
                .and_then(|v| {
                    v.first()
                        .ok_or_else(|| Error::new_message("expected 'embeddings.0' path in response body"))
                })
                .and_then(|v| {
                    v.as_array().ok_or_else(|| {
                        Error::new_message("expected 'embeddings.0' path to be an array")
                    })
                })
                .and_then(|arr| {
                    arr.iter()
                        .map(|v| {
                            v.as_f64()
                                .ok_or_else(|| {
                                    Error::new_message(
                                        "expected 'embeddings.0' array to contain floats",
                                    )
                                })
                                .map(|f| f as f32)
                        })
                        .collect()
                });
        } else {
            todo!();
        }
        output
    }
}

#[derive(Clone)]
pub enum Client {
    OpenAI(OpenAiClient),
    Nomic(NomicClient),
    Cohere(CohereClient),
    Ollama(OllamaClient),
    Llamafile(LlamafileClient),
    Jina(JinaClient),
    Mixedbread(MixedbreadClient),
    AmazonBedrock(AmazonBedrockClient),
}
