#!/usr/bin/env bash
# generate-audio.sh — Sintetiza efeitos sonoros WAV para o eSoccer Battle v3
# Formato: mono 16kHz 16-bit PCM (compatível com rodio)
# Uso: ./scripts/generate-audio.sh

set -euo pipefail

AUDIO_DIR="src-tauri/audio"
mkdir -p "$AUDIO_DIR"

AR=16000  # sample rate
AC=1      # mono
ACODEC="pcm_s16le"

echo "🔊 Gerando áudios em $AUDIO_DIR/"

# 1. goal.wav — Buzzer curto (880Hz 0.15s) + tom ascendente de comemoração
ffmpeg -y -f lavfi \
  -i "sine=frequency=880:duration=0.15" \
  -f lavfi \
  -i "sine=frequency=1200:duration=0.25" \
  -filter_complex "[0][1]concat=n=2:v=0:a=1" \
  -acodec "$ACODEC" -ar "$AR" -ac "$AC" \
  "$AUDIO_DIR/goal.wav" 2>/dev/null
echo "  ✅ goal.wav"

# 2. whistle_start.wav — Apito curto (950Hz, 0.3s, fade out)
ffmpeg -y -f lavfi \
  -i "sine=frequency=950:duration=0.3" \
  -af "afade=t=out:st=0.15:d=0.15" \
  -acodec "$ACODEC" -ar "$AR" -ac "$AC" \
  "$AUDIO_DIR/whistle_start.wav" 2>/dev/null
echo "  ✅ whistle_start.wav"

# 3. whistle_end.wav — Apito duplo (950Hz 0.25s, pausa 0.15s, 950Hz 0.35s)
ffmpeg -y -f lavfi \
  -i "sine=frequency=950:duration=0.25" \
  -f lavfi \
  -i "anullsrc=channel_layout=mono:sample_rate=$AR" \
  -f lavfi \
  -i "sine=frequency=950:duration=0.35" \
  -filter_complex \
  "[0]afade=t=out:st=0.12:d=0.13[a]; \
   [1]atrim=0:0.15[b]; \
   [2]afade=t=out:st=0.2:d=0.15[c]; \
   [a][b][c]concat=n=3:v=0:a=1" \
  -acodec "$ACODEC" -ar "$AR" -ac "$AC" \
  "$AUDIO_DIR/whistle_end.wav" 2>/dev/null
echo "  ✅ whistle_end.wav"

# 4. six_meters.wav — Dois beeps curtos (800Hz)
ffmpeg -y -f lavfi \
  -i "sine=frequency=800:duration=0.12" \
  -f lavfi \
  -i "anullsrc=channel_layout=mono:sample_rate=$AR" \
  -f lavfi \
  -i "sine=frequency=800:duration=0.12" \
  -filter_complex \
  "[0]afade=t=out:st=0.06:d=0.06[a]; \
   [1]atrim=0:0.1[b]; \
   [2]afade=t=out:st=0.06:d=0.06[c]; \
   [a][b][c]concat=n=3:v=0:a=1" \
  -acodec "$ACODEC" -ar "$AR" -ac "$AC" \
  "$AUDIO_DIR/six_meters.wav" 2>/dev/null
echo "  ✅ six_meters.wav"

# 5. challenge.wav — Beep grave (350Hz, 0.4s)
ffmpeg -y -f lavfi \
  -i "sine=frequency=350:duration=0.4" \
  -af "afade=t=out:st=0.2:d=0.2" \
  -acodec "$ACODEC" -ar "$AR" -ac "$AC" \
  "$AUDIO_DIR/challenge.wav" 2>/dev/null
echo "  ✅ challenge.wav"

echo ""
echo "📁 Arquivos gerados:"
ls -lh "$AUDIO_DIR"/*.wav
