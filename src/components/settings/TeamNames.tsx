import { SettingsCard } from './SettingsCard';

interface TeamNamesProps {
  teamA: string;
  teamB: string;
  onChangeTeamA: (name: string) => void;
  onChangeTeamB: (name: string) => void;
}

/** Inputs para nomes dos times */
export function TeamNames({ teamA, teamB, onChangeTeamA, onChangeTeamB }: TeamNamesProps) {
  return (
    <SettingsCard title="Nomes dos Times" icon="📝">
      <div className="space-y-3">
        <div>
          <label className="text-[11px] text-gray-500 uppercase tracking-wider mb-1 block">Time A</label>
          <input
            type="text"
            value={teamA}
            onChange={(e) => onChangeTeamA(e.target.value)}
            placeholder="Time A"
            maxLength={30}
            className="w-full bg-[#0a0f1a] border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                       placeholder:text-gray-600 focus:outline-none focus:border-[#00ff88] focus:ring-1
                       focus:ring-[#00ff88]/30 transition-colors"
          />
        </div>
        <div>
          <label className="text-[11px] text-gray-500 uppercase tracking-wider mb-1 block">Time B</label>
          <input
            type="text"
            value={teamB}
            onChange={(e) => onChangeTeamB(e.target.value)}
            placeholder="Time B"
            maxLength={30}
            className="w-full bg-[#0a0f1a] border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                       placeholder:text-gray-600 focus:outline-none focus:border-[#00ff88] focus:ring-1
                       focus:ring-[#00ff88]/30 transition-colors"
          />
        </div>
      </div>
    </SettingsCard>
  );
}
