use serde::{Deserialize, Serialize};

/// AST para DSL Workflow Nemesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAst {
    pub name: String,
    pub path: String,
    pub phases: Vec<WorkflowPhase>,
    pub metadata: std::collections::HashMap<String, String>,
    #[serde(rename = "rawContent")]
    pub raw_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPhase {
    pub id: String,
    pub title: String,
    pub actions: Vec<ActionNode>,
    pub gates: Vec<GateNode>,
    pub verifies: Vec<VerifyNode>,
    pub restrictions: Vec<RestrictionNode>,
    pub constants: Vec<ConstantNode>,
    pub dictionaries: Vec<DictionaryNode>,
    pub schemas: Vec<SchemaNode>,
    pub maps: Vec<MapNode>,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ActionNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct GateNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct VerifyNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct RestrictionNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct ConstantNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    pub value: String,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct DictionaryNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    pub entries: std::collections::HashMap<String, String>,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct SchemaNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    pub structure: String,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct MapNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformation: Option<String>,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
    #[serde(rename = "rawText")]
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslParseResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<WorkflowAst>,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub line: u32,
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseWarning {
    pub line: u32,
    pub message: String,
    #[serde(rename = "type")]
    pub warning_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    #[serde(rename = "workflowName")]
    pub workflow_name: String,
    #[serde(rename = "currentPhase")]
    pub current_phase: String,
    #[serde(rename = "completedSteps")]
    pub completed_steps: Vec<String>,
    pub variables: std::collections::HashMap<String, String>,
    pub artifacts: std::collections::HashMap<String, String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    #[serde(rename = "completedPhase", skip_serializing_if = "Option::is_none")]
    pub completed_phase: Option<String>,
    #[serde(rename = "nextPhase", skip_serializing_if = "Option::is_none")]
    pub next_phase: Option<String>,
    pub errors: Vec<ExecutionError>,
    pub artifacts: std::collections::HashMap<String, String>,
    pub violations: Vec<AstViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    pub phase: String,
    #[serde(rename = "nodeType")]
    pub node_type: String,
    pub message: String,
    #[serde(rename = "lineNumber")]
    pub line_number: u32,
}

/// Violation específica para uso no AST (com campos adicionais)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstViolation {
    #[serde(rename = "type")]
    pub violation_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(rename = "lineNumber", skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionState {
    #[serde(rename = "workflowName")]
    pub workflow_name: String,
    #[serde(rename = "currentPhaseIndex")]
    pub current_phase_index: u32,
    #[serde(rename = "currentNodeIndex")]
    pub current_node_index: u32,
    #[serde(rename = "completedPhases")]
    pub completed_phases: Vec<String>,
    pub variables: std::collections::HashMap<String, String>,
    pub artifacts: std::collections::HashMap<String, String>,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "lastUpdateTime")]
    pub last_update_time: String,
}

/// Regex patterns para DSL parsing — paridade com TS objeto DSL_PATTERNS (campos nomeados).
pub struct DslPatterns {
    pub action: regex::Regex,
    pub gate: regex::Regex,
    pub verify: regex::Regex,
    pub restriction: regex::Regex,
    pub constant: regex::Regex,
    pub dictionary: regex::Regex,
    pub schema: regex::Regex,
    pub map: regex::Regex,
    pub phase_header: regex::Regex,
}

lazy_static::lazy_static! {
    pub static ref DSL_PATTERNS: DslPatterns = DslPatterns {
        action: regex::Regex::new(r"^\[ACTION:\s*([^\]]+)\]").unwrap(),
        gate: regex::Regex::new(r"^\[GATE:\s*([^\]]+)\]").unwrap(),
        verify: regex::Regex::new(r"^\[VERIFY:\s*([^\]]+)\]").unwrap(),
        restriction: regex::Regex::new(r"^\[RESTRICTION:\s*([^\]]+)\]").unwrap(),
        constant: regex::Regex::new(r"^\[CONSTANT:\s*([^\]]+)\]\s*=\s*(.+)$").unwrap(),
        dictionary: regex::Regex::new(r"^\[(DICT|DICTIONARY):\s*([^\]]+)\]").unwrap(),
        schema: regex::Regex::new(r"^\[SCHEMA:\s*([^\]]+)\]").unwrap(),
        map: regex::Regex::new(r"^\[MAP\]\s*([^\]]+)\s*->\s*(.+)$").unwrap(),
        phase_header: regex::Regex::new(r"^##\s+(FASE\s+[\d\.]+|[A-Z\s]+)$").unwrap(),
    };
}

#[derive(Debug, Clone, Copy)]
pub enum PatternType {
    Action,
    Gate,
    Verify,
    Restriction,
    Constant,
    Dictionary,
    Schema,
    Map,
    PhaseHeader,
}

/// Tipo unificado de nós DSL
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DslNode {
    Action(ActionNode),
    Gate(GateNode),
    Verify(VerifyNode),
    Restriction(RestrictionNode),
    Constant(ConstantNode),
    Dictionary(DictionaryNode),
    Schema(SchemaNode),
    Map(MapNode),
}

// Aliases para os nomes em UPPERCASE usados pelos consumidores (paridade com TS).
pub type WorkflowAST = WorkflowAst;
pub type DSLParseResult = DslParseResult;
pub type DSLNode = DslNode;
