mod clients;
mod infer;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use clients::ClientsTable;
use infer::{infer_cohere, infer_nomic, infer_ollama, infer_openai};
use sqlite_loadable::{
    api, define_scalar_function, define_scalar_function_with_aux, define_virtual_table_writeablex,
    prelude::*, Result,
};
use zerocopy::AsBytes;

#[derive(Clone)]
struct OpenAiClient {
    url: String,
    model: String,
    key: String,
}
#[derive(Clone)]
struct NomicClient {
    url: String,
    model: String,
    key: String,
}
#[derive(Clone)]
struct CohereClient {
    url: String,
    model: String,
    key: String,
}
#[derive(Clone)]
struct OllamaClient {
    url: String,
    model: String,
}
#[derive(Clone)]
pub enum Client {
    OpenAI(OpenAiClient),
    Nomic(NomicClient),
    Cohere(CohereClient),
    Ollama(OllamaClient),
}

const CLIENT_OPTIONS_POINTER_NAME: &[u8] = b"sqlite-rembed-client-options\0";

pub fn rembed_client_options(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    assert!((values.len() % 2) == 0);
    let mut options: HashMap<String, String> = HashMap::new();
    for pair in values.chunks(2) {
        let key = api::value_text(&pair[0])?;
        let value = api::value_text(&pair[1])?;
        if key == "flavor" {
        } else {
            options.insert(key.to_owned(), value.to_owned());
        }
    }
    api::result_pointer(
        context,
        CLIENT_OPTIONS_POINTER_NAME,
        Client::OpenAI(OpenAiClient {
            model: "".to_string(),
            key: "".to_string(),
            url: options.get("url").unwrap().to_owned(),
        }),
    );
    Ok(())
}

pub fn rembed(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    clients: &Rc<RefCell<HashMap<String, Client>>>,
) -> Result<()> {
    let client_name = api::value_text(&values[0])?;
    let input = api::value_text(&values[1])?;
    let x = clients.borrow();
    let client = x.get(client_name).unwrap();

    let embedding = match client {
        Client::OpenAI(client) => infer_openai(client, input)?,
        Client::Ollama(client) => infer_ollama(client, input)?,
        Client::Nomic(client) => {
            let input_type = values.get(2).and_then(|v| api::value_text(v).ok());
            infer_nomic(client, input, input_type)?
        }
        Client::Cohere(client) => {
            let input_type = values.get(2).and_then(|v| api::value_text(v).ok());
            infer_cohere(client, input, input_type)?
        }
    };
    api::result_blob(context, embedding.as_bytes());
    api::result_subtype(context, 223);

    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_rembed_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;

    let c = Rc::new(RefCell::new(HashMap::new()));

    define_scalar_function_with_aux(db, "rembed", 2, rembed, flags, Rc::clone(&c))?;
    define_scalar_function_with_aux(db, "rembed", 3, rembed, flags, Rc::clone(&c))?;
    define_scalar_function(
        db,
        "rembed_client_options",
        -1,
        rembed_client_options,
        flags,
    )?;
    define_virtual_table_writeablex::<ClientsTable>(db, "rembed_clients", Some(Rc::clone(&c)))?;
    Ok(())
}
