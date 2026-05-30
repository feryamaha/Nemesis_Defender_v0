---
name: pre-writing-rule-control
description: >
  Valida o plano de implementacao contra regras Nemesis Rust
  antes da escrita formal. Recebe spec aprovada, analisa
  se o plano proposto viola regras Nemesis, aprova ou rejeita.
---

# Pre-Writing Rule Control (Nemesis Rust)

Validar planos de implementacao contra regras Nemesis antes da escrita formal.
Recebe spec aprovada, analisa se o plano proposto viola regras, aprova ou rejeita.

**Anuncio de inicio**: "Estou usando a skill pre-writing-rule-control para validar o plano contra regras Nemesis Rust."

**Pre-requisito**: Uma especificacao aprovada existe em `Feature-Documentation/SPECS/`.

## Processo

### Step 1: Carregar e Revisar Spec

Ler a especificacao aprovada. Identificar o que deve ser construido, qual(is) crate(s),
quais arquivos sao afetados, que restricoes se aplicam.

```bash
cat Feature-Documentation/SPECS/SPEC_*.md | tail -1
```

### Step 2: Validar Contra Regras Nemesis Rust

Validar o plano proposto contra **6 regras fundamentais**:

#### Regra 1: Somente Rust em .nemesis/
- **Proibido**: .ts, .js, .py, .sh dentro de .nemesis/
- **Obrigatorio**: Todos os arquivos modificados devem ser .rs
- **Validacao**: Toda tarefa lista FILES INVOLVED — todos devem ter extensao .rs

#### Regra 2: Build via Cargo Workspace
- **Obrigatorio**: Usar `cargo check -p <crate>` por tarefa
- **Obrigatorio**: Usar `cargo test -p <crate>` para validacao
- **Proibido**: Compilacao direta com rustc, cargo build sem autorizacao
- **Validacao**: Cada tarefa tem verificacao com cargo check

#### Regra 3: Maintenance Mode para Hooks
- **Alertar se**: Modificacoes em `.nemesis/hooks/`
- **Requerido**: Ler `.nemesis/nemesis-install/check.sh` antes de modificar
- **Validacao**: Se tarefa afeta .nemesis/hooks/, requer flag "maintenance_mode_required"

#### Regra 4: Scope da Spec
- **Obrigatorio**: Nao sair do escopo files listados em REQUIREMENTS/FILES INVOLVED
- **Proibido**: Modificar arquivos nao listados na spec
- **Validacao**: Cada tarefa FILE deve estar em list original da spec

#### Regra 5: Git Operations — Fernando Apenas
- **Proibido**: Plano nao deve requerer git add, git commit, git push
- **Permitido**: Unica exception: relatorios em Feature-Documentation/ (sem git)
- **Validacao**: Nenhuma tarefa executa git write operations

#### Regra 6: Sem Binarios Fora de .nemesis/target/
- **Proibido**: Copiar binarios para outro path
- **Permitido**: Somente .nemesis/target/release/
- **Validacao**: Nenhuma tarefa copia arquivo fora de .nemesis/

### Step 3: Analisar Plano Contra Regras

Verificacoes criticas:

```
- [ ] REGRA 1: Todos os arquivos sao .rs? (ou .toml, .lock para Cargo.*)
- [ ] REGRA 2: Cada tarefa usa cargo check -p <crate> para validacao?
- [ ] REGRA 3: Alertar se .nemesis/hooks/ e afetado?
- [ ] REGRA 4: Todos os FILES INVOLVED estao na spec original?
- [ ] REGRA 5: Nenhuma tarefa executa git add/commit/push?
- [ ] REGRA 6: Nenhuma copia de binarios fora de .nemesis/target/?
```

**Se violacao detectada**:
- Rejeitar o plano
- Explicar qual regra foi violada
- Sugerir ajustes
- NAO prosseguir para nemesis-writing-plans

**Se NENHUMA violacao**:
- Aprovar o plano
- Prosseguir para `nemesis-writing-plans`

### Step 4: Apresentar Decisao

**HARD-GATE**: Apresentar decisao de validacao. BLOQUEAR todas as acoes até resposta de Fernando.

**Se aprovado**:
```
Plano validado contra 6 regras Nemesis Rust:
✅ REGRA 1 (somente Rust): PASS
✅ REGRA 2 (cargo workspace): PASS
✅ REGRA 3 (maintenance mode): [PASS | ALERTAR]
✅ REGRA 4 (scope): PASS
✅ REGRA 5 (git): PASS
✅ REGRA 6 (binarios): PASS

Prosseguindo para nemesis-writing-plans.
```

**Se rejeitado**:
```
Plano rejeitado. Violacao detectada:
❌ [REGRA_N]: [descricao exata da violacao]

Ajustes necessarios: [lista de mudancas requeridas]

Aguardando revisa...
```

Respostas validas de aprovacao: "sim", "pode", "aprovado", "ok", "prossiga"

## Lembrar

- Validacao ANTES de escrita, nao depois
- 6 regras fundamentais — nao sao negociaveis
- Explicacao clara quando rejeitar
- Sempre PT-BR
- Nemesis enforcement valida codigo — skill valida planejamento

## Integracao

**Skill anterior**: `nemesis-specification-design`
**Proxima skill apos aprovacao**: `nemesis-writing-plans`