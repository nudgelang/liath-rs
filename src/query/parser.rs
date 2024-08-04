use anyhow::Result;

#[derive(Debug, PartialEq, Clone)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    CreateNamespace,
    DeleteNamespace,
    UploadFile,
    ProcessFile,
    GenerateEmbedding,
    SimilaritySearch,
    LLMQuery,
    Join,
    Aggregate,
    InstallPackage,
    ListPackages,
    ExecuteLua,
}

pub struct QueryParser;

impl QueryParser {
    pub fn parse(query: &str) -> Result<(QueryType, Vec<String>)> {
        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty query"));
        }

        let query_type = match parts[0].to_lowercase().as_str() {
            "select" => QueryType::Select,
            "insert" => QueryType::Insert,
            "update" => QueryType::Update,
            "delete" => QueryType::Delete,
            "create_namespace" => QueryType::CreateNamespace,
            "delete_namespace" => QueryType::DeleteNamespace,
            "upload_file" => QueryType::UploadFile,
            "process_file" => QueryType::ProcessFile,
            "generate_embedding" => QueryType::GenerateEmbedding,
            "similarity_search" => QueryType::SimilaritySearch,
            "llm_query" => QueryType::LLMQuery,
            "join" => QueryType::Join,
            "aggregate" => QueryType::Aggregate,
            "install_package" => QueryType::InstallPackage,
            "list_packages" => QueryType::ListPackages,
            "execute_lua" => QueryType::ExecuteLua,
            _ => return Err(anyhow::anyhow!("Unknown query type")),
        };

        let args = parts[1..].iter().map(|&s| s.to_string()).collect();

        Ok((query_type, args))
    }
}