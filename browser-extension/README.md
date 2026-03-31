# 🎤 S.O.G. Battle — Voice Commands Extension

Extensão de navegador para comandos de voz durante partidas de S.O.G. Battle.

## Comandos

| Comando (voz) | Resposta |
|---------------|----------|
| "Iniciar partida" | Partida iniciada |
| "Volta 6" | Iniciou partida de seis minutos |
| "Gol do time A" | Gol do time A |
| "Gol do time B" | Gol do time B |
| "Dúvida" | Partida pausada, devido à dúvida |
| "Retomar" | Partida retornada |
| "Encerrar partida" | Partida encerrada, fim de jogo |

## Instalação

### Chrome / Edge

1. Abra `chrome://extensions/` (ou `edge://extensions/`)
2. Ative o **Modo do desenvolvedor** (canto superior direito)
3. Clique em **Carregar sem compactação**
4. Selecione a pasta `sog-voice-extension/`
5. Aparecerá na barra de ferramentas — clique no ícone para abrir o popup

### Firefox

1. Abra `about:debugging#/runtime/this-firefox`
2. Clique em **Carregar complemento temporário**
3. Selecione o arquivo `manifest.json` dentro da pasta
4. Aparecerá na barra de ferramentas

## Preparar áudios

### Opção 1: Placeholder (para testes)
```bash
chmod +x generate-placeholders.sh
./generate-placeholders.sh
```

### Opção 2: Áudios reais
Coloque os 7 arquivos MP3 na pasta `audio/`. Veja `audio/README.md` para a lista completa.

## Uso

1. Clique no ícone da extensão
2. Clique **Ativar**
3. Fale um dos comandos — o áudio correspondente será reproduzido automaticamente
4. Clique **Desativar** para parar de escutar

## Compatibilidade

- Chrome 110+
- Firefox 112+
- Edge 110+

> ⚠️ **Nota:** O Speech Recognition (`webkitSpeechRecognition`) requer permissão de microfone. Alguns navegadores pedem permissão na primeira ativação.

## Estrutura

```
sog-voice-extension/
├── manifest.json          # MV3 config
├── background.js          # Service worker (audio playback)
├── commands.js            # Command definitions
├── popup.html             # UI
├── popup.css              # Dark theme styles
├── popup.js               # STT + matching + UI logic
├── generate-placeholders.sh
├── audio/                 # MP3 files (add yours here)
└── icons/                 # Extension icons (add yours here)
```
