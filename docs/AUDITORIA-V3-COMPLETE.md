# 🔍 AUDITORIA COMPLETA — E-Soccer Battle V3

> **Branch:** `v3-clean-rewrite` vs `master`  
> **Data:** 2026-03-29  
> **Auditor:** Code Review Agent  
> **Arquivos analisados:** 71 modificados (~11k linhas de diff)

---

## Score Geral: **0.62 / 1.0**

| Dimensão | Score | Peso | Ponderado |
|----------|-------|------|-----------|
| Funcionalidade | 0.45 | 30% | 0.135 |
| Code Quality | 0.78 | 25% | 0.195 |
| Security | 0.85 | 20% | 0.170 |
| Performance | 0.65 | 15% | 0.098 |
| Maintainability | 0.75 | 10% | 0.075 |
| **Total** | | | **0.673** |

**Nota:** Funcionalidade penalizada por **2 bugs críticos** que quebram o timer e a transcrição de voz offline. Score ajustado para **0.62** considerando severidade combinada.

---

## 🚨 Seção: "Só funciona online" — Causa Raiz

### Diagnóstico: O app NÃO funciona offline por dois motivos críticos

### BUG CRÍTICO 1: WebSpeech API é o único STT funcional

**Localização:** `src/services/stt/sttFactory.ts`, `src/services/stt/WhisperProvider.ts`

**Problema:**
O `sttFactory` no modo `auto` (padrão) tenta `WebSpeechProvider` primeiro. Se disponível, usa-o. O `WhisperProvider` existe mas é **não-funcional**:

1. `WhisperProvider.stop()` invoca `invoke('stop_listening')` no backend, que captura áudio e emite evento `voice-buffer` com samples crus, mas **nunca transcreve** — o backend apenas repassa os samples para o frontend via evento Tauri.
2. Não há nenhum código no frontend que escute o evento `voice-buffer` e faça a transcrição local.
3. O `WhisperProvider` tem um `cancel()` que invoca `invoke('cancel_listening')` — **este comando não existe no backend** (não está no `generate_handler!`). O `.catch()` silencia o erro.

**Impacto:**
- `WebSpeechProvider` usa a Web Speech API do navegador, que **requer internet** (exceto no Chrome com modelo offline instalado, que não é padrão).
- O `WhisperProvider` embora o crate `whisper-rs` esteja no `Cargo.toml`, **não é usado em nenhum lugar do backend Rust**. É só um placeholder.

**Fix sugerido:**
- Implementar transcrição Whisper no backend (Rust), no `voice_coordinator.rs`, usando o `whisper-rs` que já está nas dependências.
- Ou: usar `whisper.cpp` via WASM no frontend para transcrição offline.
- Remover a dependência `reqwest` do `Cargo.toml` (não é usada, apenas aumenta binary size).

### BUG CRÍTICO 2: Timer não funciona (sem tick)

**Localização:** `src-tauri/src/match_service.rs`, `src-tauri/src/action_dispatcher.rs`

**Problema:**
Na reescrita, o timer background thread (`spawn_timer` que fazia `game::tick()` a cada 1 segundo) foi **removido**. O novo design usa `Action::StartTimer` e `Action::StopTimer` que emitem eventos `"timer-control"` — mas **ninguém escuta esse evento** no frontend (busca por `timer-control` em `src/` retorna zero resultados).

Além disso, `Action::EmitTimeUpdated` está **definido mas nunca producido** por nenhum `process_*`. O `elapsed_secs` do estado nunca é incrementado, então o timer fica para sempre em 00:00.

**Impacto:** O cronômetro da partida não funciona de jeito nenhum. O tempo não passa.

**Fix sugerido:**
- Restaurar o timer thread no backend (como era no master) que emite `time-updated` a cada segundo.
- Ou: implementar o timer no frontend usando `setInterval` + `started_at` timestamp.
- Remover `Action::StartTimer`/`Action::StopTimer` se o timer for todo no backend, ou implementar o listener no frontend.

---

## 🐛 Bugs Encontrados por Severidade

### Critical (2)

| ID | Arquivo | Descrição |
|----|---------|-----------|
| C-1 | `sttFactory.ts`, `WhisperProvider.ts`, `voice_coordinator.rs` | **WhisperProvider é inoperante.** O backend captura áudio mas não transcreve. Nenhum modelo Whisper é carregado. `cancel_listening` não existe como Tauri command. WebSpeech é o único STT funcional e precisa de internet. |
| C-2 | `match_service.rs`, `action_dispatcher.rs`, `useMatchState.ts` | **Timer não incrementa.** `spawn_timer` removido, `Action::EmitTimeUpdated` nunca é produzido, evento `timer-control` não é escutado. `elapsed_secs` é sempre 0. |

### High (3)

| ID | Arquivo | Descrição |
|----|---------|-----------|
| H-1 | `audio.rs:play_blocking` | **Vazamento de recursos.** Cada `play()` cria novo `OutputStream` e `Sink` em thread separada. O `OutputStream` (_stream) é droppado ao final de `play_blocking`, mas `OutputStream::try_default()` pode falhar se já houver outro output stream ativo (rodio não suporta múltiplos OutputStreams simultâneos). O master usava LazyLock com leak intencional para resolver isso. |
| H-2 | `capture.rs` | **Buffer de áudio sem limite.** A nova `CaptureStream` faz `buf.extend_from_slice(data)` sem truncar. Uma sessão de captura longa (PTT pressionado por minutos) pode consumir memória ilimitada. O master tinha `BUFFER_CAPACITY` de ~5s com ring buffer. |
| H-3 | `voice_coordinator.rs:stop_listening` | **Parâmetro `_app` com nome estranho** — `_app` é usado (não deveria ter underscore), mas o nome `_app` sugere que não é usado. Isso confunde leitores. Menor: `stop_listening` retorna `Ok(None)` sempre, nunca retorna transcript — a assinatura sugere que pode retornar texto. |

### Medium (5)

| ID | Arquivo | Descrição |
|----|---------|-----------|
| M-1 | `main.rs` | **Race condition em `execute_command`.** A lock é solta entre o clone e o re-lock. Entre esses dois pontos, outro comando pode modificar o estado (ex: dois cliques rápidos em "Gol A"). O `match_service::process` usa o estado clonado, mas a escrita final pode sobrescrever um estado mais recente. |
| M-2 | `WhisperProvider.ts:cancel` | **Invoca comando inexistente.** `invoke('cancel_listening')` não existe no backend. O fallback `invoke('stop_listening')` também vai falhar porque não há capture ativa após cancel. Silenciado com `.catch(() => {})`. |
| M-3 | `config.rs:default()` | **Método `default()` sem `impl Default`.** A trait `Default` não é implementada para `AppConfig`, mas `default()` é um método associado. Isso pode causar confusão e não funciona com `Default::default()`. |
| M-4 | `Scoreboard.tsx` | **Temas hardcoded.** A classe `dark:bg-gray-700` no `CommandLog.tsx` não segue o sistema de CSS variables (`var(--bg-card)`) usado no resto do app. Inconsistência visual se tema for trocado. |
| M-5 | `Cargo.toml` | **Dependências não utilizadas.** `reqwest`, `sysinfo`, `strsim`, `bincode`, `strum`/`strum_macros` estão nas dependências mas não são importadas em nenhum arquivo `.rs`. Aumentam o tamanho do binary e tempo de compilação. |

### Low (4)

| ID | Arquivo | Descrição |
|----|---------|-----------|
| L-1 | `VoiceIndicator.tsx` | **Acessibilidade.** O botão PTT usa emoji 🎤 como label principal. Sem `aria-label` dinâmico indicando o estado (embora haja um estático). Screen readers não conseguem distinguir "ouvindo" vs "parado". |
| L-2 | `SettingsPage.tsx` | **Validação client-side fraca.** `match_duration_secs` aceita qualquer número via input (pode digitar texto). Só é validado no backend ao salvar. Deveria haver `min`/`max` no input. |
| L-3 | `MatchPage.tsx` | **`handleVoiceTranscript` e `handleButtonCommand` são idênticos.** Ambos chamam `executeCommand(text)`. Podem ser consolidados em um callback. |
| L-4 | `voice_coordinator.rs:stop_listening` | **Sem transcrição real.** O método emite `voice-buffer` com samples crus (f32 array), mas não documenta que é responsabilidade do frontend processar isso. A docstring diz "A transcrição acontece no frontend" mas não há código no frontend que escute `voice-buffer`. |

---

## ✅ Validação dos Fixes do Review Anterior (Score 0.78)

| Bug Original | Status | Detalhes |
|-------------|--------|----------|
| **BUG-1: OnceLock→Mutex** (thread safety) | ✅ **Corrigido** | `OnceLock` removido. `AppState` usa `Mutex<MatchState>`, `Mutex<AppConfig>`, `Mutex<VoiceCoordinator>`. Locks com tratamento de `PoisonError`. |
| **BUG-2: Thread leak no voice pipeline** | ✅ **Corrigido** | Nova arquitetura usa canais `mpsc` com `stop_tx`/`result_rx`. Thread de captura termina quando recebe sinal. `draining` flag impede race condition no buffer. |
| **BUG-4: Object.assign mutation no frontend** | ✅ **Corrigido** | `useMatchState.ts` usa `setState(prev => ({ ...prev, ... }))` (spread operator) — state immutability respeitada. |
| **BUG-5: Singleton state leak** | ✅ **Parcialmente corrigido** | State management reestruturado. `MatchPageWithVoice` isola o hook de voz. Mas `provider` em `MatchPage` é recriado a cada mudança de `config` sem cleanup do anterior (efeito: antigo STT provider pode ficar pendurado). |

**Veredito:** 3/5 bugs completamente corrigidos, 1 parcialmente corrigido. Os 2 bugs críticos novos (timer e whisper) não existiam no review anterior — foram introduzidos pela reescrita.

---

## 📐 Qualidade de Código (SOLID, KISS, DRY)

### Pontos Positivos
- **Separação de responsabilidades excelente.** `match_service.rs` é uma função pura — recebe estado e comando, retorna novo estado + ações. Sem efeitos colaterais. ✅ SRP
- **Command parser (`command.rs`) bem estruturado.** Tabela de aliases com normalização Unicode. Testes completos (12 testes). ✅
- **Action dispatcher separado.** `action_dispatcher.rs` isola efeitos colaterais da lógica de negócio. ✅ SRP
- **Tipos de erro customizados.** `CaptureError`, `AudioError`, `VoiceError`, `ConfigError` — todos com `Display`. ✅
- **Testes em `match_service.rs`.** 18 testes cobrindo todos os comandos e transições de estado. ✅
- **Remoção de `transcriber.rs` + `on_demand_transcriber.rs` + `parser.rs` + `buffer.rs`** — redução de ~1300 linhas. Código mais enxuto. ✅ DRY

### Pontos Negativos
- **Dependências mortas.** 5+ crates no Cargo.toml sem uso. ❌
- **Audio backend regrediu.** O LazyLock + leak intencional era correto para rodio. O novo código cria OutputStream por som e pode quebrar em reprodução simultânea. ❌ KISS
- **Timer apagado sem reposição.** A funcionalidade mais básica de uma partida de futebol — o cronômetro — foi removida sem implementar alternativa. ❌
- **Arquitetura voz incompleta.** O doc diz "transcrição no frontend" mas não implementou o pipeline completo. ❌

---

## 📊 Resumo Comparativo: Master vs v3-clean-rewrite

| Aspecto | Master | V3 Rewrite |
|---------|--------|------------|
| Timer | ✅ Funciona (thread background) | ❌ **Quebrado** (removido sem substituir) |
| Transcrição Whisper | ✅ Funciona (backend Rust) | ❌ **Quebrado** (placeholder) |
| WebSpeech | ❌ N/A | ✅ Funciona (mas precisa internet) |
| Thread Safety | ⚠️ OnceLock | ✅ Mutex com tratamento |
| Memory Leak (Voice) | ⚠️ Thread leak | ✅ Canais mpsc |
| Audio Playback | ✅ LazyLock global | ⚠️ Pode falhar em simultâneo |
| Buffer Capture | ✅ Ring buffer (5s) | ❌ Sem limite |
| Code Organization | ⚠️ Monolítico (main.rs 1100 linhas) | ✅ Modularizado |
| Testes Backend | ⚠️ Poucos | ✅ 30+ testes |
| Dependencies | ⚠️ Muitas desnecessárias | ⚠️ Muitas desnecessárias |

---

## 🎯 Conclusão

### Pronto para merge/delivery? **❌ NÃO**

### O que falta (em ordem de prioridade):

1. **[CRITICAL] Implementar timer funcional.** Sem cronômetro, o app é inutilizável como app de partida.
   - Sugestão: Restaurar `spawn_timer` do master, adaptado para emitir `time-updated` com novo formato de payload.

2. **[CRITICAL] Implementar Whisper local ou remover.** O app promete funcionar offline mas só funciona online via WebSpeech.
   - Opção A: Implementar transcrição Whisper no backend Rust (usando `whisper-rs` já dependenciado).
   - Opção B: Usar Whisper WASM no frontend.
   - Opção C: Remover Whisper como opção e documentar que o app precisa de internet para voz.

3. **[HIGH] Corrigir playback de áudio.** Usar padrão LazyLock+leak para OutputStreamHandle (como no master).

4. **[HIGH] Adicionar limite ao buffer de captura.** Ring buffer ou truncamento baseado em tempo.

5. **[MEDIUM] Proteger contra race condition em `execute_command`.** Usar uma única lock scope ou usar `compare_and_swap` pattern.

6. **[MEDIUM] Limpar dependências mortas do Cargo.toml.** `reqwest`, `sysinfo`, `strsim`, `bincode`, `strum`, `strum_macros`.

7. **[LOW] Implementar `cancel_listening` no backend ou remover chamada do frontend.**

### Nota final
A reescrita melhorou significativamente a arquitetura (separação de responsabilidades, testes, tipos), mas **regrediu em funcionalidade crítica**: o timer e a transcrição offline estão quebrados. O código precisa ser funcional antes de pensar em merge.

---

*Gerado automaticamente por Code Review Agent*  
*Score: 0.62 / 1.0 — Changes Requested*
