import { motion } from 'framer-motion';

// ── Tipos ─────────────────────────────────────────────

interface HelpPageProps {
  onBack: () => void;
}

interface Command {
  phrases: string[];
  description: string;
}

interface CommandCategory {
  icon: string;
  title: string;
  commands: Command[];
}

// ── Dados ─────────────────────────────────────────────

const categories: CommandCategory[] = [
  {
    icon: '⏱️',
    title: 'Controle de Partida',
    commands: [
      { phrases: ['Iniciar partida', 'Começar', 'Play'], description: 'Inicia a partida' },
      { phrases: ['Encerrar', 'Fim', 'Parar'], description: 'Encerra a partida' },
      { phrases: ['Pausar', 'Pause'], description: 'Pausa o cronômetro' },
      { phrases: ['Retomar', 'Continuar'], description: 'Retoma o cronômetro' },
      { phrases: ['Volta seis'], description: 'Reinicia a contagem (zera tempo)' },
    ],
  },
  {
    icon: '⚽',
    title: 'Gols',
    commands: [
      { phrases: ['Gol time A', 'Gol do time A'], description: 'Gol para o Time A' },
      { phrases: ['Gol time B', 'Gol do time B'], description: 'Gol para o Time B' },
    ],
  },
  {
    icon: '❓',
    title: 'Desafios',
    commands: [
      { phrases: ['Dúvida', 'Contestar'], description: 'Inicia um desafio (para o cronômetro)' },
      { phrases: ['Resolver', 'Aceitar', 'Ok'], description: 'Resolve o desafio (retoma o cronômetro)' },
    ],
  },
];

// ── Sub-componentes ───────────────────────────────────

function CommandItem({ command, index }: { command: Command; index: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -12 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.04 }}
      className="py-2.5 border-b border-gray-800/60 last:border-b-0"
    >
      <p className="text-gray-300 text-sm">{command.description}</p>
      <div className="flex flex-wrap gap-1.5 mt-1">
        {command.phrases.map((phrase) => (
          <kbd
            key={phrase}
            className="inline-block px-2 py-0.5 rounded-md bg-gray-800 text-[#00ff88] text-xs font-mono border border-gray-700"
          >
            &ldquo;{phrase}&rdquo;
          </kbd>
        ))}
      </div>
    </motion.div>
  );
}

function CategoryCard({ category, index }: { category: CommandCategory; index: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.1, duration: 0.3 }}
      className="bg-gray-900/70 backdrop-blur border border-gray-800 rounded-xl p-4 sm:p-5"
    >
      <h2 className="text-base font-semibold flex items-center gap-2 mb-3">
        <span className="text-lg">{category.icon}</span>
        <span className="text-white">{category.title}</span>
      </h2>
      <div className="divide-y divide-gray-800/60">
        {category.commands.map((cmd, i) => (
          <CommandItem key={cmd.description} command={cmd} index={i} />
        ))}
      </div>
    </motion.div>
  );
}

// ── Componente principal ──────────────────────────────

export function HelpPage({ onBack }: HelpPageProps) {
  return (
    <div className="min-h-screen bg-[#0a0f1a] text-white flex flex-col items-center px-4 py-6 sm:px-6 sm:py-8">
      {/* Header */}
      <motion.header
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="w-full max-w-3xl mb-6 sm:mb-8"
      >
        <div className="flex items-center justify-between">
          <button
            onClick={onBack}
            className="text-gray-400 hover:text-white transition-colors p-1.5 rounded-lg hover:bg-gray-800"
            aria-label="Voltar"
          >
            ← Voltar
          </button>
          <h1 className="text-lg sm:text-xl font-bold tracking-tight">
            <span className="text-[#00ff88]">Comandos</span>{' '}
            <span className="text-gray-400">de Voz</span>
          </h1>
          <div className="w-16" />
        </div>
      </motion.header>

      {/* Categorias */}
      <main className="w-full max-w-3xl space-y-4 sm:space-y-5">
        {categories.map((cat, i) => (
          <CategoryCard key={cat.title} category={cat} index={i} />
        ))}

        {/* Push-to-Talk */}
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: categories.length * 0.1, duration: 0.3 }}
          className="bg-gray-900/70 backdrop-blur border border-gray-800 rounded-xl p-4 sm:p-5"
        >
          <h2 className="text-base font-semibold flex items-center gap-2 mb-3">
            <span className="text-lg">🎤</span>
            <span className="text-white">Como usar o Push-to-Talk</span>
          </h2>
          <ol className="list-decimal list-inside space-y-1.5 text-gray-300 text-sm">
            <li>Toque no ícone de microfone para iniciar gravação</li>
            <li>Fale o comando</li>
            <li>Toque novamente para processar</li>
            <li>O comando será executado automaticamente</li>
          </ol>
        </motion.div>

        {/* Dica */}
        <motion.div
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: (categories.length + 1) * 0.1, duration: 0.3 }}
          className="bg-[#00ff88]/5 border border-[#00ff88]/20 rounded-xl p-4 sm:p-5"
        >
          <h2 className="text-base font-semibold flex items-center gap-2 mb-2">
            <span className="text-lg">💡</span>
            <span className="text-[#00ff88]">Dica</span>
          </h2>
          <p className="text-gray-300 text-sm">
            Comandos funcionam mesmo no meio de frases. Ex:{' '}
            <kbd className="inline-block px-2 py-0.5 rounded-md bg-gray-800 text-[#00ff88] text-xs font-mono border border-gray-700">
              &ldquo;Fala gol time a rapaz&rdquo;
            </kbd>{' '}
            → Gol Time A
          </p>
        </motion.div>
      </main>
    </div>
  );
}
