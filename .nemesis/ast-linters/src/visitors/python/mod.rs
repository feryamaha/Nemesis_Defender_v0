//! Python-specific AST visitors for security and correctness checks.
//!
//! Provides 8 specialized visitors to detect common Python vulnerabilities:
//! - no_eval_exec: eval() and exec() calls
//! - no_shell_true: subprocess.run(shell=True)
//! - sql_injection: SQL queries with string interpolation
//! - no_pickle_loads: pickle.loads() from untrusted sources
//! - no_yaml_unsafe: yaml.load() without SafeLoader
//! - no_bare_except: except: without specific exception type
//! - no_mutable_default: mutable default function arguments

pub mod no_eval_exec;
pub mod no_shell_true;
pub mod sql_injection;
pub mod no_pickle_loads;
pub mod no_yaml_unsafe;
pub mod no_bare_except;
pub mod no_mutable_default;
