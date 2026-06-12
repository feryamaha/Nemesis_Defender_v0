//! egress.rs — lógica pura da allowlist de egress (CIDR:porta).
//!
//! Sem kernel: parsing e decisão testáveis por `cargo test`. O loader usa as regras
//! expostas aqui para popular o LPM trie BPF; o programa eBPF faz a decisão real em kernel.
//! Semântica fail-closed: allowlist vazia ⇒ nenhuma conexão permitida (deny-by-default).

use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EgressParseError {
    pub entry: String,
    pub reason: String,
}

impl fmt::Display for EgressParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "entrada de egress inválida `{}`: {}", self.entry, self.reason)
    }
}

impl std::error::Error for EgressParseError {}

/// Regra IPv4: rede (já mascarada), prefixo e porta (0 = qualquer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule4 {
    pub network: u32,
    pub prefix_len: u8,
    pub port: u16,
}

/// Regra IPv6: idem, em u128.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule6 {
    pub network: u128,
    pub prefix_len: u8,
    pub port: u16,
}

#[derive(Debug, Clone, Default)]
pub struct EgressAllowlist {
    rules4: Vec<Rule4>,
    rules6: Vec<Rule6>,
}

fn mask_v4(prefix_len: u8) -> u32 {
    if prefix_len == 0 {
        0
    } else if prefix_len >= 32 {
        u32::MAX
    } else {
        u32::MAX << (32 - prefix_len)
    }
}

fn mask_v6(prefix_len: u8) -> u128 {
    if prefix_len == 0 {
        0
    } else if prefix_len >= 128 {
        u128::MAX
    } else {
        u128::MAX << (128 - prefix_len)
    }
}

impl EgressAllowlist {
    /// Faz parsing de entradas `"CIDR:porta"`, ex.: `"140.82.112.0/20:443"`,
    /// `"10.0.0.5/32:0"` (0 = qualquer porta), `"2606:4700::/32:443"`.
    pub fn parse(entries: &[String]) -> Result<Self, EgressParseError> {
        let mut rules4 = Vec::new();
        let mut rules6 = Vec::new();

        for raw in entries {
            let entry = raw.trim();
            if entry.is_empty() || entry.starts_with('#') {
                continue;
            }
            // Separa o sufixo `:porta` à direita. IPv6 contém ':' — usar o ÚLTIMO ':'
            // que segue o componente CIDR (após a '/'); por isso exigimos o formato
            // explícito `CIDR:porta` e dividimos no último ':'.
            let (cidr, port_str) = entry.rsplit_once(':').ok_or_else(|| EgressParseError {
                entry: entry.to_string(),
                reason: "formato esperado CIDR:porta".to_string(),
            })?;
            let port: u16 = port_str.parse().map_err(|_| EgressParseError {
                entry: entry.to_string(),
                reason: format!("porta inválida `{port_str}`"),
            })?;
            let (addr, prefix_str) = cidr.rsplit_once('/').ok_or_else(|| EgressParseError {
                entry: entry.to_string(),
                reason: "CIDR sem prefixo `/`".to_string(),
            })?;
            let prefix_len: u8 = prefix_str.parse().map_err(|_| EgressParseError {
                entry: entry.to_string(),
                reason: format!("prefixo inválido `{prefix_str}`"),
            })?;

            if let Ok(v4) = addr.parse::<Ipv4Addr>() {
                if prefix_len > 32 {
                    return Err(EgressParseError {
                        entry: entry.to_string(),
                        reason: "prefixo IPv4 > 32".to_string(),
                    });
                }
                let network = u32::from(v4) & mask_v4(prefix_len);
                rules4.push(Rule4 { network, prefix_len, port });
            } else if let Ok(v6) = addr.parse::<Ipv6Addr>() {
                if prefix_len > 128 {
                    return Err(EgressParseError {
                        entry: entry.to_string(),
                        reason: "prefixo IPv6 > 128".to_string(),
                    });
                }
                let network = u128::from(v6) & mask_v6(prefix_len);
                rules6.push(Rule6 { network, prefix_len, port });
            } else {
                return Err(EgressParseError {
                    entry: entry.to_string(),
                    reason: format!("endereço inválido `{addr}`"),
                });
            }
        }

        Ok(Self { rules4, rules6 })
    }

    pub fn is_empty(&self) -> bool {
        self.rules4.is_empty() && self.rules6.is_empty()
    }

    pub fn rules4(&self) -> &[Rule4] {
        &self.rules4
    }

    pub fn rules6(&self) -> &[Rule6] {
        &self.rules6
    }

    /// Decisão IPv4: true se algum CIDR cobre `ip` e a porta casa (regra.port==0 ⇒ qualquer).
    /// Lista vazia ⇒ false (fail-closed).
    pub fn allows_v4(&self, ip: Ipv4Addr, port: u16) -> bool {
        let ip_bits = u32::from(ip);
        self.rules4.iter().any(|r| {
            (ip_bits & mask_v4(r.prefix_len)) == r.network && (r.port == 0 || r.port == port)
        })
    }

    /// Decisão IPv6 (análoga).
    pub fn allows_v6(&self, ip: Ipv6Addr, port: u16) -> bool {
        let ip_bits = u128::from(ip);
        self.rules6.iter().any(|r| {
            (ip_bits & mask_v6(r.prefix_len)) == r.network && (r.port == 0 || r.port == port)
        })
    }
}
