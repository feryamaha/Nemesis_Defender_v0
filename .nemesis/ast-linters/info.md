# AST Linters — Nemesis

## Objetivo

Crate isolado para validação semântica de código-fonte via tree-sitter.
Complementa as validações regex do `workflow_enforcer` (Classe A) com análise
baseada em árvore sintática (Classe B da auditoria).

## Status Atual

- [x] Estrutura do crate criada
- [x] `language.rs` — detecção de linguagem por extensão
- [x] `parser.rs` — wrapper tree-sitter (TS/JS)
- [x] `any_via_alias.rs` — detecta `type X = any`
- [x] `conditional_hooks.rs` — detecta hooks dentro de if/for/while
- [x] `fetch_in_component.rs` — detecta fetch/axios em componente
- [x] `exhaustive_deps.rs` — detecta useEffect com deps incompletas
- [x] `unused_vars.rs` — detecta variáveis declaradas não usadas
- [x] `cache.rs` — LRU cache para AST parseado
- [x] `validator.rs` — `validate_semantic()` integrado ao workflow_enforcer
- [x] Compilação limpa (sem warnings) + 37 testes passando
- [x] Integrado ao `workflow-enforcer.rs` (library) e `pretool-hook.rs` (CLI)
- [x] Pentest integration tests: 8/8 validando detecção em padrões reais de violação
- [x] `any_via_alias` estendido: detecta `props: any`, `useState<any>`, parâmetros com `any`
- [x] `fetch_in_component` estendido: detecta fetch dentro de callbacks (useEffect, etc.)
- [ ] Etapa 5: Documentação completa + logs com layer

## Dependências

```toml
tree-sitter = "0.24"
tree-sitter-typescript = "0.23"
tree-sitter-javascript = "0.23"
tree-sitter-python = "0.23"
tree-sitter-go = "0.23"
lru = "0.12"
```

## Como Testar

```bash
cargo build -p ast-linters
cargo test -p ast-linters
```

## Decisões Arquiteturais

1. **Crate separado**: As dependências tree-sitter ficam isoladas em `ast-linters`,
   sem poluir o crate principal `nemesis`. A comunicação com o `workflow_enforcer`
   acontece via função pública `validate_semantic()`.

2. **Nunca quebrar o hook**: Se o parse falhar (linguagem não suportada, arquivo
   mal-formado), `validate_semantic()` retorna lista vazia. Falhas são logadas
   apenas em nível debug. O hook nunca deixa de bloquear por causa do AST.

3. **Cache LRU**: Evita re-parse do mesmo arquivo durante a mesma sessão.
   Chave é `(path, hash_do_conteudo)`. 32 entradas. Invalidação automática.

4. **Linguagem por extensão**: Mapeamento simples e direto. Novas linguagens
   exigem: (1) adicionar parser no Cargo.toml, (2) adicionar extensão no
   `language.rs`, (3) implementar visitors específicos.

## Estrutura

```
src/
├── lib.rs                  # Re-exports públicos
├── language.rs             # Enum Language + detecção por extensão
├── parser.rs               # Wrapper tree-sitter
├── cache.rs                # LRU cache
├── validator.rs            # validate_semantic() → Vec<Violation>
└── visitors/
    ├── mod.rs              # Re-exports dos visitors
    ├── any_via_alias.rs
    ├── conditional_hooks.rs
    ├── fetch_in_component.rs
    ├── exhaustive_deps.rs
    └── unused_vars.rs
```

## Próximos Passos

- Adaptar validações para cada linguagem
- Etapa 5: docs + logging com layer
