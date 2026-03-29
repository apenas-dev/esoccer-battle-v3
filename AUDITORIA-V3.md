# Auditoria Completa — E-Soccer Battle V3

> Data: 2026-03-29 | Branch: `v3-clean-rewrite` | Arquivos auditados: 39

## Sumário

| Categoria | Total | Crítico | Médio | Baixo |
|-----------|-------|---------|-------|-------|
| Bugs | 4 | 3 | 1 | 0 |
| SOLID/KISS | 6 | 1 | 3 | 2 |
| Dead Code | 5 | 0 | 3 | 2 |
| STT Gaps | 3 | 2 | 1 | 0 |
| Documentação | 3 | 0 | 2 | 1 |

---

## 1. BUGS CRÍTICOS

### B1 — `cancel_listening` chamado mas NÃO EXISTE no backend
- **Frontend:** `WhisperProvider.ts:45` — `invoke('cancel_listening')`
- **Backend:** `main.rs` — NENHUM comando `cancel_listening` registrado
- **Erro:** Runtime `"command cancel_listening not found"` ao cancelar gravação Whisper
- **Fallback:** Código faz `.catch(() => invoke('stop_listening'))` mas o erro ainda é logado

### B2 — Whisper NÃO transcreve (backend captura áudio mas nunca roda inferência)
- **Arquitetura esperada:** `on_demand_transcriber.rs` (planejado, nunca criado)
- **Realidade:** `voice_coordinator.rs` captura áudio e emite `voice-buffer` com samples f32
- **Resultado:** `WhisperProvider.stop()` recebe `None` do backend, retorna string vazia
- **Dependência desperdiçada:** `whisper-rs = "0.16"` no Cargo.toml mas nunca importado

### B3 — Timer NÃO atualiza durante partida
- **Backend:** `action_dispatcher.rs` emite `timer-control("start"/"stop")` mas NÃO tem thread de timer
- **Frontend:** `useMatchState.ts` NÃO escuta `timer-control` (removido no "BUG 6 FIX")
- **Resultado:** Display `00:00` congelado enquanto partida roda
- **Conflito:** ADR-007 diz "timer no frontend via setInterval" mas foi removido

### B4 — Tema NÃO sincroniza frontend ↔ backend
- **Frontend:** `App.tsx` salva tema em `localStorage`
- **Backend:** `config.rs` salva tema em `config.json`
- **SettingsPage:** Atualiza backend via `update_config` mas `App.tsx` lê de `localStorage`
- **Resultado:** Mudar tema nas Settings não reflete no App até reload

---

## 2. VIOLAÇÕES SOLID/KISS

### S1 — [CRÍTICO] SoundName e SoundFile são enums paralelos (DRY)
- **match_service.rs:** `pub enum SoundName { Goal, Whistle, SixMeters, Challenge }`
- **audio.rs:** `pub enum SoundFile { Goal, Whistle, SixMeters, Challenge }`
- **Mapeamento manual em audio.rs:**
  ```rust
  SoundFile::Goal => "goal.wav",
  SoundFile::Whistle => "whistle.wav",
  // ...
  ```
- **Risco:** Adicionar som novo exige alterar DOIS enums sincronizados
- **Correção sugerida:** Um único enum `Sound` em `game.rs` com método `filename()`

### S2 — [MÉDIO] 10 dependências Cargo NÃO USADAS
| Dep | Declarada em | Usada? |
|-----|-------------|--------|
| `anyhow` | Cargo.toml | NÃO — custom error enums |
| `reqwest` | Cargo.toml | NÃO — sem HTTP calls |
| `strsim` | Cargo.toml | NÃO — parser usa match exato |
| `strum` | Cargo.toml | NÃO — sem derive macros |
| `strum_macros` | Cargo.toml | NÃO — sem derive macros |
| `sysinfo` | Cargo.toml | NÃO — sem uso de sistema |
| `tauri-plugin-dialog` | Cargo.toml | NÃO — sem dialogs |
| `tauri-plugin-opener` | Cargo.toml | NÃO — sem abrir URLs |
| `whisper-rs` | Cargo.toml | NÃO — transcrição não implementada |
| `bincode` | Cargo.toml | NÃO — sem serialização binária |

**Impacto:** Build mais lento, binário maior, confusão para desenvolvedores

### S3 — [MÉDIO] `MatchLayout` viola SRP (renderização + lógica de log/flash)
- **Estado misturado:** `flashTeam`, `logEntries`, `loggedVoiceCommand`
- **Lógica de negócio:** `detectFlashTeam()`, deduplicação de voice commands
- **Correção sugerida:** Extrair para hook `useCommandLog()` separado

### S4 — [MÉDIO] Factory hardcode `'auto'` sem input do usuário
- **MatchPage.tsx:** `createSTTProvider('auto', config)` — sempre auto
- **config.rs:** Não tem campo `stt_backend`
- **SettingsPage.tsx:** Sem dropdown para escolher STT backend
- **KISS issue:** Funcionalidade de seleção não exposta ao usuário

### S5 — [BAIXO] Eventos emitidos mas nunca consumidos (YAGNI)
| Evento | Emitido por | Listener Frontend |
|--------|-------------|-------------------|
| `voice-buffer` | voice_coordinator.rs:107 | NENHUM |
| `voice-status` | voice_coordinator.rs:79,91 | NENHUM |
| `history-updated` | action_dispatcher.rs:84 | NENHUM |

### S6 — [BAIXO] `strsim` declarado mas parser NÃO usa fuzzy matching
- **command.rs:** Usa `match` exato após normalização
- **strsim** nunca importado nem chamado
- **Decisão:** Implementar fuzzy OU remover dependência

---

## 3. STT — AUDITORIA DETALHADA

### Providers Existentes
| Provider | Tipo | Status | Funciona? |
|----------|------|--------|-----------|
| WebSpeechProvider | Online (browser) | Implementado | SIM — usa SpeechRecognition API |
| WhisperProvider | Offline (whisper-rs) | Parcial | NÃO — captura áudio mas nunca transcreve |

### Fluxo Atual (WebSpeech — FUNCIONA)
```
PTT pressionado → provider.start() → recognition.start()
PTT solto       → provider.stop()  → recognition.stop() → transcript
                  → onTranscript(text) → executeCommand(text)
```

### Fluxo Atual (Whisper — QUEBRADO)
```
PTT pressionado → provider.start() → invoke('start_listening') → cpal captura samples
PTT solto       → provider.stop()  → invoke('stop_listening') → retorna None
                  → transcript = "" (vazio) → NADA acontece
```

### Gaps para Dual STT
| # | Gap | Arquivo | Tipo |
|---|-----|---------|------|
| G1 | Sem inferência Whisper no backend | Falta `transcriber.rs` | Feature faltando |
| G2 | Sem campo `stt_backend` no config | config.rs, types.ts | Config faltando |
| G3 | Sem UI para selecionar backend | SettingsPage.tsx | UI faltando |

---

## 4. DEAD CODE / CÓDIGO MORTO

### D1 — `voice-buffer` event (emitido, nunca consumido)
- **voice_coordinator.rs:107:** `let _ = _app.emit("voice-buffer", VoiceBufferPayload { ... });`
- **Frontend:** Nenhum listener registrado
- **Payload:** `{ samples: Vec<f32>, sample_rate: u32, channels: u16 }`

### D2 — `InvalidPhase` variant (definido, nunca usado)
- **command.rs:** `pub enum ParseErrorKind { EmptyInput, UnknownCommand, InvalidPhase }`
- `InvalidPhase` nunca é construído em lugar algum

### D3 — `voice_threshold` config (salvo, nunca usado no backend)
- **config.rs:** Campo existe e é validado
- **capture.rs:** Não usa threshold para VAD
- **Frontend:** Não usa para nada

### D4 — `reqwest` dependency (declarado, nunca importado)
- **Cargo.toml:** `reqwest = { version = "0.11.24", features = ["json"] }`
- **Código:** Nenhum `use reqwest` encontrado

### D5 — `bincode` dependency (declarado, nunca importado)
- **Cargo.toml:** `bincode = "1.3.3"`
- **Código:** Nenhum `use bincode` encontrado

---

## 5. INCONSISTÊNCIAS COM DOCUMENTAÇÃO

### Doc vs Realidade

| Item no ARCHITECTURE-V3 | Realidade |
|--------------------------|-----------|
| `on_demand_transcriber.rs` | **NÃO EXISTE** |
| `src/lib/tauri.ts` | **NÃO EXISTE** |
| ADR-007: "Timer no frontend via setInterval" | **setInterval removido** (BUG 6 FIX) |
| Backend faz Whisper transcription | **Não faz** — só captura áudio |
| `drain_buffer()` em capture.rs | **NÃO IMPLEMENTADO** |

### Push-to-Talk Plan vs Implementação
| Task Planejada | Status |
|----------------|--------|
| T01: `drain_buffer()` | NÃO implementado |
| T02: `on_demand_transcriber.rs` | NÃO criado |
| T03: Novos comandos PTT | PARCIAL (usa start/stop existentes) |
| T04: `src/lib/tauri.ts` | NÃO criado |
| T05: VoiceIndicator clicável | SIM |
| T06: PTT state machine | PARCIAL (useVoicePipeline) |
| T07: `voice_text` event flow | NÃO (usa WebSpeech) |
| T08-T10: Reviews | Desconhecido |

---

## 6. MATRIZ DE COMANDOS — VERIFICAÇÃO COMPLETA

### Backend (Rust) — Comandos Tauri Registrados
| Comando | Parâmetros | Retorna | Status |
|---------|-----------|---------|--------|
| `execute_command` | text: String | MatchState | OK |
| `start_listening` | - | () | OK |
| `stop_listening` | - | Option\<String\> | OK (mas sempre None) |
| `get_config` | - | AppConfig | OK |
| `update_config` | AppConfig | () | OK |
| `get_state` | - | MatchState | OK |
| `get_history` | Option\<usize\> | Vec\<HistoryEntry\> | OK |
| `remove_history` | id: String | () | OK |
| `clear_history` | - | () | OK |
| `list_mic_devices` | - | Vec\<String\> | OK |
| `get_available_commands` | - | Vec\<CommandHelp\> | OK |
| `reset_match` | - | () | OK |
| `cancel_listening` | - | - | **NÃO EXISTE** |

### Frontend — Chamadas invoke()
| Comando | Arquivo | Status |
|---------|---------|--------|
| `execute_command` | useMatchState.ts | OK |
| `start_listening` | WhisperProvider.ts | OK |
| `stop_listening` | WhisperProvider.ts | OK |
| `cancel_listening` | WhisperProvider.ts:45 | **FALHA** — não existe |
| `get_config` | useMatchState.ts | OK |
| `update_config` | useMatchState.ts | OK |
| `get_state` | useMatchState.ts | OK |
| `get_history` | HistoryPage.tsx | OK |
| `remove_history` | HistoryPage.tsx | OK |
| `clear_history` | HistoryPage.tsx | OK |
| `list_mic_devices` | SettingsPage.tsx | OK |
| `get_available_commands` | HelpPage.tsx | OK |
| `reset_match` | useMatchState.ts | OK |

---

## 7. SCORE SOLID/KISS DETALHADO

| Princípio | Score | Detalhes |
|-----------|-------|---------|
| **S** — Single Responsibility | 8/10 | match_service puro é exemplar; MatchLayout mistura responsabilidades |
| **O** — Open/Closed | 8/10 | ISTTProvider extensível; SoundName/SoundFile requer alteração paralela |
| **L** — Liskov Substitution | 7/10 | WhisperProvider NÃO é substituível — não retorna transcript |
| **I** — Interface Segregation | 9/10 | ISTTProvider é minimal e focado |
| **D** — Dependency Inversion | 9/10 | useVoicePipeline depende de abstração, não implementação |
| **KISS** | 6/10 | 10 deps não usadas, enums paralelos, eventos mortos, factory hardcode |
| **YAGNI** | 5/10 | reqwest, sysinfo, strsim, bincode — infraestrutura para features que não existem |
| **DRY** | 7/10 | SoundName/SoundFile duplicam estrutura; tipos sincronizados manualmente Rust↔TS |
| **Geral** | **7.4/10** | Arquitetura core é sólida, mas implementação incompleta gera ruído |

---

## 8. PRIORIZAÇÃO SUGERIDA PARA CORREÇÃO

### Prioridade 1 — Corrigir antes de usar em produção
1. **B3** — Timer congelado (partida sem cronômetro)
2. **B1** — cancel_listening faltando (crash ao cancelar)
3. **B4** — Tema não sincroniza (UX inconsistente)

### Prioridade 2 — Decisão arquitetural necessária
4. **B2 + G1** — Definir destino do Whisper: implementar transcrição OU remover
5. **S2** — Remover dependências não usadas

### Prioridade 3 — Melhorias de qualidade
6. **S1** — Unificar SoundName/SoundFile
7. **S3** — Extrair log/flash do MatchLayout
8. **S5** — Limpar eventos mortos
9. **D2-D5** — Remover código morto

### Prioridade 4 — Documentação
10. Atualizar ARCHITECTURE-V3 para refletir realidade
11. Atualizar plan-push-to-talk.json com status real

---

## 9. ARQUIVOS AUDITADOS (todos lidos integralmente)

### Backend (Rust) — 10 arquivos
- `src-tauri/src/main.rs`
- `src-tauri/src/game.rs`
- `src-tauri/src/command.rs`
- `src-tauri/src/match_service.rs`
- `src-tauri/src/action_dispatcher.rs`
- `src-tauri/src/voice_coordinator.rs`
- `src-tauri/src/capture.rs`
- `src-tauri/src/audio.rs`
- `src-tauri/src/config.rs`
- `src-tauri/src/history.rs`

### Frontend (TypeScript/React) — 16 arquivos
- `src/types.ts`
- `src/App.tsx`
- `src/main.tsx`
- `src/lib/utils.ts`
- `src/services/stt/ISTTProvider.ts`
- `src/services/stt/WebSpeechProvider.ts`
- `src/services/stt/WhisperProvider.ts`
- `src/services/stt/sttFactory.ts`
- `src/hooks/useMatchState.ts`
- `src/hooks/useVoicePipeline.ts`
- `src/components/match/Scoreboard.tsx`
- `src/components/match/Timer.tsx`
- `src/components/match/Controls.tsx`
- `src/components/match/VoiceIndicator.tsx`
- `src/components/match/CommandLog.tsx`
- `src/components/match/MatchLayout.tsx`

### Frontend (Páginas) — 4 arquivos
- `src/pages/MatchPage.tsx`
- `src/pages/SettingsPage.tsx`
- `src/pages/HistoryPage.tsx`
- `src/pages/HelpPage.tsx`

### Frontend (Layout/UI) — 4 arquivos
- `src/components/layout/AppShell.tsx`
- `src/components/layout/Sidebar.tsx`
- `src/components/ui/Button.tsx`
- `src/components/ui/ThemeToggle.tsx`

### Config/Docs — 5 arquivos
- `src-tauri/Cargo.toml`
- `package.json`
- `ARCHITECTURE-V3-FROM-SCRATCH.md`
- `docs/plan-push-to-talk.json`
- `tauri.conf.json`

**Total: 39 arquivos auditados**
