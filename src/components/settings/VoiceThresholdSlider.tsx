import { SettingsCard } from './SettingsCard';

interface VoiceThresholdSliderProps {
  value: number;
  onChange: (threshold: number) => void;
}

/** Slider de threshold de voz (0.0 – 1.0) */
export function VoiceThresholdSlider({ value, onChange }: VoiceThresholdSliderProps) {
  return (
    <SettingsCard title="Voice Threshold" icon="🎚️">
      <div className="flex items-center gap-3">
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value))}
          className="flex-1 h-1.5 bg-gray-700 rounded-full appearance-none cursor-pointer
                     [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4
                     [&::-webkit-slider-thumb]:h-4 [&::-webkit-slider-thumb]:rounded-full
                     [&::-webkit-slider-thumb]:bg-[#00ff88] [&::-webkit-slider-thumb]:shadow-[0_0_8px_#00ff88]"
        />
        <span className="text-sm font-mono text-[#00ff88] w-10 text-right">{value.toFixed(2)}</span>
      </div>
    </SettingsCard>
  );
}
