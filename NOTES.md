# E-Soccer V5 — Notas de Desenvolvimento

## 🎙️ Testes de Voz (2026-03-31)

### Resultados Whisper PT-BR

| # | Comando Esperado | Transcrição Real | Status |
|---|-----------------|-----------------|--------|
| 1 | "Volta 6" | "Volta 6" | ✅ Perfeito |
| 2 | "Gol do time A" | "Gol do time A." | ✅ OK (pontuação ignorada) |
| 3 | "Gol do time B" | "Goal do time B!" | ⚠️ "Goal" em inglês |
| 4 | "Dúvida" | (pendente) | ⏳ |
| 5 | "Retornar" | (pendente) | ⏳ |

### ⚠️ Issue: Whisper mistura "Gol" / "Goal"

- Whisper PT-BR pode transcrever **"Gol"** como **"Goal"** (variação em inglês)
- **Parser DEVE aceitar ambas as formas:** `gol` e `goal` (case-insensitive)
- Aplicar `unaccent()` + normalização no `command.rs` antes do match

### Regras de normalização sugeridas para command.rs
1. Lowercase
2. Remover acentos (ú → u)
3. Remover pontuação (!.?.,)
4. Trim
5. Aceitar variações: "gol" | "goal" | "volta 6" | "duvida" | "retornar"
