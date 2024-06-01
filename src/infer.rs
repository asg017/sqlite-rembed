use crate::StandardClient;
use sqlite_loadable::Result;

fn parse_single_embedding_openai(value: serde_json::Value) -> std::result::Result<Vec<f32>, ()> {
    Ok(value
        .get("data")
        .ok_or(())?
        .get(0)
        .ok_or(())?
        .get("embedding")
        .ok_or(())?
        .as_array()
        .ok_or(())?
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect())
}
fn parse_single_embedding_nomic(value: serde_json::Value) -> std::result::Result<Vec<f32>, ()> {
    Ok(value
        .get("embeddings")
        .ok_or(())?
        .get(0)
        .ok_or(())?
        .as_array()
        .ok_or(())?
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect())
}
fn parse_single_embedding_cohere(value: serde_json::Value) -> std::result::Result<Vec<f32>, ()> {
    Ok(value
        .get("embeddings")
        .ok_or(())?
        .get(0)
        .ok_or(())?
        .as_array()
        .ok_or(())?
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect())
}

pub fn infer_openai(client: &StandardClient, input: &str) -> Result<Vec<f32>> {
    let body = serde_json::json!({
        "input": input,
        "model": client.model
    });

    let data: serde_json::Value = ureq::post(&client.url)
        .set("Content-Type", "application/json")
        .set("Authorization", format!("Bearer {}", client.key).as_str())
        .send_bytes(serde_json::to_vec(&body).unwrap().as_ref())
        .unwrap()
        .into_json()
        .unwrap();
    Ok(parse_single_embedding_openai(data).unwrap())
}
pub fn infer_nomic(
    client: &StandardClient,
    input: &str,
    input_type: Option<&str>,
) -> Result<Vec<f32>> {
    let mut body = serde_json::Map::new();
    body.insert("texts".to_owned(), vec![input.to_owned()].into());
    body.insert("model".to_owned(), client.model.to_owned().into());

    if let Some(input_type) = input_type {
        body.insert("input_type".to_owned(), input_type.to_owned().into());
    }

    let data: serde_json::Value = ureq::post(&client.url)
        .set("Content-Type", "application/json")
        .set("Authorization", format!("Bearer {}", client.key).as_str())
        .send_bytes(serde_json::to_vec(&body).unwrap().as_ref())
        .unwrap()
        .into_json()
        .unwrap();
    Ok(parse_single_embedding_nomic(data).unwrap())
}

pub fn infer_cohere(
    client: &StandardClient,
    input: &str,
    input_type: Option<&str>,
) -> Result<Vec<f32>> {
    let mut body = serde_json::Map::new();
    body.insert("texts".to_owned(), vec![input.to_owned()].into());
    body.insert("model".to_owned(), client.model.to_owned().into());

    if let Some(input_type) = input_type {
        body.insert("input_type".to_owned(), input_type.to_owned().into());
    }

    let data: serde_json::Value = ureq::post(&client.url)
        .set("Content-Type", "application/json")
        .set("Authorization", format!("Bearer {}", client.key).as_str())
        .send_bytes(serde_json::to_vec(&body).unwrap().as_ref())
        .unwrap()
        .into_json()
        .unwrap();
    Ok(parse_single_embedding_cohere(data).unwrap())
}
