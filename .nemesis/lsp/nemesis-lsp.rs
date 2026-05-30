/// Nemesis LSP Server - Fornecedor de diagnósticos em tempo real para o editor.
///
/// Este servidor LSP fornece diagnósticos instantâneos enquanto o usuário digita,
/// reutilizando o validate_semantic do crate ast-linters.
/// O hook pre_write_code continua bloqueando (exit 2) independentemente do LSP.

use ast_linters::validator::validate_semantic;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use url::Url;

/// Estado do servidor LSP.
struct Backend {
    client: Client,
    /// Mapa de URI do documento para conteúdo atual
    documents: Arc<RwLock<HashMap<String, String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        eprintln!("[NEMESIS-LSP] Initializing server");
        
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "nemesis-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        eprintln!("[NEMESIS-LSP] Server initialized");
        
        // Enviar mensagem de boas-vindas ao cliente
        self.client.show_message(
            MessageType::INFO,
            "Nemesis LSP Server initialized. AST linting is active.".to_string(),
        ).await;
    }

    async fn shutdown(&self) -> Result<()> {
        eprintln!("[NEMESIS-LSP] Shutting down server");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let content = params.text_document.text.clone();
        
        eprintln!("[NEMESIS-LSP] Document opened: {}", uri);
        
        // Armazena o documento
        let mut docs = self.documents.write().await;
        docs.insert(uri.clone(), content);
        drop(docs);
        
        // Publica diagnósticos
        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        
        eprintln!("[NEMESIS-LSP] Document changed: {}", uri);
        
        // Atualiza o conteúdo do documento
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(&uri) {
            for change in &params.content_changes {
                *doc = change.text.clone();
            }
        }
        drop(docs);
        
        // Publica diagnósticos
        self.publish_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        
        eprintln!("[NEMESIS-LSP] Document closed: {}", uri);
        
        // Remove o documento
        let mut docs = self.documents.write().await;
        docs.remove(&uri);
        drop(docs);
        
        // Limpa diagnósticos
        self.client.publish_diagnostics(
            params.text_document.uri.clone(),
            vec![],
            None,
        ).await;
    }
}

impl Backend {
    /// Publica diagnósticos para o documento especificado.
    async fn publish_diagnostics(&self, uri: &str) {
        // Obtém o conteúdo do documento
        let docs = self.documents.read().await;
        let content = docs.get(uri).cloned().unwrap_or_default();
        drop(docs);
        
        // Converte URI para caminho de arquivo
        let file_path = if uri.starts_with("file://") {
            uri.strip_prefix("file://").unwrap_or(uri)
        } else {
            uri
        };
        
        // Valida usando ast-linters
        let violations = validate_semantic(&content, file_path);
        
        // Converte violações para diagnósticos LSP
        let diagnostics: Vec<Diagnostic> = violations
            .into_iter()
            .map(|v| Diagnostic {
                range: Range {
                    start: Position {
                        line: v.line as u32 - 1, // LSP usa 0-indexed
                        character: 0,
                    },
                    end: Position {
                        line: v.line as u32 - 1,
                        character: 100, // Aproximação - em produção seria mais preciso
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String(v.layer.to_string())),
                source: Some("nemesis-ast".to_string()),
                message: v.message,
                ..Default::default()
            })
            .collect();
        
        eprintln!("[NEMESIS-LSP] Published {} diagnostics for {}", diagnostics.len(), uri);
        
        // Publica diagnósticos
        let url = Url::parse(uri).unwrap();
        self.client.publish_diagnostics(url, diagnostics, None).await;
    }
}

/// Inicia o servidor LSP.
pub async fn run_server() -> anyhow::Result<()> {
    eprintln!("[NEMESIS-LSP] Starting Nemesis LSP Server");
    
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(RwLock::new(HashMap::new())),
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
    
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_server().await
}
