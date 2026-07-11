//! Cliente HTTP para a dashboard: registro e desregistro (ping-only).

use crate::identity::Identity;
use anyhow::Result;

/// Registra o install na dashboard.
/// POST /api/installs/register com header x-install-bootstrap.
pub fn register(identity: &Identity, bootstrap_secret: &str, dashboard_url: &str) -> Result<()> {
    let url = format!("{}/api/installs/register", dashboard_url);
    let body = serde_json::json!({
        "installId": identity.install_id,
        "alias": identity.alias,
        "environment": identity.environment,
        "optIn": true,
        "tokenHash": identity.project_token_hash,
        "ingestTokenHash": identity.ingest_token_hash
    });

    let resp = ureq::post(&url)
        .set("x-install-bootstrap", bootstrap_secret)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string());

    match resp {
        Ok(response) => {
            let status = response.status();
            if status == 201 {
                println!("[nemesis-publisher] Registro concluido.");
                Ok(())
            } else {
                anyhow::bail!("Resposta inesperada: HTTP {}", status);
            }
        }
        Err(ureq::Error::Status(code, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            match code {
                401 => anyhow::bail!("Nao autorizado: bootstrap secret invalido."),
                400 => anyhow::bail!("Payload invalido: {}", body_text),
                503 => anyhow::bail!("Registro nao configurado no servidor (fase final)."),
                _ => anyhow::bail!("HTTP {}: {}", code, body_text),
            }
        }
        Err(e) => {
            anyhow::bail!("Erro de rede: {}", e);
        }
    }
}

/// Desregistra o install na dashboard (ping de uninstall).
/// POST /api/installs/unregister com header x-install-bootstrap.
pub fn unregister(identity: &Identity, bootstrap_secret: &str, dashboard_url: &str) -> Result<()> {
    let url = format!("{}/api/installs/unregister", dashboard_url);
    let body = serde_json::json!({
        "installId": identity.install_id
    });

    let resp = ureq::post(&url)
        .set("x-install-bootstrap", bootstrap_secret)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string());

    match resp {
        Ok(response) => {
            let status = response.status();
            if status == 200 {
                Ok(())
            } else {
                anyhow::bail!("Resposta inesperada: HTTP {}", status);
            }
        }
        Err(ureq::Error::Status(code, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            match code {
                401 => anyhow::bail!("Nao autorizado: bootstrap secret invalido."),
                404 => anyhow::bail!("Install nao encontrado na dashboard."),
                503 => anyhow::bail!("Unregister nao configurado no servidor."),
                _ => anyhow::bail!("HTTP {}: {}", code, body_text),
            }
        }
        Err(e) => {
            anyhow::bail!("Erro de rede: {}", e);
        }
    }
}

