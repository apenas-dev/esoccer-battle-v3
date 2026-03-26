# E-Soccer Battle V3 — Build Guide

## Pré-requisitos

### Comuns
- Node.js 18+ (com npm)
- Rust (rustup.rs) — versão stable

### Windows (Prioridade)
- Visual Studio Build Tools 2022 (ou Visual Studio)
  - Componentes: "C++ build tools", "Windows 10/11 SDK"
- WebView2 (já vem no Windows 11)
- Whisper.cpp não precisa de instalação separada — é compilado via whisper-rs

### Linux (Dev)
- sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
- NOTA: Não roda `npm run tauri dev` sem display gráfico

## Setup Rápido

```bash
git clone git@github.com:apenas-dev/esoccer-battle-v3.git
cd esoccer-battle-v3
npm install
```

## Desenvolvimento

### Frontend only (sem Tauri — modo demo)
```bash
npm run dev
# Abre http://localhost:1420 — UI funciona sem backend
```

### App completo (Tauri + Rust)
```bash
npm run tauri dev
# Abre janela nativa com backend Rust + UI
# Requer: Visual Studio Build Tools (Windows) ou libs GTK (Linux)
```

## Build de Produção

```bash
npm run tauri build
# Gera .exe em src-tauri/target/release/bundle/
```

## Primeiro Uso (STT)

O app usa Whisper local para reconhecimento de voz. Na primeira execução:
1. O app detecta que nenhum modelo está baixado
2. Baixe o modelo "Base" (~148MB) pela UI ou via comando
3. O modelo fica em cache local (AppData)

## Comandos de Voz

- "Iniciar partida" / "Começar" → Inicia o jogo
- "Gol do time A" / "Gol time A" → Marca gol Time A
- "Gol do time B" / "Gol time B" → Marca gol Time B
- "Volta seis" → Reseta o cronômetro
- "Dúvida" / "Contestar" → Marca contestação
- "Encerrar" / "Fim" → Finaliza a partida

## Testes

```bash
# Backend Rust
cd src-tauri && cargo test

# Frontend build
npm run build

# E2E (modo demo, sem Tauri)
npx playwright test
```

## Estrutura

```
src-tauri/src/
├── main.rs          # Tauri commands + integração
├── capture.rs       # Captura de microfone (cpal)
├── buffer.rs        # Extração de chunks com overlap
├── transcriber.rs   # Whisper STT pipeline
├── parser.rs        # Fuzzy match de comandos
├── game.rs          # Game engine (estado da partida)
├── audio.rs         # Efeitos sonoros (rodio)
└── configuration.rs # Configurações persistentes
```
