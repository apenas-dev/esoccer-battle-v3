import { Button } from '../ui/Button';
import type { GamePhase } from '../../types';

interface ControlsProps {
  phase: GamePhase;
  onCommand: (text: string) => void;
}

export function Controls({ phase, onCommand }: ControlsProps) {
  return (
    <div className="flex flex-wrap items-center justify-center gap-3 mt-6">
      {phase === 'idle' && (
        <Button variant="success" onClick={() => onCommand('iniciar')}>
          🏟️ Iniciar Partida
        </Button>
      )}

      {phase === 'playing' && (
        <>
          <Button variant="primary" onClick={() => onCommand('gol do time a')}>
            ⚽ Gol Time A
          </Button>
          <Button variant="primary" onClick={() => onCommand('gol do time b')}>
            ⚽ Gol Time B
          </Button>
          <Button variant="danger" onClick={() => onCommand('encerrar')}>
            🏁 Encerrar
          </Button>
        </>
      )}

      {phase === 'paused' && (
        <Button variant="danger" onClick={() => onCommand('encerrar')}>
          🏁 Encerrar
        </Button>
      )}

      {phase === 'finished' && (
        <Button variant="success" onClick={() => onCommand('novo jogo')}>
          🔄 Novo Jogo
        </Button>
      )}
    </div>
  );
}
