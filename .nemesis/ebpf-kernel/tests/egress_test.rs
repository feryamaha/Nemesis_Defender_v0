//! Testes da lógica pura de allowlist de egress (sem kernel).
#![cfg(target_os = "linux")]

use nemesis_ebpf_kernel::egress::EgressAllowlist;
use std::net::{Ipv4Addr, Ipv6Addr};

fn al(entries: &[&str]) -> EgressAllowlist {
    let v: Vec<String> = entries.iter().map(|s| s.to_string()).collect();
    EgressAllowlist::parse(&v).expect("allowlist válida")
}

#[test]
fn v4_match_dentro_do_cidr_e_porta() {
    let a = al(&["140.82.112.0/20:443"]);
    assert!(a.allows_v4("140.82.121.4".parse::<Ipv4Addr>().unwrap(), 443));
}

#[test]
fn v4_fora_do_cidr_nega() {
    let a = al(&["140.82.112.0/20:443"]);
    assert!(!a.allows_v4("8.8.8.8".parse::<Ipv4Addr>().unwrap(), 443));
}

#[test]
fn v4_porta_errada_nega() {
    let a = al(&["140.82.112.0/20:443"]);
    assert!(!a.allows_v4("140.82.121.4".parse::<Ipv4Addr>().unwrap(), 80));
}

#[test]
fn v4_porta_zero_aceita_qualquer() {
    let a = al(&["10.0.0.0/8:0"]);
    assert!(a.allows_v4("10.1.2.3".parse::<Ipv4Addr>().unwrap(), 22));
    assert!(a.allows_v4("10.1.2.3".parse::<Ipv4Addr>().unwrap(), 443));
}

#[test]
fn v4_host_unico_32() {
    let a = al(&["192.168.1.5/32:8443"]);
    assert!(a.allows_v4("192.168.1.5".parse::<Ipv4Addr>().unwrap(), 8443));
    assert!(!a.allows_v4("192.168.1.6".parse::<Ipv4Addr>().unwrap(), 8443));
}

#[test]
fn v6_match_e_porta() {
    let a = al(&["2606:4700::/32:443"]);
    assert!(a.allows_v6("2606:4700:4700::1111".parse::<Ipv6Addr>().unwrap(), 443));
    assert!(!a.allows_v6("2001:4860::8888".parse::<Ipv6Addr>().unwrap(), 443));
}

#[test]
fn fail_closed_lista_vazia_nega() {
    let a = al(&[]);
    assert!(a.is_empty());
    assert!(!a.allows_v4("8.8.8.8".parse::<Ipv4Addr>().unwrap(), 443));
    assert!(!a.allows_v6("2001:4860::8888".parse::<Ipv6Addr>().unwrap(), 443));
}

#[test]
fn parse_entrada_invalida_falha() {
    let v = vec!["nao-e-cidr:443".to_string()];
    assert!(EgressAllowlist::parse(&v).is_err());
    let v2 = vec!["10.0.0.0/8".to_string()]; // sem :porta
    assert!(EgressAllowlist::parse(&v2).is_err());
    let v3 = vec!["10.0.0.0/40:443".to_string()]; // prefixo IPv4 > 32
    assert!(EgressAllowlist::parse(&v3).is_err());
}
