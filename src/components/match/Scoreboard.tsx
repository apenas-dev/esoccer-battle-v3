interface ScoreboardProps {
  teamAName: string;
  teamBName: string;
  scoreA: number;
  scoreB: number;
  phase: string;
}

export function Scoreboard({ teamAName, teamBName, scoreA, scoreB, phase }: ScoreboardProps) {
  const isActive = phase === 'playing';

  return (
    <div className="flex items-center justify-center gap-8 py-6">
      <div className="text-center">
        <p className="text-sm text-gray-400 mb-1">{teamAName}</p>
        <p className="text-7xl font-black text-blue-400">{scoreA}</p>
      </div>
      
      <div className="text-center">
        <p className="text-2xl font-bold text-gray-500">×</p>
        {isActive && (
          <span className="block mt-2 w-3 h-3 rounded-full bg-green-500 animate-pulse mx-auto" title="Ao vivo" />
        )}
      </div>
      
      <div className="text-center">
        <p className="text-sm text-gray-400 mb-1">{teamBName}</p>
        <p className="text-7xl font-black text-red-400">{scoreB}</p>
      </div>
    </div>
  );
}
