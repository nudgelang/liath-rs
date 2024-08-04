use tonic::{transport::Server, Request, Response, Status};
use crate::query::QueryExecutor;
use std::sync::Arc;
use anyhow::Result;

pub mod ai_first_db {
    tonic::include_proto!("ai_first_db");
}

use ai_first_db::database_server::{Database, DatabaseServer};
use ai_first_db::{QueryRequest, QueryResponse};

pub struct DatabaseService {
    query_executor: Arc<QueryExecutor>,
}

#[tonic::async_trait]
impl Database for DatabaseService {
    async fn execute_query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let query_request = request.into_inner();
        let result = self.query_executor.execute(&query_request.query, &query_request.user_id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(QueryResponse { result }))
    }
}

pub async fn run_server(port: u16, query_executor: QueryExecutor) -> Result<()> {
    let addr = format!("[::1]:{}", port).parse()?;
    let database_service = DatabaseService { query_executor: Arc::new(query_executor) };

    println!("AI-First DB Server listening on {}", addr);

    Server::builder()
        .add_service(DatabaseServer::new(database_service))
        .serve(addr)
        .await?;

    Ok(())
}