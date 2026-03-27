import { Button } from '../ui/Button';
import type { GamePhase, PlayingSubPhase } from '../../types';

interface ControlsProps {
  phase: GamePhase;
  subPhase: PlayingSubPhase;
  onExecuteCommand: (text: string) => Promise<void>;
  onResetMatch: () => Promise<void>;
}

export function Controls({ phase, subPhase, onExecuteCommand, onResetMatch }: ControlsProps) {
  const isIdle = phase === 'idle';
  const isPlaying = phase === 'playing' && subPhase === 'normal';
  const isPaused = phase === 'paused';
  const isChallenge = phase === 'playing' && subPhase === 'challenge';
  const isFinished = phase === 'finished';
  const isPlayingAny = phase === 'playing';

  const handleCommand = (cmd: string) => () => onExecuteCommand(cmd);

  return (
    <div className="flex flex-wrap items-center justify-center gap-2">
      {/* Idle: Start */}
      <Button
        variant="neon"
        onClick={handleCommand('start')}
        disabled={!isIdle}
      >
        ▶ Iniciar
      </Button>

      {/* Playing Normal: Pause, Goal A, Goal B, Doubt, End */}
      <Button variant="secondary" onClick={handleCommand('pause')} disabled={!isPlaying}>
        ⏸ Pausar
      </Button>
      <Button variant="secondary" onClick={handleCommand('gol time a')} disabled={!isPlaying}>
        ⚽ Gol {isPlaying ? 'A' : ''}
      </Button>
      <Button variant="secondary" onClick={handleCommand('gol time b')} disabled={!isPlaying}>
        ⚽ Gol {isPlaying ? 'B' : ''}
      </Button>
      <Button variant="secondary" onClick={handleCommand('dúvida')} disabled={!isPlaying}>
        ⚠️ Dúvida
      </Button>
      <Button variant="danger" onClick={handleCommand('encerrar')} disabled={!isPlayingAny && !isPaused}>
        🛑 Encerrar
      </Button>

      {/* Paused: Resume */}
      <Button variant="neon" onClick={handleCommand('retomar')} disabled={!isPaused}>
        ▶ Retomar
      </Button>

      {/* Challenge: Resolve, Volta Seis */}
      <Button variant="secondary" onClick={handleCommand('resolver')} disabled={!isChallenge}>
        ✅ Resolver
      </Button>
      <Button variant="secondary" onClick={handleCommand('volta seis')} disabled={!isChallenge}>
        🔄 Volta Seis
      </Button>

      {/* Finished: Reset */}
      <Button variant="neon" onClick={onResetMatch} disabled={!isFinished}>
        🔄 Novo Jogo
      </Button>
    </div>
  );
}
