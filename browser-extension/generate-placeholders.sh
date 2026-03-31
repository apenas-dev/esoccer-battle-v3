#!/bin/bash
# generate-placeholders.sh
# Gera arquivos MP3 de placeholder (beep sintético) para testes.
# Requer: ffmpeg
#
# Uso: chmod +x generate-placeholders.sh && ./generate-placeholders.sh

AUDIO_DIR="audio"

generate_beep() {
  local filename="$1"
  local duration="${2:-1.0}"
  local freq="${3:-880}"
  local filepath="${AUDIO_DIR}/${filename}"

  if command -v ffmpeg &>/dev/null; then
    ffmpeg -y -f lavfi -i "sine=frequency=${freq}:duration=${duration}" \
      -c:a libmp3lame -q:a 9 "$filepath" 2>/dev/null
    echo "✅ Created $filepath"
  else
    echo "❌ ffmpeg not found. Install it or add real MP3 files to $AUDIO_DIR/"
    echo "   On macOS: brew install ffmpeg"
    echo "   On Ubuntu: sudo apt install ffmpeg"
    return 1
  fi
}

mkdir -p "$AUDIO_DIR"

generate_beep "partida-iniciada.mp3" 1.0 880
generate_beep "volta-6.mp3" 1.0 660
generate_beep "gol-time-a.mp3" 0.5 1200
generate_beep "gol-time-b.mp3" 0.5 1000
generate_beep "duvida-pausada.mp3" 1.5 440
generate_beep "retomar-partida.mp3" 1.0 880
generate_beep "encerrar-partida.mp3" 2.0 330

echo ""
echo "🎉 All placeholder MP3 files generated!"
echo "⚠️  Replace with real narration audio before production use."
