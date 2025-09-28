// New lib.rs using genai - complete implementation
mod genai_client;
mod multimodal;
mod mock_provider;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use genai_client::{EmbeddingClient, parse_client_options, legacy_provider_to_model};
use multimodal::MultimodalClient;
use sqlite_loadable::{
    api, define_scalar_function, define_scalar_function_with_aux, define_virtual_table_writeablex,
    prelude::*, Error, Result,
};
use sqlite_loadable::table::{UpdateOperation, IndexInfo, VTab, VTabArguments, VTabCursor, VTabWriteable};
use sqlite_loadable::api::ValueType;
use sqlite_loadable::BestIndexError;
use std::{marker::PhantomData, mem, os::raw::c_int};
use zerocopy::AsBytes;
use base64;
use serde_json;

const FLOAT32_VECTOR_SUBTYPE: u8 = 223;
const CLIENT_OPTIONS_POINTER_NAME: &[u8] = b"sqlite-rembed-client-options\0";
const MULTIMODAL_CLIENT_OPTIONS_POINTER_NAME: &[u8] = b"sqlite-rembed-multimodal-client-options\0";

pub fn rembed_version(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, format!("v{}-genai", env!("CARGO_PKG_VERSION")))?;
    Ok(())
}

// Helper function to base64 encode a blob (useful for image processing)
pub fn readfile_base64(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let blob = api::value_blob(&values[0]);
    use base64::Engine as _;
    let encoded = base64::engine::general_purpose::STANDARD.encode(blob);
    api::result_text(context, encoded)?;
    Ok(())
}

pub fn rembed_debug(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(
        context,
        format!(
            "Version: v{}
Source: {}
Backend: genai v0.4.0-alpha.4
",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        ),
    )?;
    Ok(())
}


pub fn rembed_client_options(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    if (values.len() % 2) != 0 {
        return Err(Error::new_message(
            "Must have an even number of arguments to rembed_client_options, as key/value pairs.",
        ));
    }

    let mut options: HashMap<String, String> = HashMap::new();
    let mut format: Option<String> = None;
    for pair in values.chunks(2) {
        let key = api::value_text(&pair[0])?;
        let value = api::value_text(&pair[1])?;
        if key == "format" {
            format = Some(value.to_owned());
        } else {
            options.insert(key.to_owned(), value.to_owned());
        }
    }

    // Check if this is a multimodal client (has embedding_model option)
    if let Some(embedding_model) = options.get("embedding_model") {
        // Create MultimodalClient
        let vision_model = if let Some(format) = format {
            // Legacy compatibility: convert old format to genai model
            let model_name = options.get("model")
                .ok_or_else(|| Error::new_message("'model' option is required for vision model"))?;
            legacy_provider_to_model(&format, model_name)
        } else if let Some(model) = options.get("model") {
            model.clone()
        } else {
            return Err(Error::new_message("'model' or 'format' key is required for vision model"));
        };

        let multimodal_client = MultimodalClient::new(vision_model, embedding_model.clone())?;
        api::result_pointer(context, MULTIMODAL_CLIENT_OPTIONS_POINTER_NAME, multimodal_client);
    } else {
        // Create regular EmbeddingClient
        let model = if let Some(format) = format {
            // Legacy compatibility: convert old format to genai model
            let model_name = options.get("model")
                .ok_or_else(|| Error::new_message("'model' option is required"))?;
            legacy_provider_to_model(&format, model_name)
        } else if let Some(model) = options.get("model") {
            model.clone()
        } else {
            return Err(Error::new_message("'model' or 'format' key is required"));
        };

        let api_key = options.get("key").cloned()
            .or_else(|| options.get("api_key").cloned());

        let client = EmbeddingClient::new(model, api_key)?;
        api::result_pointer(context, CLIENT_OPTIONS_POINTER_NAME, client);
    }

    Ok(())
}

pub fn rembed(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    clients: &Rc<RefCell<HashMap<String, EmbeddingClient>>>,
) -> Result<()> {
    let client_name = api::value_text(&values[0])?;
    let input = api::value_text(&values[1])?;

    let clients_map = clients.borrow();
    let client = clients_map.get(client_name).ok_or_else(|| {
        Error::new_message(format!(
            "Client with name {} was not registered with rembed_clients.",
            client_name
        ))
    })?;

    // Generate embedding synchronously (blocks on async internally)
    let embedding = client.embed_sync(input)?;

    api::result_blob(context, embedding.as_bytes());
    api::result_subtype(context, FLOAT32_VECTOR_SUBTYPE);
    Ok(())
}

// Batch embedding function - accepts JSON array of texts
pub fn rembed_batch(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    clients: &Rc<RefCell<HashMap<String, EmbeddingClient>>>,
) -> Result<()> {
    let client_name = api::value_text(&values[0])?;
    let json_input = api::value_text(&values[1])?;

    // Parse JSON array of texts
    let texts: Vec<String> = serde_json::from_str(json_input)
        .map_err(|e| Error::new_message(format!("Invalid JSON array: {}", e)))?;

    if texts.is_empty() {
        return Err(Error::new_message("Input array cannot be empty"));
    }

    let clients_map = clients.borrow();
    let client = clients_map.get(client_name).ok_or_else(|| {
        Error::new_message(format!(
            "Client with name {} was not registered with rembed_clients.",
            client_name
        ))
    })?;

    // Generate embeddings in batch
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    let embeddings = client.embed_batch_sync(text_refs)?;

    // Return as JSON array of base64-encoded embeddings
    let result: Vec<String> = embeddings.into_iter()
        .map(|embedding| {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(embedding.as_bytes())
        })
        .collect();

    api::result_text(context, serde_json::to_string(&result)
        .map_err(|e| Error::new_message(format!("JSON serialization failed: {}", e)))?)?;
    Ok(())
}

// Virtual table implementation
enum Columns {
    Name,
    Options,
}

fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Name),
        1 => Some(Columns::Options),
        _ => None,
    }
}

// Auxiliary data structure for the virtual table
pub struct ClientsTableAux {
    pub clients: Rc<RefCell<HashMap<String, EmbeddingClient>>>,
    pub multimodal_clients: Rc<RefCell<HashMap<String, MultimodalClient>>>,
}

#[repr(C)]
pub struct ClientsTable {
    base: sqlite3_vtab,
    clients: Rc<RefCell<HashMap<String, EmbeddingClient>>>,
    multimodal_clients: Rc<RefCell<HashMap<String, MultimodalClient>>>,
}

impl<'vtab> VTab<'vtab> for ClientsTable {
    type Aux = ClientsTableAux;
    type Cursor = ClientsCursor<'vtab>;

    fn create(
        db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)> {
        Self::connect(db, aux, args)
    }

    fn connect(
        _db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, ClientsTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let aux = aux.expect("Required aux");
        let clients = aux.clients.clone();
        let multimodal_clients = aux.multimodal_clients.clone();

        let vtab = ClientsTable {
            base,
            clients,
            multimodal_clients,
        };
        let sql = "create table x(name text primary key, options)".to_owned();

        Ok((sql, vtab))
    }

    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        info.set_estimated_cost(10000.0);
        info.set_estimated_rows(10000);
        info.set_idxnum(1);
        Ok(())
    }

    fn open(&'vtab mut self) -> Result<ClientsCursor<'vtab>> {
        ClientsCursor::new(self)
    }
}

impl<'vtab> VTabWriteable<'vtab> for ClientsTable {
    fn update(&'vtab mut self, operation: UpdateOperation<'_>, _p_rowid: *mut i64) -> Result<()> {
        match operation {
            UpdateOperation::Delete(_) => {
                return Err(Error::new_message(
                    "DELETE operations on rembed_clients is not supported yet",
                ))
            }
            UpdateOperation::Update { _values } => {
                return Err(Error::new_message(
                    "UPDATE operations on rembed_clients is not supported yet",
                ))
            }
            UpdateOperation::Insert { values, rowid: _ } => {
                let name = api::value_text(&values[0])?;

                match api::value_type(&values[1]) {
                    ValueType::Text => {
                        let options = api::value_text(&values[1])?;
                        // Parse the options to get model and api key
                        let config = parse_client_options(name, options)?;
                        // Create client with the model and api key
                        let client = EmbeddingClient::new(config.model, config.api_key)?;
                        self.clients.borrow_mut().insert(name.to_owned(), client);
                    }
                    ValueType::Null => unsafe {
                        // Try multimodal client first
                        if let Some(multimodal_client) =
                            api::value_pointer::<MultimodalClient>(&values[1], MULTIMODAL_CLIENT_OPTIONS_POINTER_NAME)
                        {
                            self.multimodal_clients.borrow_mut().insert(name.to_owned(), (*multimodal_client).clone());
                        }
                        // Fallback to regular embedding client
                        else if let Some(client) =
                            api::value_pointer::<EmbeddingClient>(&values[1], CLIENT_OPTIONS_POINTER_NAME)
                        {
                            self.clients.borrow_mut().insert(name.to_owned(), (*client).clone());
                        } else {
                            return Err(Error::new_message("client options required"));
                        }
                    },
                    _ => return Err(Error::new_message("client options required")),
                };
            }
        }
        Ok(())
    }
}

#[repr(C)]
pub struct ClientsCursor<'vtab> {
    base: sqlite3_vtab_cursor,
    keys: Vec<String>,
    rowid: i64,
    clients: Rc<RefCell<HashMap<String, EmbeddingClient>>>,
    multimodal_clients: Rc<RefCell<HashMap<String, MultimodalClient>>>,
    phantom: PhantomData<&'vtab ClientsTable>,
}

impl ClientsCursor<'_> {
    fn new(table: &mut ClientsTable) -> Result<ClientsCursor<'_>> {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };

        // Collect keys from both regular and multimodal clients
        let mut keys = Vec::new();

        // Add regular embedding client keys
        let c = table.clients.borrow();
        keys.extend(c.keys().map(|k| k.to_string()));
        drop(c);

        // Add multimodal client keys
        let mc = table.multimodal_clients.borrow();
        keys.extend(mc.keys().map(|k| k.to_string()));
        drop(mc);

        let cursor = ClientsCursor {
            base,
            keys,
            rowid: 0,
            clients: table.clients.clone(),
            multimodal_clients: table.multimodal_clients.clone(),
            phantom: PhantomData,
        };
        Ok(cursor)
    }
}

impl VTabCursor for ClientsCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        (self.rowid as usize) >= self.keys.len()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let key = self
            .keys
            .get(self.rowid as usize)
            .expect("Internal rembed_clients logic error");
        match column(i) {
            Some(Columns::Name) => api::result_text(context, key)?,
            Some(Columns::Options) => {
                // Check what type of client this is for debugging
                let clients = self.clients.borrow();
                if clients.contains_key(key) {
                    api::result_text(context, "(embedding client)")?;
                } else {
                    drop(clients);
                    let multimodal = self.multimodal_clients.borrow();
                    if multimodal.contains_key(key) {
                        api::result_text(context, "(multimodal client)")?;
                    }
                    // If neither, return NULL
                }
            },
            None => (),
        };
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}

// For now, we'll focus on the scalar batch function approach
// Table function implementation can be added later when sqlite-loadable has better support

// Image embedding using hybrid approach (vision model → text → embedding)
pub fn rembed_image(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    multimodal_clients: &Rc<RefCell<HashMap<String, MultimodalClient>>>,
) -> Result<()> {
    let client_name = api::value_text(&values[0])?;
    let image_blob = api::value_blob(&values[1]);

    let clients_map = multimodal_clients.borrow();
    let client = clients_map.get(client_name).ok_or_else(|| {
        Error::new_message(format!(
            "Multimodal client with name {} was not registered.",
            client_name
        ))
    })?;

    // Generate embedding using hybrid approach
    let embedding = client.embed_image_sync(image_blob)?;

    api::result_blob(context, embedding.as_bytes());
    api::result_subtype(context, FLOAT32_VECTOR_SUBTYPE);
    Ok(())
}

// Image embedding with custom prompt
pub fn rembed_image_prompt(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    multimodal_clients: &Rc<RefCell<HashMap<String, MultimodalClient>>>,
) -> Result<()> {
    let client_name = api::value_text(&values[0])?;
    let image_blob = api::value_blob(&values[1]);
    let prompt = api::value_text(&values[2])?;

    let clients_map = multimodal_clients.borrow();
    let client = clients_map.get(client_name).ok_or_else(|| {
        Error::new_message(format!(
            "Multimodal client with name {} was not registered.",
            client_name
        ))
    })?;

    // Generate embedding with custom prompt
    let embedding = client.embed_image_with_prompt_sync(image_blob, prompt)?;

    api::result_blob(context, embedding.as_bytes());
    api::result_subtype(context, FLOAT32_VECTOR_SUBTYPE);
    Ok(())
}

// Concurrent batch image processing for high performance
pub fn rembed_images_concurrent(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    multimodal_clients: &Rc<RefCell<HashMap<String, MultimodalClient>>>,
) -> Result<()> {
    let client_name = api::value_text(&values[0])?;
    let json_input = api::value_text(&values[1])?;

    // Parse JSON array of base64-encoded images
    let images_base64: Vec<String> = serde_json::from_str(json_input)
        .map_err(|e| Error::new_message(format!("Invalid JSON array: {}", e)))?;

    if images_base64.is_empty() {
        return Err(Error::new_message("Input array cannot be empty"));
    }

    let clients_map = multimodal_clients.borrow();
    let client = clients_map.get(client_name).ok_or_else(|| {
        Error::new_message(format!(
            "Multimodal client with name {} was not registered.",
            client_name
        ))
    })?;

    // Decode base64 images
    let mut images: Vec<Vec<u8>> = Vec::new();
    for img_base64 in &images_base64 {
        use base64::Engine as _;
        let img_data = base64::engine::general_purpose::STANDARD.decode(img_base64)
            .map_err(|e| Error::new_message(format!("Base64 decode failed: {}", e)))?;
        images.push(img_data);
    }

    // Process concurrently
    let image_refs: Vec<&[u8]> = images.iter().map(|v| v.as_slice()).collect();
    let (embeddings, stats) = client.embed_images_concurrent_sync(image_refs)?;

    // Return JSON with embeddings and statistics
    let result: serde_json::Value = serde_json::json!({
        "embeddings": embeddings.iter().map(|embedding| {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD.encode(embedding.as_bytes())
        }).collect::<Vec<_>>(),
        "stats": {
            "total_processed": stats.total_processed,
            "successful": stats.successful,
            "failed": stats.failed,
            "total_duration_ms": stats.total_duration.as_millis(),
            "avg_time_per_item_ms": stats.avg_time_per_item.as_millis(),
            "throughput": if stats.total_duration.as_secs_f64() > 0.0 {
                stats.successful as f64 / stats.total_duration.as_secs_f64()
            } else {
                0.0
            }
        }
    });

    api::result_text(context, serde_json::to_string(&result)
        .map_err(|e| Error::new_message(format!("JSON serialization failed: {}", e)))?)?;
    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_rembed_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8
        | FunctionFlags::DETERMINISTIC
        | unsafe { FunctionFlags::from_bits_unchecked(0x001000000) };

    let clients: Rc<RefCell<HashMap<String, EmbeddingClient>>> =
        Rc::new(RefCell::new(HashMap::new()));

    let multimodal_clients: Rc<RefCell<HashMap<String, MultimodalClient>>> =
        Rc::new(RefCell::new(HashMap::new()));

    define_scalar_function(
        db,
        "rembed_version",
        0,
        rembed_version,
        FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC,
    )?;

    define_scalar_function(
        db,
        "rembed_debug",
        0,
        rembed_debug,
        FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC,
    )?;

    // Helper function for base64 encoding (useful with image functions)
    define_scalar_function(
        db,
        "readfile_base64",
        1,
        readfile_base64,
        FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC,
    )?;

    define_scalar_function_with_aux(db, "rembed", 2, rembed, flags, Rc::clone(&clients))?;
    define_scalar_function_with_aux(db, "rembed", 3, rembed, flags, Rc::clone(&clients))?;

    define_scalar_function(
        db,
        "rembed_client_options",
        -1,
        rembed_client_options,
        flags,
    )?;

    // Create auxiliary data for the virtual table
    let clients_table_aux = ClientsTableAux {
        clients: Rc::clone(&clients),
        multimodal_clients: Rc::clone(&multimodal_clients),
    };

    define_virtual_table_writeablex::<ClientsTable>(db, "rembed_clients", Some(clients_table_aux))?;

    // Batch embedding function
    define_scalar_function_with_aux(
        db,
        "rembed_batch",
        2,
        rembed_batch,
        flags,
        Rc::clone(&clients),
    )?;

    // Table function will be added in a future version when sqlite-loadable has better support

    // Image embedding functions (hybrid multimodal)
    define_scalar_function_with_aux(
        db,
        "rembed_image",
        2,
        rembed_image,
        flags,
        Rc::clone(&multimodal_clients),
    )?;

    define_scalar_function_with_aux(
        db,
        "rembed_image_prompt",
        3,
        rembed_image_prompt,
        flags,
        Rc::clone(&multimodal_clients),
    )?;

    // High-performance concurrent image batch processing
    define_scalar_function_with_aux(
        db,
        "rembed_images_concurrent",
        2,
        rembed_images_concurrent,
        flags,
        Rc::clone(&multimodal_clients),
    )?;

    // Register multimodal Ollama client by default
    multimodal_clients.borrow_mut().insert(
        "ollama-multimodal".to_string(),
        MultimodalClient::new(
            "ollama::llava:7b".to_string(),
            "ollama::nomic-embed-text".to_string(),
        )?,
    );

    Ok(())
}