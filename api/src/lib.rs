mod db;
mod graphql;

use std::env;

use axum_graph_sea_core::sea_orm::{ConnectionTrait, Database, Statement};
use entity::async_graphql;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use graphql::schema::{build_schema, AppSchema};

#[cfg(debug_assertions)]
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};

async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new(
        "/api/graphql",
    )))
}

async fn db_up() {
    let db = Database::connect("postgresql://postgres:postgres@localhost:5432/postgres")
        .await
        .unwrap();
    match db
        .execute(Statement::from_string(
            db.get_database_backend(),
            format!("CREATE DATABASE \"axum_graph_sea\""),
        ))
        .await
    {
        Ok(_) => println!("Database created."),
        Err(_) => println!("Database already exists."),
    };

    let db = Database::connect(env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    match Migrator::refresh(&db).await {
        Ok(_) => println!("Migration successful."),
        Err(_) => println!("Migration failed."),
    };
}

#[tokio::main]
pub async fn main() {
    #[cfg(debug_assertions)]
    dotenv().ok();

    db_up().await;

    let schema = build_schema().await;

    let app = Router::new()
        .route(
            "/api/graphql",
            get(graphql_playground).post(graphql_handler),
        )
        .layer(Extension(schema));

    println!("Playground: http://localhost:3000/api/graphql");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
