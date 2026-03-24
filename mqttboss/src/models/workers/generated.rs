/* @generated and managed by dsync */
use messages::messages::WorkerAnnouncement;
#[allow(unused)]
use diesel::*;
use diesel::associations::HasTable;
use crate::schema::*;

pub type ConnectionType = diesel::pg::PgConnection;

/// Struct representing a row in table `workers`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Queryable, diesel::Selectable, diesel::QueryableByName, diesel::Identifiable)]
#[diesel(table_name=workers, primary_key(id))]
pub struct Workers {
    /// Field representing column `id`
    pub id: i32,
    /// Field representing column `name`
    pub name: String,
    /// Field representing column `last_seen`
    pub last_seen: Option<chrono::NaiveDateTime>,
    /// Field representing column `cpus`
    pub cpus: Option<i32>,
    /// Field representing column `ram`
    pub ram: Option<i32>,
    /// Field representing column `disk`
    pub disk: Option<f64>,
    /// Field representing column `gpu`
    pub gpu: Option<i32>,
    /// Field representing column `tags`
    pub tags: Option<serde_json::Value>,
    pub available: bool,
}

/// Create Struct for a row in table `workers` for [`Workers`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Insertable)]
#[diesel(table_name=workers)]
pub struct CreateWorkers {
    /// Field representing column `id`
    pub id: Option<i32>,
    /// Field representing column `name`
    pub name: String,
    /// Field representing column `last_seen`
    pub last_seen: Option<chrono::NaiveDateTime>,
    /// Field representing column `cpus`
    pub cpus: Option<i32>,
    /// Field representing column `ram`
    pub ram: Option<i32>,
    /// Field representing column `disk`
    pub disk: Option<f64>,
    /// Field representing column `gpu`
    pub gpu: Option<i32>,
    /// Field representing column `tags`
    pub tags: Option<serde_json::Value>,
    pub available: bool,
}

/// Update Struct for a row in table `workers` for [`Workers`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::AsChangeset, PartialEq, Default)]
#[diesel(table_name=workers)]
pub struct UpdateWorkers {
    /// Field representing column `name`
    pub name: Option<String>,
    /// Field representing column `last_seen`
    pub last_seen: Option<Option<chrono::NaiveDateTime>>,
    /// Field representing column `cpus`
    pub cpus: Option<Option<i32>>,
    /// Field representing column `ram`
    pub ram: Option<Option<i32>>,
    /// Field representing column `disk`
    pub disk: Option<Option<f64>>,
    /// Field representing column `gpu`
    pub gpu: Option<Option<i32>>,
    /// Field representing column `tags`
    pub tags: Option<Option<serde_json::Value>>,
    pub available: bool,
}

/// Result of a `.paginate` function
#[derive(Debug, serde::Serialize)]
pub struct PaginationResult<T> {
    /// Resulting items that are from the current page
    pub items: Vec<T>,
    /// The count of total items there are
    pub total_items: i64,
    /// Current page, 0-based index
    pub page: i64,
    /// Size of a page
    pub page_size: i64,
    /// Number of total possible pages, given the `page_size` and `total_items`
    pub num_pages: i64,
}

impl Workers {
    /// Insert a new row into `workers` with a given [`CreateWorkers`]
    pub fn create(db: &mut ConnectionType, item: &CreateWorkers) -> diesel::QueryResult<Self> {
        use crate::schema::workers::dsl::*;

        diesel::insert_into(workers).values(item).get_result::<Self>(db)
    }

    /// Get a row from `workers`, identified by the primary key
    pub fn read(db: &mut ConnectionType, param_id: Option<i32>) -> diesel::QueryResult<Vec<Self>> {
        use crate::schema::workers::dsl::*;
        match param_id {
            None => {workers.load::<Workers>(db)}
            Some(worker_id) => {workers.filter(id.eq(worker_id)).get_results::<Self>(db)}
        }

    }

    pub fn search_by_message(db: &mut ConnectionType, message: &WorkerAnnouncement) -> diesel::QueryResult<Vec<Self>> {
        use crate::schema::workers::dsl::*;
        workers.filter(name.eq(message.message_config.worker_id.clone())).load::<Self>(db)
    }

    /// Update a row in `workers`, identified by the primary key with [`UpdateWorkers`]
    pub fn update(db: &mut ConnectionType, param_id: i32, item: &UpdateWorkers) -> diesel::QueryResult<Self> {
        use crate::schema::workers::dsl::*;

        diesel::update(workers.filter(id.eq(param_id))).set(item).get_result(db)
    }

    /// Delete a row in `workers`, identified by the primary key
    pub fn delete(db: &mut ConnectionType, param_id: i32) -> diesel::QueryResult<usize> {
        use crate::schema::workers::dsl::*;

        diesel::delete(workers.filter(id.eq(param_id))).execute(db)
    }
}
