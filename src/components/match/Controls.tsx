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
      {isIdle && (
        <Button variant="neon" onClick={handleCommand('start')}>
          ▶ Iniciar
        </Button>
      )}

      {/* Playing Normal: Pause, Goal A, Goal B, Doubt */}
      {isPlaying && (
        <Button variant="secondary" onClick={handleCommand('pause')}>
          ⏸ Pausar
        </Button>
      )}
      {isPlayingAny && (
        <Button variant="secondary" onClick={handleCommand('gol time a')}>
          ⚽ Gol A
        </Button>
      )}
      {isPlayingAny && (
        <Button variant="secondary" onClick={handleCommand('gol time b')}>
          ⚽ Gol B
        </Button>
      )}
      {isPlaying && (
        <Button variant="secondary" onClick={handleCommand('dúvida')}>
          ⚠️ Dúvida
        </Button>
      )}

      {/* Playing/Paused: End */}
      {(isPlayingAny || isPaused) && (
        <Button variant="danger" onClick={handleCommand('encerrar')}>
          🛑 Encerrar
        </Button>
      )}

      {/* Paused: Resume */}
      {isPaused && (
        <Button variant="neon" onClick={handleCommand('retomar')}>
          ▶ Retomar
        </Button>
      )}

      {/* Challenge: Resolve, Volta Seis */}
      {isChallenge && (
        <Button variant="secondary" onClick={handleCommand('resolver')}>
          ✅ Resolver
        </Button>
      )}
      {isChallenge && (
        <Button variant="secondary" onClick={handleCommand('volta seis')}>
          🔄 Volta Seis
        </Button>
      )}

      {/* Finished: Reset */}
      {isFinished && (
        <Button variant="neon" onClick={onResetMatch}>
          🔄 Novo Jogo
        </Button>
      )}
    </div>
  );
}
