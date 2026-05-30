//! AST Visitors — semantic malicious-intent detection
//!
//! Each visitor implements a specific attack vector.
//! All visitors return Vec<DefenderViolation>.

pub mod decode_exec;       // Vetor 2: base64/hex/charCode → exec/eval/spawn
pub mod dynamic_cmd;       // Vetor 6: string concat/template → exec
pub mod url_in_exec;       // Vetor 5a: http:// URL as exec argument
pub mod unicode_steg;      // Vetor 3: BiDi/PUA in AST string nodes
pub mod prompt_injection;  // Vetor 4: AI instruction injection in strings/comments
pub mod credential_harvest;// Vetor 7: secret file reads + exfiltration pattern
pub mod time_gated;        // Vetor 5b: setTimeout/date-gated payload delivery
pub mod self_clean;        // Vetor 8: __filename unlink / package.json overwrite
pub mod manifest_abuse;    // Vetor 1: postinstall/preinstall/prepare script abuse
pub mod persistence_patterns; // Vetor 9: persistence mechanisms (cron, .bashrc, SSH keys)
pub mod nemesis_bypass;    // Vetor 10: Nemesis self-protection bypass detection
pub mod python_import_injection; // Vetor 7b: suspicious Python imports in __init__.py
