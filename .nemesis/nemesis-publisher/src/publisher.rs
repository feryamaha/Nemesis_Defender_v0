//! Cliente HTTP para a dashboard: registro e ingestao.

use crate::identity::Identity;
use crate::ledger::Aggregation;
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

/// Envia contadores agregados para a dashboard.
/// POST /api/ingest com header Authorization: Bearer <ingest_token>.
pub fn publish(identity: &Identity, agg: &Aggregation, dashboard_url: &str) -> Result<()> {
    let url = format!("{}/api/ingest", dashboard_url);
    let body = agg.to_payload(&identity.install_id);

    let resp = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", identity.ingest_token))
        .set("Content-Type", "application/json")
        .send_string(&body.to_string());

    match resp {
        Ok(response) => {
            let status = response.status();
            if status == 202 {
                println!(
                    "[nemesis-publisher] Ingestao concluida: {} bloqueios enviados.",
                    agg.total_blocks
                );
                Ok(())
            } else {
                anyhow::bail!("Resposta inesperada: HTTP {}", status);
            }
        }
        Err(ureq::Error::Status(code, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            match code {
                401 => anyhow::bail!("Nao autorizado: ingest_token invalido."),
                400 => anyhow::bail!("Payload invalido: {}", body_text),
                _ => anyhow::bail!("HTTP {}: {}", code, body_text),
            }
        }
        Err(e) => {
            anyhow::bail!("Erro de rede: {}", e);
        }
    }
}
