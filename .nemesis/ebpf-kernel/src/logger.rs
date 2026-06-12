use crate::{EbpfBlockEvent, EbpfEventKind, KernelEvent};
use crate::violation::{Violation, ViolationType, ViolationLogger};

pub fn log_block_event(event: &EbpfBlockEvent) {
    let message = match event.kind {
        EbpfEventKind::CommandBlocked => format!("eBPF blocked command: {}", event.subject),
        EbpfEventKind::WritePathBlocked => format!("eBPF blocked write path: {}", event.subject),
        EbpfEventKind::EgressBlocked => {
            format!("NEMESIS SEC - CONEXAO NAO PERMITIDA · {}", event.subject)
        }
    };

    let violation = Violation {
        violation_type: ViolationType::PermissionDenied,
        message,
        rule: Some("ebpf-kernel".to_string()),
        command: Some(event.subject.clone()),
        timestamp: event.timestamp.clone(),
        llm_model: None,
        layer: Some("ebpf".to_string()),
    };

    ViolationLogger::log_violation(&violation);
}

pub fn kernel_event_to_block_event(event: &KernelEvent) -> EbpfBlockEvent {
    let kind = match event.kind {
        2 => EbpfEventKind::WritePathBlocked,
        3 => EbpfEventKind::EgressBlocked,
        _ => EbpfEventKind::CommandBlocked,
    };
    EbpfBlockEvent {
        pid: event.pid,
        tgid: event.tgid,
        kind,
        subject: event.subject_string(),
        decision: event.decision_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}
