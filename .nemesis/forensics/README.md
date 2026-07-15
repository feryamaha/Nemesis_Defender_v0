# Forensic audit of external content (issue / PR)

Before **analyzing and merging** a third-party issue or PR, run the content through the
Nemesis engine itself. It is the project's "customs": it reduces the risk of **hidden payload**,
**prompt-injection** and **poisoning** of agent configuration files entering the source.

## How to use

1. Paste the untrusted content (issue body, PR files/diff) into:

   ```
   .nemesis/forensics/incoming/
   ```

2. Run the manual scan (from the project root):

   ```bash
   bash .nemesis/forensics/scan-incoming.sh
   ```

3. Read the verdict in the terminal and in `.nemesis/forensics/forensics-report.md`:
   - **APPROVED** — no known hostile signal. *Still read the content* (the scan does not
     understand business logic).
   - **REJECTED** — one or more files with a hostile signal. **Do not merge** without understanding each finding.

4. Clean the drop zone when done (the content is disposable and is **not** versioned):

   ```bash
   rm -rf .nemesis/forensics/incoming/*
   ```

## Why this is safe (and what it is NOT)

- The `.nemesis/forensics/` folder is **exempt from daemon quarantine**
  (`denylist-folder-files.json` → `daemon_quarantine_exempt`): the daemon still **scans and
  logs**, but does **not move** the files nor lock the session during triage. The
  authoritative verdict is the **manual scan** above.
- The drop zone (`incoming/`) and the report are **not committed** (`.gitignore`): hostile
  content never enters the history.
- **Honest limit:** this is a triage layer, **not** a guarantee. An attacker can
  write a payload the scanner does not yet know. The real defense remains
  **human review** + `CODEOWNERS` + branch protection on trust-critical files.

## Why it is NOT called `src/`

This is an **untrusted content** zone, not source code. Naming it `src/` would confuse it
with the project source and could cause it to be treated as code to compile/distribute.
`forensics/incoming/` makes the purpose explicit.
