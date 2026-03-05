/* @generated and managed by dsync */

#[allow(unused)]
use crate::diesel::*;
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
    /// Field representing column `capabilities`
    pub capabilities: serde_json::Value,
    /// Field representing column `last_seen`
    pub last_seen: chrono::NaiveDateTime,
}

/// Create Struct for a row in table `workers` for [`Workers`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::Insertable)]
#[diesel(table_name=workers)]
pub struct CreateWorkers {
    /// Field representing column `id`
    pub id: i32,
    /// Field representing column `name`
    pub name: String,
    /// Field representing column `capabilities`
    pub capabilities: serde_json::Value,
    /// Field representing column `last_seen`
    pub last_seen: chrono::NaiveDateTime,
}

/// Update Struct for a row in table `workers` for [`Workers`]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, diesel::AsChangeset, PartialEq, Default)]
#[diesel(table_name=workers)]
pub struct UpdateWorkers {
    /// Field representing column `name`
    pub name: Option<String>,
    /// Field representing column `capabilities`
    pub capabilities: Option<serde_json::Value>,
    /// Field representing column `last_seen`
    pub last_seen: Option<chrono::NaiveDateTime>,
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
    pub fn read(db: &mut ConnectionType, param_id: i32) -> diesel::QueryResult<Self> {
        use crate::schema::workers::dsl::*;

        workers.filter(id.eq(param_id)).first::<Self>(db)
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
