#[macro_use]
extern crate diesel;
use diesel::{
    expression::{nullable::Nullable, operators::Eq, BoxableExpression},
    pg::Pg,
    query_dsl::{QueryDsl, RunQueryDsl},
    query_source::{
        joins::{Join, JoinOn, LeftOuter},
        AppearsInFromClause, Once,
    },
    sql_types::Bool,
    ExpressionMethods,
};

// Define the tunnel table and struct
table! {
    #[allow(unused_imports)]
    use diesel::sql_types::*;
    tunnel (id) {
        id -> BigInt,
        name -> Text,
    }
}
#[derive(Queryable, Identifiable, Clone, Debug, PartialEq, Eq)]
#[table_name = "tunnel"]
pub struct Tunnel {
    pub id: i64,
    pub name: String,
}

// Define the connection table and struct
table! {
    #[allow(unused_imports)]
    use diesel::sql_types::*;
    connection(id) {
        id -> BigInt,
        tunnel_id -> BigInt,
    }
}

#[derive(Debug, Associations, Identifiable, Queryable)]
#[table_name = "connection"]
#[primary_key(id)]
#[belongs_to(Tunnel)]
pub struct Connection {
    pub id: i64,
    pub tunnel_id: i64,
}

joinable!(connection -> tunnel(tunnel_id));
allow_tables_to_appear_in_same_query!(connection, tunnel);

// This works just fine when the query source is the tunnel table:

fn filters_t(
    name: &'static str,
) -> Vec<Box<dyn BoxableExpression<tunnel::table, Pg, SqlType = Bool>>> {
    let mut wheres: Vec<Box<dyn BoxableExpression<tunnel::table, Pg, SqlType = Bool>>> = Vec::new();
    wheres.push(Box::new(tunnel::name.eq(name)));
    wheres
}

// Or when the query source is a join that includes the tunnel table:

pub type TunnelJoinConnection = JoinOn<
    Join<tunnel::table, connection::table, LeftOuter>,
    Eq<Nullable<connection::columns::tunnel_id>, Nullable<tunnel::columns::id>>,
>;

fn filters_j(
    name: &'static str,
) -> Vec<Box<dyn BoxableExpression<TunnelJoinConnection, Pg, SqlType = Bool>>> {
    let mut wheres: Vec<Box<dyn BoxableExpression<TunnelJoinConnection, Pg, SqlType = Bool>>> =
        Vec::new();
    wheres.push(Box::new(tunnel::name.eq(name)));
    wheres
}

// But I'm having trouble writing it generic over anything joined onto the tunnel table.

fn filters<T>(name: &'static str) -> Vec<Box<dyn BoxableExpression<T, Pg, SqlType = Bool>>>
where
    T: AppearsInFromClause<tunnel::table, Count = Once>,
{
    vec![Box::new(tunnel::name.eq(name))]
}

/// This is how I'm using the filter function.
/// Get any tunnels with the given name, and all their connections.
fn get_connections(
    conn: &diesel::PgConnection,
    name: &'static str,
) -> Result<Vec<(Tunnel, Option<Connection>)>, Box<dyn std::error::Error>> {
    let mut query = tunnel::table.left_join(connection::table).into_boxed();
    for filter in filters("adam") {
        query = query.filter(filter);
    }
    let results = query.get_results(conn)?;
    Ok(results)
}

fn main() {}
