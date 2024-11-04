use crate::{Client, FLOAT32_VECTOR_SUBTYPE};
use sqlite_loadable::{
    api,
    table::{ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    BestIndexError, Result,
};
use sqlite_loadable::{prelude::*, Error};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, mem, os::raw::c_int, rc::Rc};
use zerocopy::AsBytes;

static CREATE_SQL: &str =
    "CREATE TABLE x(contents, embedding, input1 hidden, input2 hidden, source hidden)";
enum Columns {
    Contents,
    Embedding,
    Input1,
    Input2,
    Source,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Contents),
        1 => Some(Columns::Embedding),
        2 => Some(Columns::Input1),
        3 => Some(Columns::Input2),
        4 => Some(Columns::Source),
        _ => None,
    }
}

#[repr(C)]
pub struct BatchTable {
    /// must be first
    base: sqlite3_vtab,
    clients: Rc<RefCell<HashMap<String, Client>>>,
}

impl<'vtab> VTab<'vtab> for BatchTable {
    type Aux = Rc<RefCell<HashMap<String, Client>>>;
    type Cursor = BatchCursor<'vtab>;

    fn connect(
        _db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, BatchTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = BatchTable {
            base,
            clients: aux.unwrap().clone(),
        };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_input1 = false;
        let mut has_input2 = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Input1) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_input1 = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                Some(Columns::Input2) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(2);
                        has_input2 = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => (),
            }
        }
        if !has_input1 {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(2);

        Ok(())
    }

    fn open(&mut self) -> Result<BatchCursor<'_>> {
        Ok(BatchCursor::new(self.clients.clone()))
    }
}

type Entry = (serde_json::Value, Vec<f32>);
#[repr(C)]
pub struct BatchCursor<'vtab> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    clients: Rc<RefCell<HashMap<String, Client>>>,
    results: Option<Vec<Entry>>,
    curr: usize,
    phantom: PhantomData<&'vtab BatchTable>,
}
impl BatchCursor<'_> {
    fn new<'vtab>(clients: Rc<RefCell<HashMap<String, Client>>>) -> BatchCursor<'vtab> {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        BatchCursor {
            base,
            clients: clients,
            results: None,
            curr: 0,
            phantom: PhantomData,
        }
    }
}

impl VTabCursor for BatchCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        self.curr = 0;
        let first = values.get(0).unwrap();
        let (client_name, input) = match values.get(1) {
            Some(v) => (api::value_text(first).unwrap(), api::value_text(v).unwrap()),
            None => ("default", api::value_text(first).unwrap()),
        };

        let x = self.clients.borrow();
        let client = x.get(client_name).ok_or_else(|| {
            Error::new_message(format!(
                "Client with name {client_name} was not registered with rembed_clients."
            ))
        })?;

        let input: serde_json::Value = serde_json::from_str(input).unwrap();
        let input = input.as_array().unwrap();
        let x: Vec<String> = input
            .iter()
            .map(|v| {
                let contents = v.get("contents").unwrap().as_str().unwrap();
                contents.to_string()
            })
            .collect();
        let embeddings = match client {
            Client::Ollama(c) => c.infer_multiple(x).unwrap(),
            _ => todo!(),
        };
        self.results = Some(
            embeddings
                .iter()
                .zip(input)
                .map(|(emb, val)| (val.to_owned(), emb.to_owned()))
                .collect(),
        );

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.curr += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.results
            .as_ref()
            .map_or(true, |v| v.get(self.curr).is_none())
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::Contents) => {
                api::result_text(
                    context,
                    self.results
                        .as_ref()
                        .unwrap()
                        .get(self.curr)
                        .unwrap()
                        .0
                        .get("contents")
                        .unwrap()
                        .as_str()
                        .unwrap(),
                )?;
            }
            Some(Columns::Embedding) => {
                api::result_blob(
                    context,
                    self.results
                        .as_ref()
                        .unwrap()
                        .get(self.curr)
                        .unwrap()
                        .1
                        .as_bytes(),
                );
                api::result_subtype(context, FLOAT32_VECTOR_SUBTYPE);
            }
            Some(Columns::Input1) => todo!(),
            Some(Columns::Input2) => todo!(),
            Some(Columns::Source) => todo!(),
            None => todo!(),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.curr as i64)
    }
}
