# 🎙️ Transcrição por Voz — Whisper WASM (Electron + React)

Prototype mínimo de speech-to-text 100% offline usando Whisper (tiny) via Transformers.js/WASM.

## Stack

- **Electron** — desktop wrapper
- **React** + **Vite** — UI
- **@huggingface/transformers** — Whisper tiny rodando via WASM
- **Web Audio API** — captura do microfone

## Como rodar

```bash
npm install
npm run dev:electron
```

## Como funciona

1. Abre o app
2. Clique **"Iniciar Transcrição"**
3. Primeira vez: baixa o modelo Whisper (tiny, ~40MB) do HuggingFace com progresso visual
4. Depois: captura o microfone e transcreve em tempo real (chunks de ~3s)
5. Clique **"Parar"** para encerrar

## Estrutura

```
electron/
  main.ts          # Janela Electron
  preload.ts       # APIs seguras via contextBridge
src/
  App.tsx           # UI: botão + transcrição
  main.tsx          # Entry React
  lib/transcriber.ts # Captura mic + Whisper
vite.config.ts      # Vite + Electron plugin
```

## Notas

- Modelo: `onnx-community/whisper-tiny` (~40MB, cacheado localmente)
- Idioma padrão: Português
- Primeiro uso requer internet para baixar o modelo; depois funciona 100% offline
- Para rodar só o frontend (sem Electron): `npm run dev`
